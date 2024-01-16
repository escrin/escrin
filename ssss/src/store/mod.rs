#[cfg(feature = "aws")]
pub mod aws;
pub mod local;
pub mod memory;

use ethers::types::{Address, H256};

use crate::cli;

pub type IdentityId = H256;
pub type ShareVersion = u64;

const SHARE_SIZE: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShareId {
    identity: IdentityId,
    version: ShareVersion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permit {
    expiry: u64,
}

#[derive(zeroize::Zeroize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct WrappedShare(Vec<u8>);

pub trait Store: Clone + Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn create_share(&self, identity: IdentityId) -> Result<ShareId, Self::Error>;

    async fn get_share(&self, share: ShareId) -> Result<Option<WrappedShare>, Self::Error>;

    #[cfg(test)]
    async fn destroy_share(&self, share: ShareId) -> Result<(), Self::Error>;

    async fn create_permit(
        &self,
        share: ShareId,
        recipient: Address,
        expiry: u64,
    ) -> Result<Option<Permit>, Self::Error>;

    async fn read_permit(
        &self,
        share: ShareId,
        recipient: Address,
    ) -> Result<Option<Permit>, Self::Error>;

    async fn delete_permit(&self, share: ShareId, recipient: Address) -> Result<(), Self::Error>;
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
    Local,
}

impl Store for DynStore {
    type Error = DynStoreError;

    async fn create_share(&self, identity: IdentityId) -> Result<ShareId, Self::Error> {
        Ok(match &self.inner {
            DynStoreKind::Aws(ss) => ss.create_share(identity).await.map_err(anyhow::Error::from),
            DynStoreKind::Memory(ss) => {
                ss.create_share(identity).await.map_err(anyhow::Error::from)
            }
            DynStoreKind::Local => todo!(),
        }?)
    }

    async fn get_share(&self, share: ShareId) -> Result<Option<WrappedShare>, Self::Error> {
        Ok(match &self.inner {
            DynStoreKind::Aws(ss) => ss.get_share(share).await.map_err(anyhow::Error::from),
            DynStoreKind::Memory(ss) => ss.get_share(share).await.map_err(anyhow::Error::from),
            DynStoreKind::Local => todo!(),
        }?)
    }

    #[cfg(test)]
    async fn destroy_share(&self, share: ShareId) -> Result<(), Self::Error> {
        Ok(match &self.inner {
            DynStoreKind::Aws(ss) => ss.destroy_share(share).await.map_err(anyhow::Error::from),
            DynStoreKind::Memory(ss) => ss.destroy_share(share).await.map_err(anyhow::Error::from),
            DynStoreKind::Local => todo!(),
        }?)
    }

    async fn create_permit(
        &self,
        share: ShareId,
        recipient: Address,
        expiry: u64,
    ) -> Result<Option<Permit>, Self::Error> {
        Ok(match &self.inner {
            DynStoreKind::Aws(ss) => ss
                .create_permit(share, recipient, expiry)
                .await
                .map_err(anyhow::Error::from),
            DynStoreKind::Memory(ss) => ss
                .create_permit(share, recipient, expiry)
                .await
                .map_err(anyhow::Error::from),
            DynStoreKind::Local => todo!(),
        }?)
    }

    async fn read_permit(
        &self,
        share: ShareId,
        recipient: Address,
    ) -> Result<Option<Permit>, Self::Error> {
        Ok(match &self.inner {
            DynStoreKind::Aws(ss) => ss
                .read_permit(share, recipient)
                .await
                .map_err(anyhow::Error::from),
            DynStoreKind::Memory(ss) => ss
                .read_permit(share, recipient)
                .await
                .map_err(anyhow::Error::from),
            DynStoreKind::Local => todo!(),
        }?)
    }

    async fn delete_permit(&self, share: ShareId, recipient: Address) -> Result<(), Self::Error> {
        Ok(match &self.inner {
            DynStoreKind::Aws(ss) => ss
                .delete_permit(share, recipient)
                .await
                .map_err(anyhow::Error::from),
            DynStoreKind::Memory(ss) => ss
                .delete_permit(share, recipient)
                .await
                .map_err(anyhow::Error::from),
            DynStoreKind::Local => todo!(),
        }?)
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct DynStoreError(#[from] anyhow::Error);

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
    use anyhow::{ensure, Result};

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
        identity: &IdentityId,
        f: impl FnOnce(&'a S, ShareId) -> Fut,
    ) -> T
    where
        Fut: futures::Future<Output = T> + 'a,
    {
        let share_id = sstore.create_share(*identity).await.unwrap();
        let res = f(sstore, share_id).await;
        sstore.destroy_share(share_id).await.unwrap();
        res
    }

    pub async fn roundtrip_share(sstore: impl Store) {
        let identity = IdentityId::random();
        with_share(&sstore, &identity, |sstore, share_id| async move {
            ensure!(share_id.identity == identity, "unexpected share identity");
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
        with_share(&sstore, &identity, |sstore, share_id1| async move {
            let share1_1 = sstore.get_share(share_id1).await?;
            with_share(sstore, &identity, |sstore, share_id2| async move {
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
        with_share(&sstore, &identity1, |sstore, share_id1| async move {
            let share1 = sstore.get_share(share_id1).await?;
            with_share(sstore, &identity2, |sstore, share_id2| async move {
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
            identity: IdentityId::random(),
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
}
