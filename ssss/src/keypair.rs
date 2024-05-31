use std::{
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

use aes_gcm_siv::{Aes256GcmSiv, KeyInit as _};

use crate::backend::Store;

#[derive(Clone, Copy)]
pub struct KeyPair {
    pk: p384::PublicKey,
    sk: p384::NonZeroScalar,
}

pub static DEAL_SHARES_DOMAIN_SEP: &[u8] = b"deal-shares";
pub static GET_SHARE_DOMAIN_SEP: &[u8] = b"get-share";

impl KeyPair {
    pub fn ephemeral() -> Self {
        let sk = p384::NonZeroScalar::random(&mut rand::thread_rng());
        Self {
            pk: p384::PublicKey::from_secret_scalar(&sk),
            sk,
        }
    }

    pub fn derive_shared_cipher(&self, opk: p384::PublicKey, hkdf_info: &[u8]) -> Aes256GcmSiv {
        derive_shared_cipher(&self.sk, &opk, hkdf_info)
    }

    pub fn public_key(&self) -> &p384::PublicKey {
        &self.pk
    }

    pub fn fingerprint(&self) -> String {
        hex::encode(<sha2::Sha256 as sha2::Digest>::digest(
            self.pk.to_sec1_bytes(),
        ))
    }
}

pub fn derive_shared_cipher(
    sk: &p384::NonZeroScalar,
    opk: &p384::PublicKey,
    hkdf_info: &[u8],
) -> Aes256GcmSiv {
    let shared = p384::ecdh::diffie_hellman(sk, opk.as_affine());
    let hkdf = shared.extract::<sha2::Sha256>(Some(b"ssss_ecdh_aes-256-gcm-siv"));
    let mut aes_key = [0u8; 32];
    hkdf.expand(hkdf_info, &mut aes_key).unwrap();
    Aes256GcmSiv::new_from_slice(&aes_key).unwrap()
}

struct RotatedKeyPair {
    id: String,
    kp: KeyPair,
    expiry: SystemTime,
}

impl RotatedKeyPair {
    fn generate(lifetime: Duration) -> Self {
        let kp = KeyPair::ephemeral();
        let mut id = kp.fingerprint();
        id.truncate(16);
        Self {
            id,
            kp,
            expiry: SystemTime::now() + lifetime,
        }
    }
}

#[derive(Clone)]
pub struct RotatingKeyPairProvider<S> {
    // store: S, // TODO: sync with store to enable replicas & stateless load balancing
    store: std::marker::PhantomData<S>,
    keys: Arc<RwLock<(RotatedKeyPair, Option<RotatedKeyPair>)>>,

    key_lifetime: Duration,
    /// The amount of time before the old key expires that the new one becomes active.
    /// Effectively the upper bound on much time a client has to send their request.
    swap_time: Duration,
}

impl<S: Store> RotatingKeyPairProvider<S> {
    const DEFAULT_KEY_LIFETIME: Duration = Duration::from_secs(60 * 60); // 1 hr
    const DEFAULT_SWAP_TIME: Duration = Duration::from_secs(5 * 60); // 5 mins

    pub fn new(store: S) -> Self {
        Self::new_with_durations(store, Self::DEFAULT_KEY_LIFETIME, Self::DEFAULT_SWAP_TIME)
    }

    pub fn new_with_durations(_store: S, key_lifetime: Duration, buffer_time: Duration) -> Self {
        Self {
            store: std::marker::PhantomData,
            keys: Arc::new(RwLock::new((RotatedKeyPair::generate(key_lifetime), None))),
            key_lifetime,
            swap_time: buffer_time,
        }
    }

    pub async fn with_latest_key<T>(
        &self,
        f: impl FnOnce(&str, &KeyPair, SystemTime) -> T,
    ) -> Result<T, crate::backend::Error> {
        self.do_with_key(None, f).await.transpose().unwrap()
    }

    pub async fn with_key<T>(
        &self,
        id: &str,
        f: impl FnOnce(&KeyPair) -> T,
    ) -> Result<Option<T>, crate::backend::Error> {
        self.do_with_key(Some(id), |_id, key, _expiry| f(key)).await
    }

    async fn do_with_key<T>(
        &self,
        id: Option<&str>,
        f: impl FnOnce(&str, &KeyPair, SystemTime) -> T,
    ) -> Result<Option<T>, crate::backend::Error> {
        let needs_refresh = {
            let keys = self.keys.read().unwrap();
            let now = SystemTime::now();
            now > keys.0.expiry || (now + self.swap_time > keys.0.expiry && keys.1.is_none())
        };
        if needs_refresh {
            let now = SystemTime::now();
            let mut keys = self.keys.write().unwrap();
            if now + self.swap_time > keys.0.expiry && keys.1.is_none() {
                keys.1 = Some(RotatedKeyPair::generate(self.key_lifetime));
            }
            if now > keys.0.expiry {
                keys.0 = keys.1.take().expect("new key exists");
            }
        }
        let keys = self.keys.read().unwrap();
        Ok(Some(match id {
            Some(id) => {
                if keys.0.id == id {
                    f(&keys.0.id, &keys.0.kp, keys.0.expiry)
                } else if let Some(key1) = keys.1.as_ref() {
                    if key1.id == id {
                        f(&key1.id, &key1.kp, key1.expiry)
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }
            None => {
                let key = keys.1.as_ref().unwrap_or(&keys.0);
                f(&key.id, &key.kp, key.expiry)
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::memory::Backend;

    #[tokio::test]
    async fn key_rotation() {
        let lifetime = Duration::from_millis(100);
        let buffer = Duration::from_millis(50);
        let provider =
            RotatingKeyPairProvider::new_with_durations(Backend::generate(), lifetime, buffer);

        macro_rules! latest_key {
            () => {
                provider
                    .with_latest_key(|id, _kp, _expiry| id.to_string())
                    .await
                    .unwrap()
            };
        }

        macro_rules! assert_key {
            ($id:expr, $is:ident) => {
                assert!(provider.with_key($id, |_kp| ()).await.unwrap().$is());
            };
        }

        let first_key_id_1 = latest_key!();
        let first_key_id_2 = latest_key!();
        assert_eq!(first_key_id_1, first_key_id_2);

        assert_key!(&first_key_id_1, is_some);

        tokio::time::sleep(buffer).await;

        let second_key_id_1 = latest_key!();
        let second_key_id_2 = latest_key!();
        assert_eq!(second_key_id_1, second_key_id_2);
        assert_ne!(first_key_id_1, second_key_id_1);

        assert_key!(&first_key_id_1, is_some);
        assert_key!(&second_key_id_1, is_some);

        tokio::time::sleep(lifetime - buffer).await;

        let second_key_id_3 = latest_key!();
        assert_eq!(second_key_id_1, second_key_id_3);

        assert_key!(&first_key_id_1, is_none);
        assert_key!(&second_key_id_1, is_some);

        tokio::time::sleep(lifetime * 2).await;
        latest_key!();
        assert_key!(&second_key_id_1, is_none);
    }
}
