#[cfg(feature = "aws")]
pub mod aws;
#[cfg(feature = "azure")]
pub mod azure;
#[cfg(feature = "local")]
pub mod local;
pub mod memory;
#[cfg(test)]
mod tests;

use std::{future::Future, time::Duration};

use axum::http::uri::Authority;
use ethers::types::{Address, Signature, H256};

use crate::types::*;

const PRE_COMMIT_EXPIRY: Duration = Duration::from_secs(10 * 60); // 10 minutes

pub trait Store: Clone + Send + Sync + 'static {
    fn put_share(
        &self,
        id: ShareId,
        share: SecretShare,
    ) -> impl Future<Output = Result<bool, Error>> + Send;

    fn commit_share(&self, id: ShareId) -> impl Future<Output = Result<bool, Error>> + Send;

    fn get_share(
        &self,
        id: ShareId,
    ) -> impl Future<Output = Result<Option<SecretShare>, Error>> + Send;

    fn get_current_share_version(
        &self,
        identity: IdentityLocator,
        name: String,
    ) -> impl Future<Output = Result<Option<(ShareVersion, bool /* pending */)>, Error>> + Send;

    fn delete_share(&self, id: ShareId) -> impl Future<Output = Result<(), Error>> + Send;

    fn put_secret(
        &self,
        id: KeyId,
        key: WrappedKey,
    ) -> impl Future<Output = Result<bool, Error>> + Send;

    fn get_secret(
        &self,
        id: KeyId,
    ) -> impl Future<Output = Result<Option<WrappedKey>, Error>> + Send;

    fn delete_secret(&self, id: KeyId) -> impl Future<Output = Result<(), Error>> + Send;

    fn put_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
        config: Vec<u8>,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
    ) -> impl Future<Output = Result<Option<Vec<u8>>, Error>> + Send;

    #[cfg(test)]
    fn clear_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
    ) -> impl Future<Output = Result<(), Error>> + Send;
}

pub trait Signer: Clone + Send + Sync + 'static {
    fn sign(&self, hash: H256) -> impl Future<Output = Result<Signature, Error>> + Send;

    fn signer_address(&self) -> impl Future<Output = Result<Address, Error>> + Send;
}

#[derive(Clone)]
pub struct DynBackend {
    inner: DynBackendKind,
}

#[derive(Clone)]
pub enum DynBackendKind {
    Memory(memory::Backend),
    #[cfg(feature = "aws")]
    Aws(aws::Backend),
    #[cfg(feature = "azure")]
    Azure(azure::Backend),
    #[cfg(feature = "local")]
    Local(local::Local),
}

impl Store for DynBackend {
    async fn put_share(&self, id: ShareId, share: SecretShare) -> Result<bool, Error> {
        match &self.inner {
            DynBackendKind::Memory(s) => s.put_share(id, share).await,
            #[cfg(feature = "aws")]
            DynBackendKind::Aws(s) => s.put_share(id, share).await,
            #[cfg(feature = "azure")]
            DynBackendKind::Azure(s) => s.put_share(id, share).await,
            #[cfg(feature = "local")]
            DynBackendKind::Local(s) => s.put_share(id, share).await,
        }
    }

    async fn commit_share(&self, id: ShareId) -> Result<bool, Error> {
        match &self.inner {
            DynBackendKind::Memory(s) => s.commit_share(id).await,
            #[cfg(feature = "aws")]
            DynBackendKind::Aws(s) => s.commit_share(id).await,
            #[cfg(feature = "azure")]
            DynBackendKind::Azure(s) => s.commit_share(id).await,
            #[cfg(feature = "local")]
            DynBackendKind::Local(s) => s.commit_share(id).await,
        }
    }

    async fn get_share(&self, id: ShareId) -> Result<Option<SecretShare>, Error> {
        match &self.inner {
            DynBackendKind::Memory(s) => s.get_share(id).await,
            #[cfg(feature = "aws")]
            DynBackendKind::Aws(s) => s.get_share(id).await,
            #[cfg(feature = "azure")]
            DynBackendKind::Azure(s) => s.get_share(id).await,
            #[cfg(feature = "local")]
            DynBackendKind::Local(s) => s.get_share(id).await,
        }
    }

    async fn get_current_share_version(
        &self,
        identity: IdentityLocator,
        name: String,
    ) -> Result<Option<(ShareVersion, bool /* pending */)>, Error> {
        match &self.inner {
            DynBackendKind::Memory(s) => s.get_current_share_version(identity, name).await,
            #[cfg(feature = "aws")]
            DynBackendKind::Aws(s) => s.get_current_share_version(identity, name).await,
            #[cfg(feature = "azure")]
            DynBackendKind::Azure(s) => s.get_current_share_version(identity, name).await,
            #[cfg(feature = "local")]
            DynBackendKind::Local(s) => s.get_current_share_version(identity, name).await,
        }
    }

