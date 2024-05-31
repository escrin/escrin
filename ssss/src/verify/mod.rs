mod nitro;

use ethers::types::Address;

use crate::types::{IdentityLocator, PolicyDocument};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestKind {
    Grant { duration: u64 },
    Revoke,
}

pub trait Verifier {
    #[allow(clippy::too_many_arguments)]
    async fn verify(
        &self,
        raw_policy: serde_json::Value,
        req: RequestKind,
        identity: IdentityLocator,
        recipient: Address,
        authorization: &[u8],
        context: &[u8],
        relayer: Option<Address>,
    ) -> Result<Verification, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown verifier `{0}`")]
    UnknownVerifier(String),
    #[error("failed to decode policy: {0}")]
    PolicyDecode(#[source] anyhow::Error),
    #[error("failed to decode attesation document: {0}")]
    AttestationDecode(#[source] anyhow::Error),
    #[error("invalid binding provided")]
    InvalidBinding,
    #[error("binding mismatch. expected {}", hex::encode(_0))]
    BindingMismatch(smallvec::SmallVec<[u8; 32]>),
    #[error("PCR mismatch")]
    PcrMismatch(usize),
    #[error("{0}")]
    Unauthorized(String),
    #[error("timing error: {0}")]
    Timing(String),
}

#[derive(Clone, Debug)]
pub struct Verification {
    pub nonce: Vec<u8>,
    pub public_key: Vec<u8>,
    pub duration: Option<u64>,
}

pub async fn verify(
    policy_bytes: &[u8],
    req: RequestKind,
    identity: IdentityLocator,
    recipient: Address,
    auth: &[u8],
    ctx: &[u8],
    relayer: Option<Address>,
) -> Result<Verification, Error> {
    let PolicyDocument {
        verifier,
        policy: raw_policy,
    } = serde_json::from_slice(policy_bytes).map_err(|e| Error::PolicyDecode(e.into()))?;

    match verifier.as_str() {
        "nitro" => {
            nitro::NitroEnclaveVerifier
                .verify(raw_policy, req, identity, recipient, auth, ctx, relayer)
                .await
        }
        #[cfg(debug_assertions)]
        "mock" => Ok(Verification {
            nonce: ethers::utils::keccak256(ethers::types::H256::from_low_u64_ne(
                (ssss::utils::now() >> 1) << 1, // permit up to 2s drift
            ))
            .into(),
            public_key: vec![],
            duration: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + 60,
            ),
        }),
        sel => Err(Error::UnknownVerifier(sel.into())),
    }
}
