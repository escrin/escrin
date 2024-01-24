use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use rand::RngCore as _;

use super::*;

#[derive(Clone, Default)]
pub struct MemoryStore {
    state: Arc<State>,
}

type ShareGrantee = (ShareId, Address);
type PermitterIdentityLocator = (PermitterLocator, IdentityId);
type VerionedVerifierConfig = (Vec<u8>, EventIndex);

#[derive(Default)]
struct State {
    shares: RwLock<HashMap<IdentityLocator, Vec<Option<Vec<u8>>>>>,
    permits: RwLock<HashMap<ShareGrantee, Permit>>,
    verifiers: RwLock<HashMap<PermitterIdentityLocator, VerionedVerifierConfig>>,
    chain: RwLock<HashMap<u64, ChainState>>,
}

impl Store for MemoryStore {
    async fn create_share(&self, identity: IdentityLocator) -> Result<ShareId, Error> {
        let mut shares = self.state.shares.write().unwrap();
        let mut share = vec![0u8; SHARE_SIZE];
        rand::thread_rng().fill_bytes(&mut share);
        let identity_shares = shares.entry(identity).or_default();
        identity_shares.push(Some(share));
        Ok(ShareId {
            identity,
            version: identity_shares.len() as u64,
        })
    }

    async fn get_share(&self, share: ShareId) -> Result<Option<WrappedShare>, Error> {
        Ok(
            match self.state.shares.read().unwrap().get(&share.identity) {
                Some(versions) if versions.len() >= share.version as usize => {
                    versions[share.version as usize - 1].clone().map(Into::into)
                }
                _ => None,
            },
        )
    }

    #[cfg(test)]
    async fn destroy_share(&self, share: ShareId) -> Result<(), Error> {
        if let Some(versions) = self.state.shares.write().unwrap().get_mut(&share.identity) {
            if versions.len() <= share.version as usize {
                versions[share.version as usize - 1] = None;
            }
        }
        Ok(())
    }

    async fn create_permit(
        &self,
        share: ShareId,
        recipient: Address,
        expiry: u64,
    ) -> Result<Option<Permit>, Error> {
        Ok(
            match self
                .state
                .permits
                .write()
                .unwrap()
                .entry((share, recipient))
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
        share: ShareId,
        recipient: Address,
    ) -> Result<Option<Permit>, Error> {
        Ok(
            match self.state.permits.read().unwrap().get(&(share, recipient)) {
                Some(permit) if permit.expiry > now() => Some(permit.clone()),
                _ => None,
            },
        )
    }

    async fn delete_permit(&self, share: ShareId, recipient: Address) -> Result<(), Error> {
        self.state
            .permits
            .write()
            .unwrap()
            .remove(&(share, recipient));
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
