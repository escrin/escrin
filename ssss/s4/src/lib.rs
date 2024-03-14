use aes_gcm_siv::AeadInPlace;
use ethers::{
    core::utils::keccak256,
    signers::{LocalWallet, Signer as _},
    types::transaction::eip712::Eip712 as _,
};
use eyre::Result;
use futures::TryFutureExt as _;
use headers::Header as _;
use reqwest::StatusCode;
use ssss::types::{api::*, *};

#[derive(Clone, Debug)]
pub struct SsssClient {
    client: reqwest::Client,
    url: url::Url,
}

impl SsssClient {
    pub fn new(ssss: url::Url) -> Self {
        Self {
            client: Default::default(),
            url: ssss,
        }
    }

    // TODO: cache this
    pub async fn get_ssss_identity(&self) -> Result<IdentityResponse> {
        Ok(reqwest::get(self.url.join("/v1/identity").unwrap())
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Returns whether the SSSS optimistically granted the permit.
    pub async fn acquire_identity(
        &self,
        il: IdentityLocator,
        params: &AcqRelIdentityRequest,
        signer: Option<&LocalWallet>,
    ) -> Result<bool> {
        Ok(self.acqrel_identity(il, params, signer, true).await? == StatusCode::CREATED)
    }

    pub async fn release_identity(
        &self,
        il: IdentityLocator,
        params: &AcqRelIdentityRequest,
        signer: Option<&LocalWallet>,
    ) -> Result<()> {
        self.acqrel_identity(il, params, signer, false).await?;
        Ok(())
    }

    async fn acqrel_identity(
        &self,
        IdentityLocator {
            chain,
            registry,
            id: IdentityId(identity),
        }: IdentityLocator,
        params: &AcqRelIdentityRequest,
        signer: Option<&LocalWallet>,
        acquire: bool,
    ) -> Result<StatusCode> {
        let paq = format!("/v1/permits/{chain}/{registry:x}/{identity:x}");
        let url = self.url.join(&paq)?;
        let body = serde_json::to_vec(&params)?;
        let method = if acquire {
            reqwest::Method::POST
        } else {
            reqwest::Method::DELETE
        };

        let req = self.client.request(method.clone(), url.clone());
        let res = match signer {
            Some(signer) => Self::attach_escrin1_sig(
                req,
                SsssRequest {
                    method: method.to_string(),
                    host: url.authority().to_string(),
                    path_and_query: paq,
                    body: keccak256(&body).into(),
                },
                signer,
            )?,
            None => req,
        }
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await?;

        if !res.status().is_success() {
            let res_text = res.text().await?;
            let ErrorResponse { error } =
                serde_json::from_str(&res_text).unwrap_or(ErrorResponse { error: res_text });
            return Err(eyre::eyre!("failed to acquire identity: {error}"));
        }

        Ok(res.status())
    }

    pub async fn get_share(
        &self,
        name: &str,
        IdentityLocator {
            chain,
            registry,
            id: IdentityId(identity),
        }: IdentityLocator,
        version: u64,
        signer: &LocalWallet,
        sk: Option<&p384::SecretKey>,
    ) -> Result<(u64, Vec<u8>)> {
        let paq = format!("/v1/shares/{name}/{chain}/{registry:x}/{identity:x}?version={version}");
        let url = self.url.join(&paq)?;

        let sk: std::borrow::Cow<p384::SecretKey> = match sk {
            Some(sk) => std::borrow::Cow::Borrowed(sk),
            None => std::borrow::Cow::Owned(p384::SecretKey::random(&mut rand::thread_rng())),
        };

        let shares_req = Self::attach_escrin1_sig(
            self.client.get(url.clone()),
            SsssRequest {
                method: "GET".into(),
                host: url.authority().to_string(),
                path_and_query: paq,
                body: Default::default(),
            },
            signer,
        )?
        .header(
            RequesterPublicKeyHeader::name().as_str(),
            RequesterPublicKeyHeader(sk.public_key()).to_string(),
        )
        .send()
        .map_err(eyre::Error::from);

        let (shares_res, ssss_identity) = tokio::try_join!(shares_req, self.get_ssss_identity())?;

        if !shares_res.status().is_success() {
            let res_text = shares_res.text().await?;
            let ErrorResponse { error } =
                serde_json::from_str(&res_text).unwrap_or(ErrorResponse { error: res_text });
            return Err(eyre::eyre!(
                "failed to get shares from {}: {error}",
                self.url
            ));
        }

        let res: ShareResponse = shares_res.error_for_status()?.json().await?;

        let share = match res.format {
            ShareResponseFormat::Plain => res.ss.share,
            ShareResponseFormat::EncAes256GcmSiv { nonce } => {
                let mut share = res.ss.share;
                let ssss_pk = p384::PublicKey::from_jwk(&ssss_identity.ephemeral)?;
                let cipher = ssss::identity::derive_shared_cipher(
                    &sk.to_nonzero_scalar(),
                    &ssss_pk,
                    ssss::identity::GET_SHARE_DOMAIN_SEP,
                );
                cipher
                    .decrypt_in_place(&nonce.into(), &[], &mut share)
                    .map_err(|_| eyre::eyre!("share decryption failed"))?;
                share
            }
        };

        Ok((res.ss.index, share))
    }

    fn attach_escrin1_sig(
        req: reqwest::RequestBuilder,
        req721: SsssRequest,
        signer: &LocalWallet,
    ) -> Result<reqwest::RequestBuilder> {
        let req_hash = req721.encode_eip712()?;
        let sig = signer.sign_hash(req_hash.into())?;
        Ok(req
            .header(
                SignatureHeader::name().as_str(),
                SignatureHeader(sig).to_string(),
            )
            .header(
                RequesterHeader::name().as_str(),
                RequesterHeader(signer.address()).to_string(),
            ))
    }
}
