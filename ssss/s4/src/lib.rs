use std::{
    io::Write as _,
    sync::{Arc, Mutex},
};

use aes_gcm_siv::AeadInPlace as _;
use ethers::{
    core::utils::keccak256,
    signers::{LocalWallet, Signer as _},
    types::{transaction::eip712::Eip712 as _, Address, H256},
};
use eyre::{ensure, Context, Result};
use headers::Header as _;
use rand::RngCore as _;
use reqwest::{Method, RequestBuilder, Response, StatusCode};
use ssss::types::{api::*, *};
use tokio::sync::OnceCell;

#[derive(Clone, Debug)]
pub struct SsssClient {
    pub url: url::Url,
    client: reqwest::Client,
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
        Ok(send_request(self.client.get(self.url("v1/identity")))
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
        permitter: Address,
        policy: &PolicyDocument,
    ) -> Result<()> {
        let res = send_request(
            self.client
                .post(self.url(format!("v1/policies/{chain}/{registry:x}/{id}")))
                .json(&SetPolicyRequest {
                    permitter,
                    policy: serde_json::value::RawValue::from_string(serde_json::to_string(
                        &policy,
                    )?)?,
                }),
        )
        .await?;
        ensure!(
            res.status() == StatusCode::NO_CONTENT,
            "failed to set policy: received unexpected response status: {}",
            res.status()
        );
        Ok(())
    }

    pub async fn deal_share(
        &self,
        id: &ShareId,
        share: api::SecretShare,
        signer: &LocalWallet,
    ) -> Result<()> {
        let mut payload = serde_json::to_vec(&share)?;

        let kp = ssss::keypair::KeyPair::ephemeral();

        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);

        let ssss_key = self.ephemeral_key().await?;

        let cipher = kp.derive_shared_cipher(ssss_key.pk, ssss::keypair::DEAL_SHARES_DOMAIN_SEP);
        cipher
            .encrypt_in_place(&nonce.into(), &[], &mut payload)
            .unwrap();

        let body = EncryptedPayload {
            format: EncryptedPayloadFormat::P384EcdhAes256GcmSiv {
                curve: CurveP384,
                pk: *kp.public_key(),
                nonce,
                recipient_key_id: ssss_key.key_id,
            },
            payload: payload.into(),
        };

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
        let paq =
            format!("v1/shares/{secret_name}/{chain}/{registry:x}/{identity}?version={version}");

        let res = send_request(self.make_escrin1_req(Method::POST, paq, &body, signer)?).await?;
        ensure!(res.status() == StatusCode::CREATED, "share not dealt");

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
        let paq = format!(
            "v1/shares/{secret_name}/{chain}/{registry:x}/{identity}/commit?version={version}"
        );

        send_request(self.make_escrin1_req(Method::POST, paq, &(), signer)?).await?;

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
        let paq = format!(
            "v1/shares/{secret_name}/{chain}/{registry:x}/{identity}?version={version}&pk={}",
            kp.fingerprint() // bind the requester public key to the request
        );

        let res = send_request(
            self.make_escrin1_req(Method::GET, paq, &(), signer)?
                .header(
                    RequesterPublicKeyHeader::name().as_str(),
                    RequesterPublicKeyHeader(*kp.public_key()).to_string(),
                ),
        )
        .await?;

