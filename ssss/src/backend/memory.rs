use std::{
    collections::{
        btree_map::{self, BTreeMap},
        HashMap,
    },
    sync::{Arc, RwLock},
    time::Instant,
};

use ethers::signers::{LocalWallet, Signer as _};

use super::*;

type PermitterIdentityLocator = (PermitterLocator, IdentityLocator);
type IdentityNamedItem = (IdentityLocator, String);
type ExpiringSecretShare = (SecretShare, Option<Instant>);

#[derive(Default)]
struct State {
    shares: RwLock<HashMap<IdentityNamedItem, BTreeMap<u64, Option<ExpiringSecretShare>>>>,
    keys: RwLock<HashMap<IdentityNamedItem, BTreeMap<u64, Option<WrappedKey>>>>,
    verifiers: RwLock<HashMap<PermitterIdentityLocator, Vec<u8>>>,
}

#[derive(Clone)]
pub struct Backend {
    state: Arc<State>,
    wallet: LocalWallet,
}

impl Backend {
    pub fn generate() -> Self {
        Self {
            state: Default::default(),
            wallet: LocalWallet::new(&mut rand::thread_rng()),
        }
    }
}

impl Signer for Backend {
    async fn sign(&self, hash: H256) -> Result<Signature, Error> {
        Ok(self.wallet.sign_hash(hash)?)
    }

    async fn signer_address(&self) -> Result<Address, Error> {
        Ok(self.wallet.address())
    }
}

impl Store for Backend {
    async fn put_share(&self, id: ShareId, share: SecretShare) -> Result<bool, Error> {
        let mut shares = self.state.shares.write().unwrap();
        let versions = shares
            .entry((id.identity, id.secret_name.clone()))
            .or_default();
        let current_version = versions
            .last_key_value()
            .map(|(k, _)| *k)
            .unwrap_or_default();
        if id.version != current_version + 1 {
            return Ok(false);
        }
        versions.insert(
            id.version,
            Some((share, Some(Instant::now() + PRE_COMMIT_EXPIRY))),
        );
        Ok(true)
    }

    async fn commit_share(&self, id: ShareId) -> Result<bool, Error> {
        let (committed, to_delete) = {
            let mut shares = self.state.shares.write().unwrap();
            let Some(Some((_, expiry))) = shares
                .get_mut(&(id.identity, id.secret_name.clone()))
                .and_then(|versions| versions.get_mut(&id.version))
            else {
                return Ok(false);
            };
            if let Some(expiry) = expiry.take() {
                let current = expiry > Instant::now();
                (
                    current,
                    current
                        .then_some(())
                        .and((id.version > 1).then_some(id.version - 1)),
                )
            } else {
                (true, None)
            }
        };
        if let Some(version) = to_delete {
            self.delete_share(ShareId { version, ..id }).await?;
        }
        Ok(committed)
    }

    async fn get_share(&self, id: ShareId) -> Result<Option<SecretShare>, Error> {
        let Some((ss, expiry)) = self
            .state
            .shares
            .read()
            .unwrap()
            .get(&(id.identity, id.secret_name.clone()))
            .and_then(|versions| versions.get(&id.version).cloned())
            .flatten()
        else {
            return Ok(None);
        };
        Ok(match expiry {
            Some(expiry) => {
                if expiry <= Instant::now() {
                    self.delete_share(id).await?;
                }
                None
            }
            _ => Some(ss),
        })
    }

    async fn get_current_share_version(
        &self,
        identity: IdentityLocator,
        name: String,
    ) -> Result<Option<(ShareVersion, bool /* pending */)>, Error> {
        Ok(self
            .state
            .shares
            .read()
            .unwrap()
            .get(&(identity, name))
            .and_then(|versions| {
                let (&version, Some((_, expiry))) = versions.last_key_value()? else {
                    return None;
                };
                Some((version, expiry.is_some()))
            }))
    }

    async fn delete_share(&self, id: ShareId) -> Result<(), Error> {
        if let Some(versions) = self
            .state
            .shares
            .write()
            .unwrap()
            .get_mut(&(id.identity, id.secret_name))
        {
            if let btree_map::Entry::Occupied(mut oe) = versions.entry(id.version) {
                oe.insert(None);
            }
        }
        Ok(())
    }

    async fn put_secret(&self, id: KeyId, key: WrappedKey) -> Result<bool, Error> {
        let mut keys = self.state.keys.write().unwrap();
        let versions = keys.entry((id.identity, id.name)).or_default();
        let current_version = versions
            .last_key_value()
            .map(|(k, _)| *k)
            .unwrap_or_default();
        if id.version != current_version + 1 {
            return Ok(false);
        }
        versions.insert(id.version, Some(key));
        Ok(true)
    }

    async fn get_secret(&self, id: KeyId) -> Result<Option<WrappedKey>, Error> {
        Ok(self
            .state
            .keys
            .read()
            .unwrap()
            .get(&(id.identity, id.name))
            .and_then(|versions| versions.get(&id.version).cloned())
            .flatten())
    }

    async fn delete_secret(&self, id: KeyId) -> Result<(), Error> {
        if let Some(versions) = self
            .state
            .keys
            .write()
            .unwrap()
            .get_mut(&(id.identity, id.name))
        {
            if let btree_map::Entry::Occupied(mut oe) = versions.entry(id.version) {
                oe.insert(None);
            }
        }
        Ok(())
    }

    async fn put_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
        config: Vec<u8>,
    ) -> Result<(), Error> {
        let mut config = Some(config);
        self.state
            .verifiers
            .write()
            .unwrap()
            .entry((permitter, identity))
            .and_modify(|current_config| {
                *current_config = config.take().unwrap();
            })
            .or_insert_with(|| config.take().unwrap());
        Ok(())
    }

    async fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
    ) -> Result<Option<Vec<u8>>, Error> {
        Ok(self
            .state
            .verifiers
            .read()
            .unwrap()
            .get(&(permitter, identity))
            .cloned())
    }

    #[cfg(test)]
    async fn clear_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
    ) -> Result<(), Error> {
        self.state
            .verifiers
            .write()
            .unwrap()
            .remove(&(permitter, identity));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::make_backend_tests!(async { Backend::generate() });
}
