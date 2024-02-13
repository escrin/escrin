#[cfg(feature = "aws")]
mod nitro;

use ethers::types::Address;

use crate::types::IdentityLocator;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestKind {
    Grant { duration: u64 },
    Revoke,
}

pub trait Verifier {
    async fn verify(
        &self,
        policy_bytes: &[u8],
        req: RequestKind,
        identity: IdentityLocator,
        recipient: Address,
        authorization: &[u8],
        context: &[u8],
    ) -> Result<Verification, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown verifier `{0}` requested")]
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
}

#[derive(Clone, Debug)]
pub struct Verification {
    pub nonce: Vec<u8>,
    pub public_key: Vec<u8>,
    pub expiry: Option<u64>,
}

pub async fn verify(
    policy_bytes: &[u8],
    req: RequestKind,
    identity: IdentityLocator,
    recipient: Address,
    auth: &[u8],
    ctx: &[u8],
) -> Result<Verification, Error> {
    let (verifier_sel, _): (String, Vec<u8>) = ethers::abi::AbiDecode::decode(ctx)
        .map_err(|_| Error::UnknownVerifier("<unparseable>".into()))?;
    match verifier_sel.as_str() {
        "nitro" => {
            nitro::NitroEnclaveVerifier.verify(policy_bytes, req, identity, recipient, auth, ctx)
        }
        sel => return Err(Error::UnknownVerifier(sel.into())),
    }
    .await
}