        Ok(decrypt_enc_payload::<ShareBody>(res.json().await?, kp)?.share)
    }

    /// Returns whether the SSSS optimistically granted the permit.
    pub async fn request_acquire_identity_permit(
        &self,
        il: IdentityLocator,
        params: &AcqRelIdentityRequest,
        signer: Option<&LocalWallet>,
    ) -> Result<PermitResponse> {
        self.request_identity_permit(il, params, signer, true).await
    }

    pub async fn request_release_identity_permit(
        &self,
        il: IdentityLocator,
        params: &AcqRelIdentityRequest,
        signer: Option<&LocalWallet>,
    ) -> Result<PermitResponse> {
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
    ) -> Result<PermitResponse> {
        let paq = format!("v1/permits/{chain}/{registry:x}/{identity:x}");
        let method = if acquire {
            Method::POST
        } else {
            Method::DELETE
        };

        let req = match signer {
            Some(signer) => self.make_escrin1_req(method, paq, &params, signer)?,
            None => self
                .client
                .request(method.clone(), self.url(paq))
                .json(&params),
        };

        Ok(send_request(req).await?.json().await?)
    }

    fn make_escrin1_req(
        &self,
        method: Method,
        paq: impl AsRef<str>,
        body: &impl serde::Serialize,
        signer: &LocalWallet,
    ) -> Result<RequestBuilder> {
        let req = self.client.request(method.clone(), self.url(paq.as_ref()));
        let (body_hash, req) = if matches!(method, Method::GET | Method::HEAD) {
            (Default::default(), req)
        } else {
            let body_bytes = serde_json::to_vec(&body)?;
            (keccak256(body_bytes).into(), req.json(body))
        };
        let req721 = SsssRequest {
            method: method.to_string(),
            url: format!("{}/{}", self.url.authority(), paq.as_ref()),
            body: body_hash,
        };
        Self::attach_escrin1_sig(req, req721, signer)
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

pub fn calculate_threshold(num_sssss: u64, threshold: f64) -> u64 {
    if threshold > 1.0 {
        threshold as u64
    } else {
        (threshold * (num_sssss as f64)).ceil() as u64
    }
}

pub fn generate_signer_proof(
    signers: &[Address],
    signatories: &[Address],
) -> Result<(Vec<H256>, Vec<bool>, Vec<Address>)> {
    let tempdir = tempfile::tempdir()?;

    ensure!(
        std::process::Command::new("npm")
            .args(["install", "@openzeppelin/merkle-tree@1"])
            .current_dir(tempdir.path())
            .output()
            .wrap_err("npm install @openzeppelin/merkle-tree@1")?
            .status
            .success(),
        "`npm install @openzeppelin/merkle-tree` failed"
    );

    static PROOF_GENERATOR_CJS: &str = r#"
        const { StandardMerkleTree } = require('@openzeppelin/merkle-tree');

        let chunks = [];
        process.stdin.on('readable', () => {
          let chunk;
          while ((chunk = process.stdin.read()) !== null) chunks.push(chunk);
        });

        process.stdin.on('end', () => {
          let { signers, signatories } = JSON.parse(Buffer.concat(chunks));
          const tree = StandardMerkleTree.of(
            signers.map((s) => [s]),
            ['address'],
          );
          process.stdout.write(
            JSON.stringify(
              signatories.length > 0
                ? tree.getMultiProof(signatories.map((s) => [s]))
                : { proof: [tree.root], proofFlags: [], leaves: [] },
            ),
          );
        });
    "#;
    static GENERATE_PROOF_CJS: &str = "generate-proof.cjs";
    std::fs::write(tempdir.path().join(GENERATE_PROOF_CJS), PROOF_GENERATOR_CJS)?;
    let mut cp = std::process::Command::new("node")
        .arg(GENERATE_PROOF_CJS)
        .current_dir(tempdir.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .wrap_err("node generate-proof.cjs")?;
    cp.stdin
        .as_mut()
        .unwrap()
        .write_all(&serde_json::to_vec(&serde_json::json!({
            "signers": signers,
            "signatories": signatories,
        }))?)?;
    let output = cp.wait_with_output()?;
    ensure!(
        output.status.success(),
        "proof generation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ProofGeneratorOutput {
        proof: Vec<H256>,
        proof_flags: Vec<bool>,
        leaves: Vec<(Address,)>,
    }
    let ProofGeneratorOutput {
        proof,
        proof_flags,
        leaves,
    } = serde_json::from_slice(&output.stdout)?;
    Ok((
        proof,
        proof_flags,
        leaves.into_iter().map(|l| l.0).collect(),
    ))
}
