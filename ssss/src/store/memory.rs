use std::{collections::HashMap, sync::{Arc, RwLock}};

use rand::RngCore as _;

use super::*;

#[derive(Clone, Default)]
pub struct MemoryStore {
    state: Arc<State>
}

#[derive(Default)]
struct State {
    shares: RwLock<HashMap<IdentityLocator, Vec<Option<Vec<u8>>>>>,
    permits: RwLock<HashMap<(ShareId, Address), Permit>>,
}

impl Store for MemoryStore {
    type Error = std::convert::Infallible;

    async fn create_share(&self, identity: IdentityLocator) -> Result<ShareId, Self::Error> {
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

    async fn get_share(&self, share: ShareId) -> Result<Option<WrappedShare>, Self::Error> {
        Ok(match self.state.shares.read().unwrap().get(&share.identity) {
            Some(versions) if versions.len() >= share.version as usize => versions
                [share.version as usize - 1]
                .clone()
                .map(WrappedShare),
            _ => None,
        })
    }

    #[cfg(test)]
    async fn destroy_share(&self, share: ShareId) -> Result<(), Self::Error> {
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
    ) -> Result<Option<Permit>, Self::Error> {
        Ok(
            match self.state.permits.write().unwrap().entry((share, recipient)) {
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
    ) -> Result<Option<Permit>, Self::Error> {
        Ok(
            match self.state.permits.read().unwrap().get(&(share, recipient)) {
                Some(permit) if permit.expiry > now() => Some(permit.clone()),
                _ => None,
            },
        )
    }

    async fn delete_permit(&self, share: ShareId, recipient: Address) -> Result<(), Self::Error> {
        self.state.permits.write().unwrap().remove(&(share, recipient));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::make_sstore_tests!(MemoryStore::default());
}
