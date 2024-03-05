use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use super::*;

#[derive(Clone, Default)]
pub struct MemoryStore {
    state: Arc<State>,
}

type Grantee = (IdentityLocator, Address);
type PermitterIdentityLocator = (PermitterLocator, IdentityId);
type VerionedVerifierConfig = (Vec<u8>, EventIndex);
type IdentityNamedItem = (IdentityLocator, String);

#[derive(Default)]
struct State {
    shares: RwLock<HashMap<IdentityLocator, Vec<Option<SecretShare>>>>,
    keys: RwLock<HashMap<IdentityNamedItem, Vec<Option<WrappedKey>>>>,
    permits: RwLock<HashMap<Grantee, Permit>>,
    verifiers: RwLock<HashMap<PermitterIdentityLocator, VerionedVerifierConfig>>,
    chain: RwLock<HashMap<u64, ChainState>>,
    nonces: RwLock<HashSet<Nonce>>,
}

impl Store for MemoryStore {
    async fn put_share(&self, id: ShareId, share: SecretShare) -> Result<bool, Error> {
        let mut shares = self.state.shares.write().unwrap();
        let identity_shares = shares.entry(id.identity).or_default();
        if id.version as usize != identity_shares.len() + 1 {
            return Ok(false);
        }
        identity_shares.push(Some(share));
        Ok(true)
    }

    async fn get_share(&self, id: ShareId) -> Result<Option<SecretShare>, Error> {
        Ok(match self.state.shares.read().unwrap().get(&id.identity) {
            Some(versions) if versions.len() >= id.version as usize => {
                versions[id.version as usize - 1].clone().map(Into::into)
            }
            _ => None,
        })
    }

    async fn delete_share(&self, id: ShareId) -> Result<(), Error> {
        if let Some(versions) = self.state.shares.write().unwrap().get_mut(&id.identity) {
            if id.version < versions.len() as u64 {
                versions[id.version as usize - 1] = None;
            }
        }
        Ok(())
    }

    async fn put_key(&self, id: KeyId, key: WrappedKey) -> Result<bool, Error> {
        let mut keys = self.state.keys.write().unwrap();
        let identified_keys = keys.entry((id.identity, id.name)).or_default();
        if id.version as usize != identified_keys.len() + 1 {
            return Ok(false);
        }
        identified_keys.push(Some(key));
        Ok(true)
    }

    async fn get_key(&self, id: KeyId) -> Result<Option<WrappedKey>, Error> {
        Ok(
            match self.state.keys.read().unwrap().get(&(id.identity, id.name)) {
                Some(versions) if versions.len() >= id.version as usize => {
                    versions[id.version as usize - 1].clone().map(Into::into)
                }
                _ => None,
            },
        )
    }

    async fn delete_key(&self, id: KeyId) -> Result<(), Error> {
        if let Some(versions) = self
            .state
            .keys
            .write()
            .unwrap()
            .get_mut(&(id.identity, id.name))
        {
            if versions.len() <= id.version as usize {
                versions[id.version as usize - 1] = None;
            }
        }
        Ok(())
    }

    async fn create_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
        expiry: u64,
        nonce: Nonce,
    ) -> Result<Option<Permit>, Error> {
        if !self.state.nonces.write().unwrap().insert(nonce) {
            return Ok(None);
        }
        Ok(
            match self
                .state
                .permits
                .write()
                .unwrap()
                .entry((identity, recipient))
            {
                std::collections::hash_map::Entry::Occupied(mut oe) => {
                    let permit = oe.get_mut();
                    if permit.expiry < expiry {
                        permit.expiry = expiry;
                        Some(permit.to_owned())
                    } else {
                        None
                    }
                }
                std::collections::hash_map::Entry::Vacant(ve) => {
                    Some(ve.insert(Permit { expiry }).to_owned())
                }
            },
        )
    }

    async fn read_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
    ) -> Result<Option<Permit>, Error> {
        Ok(
            match self
                .state
                .permits
                .read()
                .unwrap()
                .get(&(identity, recipient))
            {
                Some(permit) if permit.expiry > now() => Some(permit.clone()),
                _ => None,
            },
        )
    }

    async fn delete_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
    ) -> Result<(), Error> {
        self.state
            .permits
            .write()
            .unwrap()
            .remove(&(identity, recipient));
        Ok(())
    }

    async fn get_chain_state(&self, chain: u64) -> Result<Option<ChainState>, Error> {
        Ok(self.state.chain.read().unwrap().get(&chain).cloned())
    }

    async fn update_chain_state(&self, chain: u64, update: ChainStateUpdate) -> Result<(), Error> {
        let ChainStateUpdate { block } = update;
        let new_block = match block {
            Some(block) => block,
            None => return Ok(()),
        };
        let mut chain_state = self.state.chain.write().unwrap();
        let current_state = chain_state.entry(chain).or_default();
        if current_state.block < new_block {
            current_state.block = new_block;
        }
        Ok(())
    }

    #[cfg(test)]
    async fn clear_chain_state(&self, chain: u64) -> Result<(), Error> {
        self.state.chain.write().unwrap().remove(&chain);
        Ok(())
    }

    async fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<Option<Vec<u8>>, Error> {
        Ok(self
            .state
            .verifiers
            .read()
            .unwrap()
            .get(&(permitter, identity))
            .map(|(config, _)| config.clone()))
    }

    async fn update_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
        config: Vec<u8>,
        version: EventIndex,
    ) -> Result<(), Error> {
        let mut config = Some(config);
        self.state
            .verifiers
            .write()
            .unwrap()
            .entry((permitter, identity))
            .and_modify(|(current_config, current_version)| {
                if version <= *current_version {
                    return;
                }
                *current_config = config.take().unwrap();
                *current_version = version;
            })
            .or_insert_with(|| (config.take().unwrap(), version));
        Ok(())
    }

    #[cfg(test)]
    async fn clear_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
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

    crate::make_store_tests!(MemoryStore::default());
}
