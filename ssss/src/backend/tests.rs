use anyhow::{anyhow, ensure};
use futures_util::StreamExt as _;

use super::*;

#[macro_export]
macro_rules! make_backend_tests {
    ($store_factory:expr) => {
        $crate::make_backend_tests!(
            $store_factory,
            roundtrip_share,
            create_second_share_version,
            create_duplicate_share_version,
            create_discontinuous_share_version,
            create_delete_create_share_version,
            create_second_share,
            get_uncommitted_share,
            overwrite_uncommitted_share,
            commit_share_twice,
            roundtrip_key,
            create_second_key_version,
            create_duplicate_key_version,
            create_discontinuous_key_version,
            create_delete_create_key_version,
            create_second_key,
            roundtrip_verifier,
            roundtrip_signer,
        );
    };
    ($store_factory:expr, $($test:ident),+ $(,)?) => {
        $(
            #[tokio::test]
            async fn $test() {
                let store = $store_factory.await;
                $crate::backend::tests::$test(store).await;
            }
        )+
    }
}

fn random_bytes() -> Vec<u8> {
    let mut bytes = vec![0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
    bytes
}

fn make_share(identity: IdentityId, version: u64) -> (ShareId, SecretShare) {
    let share_id = ShareId {
        secret_name: "test".into(),
        identity: IdentityLocator {
            chain: 31337,
            registry: Address::repeat_byte(1),
            id: identity,
        },
        version,
    };
    (
        share_id,
        SecretShare {
            meta: SecretShareMeta {
                index: 1,
                commitments: std::iter::repeat(())
                    .map(|_| random_bytes())
                    .take(16)
                    .collect(),
            },
            share: random_bytes().into(),
            blinder: random_bytes().into(),
        },
    )
}

async fn with_new_share<'a, S: Store, Fut, T>(
    store: &'a S,
    identity: IdentityId,
    version: ShareVersion,
    f: impl FnOnce(&'a S, ShareId) -> Fut + 'a,
) -> Result<T, Error>
where
    Fut: std::future::Future<Output = T> + 'a,
{
    let (share_id, share) = make_share(identity, version);
    with_share(store, share_id, share, f).await
}

async fn with_share<'a, S: Store, Fut, T>(
    store: &'a S,
    share_id: ShareId,
    share: SecretShare,
    f: impl FnOnce(&'a S, ShareId) -> Fut + 'a,
) -> Result<T, Error>
where
    Fut: std::future::Future<Output = T> + 'a,
{
    with_share_no_commit(store, share_id, share, |store, share_id| async move {
        ensure!(
            store.commit_share(share_id.clone()).await?,
            "share not committed"
        );
        Ok(f(store, share_id).await)
    })
    .await?
}

async fn with_share_no_commit<'a, S: Store, Fut, T>(
    store: &'a S,
    share_id: ShareId,
    share: SecretShare,
    f: impl FnOnce(&'a S, ShareId) -> Fut,
) -> Result<T, Error>
where
    Fut: std::future::Future<Output = T> + 'a,
{
    let created = store.put_share(share_id.clone(), share).await?;
    ensure!(
        created,
        "share not created due to duplicate or non-contiguous version"
    );
    let res = f(store, share_id.clone()).await;
    store.delete_share(share_id.clone()).await?;
    Ok(res)
}

pub async fn roundtrip_share(store: impl Store) {
    let identity = IdentityId::random();
    with_new_share(&store, identity, 1, |store, share_id| async move {
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
        let share = store
            .get_share(share_id.clone())
            .await?
            .ok_or_else(|| anyhow!("no share returned"))?;
        let share2 = store
            .get_share(share_id.clone())
            .await?
            .ok_or_else(|| anyhow!("no share returned"))?;
        ensure!(share == share2, "retrieved shares mismatched");
        Ok(())
    })
    .await
    .expect("test failed")
    .expect("share creation failed");
}

pub async fn create_second_share_version(store: impl Store) {
    let identity = IdentityId::random();
    with_new_share(&store, identity, 1, |store, share_id1| async move {
        let share1 = store.get_share(share_id1.clone()).await?;
        with_new_share(store, identity, 2, |store, share_id2| async move {
            ensure!(
                share_id1.identity == share_id2.identity,
                "share identity changed"
            );
            ensure!(share_id2.version == 2, "share version did not increment");
            ensure!(
                store.get_share(share_id1).await?.is_none(),
                "old share not deleted"
            );
            let share2 = store.get_share(share_id2).await?;
            ensure!(share1 != share2, "wrong share returned");
            Ok(())
        })
        .await
    })
    .await
    .expect("test failed")
    .expect("first share creation failed")
    .expect("second share creation failed");
}

pub async fn get_uncommitted_share(store: impl Store) {
    let identity = IdentityId::random();
    let (share_id, share) = make_share(identity, 1);
    with_share_no_commit(&store, share_id, share, |store, share_id| async {
        ensure!(
            store.get_share(share_id).await?.is_none(),
            "uncommitted share visible"
        );
        Ok(())
    })
    .await
    .expect("test failed")
    .expect("share not created");
}

pub async fn overwrite_uncommitted_share(store: impl Store) {
    let identity = IdentityId::random();
    let (share_id, share) = make_share(identity, 1);
    with_share_no_commit(
        &store,
        share_id,
        share.clone(),
        |store, share_id| async move {
            ensure!(
                !store.put_share(share_id, share).await?,
                "overwrote uncommitted share"
            );
            Ok(())
        },
    )
    .await
    .expect("test failed")
    .expect("share not created");
}

pub async fn commit_share_twice(store: impl Store) {
    let identity = IdentityId::random();
    let (share_id, share) = make_share(identity, 1);
    with_share_no_commit(&store, share_id, share, |store, share_id| async {
        ensure!(store.commit_share(share_id.clone()).await?, "commit failed");
        ensure!(
            store.get_share(share_id.clone()).await?.is_some(),
            "not committed"
        );
        ensure!(
            store.commit_share(share_id.clone()).await?,
            "commit not idempotent"
        );
        ensure!(store.get_share(share_id).await?.is_some(), "lost share");
        Ok(())
    })
    .await
    .expect("test failed")
    .expect("share not created");
}

pub async fn create_duplicate_share_version(store: impl Store) {
    let identity = IdentityId::random();
    with_new_share(&store, identity, 1, |store, _| async move {
        ensure!(
            with_new_share(store, identity, 1, |_, _| async move {})
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

pub async fn create_delete_create_share_version(store: impl Store) {
    let identity = IdentityId::random();
    let (share_id_1, share) = make_share(identity, 1);
    let share_id_2 = ShareId {
        version: 2,
        ..share_id_1.clone()
    };
    let share_id_3 = ShareId {
        version: 3,
        ..share_id_1.clone()
    };
    with_share(&store, share_id_1, share.clone(), |_, _| async {})
        .await
        .unwrap();
    with_share(&store, share_id_2, share.clone(), |_, _| async {})
        .await
        .unwrap();
    with_share(&store, share_id_3, share, |_, _| async {})
        .await
        .unwrap();
}

pub async fn create_discontinuous_share_version(store: impl Store) {
    let identity = IdentityId::random();
    with_new_share(&store, identity, 1, |store, _| async move {
        ensure!(
            with_new_share(store, identity, 3, |_, _| async move {})
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
    with_new_share(&store, identity1, 1, |store, share_id1| async move {
        let share1 = store.get_share(share_id1).await?;
        with_new_share(store, identity2, 1, |store, share_id2| async move {
            let share2 = store.get_share(share_id2).await?;
            ensure!(share1 != share2, "wrong share returned");
            Ok(())
        })
        .await
    })
    .await
    .expect("test failed")
    .expect("first share creation failed")
    .expect("second share creation failed");
}

fn make_key(identity: IdentityId, version: u64) -> (KeyId, WrappedKey) {
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
    (key_id, key.into())
}

async fn with_new_key<'a, S: Store, Fut, T>(
    store: &'a S,
    identity: IdentityId,
    version: KeyVersion,
    f: impl FnOnce(&'a S, KeyId) -> Fut,
) -> Result<T, Error>
where
    Fut: std::future::Future<Output = T> + 'a,
{
    let (key_id, key) = make_key(identity, version);
    with_key(store, key_id, key, f).await
}

async fn with_key<'a, S: Store, Fut, T>(
    store: &'a S,
    key_id: KeyId,
    key: WrappedKey,
    f: impl FnOnce(&'a S, KeyId) -> Fut,
) -> Result<T, Error>
where
    Fut: std::future::Future<Output = T> + 'a,
{
    let created = store.put_secret(key_id.clone(), key).await?;
    ensure!(
        created,
        "key not created due to duplicate or non-contiguous version"
    );
    let res = f(store, key_id.clone()).await;
    store.delete_secret(key_id).await?;
    Ok(res)
}

pub async fn roundtrip_key(store: impl Store) {
    let identity = IdentityId::random();
    with_new_key(&store, identity, 1, |store, key_id| async move {
        ensure!(key_id.identity.chain == 31337, "unexpected key chain");
        ensure!(
            key_id.identity.registry == Address::repeat_byte(1),
            "unexpected key registry"
        );
        ensure!(key_id.identity.id == identity, "unexpected key identity");
        ensure!(key_id.version == 1, "unexpected key version");
        let key = store.get_secret(key_id.clone()).await?;
        let key2 = store.get_secret(key_id).await?;
        ensure!(key == key2, "retrieved keys mismatched");
        Ok(())
    })
    .await
    .expect("test failed")
    .expect("key creation failed");
}

pub async fn create_second_key_version(store: impl Store) {
    let identity = IdentityId::random();
    with_new_key(&store, identity, 1, |store, key_id1| async move {
        let key1_1 = store.get_secret(key_id1.clone()).await?;
        with_new_key(store, identity, 2, |store, key_id2| async move {
            ensure!(key_id1.identity == key_id2.identity, "key identity changed");
            ensure!(key_id2.version == 2, "key version did not increment");
            let key1_2 = store.get_secret(key_id1).await?;
            let key2 = store.get_secret(key_id2).await?;
            ensure!(key1_1 == key1_2, "key changed");
            ensure!(key1_1 != key2, "wrong key returned");
            Ok(())
        })
        .await
    })
    .await
    .expect("test failed")
    .expect("first key creation failed")
    .expect("second key creation failed");
}

pub async fn create_duplicate_key_version(store: impl Store) {
    let identity = IdentityId::random();
    with_new_key(&store, identity, 1, |store, _| async move {
        ensure!(
            with_new_key(store, identity, 1, |_, _| async move {})
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

pub async fn create_delete_create_key_version(store: impl Store) {
    let identity = IdentityId::random();
    let (key_id_1, key) = make_key(identity, 1);
    let key_id_2 = KeyId {
        version: 2,
        ..key_id_1.clone()
    };
    let key_id_3 = KeyId {
        version: 3,
        ..key_id_1.clone()
    };
    with_key(&store, key_id_1, key.clone(), |_, _| async {})
        .await
        .unwrap();
    with_key(&store, key_id_2, key.clone(), |_, _| async {})
        .await
        .unwrap();
    with_key(&store, key_id_3, key, |_, _| async {})
        .await
        .unwrap();
}

pub async fn create_discontinuous_key_version(store: impl Store) {
    let identity = IdentityId::random();
    with_new_key(&store, identity, 1, |store, _| async move {
        ensure!(
            with_new_key(store, identity, 3, |_, _| async move {})
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
    with_new_key(&store, identity1, 1, |store, key_id1| async move {
        let key1 = store.get_secret(key_id1).await?;
        with_new_key(store, identity2, 1, |store, key_id2| async move {
            let key2 = store.get_secret(key_id2).await?;
            ensure!(key1 != key2, "wrong key returned");
            Ok(())
        })
        .await
    })
    .await
    .expect("test failed")
    .expect("first key creation failed")
    .expect("second key creation failed");
}

pub async fn roundtrip_verifier(store: impl Store) {
    let chains: [u64; 2] = rand::random();
    let identity_ids: [IdentityId; 2] = rand::random();
    let registries: [Address; 2] = rand::random();
    let permitters: [Address; 2] = rand::random();

    let mut configs = vec![];
    for &pchain in chains.iter() {
        for &ichain in chains.iter() {
            for &identity in identity_ids.iter() {
                for &registry in registries.iter() {
                    for &permitter in permitters.iter() {
                        let permitter_locator = PermitterLocator {
                            chain: pchain,
                            permitter,
                        };
                        let identity_locator = IdentityLocator {
                            chain: ichain,
                            registry,
                            id: identity,
                        };
                        configs.push((permitter_locator, identity_locator));
                    }
                }
            }
        }
    }

    futures_util::stream::iter(configs.iter().copied())
        .for_each_concurrent(None, |(permitter, identity)| {
            let store = store.clone();
            async move {
                let config = store.get_verifier(permitter, identity).await.unwrap();
                assert!(config.is_none());
            }
        })
        .await;

    futures_util::stream::iter(configs.iter().copied().enumerate())
        .for_each_concurrent(None, |(i, (permitter, identity))| {
            let store = store.clone();
            async move {
                store
                    .put_verifier(permitter, identity, format!("config{i}").into_bytes())
                    .await
                    .unwrap();
            }
        })
        .await;

    futures_util::stream::iter(configs.iter().copied().enumerate())
        .for_each_concurrent(None, |(i, (permitter, identity))| {
            let store = store.clone();
            async move {
                let config = store.get_verifier(permitter, identity).await.unwrap();
                assert_eq!(config.unwrap(), format!("config{i}").into_bytes())
            }
        })
        .await;

    futures_util::stream::iter(configs.iter().copied().enumerate())
        .for_each_concurrent(None, |(i, (permitter, identity))| {
            let store = store.clone();
            async move {
                store
                    .put_verifier(permitter, identity, format!("config{i}-2").into_bytes())
                    .await
                    .unwrap();
            }
        })
        .await;

    futures_util::stream::iter(configs.iter().copied().enumerate())
        .for_each_concurrent(None, |(i, (permitter, identity))| {
            let store = store.clone();
            async move {
                let config = store.get_verifier(permitter, identity).await.unwrap();
                assert_eq!(config.unwrap(), format!("config{i}-2").into_bytes())
            }
        })
        .await;

    futures_util::stream::iter(configs.iter().copied())
        .for_each_concurrent(None, |(permitter, identity)| {
            let store = store.clone();
            async move {
                store.clear_verifier(permitter, identity).await.unwrap();
            }
        })
        .await;
}

pub async fn roundtrip_signer(signer: impl Signer) {
    let addr = signer.signer_address().await.unwrap();
    futures_util::stream::repeat(())
        .take(10)
        .for_each_concurrent(None, |_| async {
            let message = H256::random();
            let sig = signer.sign(message).await.unwrap();
            sig.verify(message, addr).unwrap();
        })
        .await;
}
