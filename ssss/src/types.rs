use ethers::types::{Address, H256};
use serde::{Deserialize, Serialize};

pub trait ToKey: Copy {
    fn to_key(self) -> String;
}

pub type ChainId = u64;

pub type ShareVersion = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IdentityId(pub H256);

impl From<H256> for IdentityId {
    fn from(h: H256) -> Self {
        Self(h)
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

impl ToKey for IdentityId {
    fn to_key(self) -> String {
        format!("{:#x}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdentityLocator {
    pub chain: u64,
    pub registry: Address,
    pub id: IdentityId,
}

impl ToKey for IdentityLocator {
    fn to_key(self) -> String {
        let Self {
            chain,
            registry,
            id: IdentityId(identity),
        } = &self;
        format!("{chain}-{registry:#x}-{identity:#x}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShareId {
    pub identity: IdentityLocator,
    pub version: ShareVersion,
}

impl ToKey for ShareId {
    fn to_key(self) -> String {
        let Self { identity, version } = &self;
        format!("{}-{version}", identity.to_key())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PermitterLocator {
    pub chain: u64,
    pub permitter: Address,
}

impl PermitterLocator {
    pub fn new(chain: u64, permitter: Address) -> Self {
        Self { chain, permitter }
    }
}

impl ToKey for PermitterLocator {
    fn to_key(self) -> String {
        let Self { chain, permitter } = &self;
        format!("{chain}-{permitter:#x}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permit {
    pub expiry: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChainState {
    pub block: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChainStateUpdate {
    pub block: Option<u64>,
}

#[derive(zeroize::Zeroize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct SecretShare(Vec<u8>);

impl SecretShare {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl From<Vec<u8>> for SecretShare {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventIndex {
    pub block: u64,
    pub log_index: u64,
}
