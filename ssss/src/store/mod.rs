#[cfg(feature = "aws")]
pub mod aws;
#[cfg(feature = "local")]
pub mod local;
pub mod memory;

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
                create_second_share_version,
                create_duplicate_share_version,
                create_discontinuous_share_version,
                create_second_share,
                roundtrip_key,
                create_second_key_version,
                create_duplicate_key_version,
                create_discontinuous_key_version,
                create_second_key,
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
        version: ShareVersion,
        f: impl FnOnce(&'a S, ShareId) -> Fut,
    ) -> Result<T, Error>
    where
        Fut: futures::Future<Output = T> + 'a,
    {
        let share_id = ShareId {
            identity: IdentityLocator {
                chain: 31337,
                registry: Address::repeat_byte(1),
                id: identity,
            },
            version,
        };
        let mut share = vec![0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut share);
        let created = store
            .put_share(
                share_id,
                SecretShare {
                    index: 1,
                    share: share.into(),
                    blinding: Default::default()
                },
            )
            .await?;
        ensure!(
            created,
            "share not created due to duplicate or non-contiguous version"
        );
        let res = f(store, share_id).await;
        store.delete_share(share_id).await?;
        Ok(res)
    }

    pub async fn roundtrip_share(store: impl Store) {
        let identity = IdentityId::random();
        with_share(&store, identity, 1, |store, share_id| async move {
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
        .expect("test failed")
        .expect("share creation failed");
    }

    pub async fn create_second_share_version(store: impl Store) {
        let identity = IdentityId::random();
        with_share(&store, identity, 1, |store, share_id1| async move {
            let share1_1 = store.get_share(share_id1).await?;
            with_share(store, identity, 2, |store, share_id2| async move {
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
        .expect("test failed")
        .expect("second share creation failed")
        .expect("first share creation failed");
    }

    pub async fn create_duplicate_share_version(store: impl Store) {
        let identity = IdentityId::random();
        with_share(&store, identity, 1, |store, _| async move {
            ensure!(
                with_share(store, identity, 1, |_, _| async move {})
                    .await
                    .is_err(),
                "duplicate share version wrongly created"
            );
            Ok(())
        })
        .await
        .expect("test failed")
        .expect("first share creation failed");
    }

    pub async fn create_discontinuous_share_version(store: impl Store) {
        let identity = IdentityId::random();
        with_share(&store, identity, 1, |store, _| async move {
            ensure!(
                with_share(store, identity, 3, |_, _| async move {})
                    .await
                    .is_err(),
                "discontinuous share version wrongly created"
            );
            Ok(())
        })
        .await
        .expect("test failed")
        .expect("first share creation failed");
    }

    pub async fn create_second_share(store: impl Store) {
        let identity1 = IdentityId::random();
        let identity2 = IdentityId::random();
        with_share(&store, identity1, 1, |store, share_id1| async move {
            let share1 = store.get_share(share_id1).await?;
            with_share(store, identity2, 1, |store, share_id2| async move {
                let share2 = store.get_share(share_id2).await?;
                ensure!(share1 != share2, "wrong share returned");
                Ok(())
            })
            .await
        })
        .await
        .expect("test failed")
        .expect("second share creation failed")
        .expect("first share creation failed");
    }

    async fn with_key<'a, S: Store, Fut, T>(
        store: &'a S,
        identity: IdentityId,
        version: KeyVersion,
        f: impl FnOnce(&'a S, KeyId) -> Fut,
    ) -> Result<T, Error>
    where
        Fut: futures::Future<Output = T> + 'a,
    {
        let key_id = KeyId {
            name: "omni".to_string(),
            identity: IdentityLocator {
                chain: 31337,
                registry: Address::repeat_byte(1),
                id: identity,
            },
            version,
        };
        let mut key = vec![0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut key);
        let created = store.put_key(key_id.clone(), key.into()).await?;
        ensure!(
            created,
            "key not created due to duplicate or non-contiguous version"
        );
        let res = f(store, key_id.clone()).await;
        store.delete_key(key_id).await?;
        Ok(res)
    }

    pub async fn roundtrip_key(store: impl Store) {
        let identity = IdentityId::random();
        with_key(&store, identity, 1, |store, key_id| async move {
            ensure!(key_id.identity.chain == 31337, "unexpected key chain");
            ensure!(
                key_id.identity.registry == Address::repeat_byte(1),
                "unexpected key registry"
            );
            ensure!(key_id.identity.id == identity, "unexpected key identity");
            ensure!(key_id.version == 1, "unexpected key version");
            let key = store.get_key(key_id.clone()).await?;
            let key2 = store.get_key(key_id).await?;
            ensure!(key == key2, "retrieved keys mismatched");
            Ok(())
        })
        .await
        .expect("test failed")
        .expect("key creation failed");
    }

    pub async fn create_second_key_version(store: impl Store) {
        let identity = IdentityId::random();
        with_key(&store, identity, 1, |store, key_id1| async move {
            let key1_1 = store.get_key(key_id1.clone()).await?;
            with_key(store, identity, 2, |store, key_id2| async move {
                ensure!(key_id1.identity == key_id2.identity, "key identity changed");
                ensure!(key_id2.version == 2, "key version did not increment");
                let key1_2 = store.get_key(key_id1).await?;
                let key2 = store.get_key(key_id2).await?;
                ensure!(key1_1 == key1_2, "key changed");
                ensure!(key1_1 != key2, "wrong key returned");
                Ok(())
            })
            .await
        })
        .await
        .expect("test failed")
        .expect("second key creation failed")
        .expect("first key creation failed");
    }

    pub async fn create_duplicate_key_version(store: impl Store) {
        let identity = IdentityId::random();
        with_key(&store, identity, 1, |store, _| async move {
            ensure!(
                with_key(store, identity, 1, |_, _| async move {})
                    .await
                    .is_err(),
                "duplicate key version wrongly created"
            );
            Ok(())
        })
        .await
        .expect("test failed")
        .expect("first key creation failed");
    }

    pub async fn create_discontinuous_key_version(store: impl Store) {
        let identity = IdentityId::random();
        with_key(&store, identity, 1, |store, _| async move {
            ensure!(
                with_key(store, identity, 3, |_, _| async move {})
                    .await
                    .is_err(),
                "discontinuous key version wrongly created"
            );
            Ok(())
        })
        .await
        .expect("test failed")
        .expect("first key creation failed");
    }

    pub async fn create_second_key(store: impl Store) {
        let identity1 = IdentityId::random();
        let identity2 = IdentityId::random();
        with_key(&store, identity1, 1, |store, key_id1| async move {
            let key1 = store.get_key(key_id1).await?;
            with_key(store, identity2, 1, |store, key_id2| async move {
                let key2 = store.get_key(key_id2).await?;
                ensure!(key1 != key2, "wrong key returned");
                Ok(())
            })
            .await
        })
        .await
        .expect("test failed")
        .expect("second key creation failed")
        .expect("first key creation failed");
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
            .create_permit(share.identity, recipient, expiry, nonce)
            .await
            .unwrap();
        match permit {
            Some(p) => {
                let res = f(store, p).await;
                store
                    .delete_permit(share.identity, recipient)
                    .await
                    .unwrap();
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
                let read_permit = store.read_permit(share.identity, recipient).await?;
                ensure!(read_permit.is_some(), "permit not created");
                ensure!(read_permit.unwrap() == permit, "permit mismatch");
                Ok(())
            },
        )
        .await
        .expect("test failed")
        .expect("permit creation failed");
    }

    pub async fn expired_permit(store: impl Store) {
        let share = mock_share();
        let recipient = Address::random();
        let expiry = now() - 60;
        with_permit(&store, share, recipient, expiry, |store, _| async move {
            let read_permit = store.read_permit(share.identity, recipient).await?;
            ensure!(read_permit.is_none(), "permit not expired");
            Ok(())
        })
        .await
        .expect("test failed")
        .expect("permit creation failed");
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
                    let read_permit = store.read_permit(share.identity, recipient).await?;
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
        .expect("test failed")
        .expect("second permit creation failed")
        .expect("first permit creation failed");
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
        .expect("test failed")
        .expect("permit creation failed");
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
                store.delete_permit(share.identity, recipient).await?;
                let outcome = with_permit(
                    store,
                    share,
                    recipient,
                    expiry_soon,
                    |store, permit| async move {
                        let read_permit = store.read_permit(share.identity, recipient).await?;
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
        .expect("test failed")
        .expect("permit creation failed");
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
