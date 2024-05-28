use std::sync::{Arc, Mutex};

use aes_gcm_siv::AeadInPlace as _;
use ethers::{
    core::utils::keccak256,
    signers::{LocalWallet, Signer as _},
    types::{transaction::eip712::Eip712 as _, Address, Signature},
};
use eyre::{ensure, Result};
use headers::Header as _;
use rand::RngCore as _;
use reqwest::{RequestBuilder, Response, StatusCode};
use ssss::types::{api::*, *};
use tokio::sync::OnceCell;

#[derive(Clone, Debug)]
pub struct SsssClient {
    client: reqwest::Client,
    url: url::Url,
    remote_signer: OnceCell<Address>,
    remote_ek: Arc<Mutex<Option<EphemeralKey>>>,
}

impl SsssClient {
    pub fn new(ssss: url::Url) -> Self {
        Self {
            client: Default::default(),
            url: ssss,
            remote_signer: Default::default(),
            remote_ek: Default::default(),
        }
    }

    pub async fn signer(&self) -> Result<Address> {
        self.remote_signer
            .get_or_try_init(|| async {
                let identity = self.fetch_remote_identity().await?;
                let mut remote_ek = self.remote_ek.lock().unwrap();
                if identity.ephemeral.expiry
                    > remote_ek.as_ref().map(|k| k.expiry).unwrap_or_default()
                {
                    *remote_ek = Some(identity.ephemeral);
                }
                Ok(identity.signer)
            })
            .await
            .copied()
    }

    async fn fetch_remote_identity(&self) -> Result<IdentityResponse> {
        Ok(send_request(self.client.get(self.url("/v1/identity")))
            .await?
            .json()
            .await?)
    }

    async fn ephemeral_key(&self) -> Result<EphemeralKey> {
        {
            let remote_ek = self.remote_ek.lock().unwrap();
            match &*remote_ek {
                Some(ek) if ek.expiry > ssss::utils::now() + 5 * 60 => return Ok(ek.clone()),
                _ => {}
            }
        }
        let identity = self.fetch_remote_identity().await?;
        let mut remote_ek = self.remote_ek.lock().unwrap();
        if identity.ephemeral.expiry > remote_ek.as_ref().map(|k| k.expiry).unwrap_or_default() {
            *remote_ek = Some(identity.ephemeral.clone());
        }
        Ok(identity.ephemeral)
    }

    pub async fn set_policy(
        &self,
        IdentityLocator {
            chain,
            registry,
            id,
        }: IdentityLocator,
        policy: &PolicyDocument,
    ) -> Result<()> {
        ensure!(
            send_request(
                self.client
                    .post(self.url(format!("/v1/policies/{chain}/{registry}/{id}")))
                    .json(policy),
            )
            .await?
            .status()
                != StatusCode::NO_CONTENT,
            "failed to set policy: received unexpected response status"
        );
        Ok(())
    }

    pub async fn deal_share(
        &self,
        id: &ShareId,
        share: api::SecretShare,
        signer: &LocalWallet,
    ) -> Result<()> {
        let mut payload = serde_json::to_vec(&ShareBody { share })?;

        let kp = ssss::keypair::KeyPair::ephemeral();

        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);

        let ssss_key = self.ephemeral_key().await?;

        let cipher = kp.derive_shared_cipher(ssss_key.pk, ssss::keypair::DEAL_SHARES_DOMAIN_SEP);
        cipher
            .encrypt_in_place(&nonce.into(), &[], &mut payload)
            .unwrap();

        let body = serde_json::to_vec(&EncryptedPayload {
            format: EncryptedPayloadFormat::P384EcdhAes256GcmSiv {
                curve: CurveP384,
                pk: *kp.public_key(),
                nonce,
                recipient_key_id: ssss_key.key_id,
            },
            payload: payload.into(),
        })?;

        let ShareId {
            identity:
                IdentityLocator {
                    chain,
                    registry,
                    id: identity,
                },
            secret_name,
            version,
        } = id;
        let url = self.url(format!(
            "/shares/{secret_name}/{chain}/{registry}/{identity}?version={version}"
        ));

        let ssss_req = SsssRequest {
            method: "POST".into(),
            url: url.to_string(),
            body: keccak256(&body).into(),
        };

        send_request(
            Self::attach_escrin1_sig(
                self.client
                    .post(url)
                    .header("content-type", "application/json"),
                ssss_req,
                signer,
            )?
            .body(body),
        )
        .await?;

