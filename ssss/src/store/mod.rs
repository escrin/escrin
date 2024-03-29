#[cfg(feature = "aws")]
pub mod aws;
#[cfg(feature = "azure")]
pub mod azure;
#[cfg(feature = "local")]
pub mod local;
pub mod memory;
#[cfg(test)]
mod tests;

use std::future::Future;

use axum::http::uri::Authority;
use ethers::types::Address;

use crate::types::*;

type Nonce = Vec<u8>;

pub trait Store: Clone + Send + Sync + 'static {
    fn put_share(
        &self,
        id: ShareId,
        share: SecretShare,
    ) -> impl Future<Output = Result<bool, Error>> + Send;

    fn get_share(
        &self,
        id: ShareId,
    ) -> impl Future<Output = Result<Option<SecretShare>, Error>> + Send;

    fn delete_share_version(&self, id: ShareId) -> impl Future<Output = Result<(), Error>> + Send;

    fn put_key(
        &self,
        id: KeyId,
        key: WrappedKey,
    ) -> impl Future<Output = Result<bool, Error>> + Send;

    fn get_key(&self, id: KeyId) -> impl Future<Output = Result<Option<WrappedKey>, Error>> + Send;

    fn delete_key_version(&self, id: KeyId) -> impl Future<Output = Result<(), Error>> + Send;

    fn create_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
        expiry: u64,
        nonce: Nonce,
    ) -> impl Future<Output = Result<Option<Permit>, Error>> + Send;

    fn read_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
    ) -> impl Future<Output = Result<Option<Permit>, Error>> + Send;

    fn delete_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    fn get_chain_state(
        &self,
        chain: u64,
    ) -> impl Future<Output = Result<Option<ChainState>, Error>> + Send;

    fn update_chain_state(
        &self,
        chain: u64,
        update: ChainStateUpdate,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    #[cfg(test)]
    fn clear_chain_state(&self, chain: u64) -> impl Future<Output = Result<(), Error>> + Send;

    fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> impl Future<Output = Result<Option<Vec<u8>>, Error>> + Send;

    fn update_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
        config: Vec<u8>,
        version: EventIndex,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    #[cfg(test)]
    fn clear_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> impl Future<Output = Result<(), Error>> + Send;
}

#[derive(Clone)]
pub struct DynStore {
    inner: DynStoreKind,
}

#[derive(Clone)]
pub enum DynStoreKind {
    Memory(memory::MemoryStore),
    #[cfg(feature = "aws")]
    Aws(aws::Client),
    #[cfg(feature = "azure")]
    Azure(azure::Client),
    #[cfg(feature = "local")]
    Local(local::LocalStore),
}

impl Store for DynStore {
    async fn put_share(&self, id: ShareId, share: SecretShare) -> Result<bool, Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.put_share(id, share).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.put_share(id, share).await,
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => s.put_share(id, share).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.put_share(id, share).await,
        }
    }

    async fn get_share(&self, id: ShareId) -> Result<Option<SecretShare>, Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.get_share(id).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.get_share(id).await,
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => s.get_share(id).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.get_share(id).await,
        }
    }

    async fn delete_share_version(&self, id: ShareId) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.delete_share_version(id).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.delete_share_version(id).await,
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => s.delete_share_version(id).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.delete_share_version(id).await,
        }
    }

    async fn put_key(&self, id: KeyId, key: WrappedKey) -> Result<bool, Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.put_key(id, key).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.put_key(id, key).await,
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => s.put_key(id, key).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.put_key(id, key).await,
        }
    }

    async fn get_key(&self, id: KeyId) -> Result<Option<WrappedKey>, Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.get_key(id).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.get_key(id).await,
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => s.get_key(id).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.get_key(id).await,
        }
    }

    async fn delete_key_version(&self, id: KeyId) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.delete_key_version(id).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.delete_key_version(id).await,
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => s.delete_key_version(id).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.delete_key_version(id).await,
        }
    }

    async fn create_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
        expiry: u64,
        nonce: Nonce,
    ) -> Result<Option<Permit>, Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.create_permit(identity, recipient, expiry, nonce).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.create_permit(identity, recipient, expiry, nonce).await,
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => s.create_permit(identity, recipient, expiry, nonce).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.create_permit(identity, recipient, expiry, nonce).await,
        }
    }

    async fn read_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
    ) -> Result<Option<Permit>, Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.read_permit(identity, recipient).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.read_permit(identity, recipient).await,
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => s.read_permit(identity, recipient).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.read_permit(identity, recipient).await,
        }
    }

    async fn delete_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
    ) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.delete_permit(identity, recipient).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.delete_permit(identity, recipient).await,
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => s.delete_permit(identity, recipient).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.delete_permit(identity, recipient).await,
        }
    }

    async fn get_chain_state(&self, chain: u64) -> Result<Option<ChainState>, Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.get_chain_state(chain).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.get_chain_state(chain).await,
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => s.get_chain_state(chain).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.get_chain_state(chain).await,
        }
    }

    async fn update_chain_state(&self, chain: u64, update: ChainStateUpdate) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.update_chain_state(chain, update).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.update_chain_state(chain, update).await,
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => s.update_chain_state(chain, update).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.update_chain_state(chain, update).await,
        }
    }

    #[cfg(test)]
    async fn clear_chain_state(&self, chain: u64) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.clear_chain_state(chain).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.clear_chain_state(chain).await,
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => s.clear_chain_state(chain).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.clear_chain_state(chain).await,
        }
    }

    async fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<Option<Vec<u8>>, Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.get_verifier(permitter, identity).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.get_verifier(permitter, identity).await,
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => s.get_verifier(permitter, identity).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.get_verifier(permitter, identity).await,
        }
    }

    async fn update_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
        config: Vec<u8>,
        version: EventIndex,
    ) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => {
                s.update_verifier(permitter, identity, config, version)
                    .await
            }
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => {
                s.update_verifier(permitter, identity, config, version)
                    .await
            }
            #[cfg(feature = "azure")]
            DynStoreKind::Azure(s) => {
                s.update_verifier(permitter, identity, config, version)
                    .await
            }
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => {
                s.update_verifier(permitter, identity, config, version)
                    .await
            }
        }
    }

    #[cfg(test)]
    async fn clear_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Aws(s) => s.clear_verifier(permitter, identity).await,
            DynStoreKind::Azure(s) => s.clear_verifier(permitter, identity).await,
            DynStoreKind::Memory(s) => s.clear_verifier(permitter, identity).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.clear_verifier(permitter, identity).await,
        }
    }
}

