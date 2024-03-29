use anyhow::ensure;

use super::*;

#[macro_export]
macro_rules! make_store_tests {
    ($store_factory:expr) => {
        $crate::make_store_tests!(
            $store_factory,
            roundtrip_share,
            create_second_share_version,
            create_duplicate_share_version,
            create_discontinuous_share_version,
            create_delete_create_share_version,
            create_second_share,
            roundtrip_key,
            create_second_key_version,
            create_duplicate_key_version,
            create_discontinuous_key_version,
            create_delete_create_key_version,
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
    ($store_factory:expr, $($test:ident),+ $(,)?) => {
        $(
            #[tokio::test]
            async fn $test() {
                let store = $store_factory.await;
                $crate::store::tests::$test(store).await;
            }
        )+
    }
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
    let mut share = vec![0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut share);
    (
        share_id,
        SecretShare {
            index: 1,
            share: share.into(),
        },
    )
}

async fn with_new_share<'a, S: Store, Fut, T>(
    store: &'a S,
    identity: IdentityId,
    version: ShareVersion,
    f: impl FnOnce(&'a S, ShareId) -> Fut,
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
    store.delete_share_version(share_id.clone()).await?;
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
        let share = store.get_share(share_id.clone()).await?;
        let share2 = store.get_share(share_id.clone()).await?;
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
        let share1_1 = store.get_share(share_id1.clone()).await?;
        with_new_share(store, identity, 2, |store, share_id2| async move {
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
    .expect("first share creation failed")
    .expect("second share creation failed");
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
    let created = store.put_key(key_id.clone(), key).await?;
    ensure!(
        created,
        "key not created due to duplicate or non-contiguous version"
    );
    let res = f(store, key_id.clone()).await;
    store.delete_key_version(key_id).await?;
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
    with_new_key(&store, identity, 1, |store, key_id1| async move {
        let key1_1 = store.get_key(key_id1.clone()).await?;
        with_new_key(store, identity, 2, |store, key_id2| async move {
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
        let key1 = store.get_key(key_id1).await?;
        with_new_key(store, identity2, 1, |store, key_id2| async move {
            let key2 = store.get_key(key_id2).await?;
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

async fn with_permit<'a, S: Store, Fut, T>(
    store: &'a S,
    share: ShareId,
    recipient: Address,
    expiry: u64,
    f: impl FnOnce(&'a S, Permit) -> Fut,
) -> Option<T>
where
    Fut: std::future::Future<Output = T> + 'a,
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
    Fut: std::future::Future<Output = T> + 'a,
{
    let permit = store
        .create_permit(share.identity, recipient, expiry, nonce)
        .await
        .expect("permit not created");
    match permit {
        Some(p) => {
            let res = f(store, p).await;
            store
                .delete_permit(share.identity, recipient)
                .await
                .expect("permit not deleted");
            Some(res)
        }
        None => None,
    }
}

fn mock_share() -> ShareId {
    ShareId {
        secret_name: "test".into(),
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
    let expiry = now() + 30;
    with_permit(
        &store,
        share.clone(),
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
    with_permit(
        &store,
        share.clone(),
        recipient,
        expiry,
        |store, _| async move {
            let read_permit = store.read_permit(share.identity, recipient).await?;
            ensure!(read_permit.is_none(), "permit not expired");
            Ok(())
        },
    )
    .await
    .expect("test failed")
    .expect("permit creation failed");
}

pub async fn used_nonce_permit(store: impl Store) {
    let share = mock_share();
    let recipient = Address::random();
    let expiry = now() + 30;
    let mut nonce = vec![0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce);

    with_permit_nonce(
        &store,
        share.clone(),
        recipient,
        expiry,
        |_, _| async {},
        nonce.clone(),
    )
    .await
    .unwrap();

    assert!(with_permit_nonce(
        &store,
        share.clone(),
        recipient,
        expiry,
        |_, _| async {},
        nonce
    )
    .await
    .is_none());
}

pub async fn refresh_permit(store: impl Store) {
    let share = mock_share();
    let recipient = Address::random();
    let expiry_soon = now() + 30;
    let expiry_far = now() + 60;
    with_permit(
        &store,
        share.clone(),
        recipient,
        expiry_soon,
        |store, _| async move {
            with_permit(
                store,
                share.clone(),
                recipient,
                expiry_far,
                |store, _| async move {
                    let read_permit = store.read_permit(share.identity, recipient).await?;
                    ensure!(read_permit.is_some(), "permit not created");
                    ensure!(
                        read_permit.unwrap().expiry == expiry_far,
                        "permit expiry not refreshed"
                    );
                    Ok(())
                },
            )
            .await
        },
    )
    .await
    .expect("test failed")
    .expect("first permit creation failed")
    .expect("second permit creation failed");
}

pub async fn defresh_permit_fail(store: impl Store) {
    let share = mock_share();
    let recipient = Address::random();
    let expiry_soon = now() + 30;
    let expiry_far = now() + 60;
    with_permit(
        &store,
        share.clone(),
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
    let expiry_soon = now() + 30;
    let expiry_far = now() + 60;
    with_permit(
        &store,
        share.clone(),
        recipient,
        expiry_far,
        |store, _| async move {
            store.delete_permit(share.identity, recipient).await?;
            let outcome = with_permit(
                store,
                share.clone(),
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
