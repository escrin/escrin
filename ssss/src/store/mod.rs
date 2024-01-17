#[cfg(feature = "aws")]
pub mod aws;
pub mod local;
pub mod memory;

use std::future::Future;

use ethers::types::{Address, H256};

use crate::cli;

pub type IdentityId = H256;
pub type ShareVersion = u64;

const SHARE_SIZE: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IdentityLocator {
    chain: u64,
    registry: Address,
    id: IdentityId,
}

impl IdentityLocator {
    pub fn to_key(self) -> String {
        let Self {
            chain,
            registry,
            id: identity,
        } = &self;
        format!("{chain}-{registry:#x}-{identity:#x}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShareId {
    identity: IdentityLocator,
    version: ShareVersion,
}

impl ShareId {
    pub fn to_key(self) -> String {
        let Self { identity, version } = &self;
        format!("{}-{version}", identity.to_key())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
pub struct WrappedShare(Vec<u8>);

pub trait Store: Clone + Send + Sync {
    async fn create_share(&self, identity: IdentityLocator) -> Result<ShareId, Error>;

    async fn get_share(&self, share: ShareId) -> Result<Option<WrappedShare>, Error>;

    #[cfg(test)]
    async fn destroy_share(&self, share: ShareId) -> Result<(), Error>;

    async fn create_permit(
        &self,
        share: ShareId,
        recipient: Address,
        expiry: u64,
    ) -> Result<Option<Permit>, Error>;

    async fn read_permit(
        &self,
        share: ShareId,
        recipient: Address,
    ) -> Result<Option<Permit>, Error>;

    async fn delete_permit(&self, share: ShareId, recipient: Address) -> Result<(), Error>;

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
    async fn clear_chain_state(&self, chain: u64) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct DynStore {
    inner: DynStoreKind,
}

#[derive(Clone)]
pub enum DynStoreKind {
    #[cfg(feature = "aws")]
    Aws(aws::Client),
    Memory(memory::MemoryStore),
    #[allow(unused)]
    Local,
}

impl Store for DynStore {
    async fn create_share(&self, identity: IdentityLocator) -> Result<ShareId, Error> {
        match &self.inner {
            DynStoreKind::Aws(s) => s.create_share(identity).await,
            DynStoreKind::Memory(s) => s.create_share(identity).await,
            DynStoreKind::Local => todo!(),
        }
    }

    async fn get_share(&self, share: ShareId) -> Result<Option<WrappedShare>, Error> {
        match &self.inner {
            DynStoreKind::Aws(s) => s.get_share(share).await,
            DynStoreKind::Memory(s) => s.get_share(share).await,
            DynStoreKind::Local => todo!(),
        }
    }

    #[cfg(test)]
    async fn destroy_share(&self, share: ShareId) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Aws(s) => s.destroy_share(share).await,
            DynStoreKind::Memory(s) => s.destroy_share(share).await,
            DynStoreKind::Local => todo!(),
        }
    }

    async fn create_permit(
        &self,
        share: ShareId,
        recipient: Address,
        expiry: u64,
    ) -> Result<Option<Permit>, Error> {
        match &self.inner {
            DynStoreKind::Aws(s) => s.create_permit(share, recipient, expiry).await,
            DynStoreKind::Memory(s) => s.create_permit(share, recipient, expiry).await,
            DynStoreKind::Local => todo!(),
        }
    }

    async fn read_permit(
        &self,
        share: ShareId,
        recipient: Address,
    ) -> Result<Option<Permit>, Error> {
        match &self.inner {
            DynStoreKind::Aws(s) => s.read_permit(share, recipient).await,
            DynStoreKind::Memory(s) => s.read_permit(share, recipient).await,
            DynStoreKind::Local => todo!(),
        }
    }

    async fn delete_permit(&self, share: ShareId, recipient: Address) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Aws(s) => s.delete_permit(share, recipient).await,
            DynStoreKind::Memory(ss) => ss.delete_permit(share, recipient).await,
            DynStoreKind::Local => todo!(),
        }
    }

    async fn get_chain_state(&self, chain: u64) -> Result<Option<ChainState>, Error> {
        match &self.inner {
            DynStoreKind::Aws(s) => s.get_chain_state(chain).await,
            DynStoreKind::Memory(s) => s.get_chain_state(chain).await,
            DynStoreKind::Local => todo!(),
        }
    }

    async fn update_chain_state(&self, chain: u64, update: ChainStateUpdate) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Aws(s) => s.update_chain_state(chain, update).await,
            DynStoreKind::Memory(s) => s.update_chain_state(chain, update).await,
            DynStoreKind::Local => todo!(),
        }
    }

    #[cfg(test)]
    async fn clear_chain_state(&self, chain: u64) -> Result<(), Error> {
        match &self.inner {
            DynStoreKind::Aws(s) => s.clear_chain_state(chain).await,
            DynStoreKind::Memory(s) => s.clear_chain_state(chain).await,
            DynStoreKind::Local => todo!(),
        }
    }
}

