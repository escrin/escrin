#[cfg(feature = "aws")]
mod nitro;

pub use nitro::NitroEnclaveVerifier;

use crate::sync::PermitRequestEvent;

pub trait Verifier {
    async fn verify(&self, req: PermitRequestEvent, policy_bytes: &[u8]) -> Option<Verification>;
}

#[derive(Clone, Debug)]
pub struct Verification {
    pub nonce: Vec<u8>,
    pub public_key: Vec<u8>,
}
