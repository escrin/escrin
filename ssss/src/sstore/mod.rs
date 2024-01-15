#[cfg(feature = "aws")]
pub mod dynamodb;
pub mod local;

use ethers::types::H256;

pub type ShareId = H256;

pub enum ShareVersion {
    Latest,
    Numbered(u64),
}

#[derive(zeroize::Zeroize)]
pub struct WrappedShare(Vec<u8>);

pub trait ShareStore {
    async fn create(&self, id: ShareId) -> Result<(), Error>;

    async fn get(&self, id: ShareId, version: ShareVersion) -> Result<Option<WrappedShare>, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}