// #[derive(Debug, thiserror::Error)]
// #[error(transparent)]
// pub struct Error(#[from] anyhow::Error);
pub type Error = anyhow::Error;

pub async fn create(backend: cli::Store, env: cli::Environment) -> impl Store {
    DynStore {
        inner: match backend {
            #[cfg(feature = "aws")]
            crate::cli::Store::Aws => DynStoreKind::Aws(aws::Client::connect(env).await),
            crate::cli::Store::Memory => DynStoreKind::Memory(Default::default()),
            crate::cli::Store::Local => todo!(),
        },
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use anyhow::ensure;

    use super::*;

    #[macro_export]
    macro_rules! make_sstore_tests {
        ($sstore:expr) => {
            $crate::make_sstore_tests!(
                $sstore,
                roundtrip_share,
                create_second_version,
                create_second_share,
                roundtrip_permit,
                refresh_permit,
                expired_permit,
                defresh_permit_fail,
                delete_defresh_permit,
                roundtrip_chain_state,
            );
        };
        ($sstore:expr, $($test:ident),+ $(,)?) => {
            $(
                #[tokio::test]
                async fn $test() {
                    $crate::store::tests::$test($sstore).await;
                }
            )+
        }
    }

    async fn with_share<'a, S: Store, Fut, T>(
        sstore: &'a S,
        identity: IdentityId,
        f: impl FnOnce(&'a S, ShareId) -> Fut,
    ) -> T
    where
        Fut: futures::Future<Output = T> + 'a,
    {
        let share_id = sstore
            .create_share(IdentityLocator {
                chain: 31337,
                registry: Address::repeat_byte(1),
                id: identity,
            })
            .await
            .unwrap();
        let res = f(sstore, share_id).await;
        sstore.destroy_share(share_id).await.unwrap();
        res
    }

    pub async fn roundtrip_share(sstore: impl Store) {
        let identity = IdentityId::random();
        with_share(&sstore, identity, |sstore, share_id| async move {
            ensure!(share_id.identity.chain == 31337, "unexpected share chain");
            ensure!(
                share_id.identity.registry == Address::repeat_byte(1),
                "unexpected share registry"
            );
            ensure!(
                share_id.identity.id == identity,
                "unexpected share identity"
            );
            ensure!(share_id.version == 1, "unexpected share version");
            let share = sstore.get_share(share_id).await?;
            let share2 = sstore.get_share(share_id).await?;
            ensure!(share == share2, "retrieved shares mismatched");
            Ok(())
        })
        .await
        .unwrap();
    }

    pub async fn create_second_version(sstore: impl Store) {
        let identity = IdentityId::random();
        with_share(&sstore, identity, |sstore, share_id1| async move {
            let share1_1 = sstore.get_share(share_id1).await?;
            with_share(sstore, identity, |sstore, share_id2| async move {
                ensure!(
                    share_id1.identity == share_id2.identity,
                    "share identity changed"
                );
                ensure!(share_id2.version == 2, "share version did not increment");
                let share1_2 = sstore.get_share(share_id1).await?;
                let share2 = sstore.get_share(share_id2).await?;
                ensure!(share1_1 == share1_2, "share changed");
                ensure!(share1_1 != share2, "wrong share returned");
                Ok(())
            })
            .await
        })
        .await
        .unwrap();
    }

    pub async fn create_second_share(sstore: impl Store) {
        let identity1 = IdentityId::random();
        let identity2 = IdentityId::random();
        with_share(&sstore, identity1, |sstore, share_id1| async move {
            let share1 = sstore.get_share(share_id1).await?;
            with_share(sstore, identity2, |sstore, share_id2| async move {
                let share2 = sstore.get_share(share_id2).await?;
                ensure!(share1 != share2, "wrong share returned");
                Ok(())
            })
            .await
        })
        .await
        .unwrap();
    }

    async fn with_permit<'a, S: Store, Fut, T>(
        sstore: &'a S,
        share: ShareId,
        recipient: Address,
        expiry: u64,
        f: impl FnOnce(&'a S, Permit) -> Fut,
    ) -> Option<T>
    where
        Fut: futures::Future<Output = T> + 'a,
    {
        let permit = sstore
            .create_permit(share, recipient, expiry)
            .await
            .unwrap();
        match permit {
            Some(p) => {
                let res = f(sstore, p).await;
                sstore.delete_permit(share, recipient).await.unwrap();
                Some(res)
            }
            None => None,
        }
    }

    fn mock_share() -> ShareId {
        ShareId {
            identity: IdentityLocator {
                chain: 31337,
                registry: Address::repeat_byte(42),
                id: IdentityId::random(),
            },
            version: 1,
        }
    }

    pub async fn roundtrip_permit(sstore: impl Store) {
        let share = mock_share();
        let recipient = Address::random();
        let expiry = now() + 240;
        with_permit(
            &sstore,
            share,
            recipient,
            expiry,
            |sstore, permit| async move {
                let read_permit = sstore.read_permit(share, recipient).await?;
                ensure!(read_permit.is_some(), "permit not created");
                ensure!(read_permit.unwrap() == permit, "permit mismatch");
                Ok(())
            },
        )
        .await
        .unwrap()
        .unwrap();
    }

    pub async fn expired_permit(sstore: impl Store) {
        let share = mock_share();
        let recipient = Address::random();
        let expiry = now() - 60;
        with_permit(&sstore, share, recipient, expiry, |sstore, _| async move {
            let read_permit = sstore.read_permit(share, recipient).await?;
            ensure!(read_permit.is_none(), "permit not expired");
            Ok(())
        })
        .await
        .unwrap()
        .unwrap();
    }

    pub async fn refresh_permit(sstore: impl Store) {
        let share = mock_share();
        let recipient = Address::random();
        let expiry_soon = now() + 60;
        let expiry_far = now() + 120;
        with_permit(
            &sstore,
            share,
            recipient,
            expiry_soon,
            |sstore, _| async move {
                with_permit(
                    sstore,
                    share,
                    recipient,
                    expiry_far,
                    |sstore, _| async move {
                        let read_permit = sstore.read_permit(share, recipient).await?;
                        ensure!(read_permit.is_some(), "permit not created");
                        ensure!(
                            read_permit.unwrap().expiry == expiry_far,
                            "permit exiry not refrshed"
                        );
                        Ok(())
                    },
                )
                .await
            },
        )
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    }

    pub async fn defresh_permit_fail(sstore: impl Store) {
        let share = mock_share();
        let recipient = Address::random();
        let expiry_soon = now() + 60;
        let expiry_far = now() + 120;
        with_permit(
            &sstore,
            share,
            recipient,
            expiry_far,
            |sstore, _| async move {
                let outcome =
                    with_permit(sstore, share, recipient, expiry_soon, |_, _| async move {}).await;
                ensure!(outcome.is_none(), "permit wrongly defreshed");
                Ok(())
            },
        )
        .await
        .unwrap()
        .unwrap();
    }

    pub async fn delete_defresh_permit(sstore: impl Store) {
        let share = mock_share();
        let recipient = Address::random();
        let expiry_soon = now() + 60;
        let expiry_far = now() + 120;
        with_permit(
            &sstore,
            share,
            recipient,
            expiry_far,
            |sstore, _| async move {
                sstore.delete_permit(share, recipient).await?;
                let outcome = with_permit(
                    sstore,
                    share,
                    recipient,
                    expiry_soon,
                    |sstore, permit| async move {
                        let read_permit = sstore.read_permit(share, recipient).await?;
                        ensure!(read_permit.is_some(), "permit not re-created");
                        ensure!(read_permit.unwrap() == permit, "permit mismatch");
                        Ok(())
                    },
                )
                .await;
                ensure!(outcome.is_some(), "permit wrongly defreshed");
                Ok(())
            },
        )
        .await
        .unwrap()
        .unwrap();
    }

    pub async fn roundtrip_chain_state(sstore: impl Store) {
        let chain_id = (u32::max_value() as u64)
            .checked_add(rand::random())
            .unwrap();
        let start_state = sstore.get_chain_state(chain_id).await.unwrap();
        assert!(start_state.is_none());

        sstore
            .update_chain_state(chain_id, ChainStateUpdate { block: Some(42) })
            .await
            .unwrap();
        let updated_state = sstore.get_chain_state(chain_id).await.unwrap();
        assert_eq!(updated_state, Some(ChainState { block: 42 }));

        sstore
            .update_chain_state(chain_id, ChainStateUpdate { block: Some(41) })
            .await
            .unwrap();
        let not_updated_state = sstore.get_chain_state(chain_id).await.unwrap();
        assert_eq!(not_updated_state, Some(ChainState { block: 42 }));

        sstore
            .update_chain_state(chain_id, ChainStateUpdate { block: None })
            .await
            .unwrap();
        let not_updated_state = sstore.get_chain_state(chain_id).await.unwrap();
        assert_eq!(not_updated_state, Some(ChainState { block: 42 }));

        sstore
            .update_chain_state(chain_id, ChainStateUpdate { block: Some(43) })
            .await
            .unwrap();
        let re_updated_state = sstore.get_chain_state(chain_id).await.unwrap();
        assert_eq!(re_updated_state, Some(ChainState { block: 43 }));

        sstore.clear_chain_state(chain_id).await.unwrap();
    }
}
