pub mod api;

use ethers::{
    middleware::contract::{Eip712, EthAbiType},
    types::{Address, Bytes, H256, U256},
};
use serde::{Deserialize, Serialize};

pub type ChainId = u64;

pub type ShareVersion = u64;
pub type KeyVersion = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IdentityId(pub H256);

impl From<H256> for IdentityId {
    fn from(h: H256) -> Self {
        Self(h)
    }
}

impl std::fmt::Display for IdentityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[cfg(test)]
impl rand::distributions::Distribution<IdentityId> for rand::distributions::Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> IdentityId {
        IdentityId(rng.gen())
    }
}

#[cfg(test)]
impl IdentityId {
    pub fn random() -> Self {
        rand::random()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdentityLocator {
    pub chain: u64,
    pub registry: Address,
    pub id: IdentityId,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShareId {
    pub identity: IdentityLocator,
    pub secret_name: String,
    pub version: ShareVersion,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyId {
    pub name: String,
    pub identity: IdentityLocator,
    pub version: KeyVersion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermitterLocator {
    pub chain: u64,
    pub permitter: Address,
}

impl PermitterLocator {
    pub fn new(chain: u64, permitter: Address) -> Self {
        Self { chain, permitter }
    }
}

#[derive(Clone)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct SecretShare {
    pub meta: SecretShareMeta,
    pub share: zeroize::Zeroizing<Vec<u8>>,
    pub blinder: zeroize::Zeroizing<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "EncodableSecretShareMeta", into = "EncodableSecretShareMeta")]
pub struct SecretShareMeta {
    pub index: u64,
    pub commitments: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
struct EncodableSecretShareMeta {
    index: u64,
    commitments: Vec<Bytes>,
}

impl From<EncodableSecretShareMeta> for SecretShareMeta {
    fn from(m: EncodableSecretShareMeta) -> Self {
        Self {
            index: m.index,
            commitments: m.commitments.into_iter().map(|b| b.0.into()).collect(),
        }
    }
}

impl From<SecretShareMeta> for EncodableSecretShareMeta {
    fn from(m: SecretShareMeta) -> Self {
        Self {
            index: m.index,
            commitments: m.commitments.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, zeroize::Zeroize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct WrappedKey(Vec<u8>);

impl WrappedKey {
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for WrappedKey {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for WrappedKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Debug, Default, EthAbiType, Eip712)]
#[eip712(name = "SSSS", version = "1")]
pub struct SsssRequest {
    pub method: String,
    pub url: String,
    pub body: H256,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyDocument {
    pub verifier: String,
    pub policy: serde_json::Value,
}

#[derive(Clone, Debug, Default, EthAbiType, Eip712, Serialize, Deserialize)]
pub struct SsssPermit {
    pub registry: Address,
    pub identity: H256,
    pub recipient: Address,
    pub grant: bool,
    pub duration: u64,
    pub nonce: Bytes,
    pub pk: Bytes,
    pub baseblock: U256,
}