// #[derive(Debug, thiserror::Error)]
// #[error(transparent)]
// pub struct Error(#[from] anyhow::Error);
pub type Error = anyhow::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum StoreKind {
    Memory,
    #[cfg(feature = "aws")]
    Aws,
    #[cfg(feature = "local")]
    Local,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum Environment {
    Dev,
    Prod,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Dev => "dev",
                Self::Prod => "prod",
            }
        )
    }
}

#[allow(unused)]
pub async fn create(
    backend: StoreKind,
    env: Environment,
    host: &Authority,
) -> Result<impl Store, Error> {
    Ok(DynStore {
        inner: match backend {
            StoreKind::Memory => DynStoreKind::Memory(Default::default()),
            #[cfg(feature = "aws")]
            StoreKind::Aws => DynStoreKind::Aws(aws::Client::connect(env).await),
            #[cfg(feature = "azure")]
            StoreKind::Aws => {
                todo!("account");
                DynStoreKind::Azure(azure::Client::connect(host, env).await?)
            }
            #[cfg(feature = "local")]
            StoreKind::Local => todo!(),
        },
    })
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub trait ToKey {
    fn to_key(&self) -> String;
}

pub trait FromKey
where
    Self: Sized,
{
    fn from_key(key: &str) -> anyhow::Result<Self>;
}

impl ToKey for IdentityId {
    fn to_key(&self) -> String {
        format!("{:#x}", self.0)
    }
}

impl ToKey for IdentityLocator {
    fn to_key(&self) -> String {
        let Self {
            chain,
            registry,
            id: IdentityId(identity),
        } = &self;
        format!("{chain}-{registry:#x}-{identity:#x}")
    }
}

impl FromKey for IdentityLocator {
    fn from_key(key: &str) -> anyhow::Result<Self> {
        let mut parts = key.split('-');
        let Some(chain) = parts.next().and_then(|v| v.parse().ok()) else {
            anyhow::bail!("missing or invalid chain id");
        };
        let Some(registry) = parts.next().and_then(|v| v.parse().ok()) else {
            anyhow::bail!("missing or invalid registry");
        };
        let Some(id) = parts.next().and_then(|v| v.parse().ok()).map(IdentityId) else {
            anyhow::bail!("missing or invalid identity id");
        };
        anyhow::ensure!(parts.next().is_none(), "extra identity locator parts");
        Ok(Self {
            chain,
            registry,
            id,
        })
    }
}

impl ToKey for ShareId {
    fn to_key(&self) -> String {
        let Self {
            secret_name,
            identity,
            version: _,
        } = &self;
        format!("share-{secret_name}-{}", identity.to_key())
    }
}

impl ToKey for KeyId {
    fn to_key(&self) -> String {
        let Self {
            name,
            identity,
            version: _,
        } = &self;
        format!("key-{name}-{}", identity.to_key())
    }
}

impl ToKey for PermitterLocator {
    fn to_key(&self) -> String {
        let Self { chain, permitter } = &self;
        format!("{chain}-{permitter:#x}")
    }
}

impl FromKey for PermitterLocator {
    fn from_key(key: &str) -> anyhow::Result<Self> {
        let mut parts = key.split('-');
        let Some(chain) = parts.next().and_then(|v| v.parse().ok()) else {
            anyhow::bail!("missing or invalid chain id");
        };
        let Some(permitter) = parts.next().and_then(|v| v.parse().ok()) else {
            anyhow::bail!("missing or invalid permitter");
        };
        anyhow::ensure!(parts.next().is_none(), "extra permitter locator parts");
        Ok(Self { chain, permitter })
    }
}

impl ToKey for Address {
    fn to_key(&self) -> String {
        format!("{self:#x}")
    }
}

impl ToKey for ChainId {
    fn to_key(&self) -> String {
        self.to_string()
    }
}

impl ToKey for () {
    fn to_key(&self) -> String {
        Default::default()
    }
}

impl ToKey for &[u8] {
    fn to_key(&self) -> String {
        hex::encode(self)
    }
}

mod serde_key {
    use serde::{
        de::{self, Deserialize, Deserializer},
        ser::{Serialize, Serializer},
    };

    use super::{FromKey, ToKey};

    pub fn deserialize<'de, T: FromKey, D: Deserializer<'de>>(d: D) -> Result<T, D::Error> {
        let s = String::deserialize(d)?;
        T::from_key(&s).map_err(|e| de::Error::custom(e.to_string()))
    }

    pub fn serialize<T: ToKey, S: Serializer>(t: &T, s: S) -> Result<S::Ok, S::Error> {
        t.to_key().serialize(s)
    }
}