        Ok(())
    }

    pub async fn commit_share(&self, id: &ShareId, signer: &LocalWallet) -> Result<()> {
        let ShareId {
            identity:
                IdentityLocator {
                    chain,
                    registry,
                    id: identity,
                },
            secret_name,
            version,
        } = id;
        let url = self.url(format!(
            "/shares/{secret_name}/{chain}/{registry}/{identity}/commit?version={version}"
        ));

        let ssss_req = SsssRequest {
            method: "POST".into(),
            url: url.to_string(),
            body: keccak256(b"").into(),
        };

        send_request(Self::attach_escrin1_sig(
            self.client.post(url),
            ssss_req,
            signer,
        )?)
        .await?;

        Ok(())
    }

    pub async fn get_share(
        &self,
        id: &ShareId,
        signer: &LocalWallet,
    ) -> Result<ssss::types::api::SecretShare> {
        let kp = ssss::keypair::KeyPair::ephemeral();

        let ShareId {
            identity:
                IdentityLocator {
                    chain,
                    registry,
                    id: identity,
                },
            secret_name,
            version,
        } = id;
        let url = self.url(format!(
            "/shares/{secret_name}/{chain}/{registry}/{identity}/commit?version={version}&pk={}",
            kp.fingerprint() // bind the requester public key to the request
        ));

        let ssss_req = SsssRequest {
            method: "GET".into(),
            url: url.to_string(),
            body: Default::default(),
        };

        let res = send_request(Self::attach_escrin1_sig(
            self.client.get(url).header(
                RequesterPublicKeyHeader::name().as_str(),
                RequesterPublicKeyHeader(*kp.public_key()).to_string(),
            ),
            ssss_req,
            signer,
        )?)
        .await?;

        Ok(decrypt_enc_payload::<ShareBody>(res.json().await?, kp)?.share)
    }

    /// Returns whether the SSSS optimistically granted the permit.
    pub async fn request_acquire_identity_permit(
        &self,
        il: IdentityLocator,
        params: &AcqRelIdentityRequest,
        signer: Option<&LocalWallet>,
    ) -> Result<(Address, Signature)> {
        self.request_identity_permit(il, params, signer, true).await
    }

    pub async fn request_release_identity_permit(
        &self,
        il: IdentityLocator,
        params: &AcqRelIdentityRequest,
        signer: Option<&LocalWallet>,
    ) -> Result<(Address, Signature)> {
        self.request_identity_permit(il, params, signer, false)
            .await
    }

    async fn request_identity_permit(
        &self,
        IdentityLocator {
            chain,
            registry,
            id: IdentityId(identity),
        }: IdentityLocator,
        params: &AcqRelIdentityRequest,
        signer: Option<&LocalWallet>,
        acquire: bool,
    ) -> Result<(Address, Signature)> {
        let url = self.url(format!("/v1/permits/{chain}/{registry:x}/{identity:x}"));
        let body = serde_json::to_vec(&params)?;
        let method = if acquire {
            reqwest::Method::POST
        } else {
            reqwest::Method::DELETE
        };

        let kp = ssss::keypair::KeyPair::ephemeral();

        let req = self
            .client
            .request(method.clone(), url.clone())
            .header(
                RequesterPublicKeyHeader::name().as_str(),
                RequesterPublicKeyHeader(*kp.public_key()).to_string(),
            )
            .header("content-type", "application/json");
        let req = match signer {
            Some(signer) => Self::attach_escrin1_sig(
                req,
                SsssRequest {
                    url: url.to_string(),
                    method: method.to_string(),
                    body: keccak256(&body).into(),
                },
                signer,
            )?,
            None => req,
        }
        .body(body);
        let res = send_request(req).await?;

        decrypt_enc_payload(res.json().await?, kp)
    }

    fn attach_escrin1_sig(
        req: RequestBuilder,
        req721: SsssRequest,
        signer: &LocalWallet,
    ) -> Result<RequestBuilder> {
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

    fn url(&self, u: impl AsRef<str>) -> url::Url {
        self.url.join(u.as_ref()).unwrap()
    }
}

fn decrypt_enc_payload<T: serde::de::DeserializeOwned>(
    enc_payload: EncryptedPayload,
    kp: ssss::keypair::KeyPair,
) -> Result<T> {
    let EncryptedPayloadFormat::P384EcdhAes256GcmSiv {
        curve: CurveP384,
        pk: ppk,
        nonce,
        ..
    } = enc_payload.format
    else {
        unreachable!("unsupported encrypted response format");
    };
    let mut payload: Vec<u8> = enc_payload.payload.0.into();

    let cipher = kp.derive_shared_cipher(ppk, ssss::keypair::GET_SHARE_DOMAIN_SEP);
    cipher
        .decrypt_in_place(&nonce.into(), &[], &mut payload)
        .unwrap();

    Ok(serde_json::from_slice(&payload)?)
}

async fn send_request(req: RequestBuilder) -> Result<Response> {
    let res = req.send().await?;

    if !res.status().is_success() {
        let res_text = res.text().await?;
        let ErrorResponse { error } =
            serde_json::from_str(&res_text).unwrap_or(ErrorResponse { error: res_text });
        Err(eyre::eyre!("request failed: {error}"))
    } else {
        Ok(res)
    }
}
