#[cfg(feature = "aws")]
pub mod aws;
pub mod local;
pub mod memory;

use std::future::Future;

use ethers::types::Address;

use crate::{cli, types::*};

const SHARE_SIZE: usize = 32;

type Nonce = Vec<u8>;

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
        nonce: Nonce,
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
    async fn clear_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<(), Error>;
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
        nonce: Nonce,
    ) -> Result<Option<Permit>, Error> {
        match &self.inner {
            DynStoreKind::Aws(s) => s.create_permit(share, recipient, expiry, nonce).await,
            DynStoreKind::Memory(s) => s.create_permit(share, recipient, expiry, nonce).await,
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

    async fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<Option<Vec<u8>>, Error> {
        match &self.inner {
            DynStoreKind::Aws(s) => s.get_verifier(permitter, identity).await,
            DynStoreKind::Memory(s) => s.get_verifier(permitter, identity).await,
            DynStoreKind::Local => todo!(),
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
            DynStoreKind::Aws(s) => {
                s.update_verifier(permitter, identity, config, version)
                    .await
            }
            DynStoreKind::Memory(s) => {
                s.update_verifier(permitter, identity, config, version)
                    .await
            }
            DynStoreKind::Local => todo!(),
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
    macro_rules! make_store_tests {
        ($store:expr) => {
            $crate::make_store_tests!(
                $store,
                roundtrip_share,
                create_second_version,
                create_second_share,
                roundtrip_permit,
                refresh_permit,
                expired_permit,
                used_nonce_permit,
                defresh_permit_fail,
                delete_defresh_permit,
                roundtrip_chain_state,
                roundtrip_verifier,
            );
        };
        ($store:expr, $($test:ident),+ $(,)?) => {
            $(
                #[tokio::test]
                async fn $test() {
                    $crate::store::tests::$test($store).await;
                }
            )+
        }
    }

    async fn with_share<'a, S: Store, Fut, T>(
        store: &'a S,
        identity: IdentityId,
        f: impl FnOnce(&'a S, ShareId) -> Fut,
    ) -> T
    where
        Fut: futures::Future<Output = T> + 'a,
    {
        let share_id = store
            .create_share(IdentityLocator {
                chain: 31337,
                registry: Address::repeat_byte(1),
                id: identity,
            })
            .await
            .unwrap();
        let res = f(store, share_id).await;
        store.destroy_share(share_id).await.unwrap();
        res
    }

    pub async fn roundtrip_share(store: impl Store) {
        let identity = IdentityId::random();
        with_share(&store, identity, |store, share_id| async move {
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
            let share = store.get_share(share_id).await?;
            let share2 = store.get_share(share_id).await?;
            ensure!(share == share2, "retrieved shares mismatched");
            Ok(())
        })
        .await
        .unwrap();
    }

    pub async fn create_second_version(store: impl Store) {
        let identity = IdentityId::random();
        with_share(&store, identity, |store, share_id1| async move {
            let share1_1 = store.get_share(share_id1).await?;
            with_share(store, identity, |store, share_id2| async move {
                ensure!(
                    share_id1.identity == share_id2.identity,
                    "share identity changed"
                );
                ensure!(share_id2.version == 2, "share version did not increment");
                let share1_2 = store.get_share(share_id1).await?;
                let share2 = store.get_share(share_id2).await?;
                ensure!(share1_1 == share1_2, "share changed");
                ensure!(share1_1 != share2, "wrong share returned");
                Ok(())
            })
            .await
        })
        .await
        .unwrap();
    }

    pub async fn create_second_share(store: impl Store) {
        let identity1 = IdentityId::random();
        let identity2 = IdentityId::random();
        with_share(&store, identity1, |store, share_id1| async move {
            let share1 = store.get_share(share_id1).await?;
            with_share(store, identity2, |store, share_id2| async move {
                let share2 = store.get_share(share_id2).await?;
                ensure!(share1 != share2, "wrong share returned");
                Ok(())
            })
            .await
        })
        .await
        .unwrap();
    }

    async fn with_permit<'a, S: Store, Fut, T>(
        store: &'a S,
        share: ShareId,
        recipient: Address,
        expiry: u64,
        f: impl FnOnce(&'a S, Permit) -> Fut,
    ) -> Option<T>
    where
        Fut: futures::Future<Output = T> + 'a,
    {
        let mut nonce = vec![0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce);
        with_permit_nonce(store, share, recipient, expiry, f, nonce).await
    }

    async fn with_permit_nonce<'a, S: Store, Fut, T>(
        store: &'a S,
        share: ShareId,
        recipient: Address,
        expiry: u64,
        f: impl FnOnce(&'a S, Permit) -> Fut,
        nonce: Vec<u8>,
    ) -> Option<T>
    where
        Fut: futures::Future<Output = T> + 'a,
    {
        let permit = store
            .create_permit(share, recipient, expiry, nonce)
            .await
            .unwrap();
        match permit {
            Some(p) => {
                let res = f(store, p).await;
                store.delete_permit(share, recipient).await.unwrap();
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

    pub async fn roundtrip_permit(store: impl Store) {
        let share = mock_share();
        let recipient = Address::random();
        let expiry = now() + 20;
        with_permit(
            &store,
            share,
            recipient,
            expiry,
            |store, permit| async move {
                let read_permit = store.read_permit(share, recipient).await?;
                ensure!(read_permit.is_some(), "permit not created");
                ensure!(read_permit.unwrap() == permit, "permit mismatch");
                Ok(())
            },
        )
        .await
        .unwrap()
        .unwrap();
    }

    pub async fn expired_permit(store: impl Store) {
        let share = mock_share();
        let recipient = Address::random();
        let expiry = now() - 60;
        with_permit(&store, share, recipient, expiry, |store, _| async move {
            let read_permit = store.read_permit(share, recipient).await?;
            ensure!(read_permit.is_none(), "permit not expired");
            Ok(())
        })
        .await
        .unwrap()
        .unwrap();
    }

    pub async fn used_nonce_permit(store: impl Store) {
        let share = mock_share();
        let recipient = Address::random();
        let expiry = now() + 5;
        let mut nonce = vec![0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce);

        with_permit_nonce(
            &store,
            share,
            recipient,
            expiry,
            |_, _| async {},
            nonce.clone(),
        )
        .await
        .unwrap();

        assert!(
            with_permit_nonce(&store, share, recipient, expiry, |_, _| async {}, nonce)
                .await
                .is_none()
        );
    }

    pub async fn refresh_permit(store: impl Store) {
        let share = mock_share();
        let recipient = Address::random();
        let expiry_soon = now() + 5;
        let expiry_far = now() + 10;
        with_permit(
            &store,
            share,
            recipient,
            expiry_soon,
            |store, _| async move {
                with_permit(store, share, recipient, expiry_far, |store, _| async move {
                    let read_permit = store.read_permit(share, recipient).await?;
                    ensure!(read_permit.is_some(), "permit not created");
                    ensure!(
                        read_permit.unwrap().expiry == expiry_far,
                        "permit exiry not refrshed"
                    );
                    Ok(())
                })
                .await
            },
        )
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    }

    pub async fn defresh_permit_fail(store: impl Store) {
        let share = mock_share();
        let recipient = Address::random();
        let expiry_soon = now() + 5;
        let expiry_far = now() + 10;
        with_permit(
            &store,
            share,
            recipient,
            expiry_far,
            |store, _| async move {
                let outcome =
                    with_permit(store, share, recipient, expiry_soon, |_, _| async move {}).await;
                ensure!(outcome.is_none(), "permit wrongly defreshed");
                Ok(())
            },
        )
        .await
        .unwrap()
        .unwrap();
    }

    pub async fn delete_defresh_permit(store: impl Store) {
        let share = mock_share();
        let recipient = Address::random();
        let expiry_soon = now() + 5;
        let expiry_far = now() + 10;
        with_permit(
            &store,
            share,
            recipient,
            expiry_far,
            |store, _| async move {
                store.delete_permit(share, recipient).await?;
                let outcome = with_permit(
                    store,
                    share,
                    recipient,
                    expiry_soon,
                    |store, permit| async move {
                        let read_permit = store.read_permit(share, recipient).await?;
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

    pub async fn roundtrip_chain_state(store: impl Store) {
        let chain_id = (u32::max_value() as u64)
            .checked_add(rand::random())
            .unwrap();
        let start_state = store.get_chain_state(chain_id).await.unwrap();
        assert!(start_state.is_none());

        store
            .update_chain_state(chain_id, ChainStateUpdate { block: Some(42) })
            .await
            .unwrap();
        let updated_state = store.get_chain_state(chain_id).await.unwrap();
        assert_eq!(updated_state, Some(ChainState { block: 42 }));

        store
            .update_chain_state(chain_id, ChainStateUpdate { block: Some(41) })
            .await
            .unwrap();
        let not_updated_state = store.get_chain_state(chain_id).await.unwrap();
        assert_eq!(not_updated_state, Some(ChainState { block: 42 }));

        store
            .update_chain_state(chain_id, ChainStateUpdate { block: None })
            .await
            .unwrap();
        let not_updated_state = store.get_chain_state(chain_id).await.unwrap();
        assert_eq!(not_updated_state, Some(ChainState { block: 42 }));

        store
            .update_chain_state(chain_id, ChainStateUpdate { block: Some(43) })
            .await
            .unwrap();
        let re_updated_state = store.get_chain_state(chain_id).await.unwrap();
        assert_eq!(re_updated_state, Some(ChainState { block: 43 }));

        store.clear_chain_state(chain_id).await.unwrap();
    }

    pub async fn roundtrip_verifier(store: impl Store) {
        let config1 = b"config1".as_slice();
        let config2 = b"config2".as_slice();
        let config3 = b"config3".as_slice();
        let config4 = b"config4".as_slice();

        let chain1 = (u32::max_value() as u64)
            .checked_add(rand::random())
            .unwrap();
        let chain2 = (u32::max_value() as u64)
            .checked_add(rand::random())
            .unwrap();

        let chain1_permitter1 = PermitterLocator {
            chain: chain1,
            permitter: rand::random(),
        };
        let chain1_permitter2 = PermitterLocator {
            chain: chain1,
            permitter: rand::random(),
        };
        let chain2_permitter1 = PermitterLocator {
            chain: chain2,
            permitter: chain1_permitter2.permitter,
        };

        let identity1 = rand::random();
        let identity2 = rand::random();

        store
            .update_verifier(
                chain2_permitter1,
                identity1,
                config4.to_vec(),
                EventIndex {
                    block: 1,
                    log_index: 1,
                },
            )
            .await
            .unwrap();
        let updated_state = store
            .get_verifier(chain2_permitter1, identity1)
            .await
            .unwrap();
        assert_eq!(updated_state.as_deref(), Some(config4));

        // Assert that identities on the same permitter do not interfere.
        let start_config = store
            .get_verifier(chain2_permitter1, identity2)
            .await
            .unwrap();
        assert!(start_config.is_none());

        // Assert that different permitters do not interfere
        let start_config = store
            .get_verifier(chain1_permitter1, identity1)
            .await
            .unwrap();
        assert!(start_config.is_none());

        // Assert no rollbacks
        store
            .update_verifier(
                chain1_permitter1,
                identity1,
                config1.to_vec(),
                EventIndex {
                    block: 1,
                    log_index: 1,
                },
            )
            .await
            .unwrap();
        let updated_state = store
            .get_verifier(chain1_permitter1, identity1)
            .await
            .unwrap();
        assert_eq!(updated_state.as_deref(), Some(config1));
        store
            .update_verifier(
                chain1_permitter1,
                identity1,
                config2.to_vec(),
                EventIndex {
                    block: 1,
                    log_index: 1,
                },
            )
            .await
            .unwrap();
        let updated_state = store
            .get_verifier(chain1_permitter1, identity1)
            .await
            .unwrap();
        assert_eq!(updated_state.as_deref(), Some(config1));

        store
            .update_verifier(
                chain1_permitter1,
                identity1,
                config2.to_vec(),
                EventIndex {
                    block: 1,
                    log_index: 2,
                },
            )
            .await
            .unwrap();
        let updated_state = store
            .get_verifier(chain1_permitter1, identity1)
            .await
            .unwrap();
        assert_eq!(updated_state.as_deref(), Some(config2));

        store
            .update_verifier(
                chain1_permitter1,
                identity1,
                config3.to_vec(),
                EventIndex {
                    block: 2,
                    log_index: 0,
                },
            )
            .await
            .unwrap();
        let updated_state = store
            .get_verifier(chain1_permitter1, identity1)
            .await
            .unwrap();
        assert_eq!(updated_state.as_deref(), Some(config3));

        for permitter in [chain1_permitter1, chain1_permitter2, chain2_permitter1] {
            for identity in [identity1, identity2] {
                store.clear_verifier(permitter, identity).await.unwrap();
            }
        }
    }
}
