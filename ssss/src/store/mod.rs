#[cfg(feature = "aws")]
pub mod aws;
#[cfg(feature = "local")]
pub mod local;
pub mod memory;
#[cfg(test)]
mod tests;

use std::future::Future;

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

    fn delete_share(&self, id: ShareId) -> impl Future<Output = Result<(), Error>> + Send;

    fn put_key(
        &self,
        id: KeyId,
        key: WrappedKey,
    ) -> impl Future<Output = Result<bool, Error>> + Send;

    fn get_key(&self, id: KeyId) -> impl Future<Output = Result<Option<WrappedKey>, Error>> + Send;

    fn delete_key(&self, id: KeyId) -> impl Future<Output = Result<(), Error>> + Send;

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
    #[cfg(feature = "local")]
    Local(local::LocalStore),
}

impl Store for DynStore {
    async fn put_share(&self, id: ShareId, share: SecretShare) -> Result<bool, Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.put_share(id, share).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.put_share(id, share).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.put_share(id, share).await,
        }
    }

    async fn get_share(&self, id: ShareId) -> Result<Option<SecretShare>, Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.get_share(id).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.get_share(id).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.get_share(id).await,
        }
    }

    async fn delete_share(&self, id: ShareId) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.delete_share(id).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.delete_share(id).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.delete_share(id).await,
        }
    }

    async fn put_key(&self, id: KeyId, key: WrappedKey) -> Result<bool, Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.put_key(id, key).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.put_key(id, key).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.put_key(id, key).await,
        }
    }

    async fn get_key(&self, id: KeyId) -> Result<Option<WrappedKey>, Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.get_key(id).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.get_key(id).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.get_key(id).await,
        }
    }

    async fn delete_key(&self, id: KeyId) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.delete_key(id).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.delete_key(id).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.delete_key(id).await,
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
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.delete_permit(identity, recipient).await,
        }
    }

    async fn get_chain_state(&self, chain: u64) -> Result<Option<ChainState>, Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.get_chain_state(chain).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.get_chain_state(chain).await,
            #[cfg(feature = "local")]
            DynStoreKind::Local(s) => s.get_chain_state(chain).await,
        }
    }

    async fn update_chain_state(&self, chain: u64, update: ChainStateUpdate) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Memory(s) => s.update_chain_state(chain, update).await,
            #[cfg(feature = "aws")]
            DynStoreKind::Aws(s) => s.update_chain_state(chain, update).await,
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

#[cfg_attr(not(feature = "aws"), allow(unused))]
pub async fn create(backend: StoreKind, env: Environment) -> impl Store {
    DynStore {
        inner: match backend {
            StoreKind::Memory => DynStoreKind::Memory(Default::default()),
            #[cfg(feature = "aws")]
            StoreKind::Aws => DynStoreKind::Aws(aws::Client::connect(env).await),
            #[cfg(feature = "local")]
            StoreKind::Local => todo!(),
        },
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