    async fn delete_share(&self, id: ShareId) -> Result<(), Error> {
        match &self.inner {
            DynBackendKind::Memory(s) => s.delete_share(id).await,
            #[cfg(feature = "aws")]
            DynBackendKind::Aws(s) => s.delete_share(id).await,
            #[cfg(feature = "azure")]
            DynBackendKind::Azure(s) => s.delete_share(id).await,
            #[cfg(feature = "local")]
            DynBackendKind::Local(s) => s.delete_share(id).await,
        }
    }

    async fn put_secret(&self, id: KeyId, key: WrappedKey) -> Result<bool, Error> {
        match &self.inner {
            DynBackendKind::Memory(s) => s.put_secret(id, key).await,
            #[cfg(feature = "aws")]
            DynBackendKind::Aws(s) => s.put_secret(id, key).await,
            #[cfg(feature = "azure")]
            DynBackendKind::Azure(s) => s.put_secret(id, key).await,
            #[cfg(feature = "local")]
            DynBackendKind::Local(s) => s.put_secret(id, key).await,
        }
    }

    async fn get_secret(&self, id: KeyId) -> Result<Option<WrappedKey>, Error> {
        match &self.inner {
            DynBackendKind::Memory(s) => s.get_secret(id).await,
            #[cfg(feature = "aws")]
            DynBackendKind::Aws(s) => s.get_secret(id).await,
            #[cfg(feature = "azure")]
            DynBackendKind::Azure(s) => s.get_secret(id).await,
            #[cfg(feature = "local")]
            DynBackendKind::Local(s) => s.get_secret(id).await,
        }
    }

    async fn delete_secret(&self, id: KeyId) -> Result<(), Error> {
        match &self.inner {
            DynBackendKind::Memory(s) => s.delete_secret(id).await,
            #[cfg(feature = "aws")]
            DynBackendKind::Aws(s) => s.delete_secret(id).await,
            #[cfg(feature = "azure")]
            DynBackendKind::Azure(s) => s.delete_secret(id).await,
            #[cfg(feature = "local")]
            DynBackendKind::Local(s) => s.delete_secret(id).await,
        }
    }

    async fn put_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
        config: Vec<u8>,
    ) -> Result<(), Error> {
        match &self.inner {
            DynBackendKind::Memory(s) => s.put_verifier(permitter, identity, config).await,
            #[cfg(feature = "aws")]
            DynBackendKind::Aws(s) => s.put_verifier(permitter, identity, config).await,
            #[cfg(feature = "azure")]
            DynBackendKind::Azure(s) => s.put_verifier(permitter, identity, config).await,
            #[cfg(feature = "local")]
            DynBackendKind::Local(s) => s.put_verifier(permitter, identity, config).await,
        }
    }

    async fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
    ) -> Result<Option<Vec<u8>>, Error> {
        match &self.inner {
            DynBackendKind::Memory(s) => s.get_verifier(permitter, identity).await,
            #[cfg(feature = "aws")]
            DynBackendKind::Aws(s) => s.get_verifier(permitter, identity).await,
            #[cfg(feature = "azure")]
            DynBackendKind::Azure(s) => s.get_verifier(permitter, identity).await,
            #[cfg(feature = "local")]
            DynBackendKind::Local(s) => s.get_verifier(permitter, identity).await,
        }
    }

    #[cfg(test)]
    async fn clear_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
    ) -> Result<(), Error> {
        match &self.inner {
            DynBackendKind::Aws(s) => s.clear_verifier(permitter, identity).await,
            DynBackendKind::Azure(s) => s.clear_verifier(permitter, identity).await,
            DynBackendKind::Memory(s) => s.clear_verifier(permitter, identity).await,
            #[cfg(feature = "local")]
            DynBackendKind::Local(s) => s.clear_verifier(permitter, identity).await,
        }
    }
}

impl Signer for DynBackend {
    async fn sign(&self, hash: H256) -> Result<Signature, Error> {
        match &self.inner {
            DynBackendKind::Memory(s) => s.sign(hash).await,
            #[cfg(feature = "aws")]
            DynBackendKind::Aws(s) => s.sign(hash).await,
            #[cfg(feature = "azure")]
            DynBackendKind::Azure(s) => s.sign(hash).await,
            #[cfg(feature = "local")]
            DynBackendKind::Local(s) => s.sign(hash).await,
        }
    }

    async fn signer_address(&self) -> Result<Address, Error> {
        match &self.inner {
            DynBackendKind::Memory(s) => s.signer_address().await,
            #[cfg(feature = "aws")]
            DynBackendKind::Aws(s) => s.signer_address().await,
            #[cfg(feature = "azure")]
            DynBackendKind::Azure(s) => s.signer_address().await,
            #[cfg(feature = "local")]
            DynBackendKind::Local(s) => s.signer_address().await,
        }
    }
}

// #[derive(Debug, thiserror::Error)]
// #[error(transparent)]
// pub struct Error(#[from] anyhow::Error);
pub type Error = anyhow::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum StoreKind {
    Memory,
    #[cfg(feature = "aws")]
    Aws,
    #[cfg(feature = "local")]
    Local,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum Environment {
    Dev,
    Prod,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Dev => "dev",
                Self::Prod => "prod",
            }
        )
    }
}

