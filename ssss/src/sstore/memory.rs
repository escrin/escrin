use std::{collections::HashMap, sync::RwLock};

use rand::RngCore as _;

use super::*;

#[derive(Default)]
pub struct Client {
    shares: RwLock<HashMap<IdentityId, Vec<Option<Vec<u8>>>>>,
    permits: RwLock<HashMap<(ShareId, Address), Permit>>,
}

impl ShareStore for Client {
    type Error = std::convert::Infallible;

    async fn create_share(&self, identity: IdentityId) -> Result<ShareId, Self::Error> {
        let mut shares = self.shares.write().unwrap();
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
        Ok(match self.shares.read().unwrap().get(&share.identity) {
            Some(versions) if versions.len() >= share.version as usize => versions
                [share.version as usize - 1]
                .clone()
                .map(WrappedShare),
            _ => None,
        })
    }

    #[cfg(test)]
    async fn destroy_share(&self, share: ShareId) -> Result<(), Self::Error> {
        if let Some(versions) = self.shares.write().unwrap().get_mut(&share.identity) {
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
            match self.permits.write().unwrap().entry((share, recipient)) {
                std::collections::hash_map::Entry::Occupied(mut oe) => {
                    let mut permit = oe.get_mut();
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
            match self.permits.read().unwrap().get(&(share, recipient)) {
                Some(permit) if permit.expiry > now() => Some(permit.clone()),
                _ => None,
            },
        )
    }

    async fn delete_permit(&self, share: ShareId, recipient: Address) -> Result<(), Self::Error> {
        self.permits.write().unwrap().remove(&(share, recipient));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::make_sstore_tests!(Client::default());
}
