#[cfg(feature = "aws")]
pub mod aws;
pub mod local;

use ethers::types::{Address, H256};

pub type IdentityId = H256;
pub type ShareVersion = u64;

const SHARE_SIZE: usize = 32;

pub struct ShareId {
    identity: IdentityId,
    version: ShareVersion,
}

pub struct Permit {
    expiry: u64,
}

#[derive(zeroize::Zeroize)]
pub struct WrappedShare(Vec<u8>);

pub trait ShareStore {
    type Error: std::error::Error;

    async fn create_share(&self, identity: IdentityId) -> Result<ShareId, Self::Error>;

    async fn get_share(&self, share: ShareId) -> Result<Option<WrappedShare>, Self::Error>;

    async fn create_permit(
        &self,
        share: ShareId,
        recipient: Address,
        expiry: u64,
    ) -> Result<(), Self::Error>;

    async fn read_permit(
        &self,
        share: ShareId,
        recipient: Address,
    ) -> Result<Option<Permit>, Self::Error>;

    async fn delete_permit(&self, share: ShareId, recipient: Address) -> Result<(), Self::Error>;
}