#[allow(unused)]
pub async fn create(
    backend: StoreKind,
    env: Environment,
    host: &Authority,
) -> Result<impl Store + Signer, Error> {
    Ok(DynBackend {
        inner: match backend {
            StoreKind::Memory => DynBackendKind::Memory(memory::Backend::generate()),
            #[cfg(feature = "aws")]
            StoreKind::Aws => DynBackendKind::Aws(aws::Backend::connect(env).await),
            #[cfg(feature = "azure")]
            StoreKind::Aws => DynBackendKind::Azure(azure::Backend::connect(host, env).await?),
            #[cfg(feature = "local")]
            StoreKind::Local => todo!(),
        },
    })
}

pub trait ToKey {
    fn to_key(&self) -> String;
}

pub trait FromKey
where
    Self: Sized,
{
    fn from_key(key: &str) -> anyhow::Result<Self>;
}

impl ToKey for IdentityId {
    fn to_key(&self) -> String {
        format!("{:#x}", self.0)
    }
}

impl ToKey for IdentityLocator {
    fn to_key(&self) -> String {
        let Self {
            chain,
            registry,
            id: IdentityId(identity),
        } = &self;
        format!("{chain}-{registry:#x}-{identity:#x}")
    }
}

impl FromKey for IdentityLocator {
    fn from_key(key: &str) -> anyhow::Result<Self> {
        let mut parts = key.split('-');
        let Some(chain) = parts.next().and_then(|v| v.parse().ok()) else {
            anyhow::bail!("missing or invalid chain id");
        };
        let Some(registry) = parts.next().and_then(|v| v.parse().ok()) else {
            anyhow::bail!("missing or invalid registry");
        };
        let Some(id) = parts.next().and_then(|v| v.parse().ok()).map(IdentityId) else {
            anyhow::bail!("missing or invalid identity id");
        };
        anyhow::ensure!(parts.next().is_none(), "extra identity locator parts");
        Ok(Self {
            chain,
            registry,
            id,
        })
    }
}

impl ToKey for ShareId {
    fn to_key(&self) -> String {
        let Self {
            identity,
            secret_name,
            version: _,
        } = &self;
        (identity, secret_name.as_str()).to_key()
    }
}

impl ToKey for (&IdentityLocator, &str) {
    fn to_key(&self) -> String {
        let (identity, secret_name) = self;
        format!("share-{secret_name}-{}", identity.to_key())
    }
}

impl ToKey for KeyId {
    fn to_key(&self) -> String {
        let Self {
            name,
            identity,
            version: _,
        } = &self;
        format!("key-{name}-{}", identity.to_key())
    }
}

impl ToKey for PermitterLocator {
    fn to_key(&self) -> String {
        let Self { chain, permitter } = &self;
        format!("{chain}-{permitter:#x}")
    }
}

impl FromKey for PermitterLocator {
    fn from_key(key: &str) -> anyhow::Result<Self> {
        let mut parts = key.split('-');
        let Some(chain) = parts.next().and_then(|v| v.parse().ok()) else {
            anyhow::bail!("missing or invalid chain id");
        };
        let Some(permitter) = parts.next().and_then(|v| v.parse().ok()) else {
            anyhow::bail!("missing or invalid permitter");
        };
        anyhow::ensure!(parts.next().is_none(), "extra permitter locator parts");
        Ok(Self { chain, permitter })
    }
}

impl ToKey for Address {
    fn to_key(&self) -> String {
        format!("{self:#x}")
    }
}

impl ToKey for ChainId {
    fn to_key(&self) -> String {
        self.to_string()
    }
}

impl ToKey for () {
    fn to_key(&self) -> String {
        Default::default()
    }
}

impl ToKey for &[u8] {
    fn to_key(&self) -> String {
        hex::encode(self)
    }
}

#[cfg(any(feature = "aws", feature = "azure"))]
mod serde_key {
    use serde::{
        de::{self, Deserialize, Deserializer},
        ser::{Serialize, Serializer},
    };

    use super::{FromKey, ToKey};

    pub fn deserialize<'de, T: FromKey, D: Deserializer<'de>>(d: D) -> Result<T, D::Error> {
        let s = String::deserialize(d)?;
        T::from_key(&s).map_err(|e| de::Error::custom(e.to_string()))
    }

    pub fn serialize<T: ToKey, S: Serializer>(t: &T, s: S) -> Result<S::Ok, S::Error> {
        t.to_key().serialize(s)
    }
}

#[cfg(any(feature = "aws", feature = "azure"))]
fn signature_to_rsv(
    hash: H256,
    signer: Address,
    sig: ethers::core::k256::ecdsa::Signature,
) -> Signature {
    let eth_sig = Signature {
        r: <[u8; 32]>::from(sig.r().to_bytes()).into(),
        s: <[u8; 32]>::from(sig.s().to_bytes()).into(),
        v: 27,
    };

    eth_sig
        .verify(hash, signer)
        .map(|_| eth_sig)
        .unwrap_or_else(|_| Signature { v: 28, ..eth_sig })
}
