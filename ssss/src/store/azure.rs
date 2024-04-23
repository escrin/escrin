#![allow(unused)]
use std::sync::Arc;

use azure_core::Etag;
use azure_data_tables::prelude::*;
use futures_util::{StreamExt as _, TryStreamExt as _};
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone)]
pub struct Client {
    env: Environment,
    secrets: Arc<azure_security_keyvault::SecretClient>,
    db: Arc<TableServiceClient>,
}

static SECRET_VERSIONS_TABLE: &str = "secretversions";
static PERMITS_TABLE: &str = "permits";
static NONCES_TABLE: &str = "nonces";
static VERIFIERS_TABLE: &str = "verifiers";
static CHAIN_STATE_TABLE: &str = "chainstate";

impl Client {
    pub async fn connect(host: &Authority, env: Environment) -> Result<Self, Error> {
        let unique_name = hex::encode(&<sha2::Sha256 as sha2::Digest>::digest(host.as_str())[0..8]);
        let sa_name = format!("{env}{unique_name}");
        let kv_url = format!("https://{env}{unique_name}.vault.azure.net");

        let creds = Arc::new(azure_identity::DefaultAzureCredential::default());
        let secrets = Arc::new(azure_security_keyvault::SecretClient::new(
            &kv_url,
            creds.clone(),
        )?);
        let creds: Arc<dyn azure_core::auth::TokenCredential + 'static> = creds;
        let db = Arc::new(TableServiceClient::new(sa_name, creds));

        Ok(Self { env, secrets, db })
    }

    async fn get_current_chain_state(
        &self,
        chain: ChainId,
    ) -> Result<Option<(Etag, ChainState)>, Error> {
        self.get_current(CHAIN_STATE_TABLE, &chain, None::<&()>)
            .await
    }

    async fn get_current_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<Option<(Etag, VerifierEntity)>, Error> {
        self.get_current(VERIFIERS_TABLE, &permitter, Some(&identity))
            .await
    }

    async fn get_current<T: serde::de::DeserializeOwned + Send + Sync>(
        &self,
        table: &'static str,
        partition_key: &impl ToKey,
        row_key: Option<&impl ToKey>,
    ) -> Result<Option<(Etag, T)>, Error> {
        let partition_key_filter = format!("PartitionKey eq '{}'", partition_key.to_key());
        let filter = match row_key {
            Some(row_key) => format!(
                "{partition_key_filter} and RowKey eq '{}'",
                row_key.to_key()
            ),
            None => partition_key_filter,
        };
        let Some(res) = self
            .db
            .table_client(table)
            .query()
            .filter(filter)
            .top(1)
            .into_stream::<T>()
            .try_next()
            .await?
        else {
            return Ok(None);
        };
        let Some(entity) = res.entities.into_iter().nth(0) else {
            return Ok(None);
        };
        Ok(Some(("".parse().unwrap(), entity)))
    }

    async fn get_secret_guid(
        &self,
        key: &impl ToKey,
        version: SecretVersion,
    ) -> Result<Option<(u64, String)>, Error> {
        let version = match version {
            SecretVersion::Latest => None,
            SecretVersion::Numbered(n) => Some(InvSortableInt(n)),
        };
        let Some((_, SecretVersionEntity { version, guid, .. })) = self
            .get_current::<SecretVersionEntity>(SECRET_VERSIONS_TABLE, key, version.as_ref())
            .await?
        else {
            return Ok(None);
        };
        Ok(Some((version.0, guid)))
    }

    async fn put_secret(
        &self,
        id: &impl ToKey,
        version: u64,
        secret: String,
    ) -> Result<bool, Error> {
        let (current_version, guid) = self
            .get_secret_guid(id, SecretVersion::Latest)
            .await?
            .unwrap_or_default();
        if version != current_version + 1 {
            return Ok(false);
        }
        self.secrets.set(id.to_key(), secret).into_future().await?;
        let secret = self.secrets.get(id.to_key()).into_future().await?;
        self.db
            .table_client(SECRET_VERSIONS_TABLE)
            .insert::<_, ()>(SecretVersionEntity {
                id: id.to_key(),
                version: InvSortableInt(version),
                guid: secret.id.rsplit_once('/').unwrap().1.to_string(),
            })?
            .return_entity(false)
            .into_future()
            .await;
        Ok(true)
    }

    async fn get_secret(&self, id: &impl ToKey, version: u64) -> Result<Option<String>, Error> {
        let Some((_, guid)) = self
            .get_secret_guid(id, SecretVersion::Numbered(version))
            .await?
        else {
            return Ok(None);
        };
        let s = self
            .secrets
            .get(id.to_key())
            .version(&guid)
            .into_future()
            .await?;
        if !s.attributes.enabled {
            return Ok(None);
        }
        Ok(Some(s.value))
    }

    async fn delete_secret_version(&self, id: &impl ToKey, version: u64) -> Result<(), Error> {
        let Some((_, guid)) = self
            .get_secret_guid(id, SecretVersion::Numbered(version))
            .await?
        else {
            return Ok(());
        };
        self.secrets
            .update(id.to_key())
            .version(guid)
            .enabled(false)
            .into_future()
            .await
            .or_else(default_if_notfound)
    }
}

fn encode_ss(SecretShare { index, share }: SecretShare) -> String {
    format!("{index}-{}", hex::encode(&share))
}

fn decode_ss(s: String) -> Result<SecretShare, Error> {
    let s = zeroize::Zeroizing::new(s);
    let Some((index_str, share_hex)) = s.split_once('-') else {
        return Err(anyhow::anyhow!("invalid encoded secret share"));
    };
    let index = index_str.parse()?;
    let share = hex::decode(share_hex)?.into();
    Ok(SecretShare { index, share })
}

impl Store for Client {
    async fn put_share(&self, id: ShareId, ss: SecretShare) -> Result<bool, Error> {
        self.put_secret(&id, id.version, encode_ss(ss)).await
    }

    async fn get_share(&self, id: ShareId) -> Result<Option<SecretShare>, Error> {
        let Some(s) = self.get_secret(&id, id.version).await? else {
            return Ok(None);
        };
        decode_ss(s).map(Some)
    }

    async fn delete_share_version(&self, id: ShareId) -> Result<(), Error> {
        self.delete_secret_version(&id, id.version).await
    }

    async fn put_key(&self, id: KeyId, key: WrappedKey) -> Result<bool, Error> {
        self.put_secret(&id, id.version, hex::encode(&key)).await
    }

    async fn get_key(&self, id: KeyId) -> Result<Option<WrappedKey>, Error> {
        let Some(k) = self.get_secret(&id, id.version).await? else {
            return Ok(None);
        };
        Ok(Some(hex::decode(k)?.into()))
    }

    async fn delete_key_version(&self, id: KeyId) -> Result<(), Error> {
        self.delete_secret_version(&id, id.version).await
    }

    async fn create_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
        expiry: u64,
        nonce: Nonce,
    ) -> Result<Option<Permit>, Error> {
        let nonces = self
            .db
            .table_client(NONCES_TABLE)
            .partition_key_client(identity.to_key())
            .entity_client(nonce.as_slice().to_key());

        let nonce_used = nonces
            .get::<Expiring<()>>()
            .into_future()
            .await
            .map(|res| res.entity.expiry > now())
            .or_else(default_if_notfound)?;
        if nonce_used {
            return Ok(None);
        }

        let nonce_res = nonces
            .insert_or_replace(Expiring { item: (), expiry })?
            .into_future()
            .await;

        let cleanup_nonce = || async {
            nonces.delete().into_future().await.ok();
        };

        if let Err(e) = nonce_res {
            cleanup_nonce().await;
            return Err(e.into());
        }

        match self.read_permit(identity, recipient).await {
            Ok(Some(Permit {
                expiry: prev_expiry,
            })) if prev_expiry >= expiry => return Ok(None),
            Err(e) => {
                cleanup_nonce().await;
                return Err(e);
            }
            _ => {}
        }

        let permit_res = self
            .db
            .table_client(PERMITS_TABLE)
            .partition_key_client(identity.to_key())
            .entity_client(recipient.to_key())
            .insert_or_merge(Expiring { item: (), expiry })?
            .into_future()
            .await;

        if let Err(e) = permit_res {
            cleanup_nonce().await;
            return Err(e.into());
        }

        Ok(Some(Permit { expiry }))
    }

    async fn read_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
    ) -> Result<Option<Permit>, Error> {
        let Some(permit) = self
            .db
            .table_client(PERMITS_TABLE)
            .partition_key_client(identity.to_key())
            .entity_client(recipient.to_key())
            .get::<Expiring<()>>()
            .into_future()
            .await
            .map(|res| (res.entity.expiry > now()).then_some(res.entity))
            .or_else(default_if_notfound)?
        else {
            return Ok(None);
        };
        Ok(Some(Permit {
            expiry: permit.expiry,
        }))
    }

    async fn delete_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
    ) -> Result<(), Error> {
        self.db
            .table_client(PERMITS_TABLE)
            .partition_key_client(identity.to_key())
            .entity_client(recipient.to_key())
            .delete()
            .into_future()
            .await
            .map(|_| ())
            .or_else(default_if_notfound)
    }

    async fn get_chain_state(&self, chain: ChainId) -> Result<Option<ChainState>, Error> {
        Ok(self
            .get_current_chain_state(chain)
            .await?
            .map(|(etag, state)| state))
    }

    async fn update_chain_state(
        &self,
        chain: ChainId,
        update: ChainStateUpdate,
    ) -> Result<(), Error> {
        let Some(block) = update.block else {
            return Ok(());
        };
        // TODO: use etag and conditional insert once etag is supported
        let current_chain_state = self
            .get_current_chain_state(chain)
            .await?
            .map(|(_, s)| s)
            .unwrap_or_default();
        if current_chain_state.block >= block {
            return Ok(());
        }

        self.db
            .table_client(CHAIN_STATE_TABLE)
            .partition_key_client(chain.to_key())
            .entity_client("")
            .insert_or_merge(update)?
            .into_future()
            .await?;
        Ok(())
    }

    #[cfg(test)]
    async fn clear_chain_state(&self, chain: ChainId) -> Result<(), Error> {
        self.db
            .table_client(CHAIN_STATE_TABLE)
            .partition_key_client(chain.to_key())
            .entity_client("")
            .delete()
            .into_future()
            .await
            .map(|_| ())
            .or_else(default_if_notfound)
    }

    async fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<Option<Vec<u8>>, Error> {
        Ok(self
            .get_current_verifier(permitter, identity)
            .await?
            .map(|(_, v)| v.config))
    }

    async fn update_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
        config: Vec<u8>,
        EventIndex { block, log_index }: EventIndex,
    ) -> Result<(), Error> {
        let current_verifier_ix = self
            .get_current_verifier(permitter, identity)
            .await?
            .map(|(_, v)| (v.block, v.log_index))
            .unwrap_or_default();
        if current_verifier_ix >= (block, log_index) {
            return Ok(());
        }
        self.db
            .table_client(VERIFIERS_TABLE)
            .partition_key_client(permitter.to_key())
            .entity_client(identity.to_key())
            .insert_or_replace(VerifierEntity {
                permitter,
                identity,
                config,
                block,
                log_index,
            })?
            .into_future()
            .await?;
        Ok(())
    }

    #[cfg(test)]
    async fn clear_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<(), Error> {
        self.db
            .table_client(VERIFIERS_TABLE)
            .partition_key_client(permitter.to_key())
            .entity_client(identity.to_key())
            .delete()
            .into_future()
            .await
            .map(|_| ())
            .or_else(default_if_notfound)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SecretVersion {
    Latest,
    Numbered(u64),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Expiring<T> {
    #[serde(flatten)]
    item: T,
    expiry: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NonceEntity {
    #[serde(rename = "PartitionKey", with = "serde_key")]
    identity: IdentityLocator,
    #[serde(rename = "RowKey", with = "hex::serde")]
    nonce: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct PermitEntity {
    #[serde(rename = "PartitionKey", with = "serde_key")]
    identity: IdentityLocator,
    #[serde(rename = "RowKey")]
    recipient: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct VerifierEntity {
    #[serde(rename = "PartitionKey", with = "serde_key")]
    permitter: PermitterLocator,
    #[serde(rename = "RowKey")]
    identity: IdentityId,
    #[serde(with = "hex::serde")]
    config: Vec<u8>,
    block: u64,
    log_index: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SecretVersionEntity {
    #[serde(rename = "PartitionKey")]
    id: String,
    #[serde(rename = "RowKey")]
    version: InvSortableInt,
    guid: String,
}

/// An integer that sorts inverse numerically when stringified,
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
struct InvSortableInt(u64);

impl std::fmt::Display for InvSortableInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode((u64::MAX - self.0).to_be_bytes()))
    }
}

impl From<InvSortableInt> for String {
    fn from(v: InvSortableInt) -> Self {
        format!("{}", v)
    }
}

impl TryFrom<String> for InvSortableInt {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let mut b = [0u8; 8];
        hex::decode_to_slice(&s, &mut b)?;
        Ok(Self(u64::MAX - u64::from_be_bytes(b)))
    }
}

impl ToKey for InvSortableInt {
    fn to_key(&self) -> String {
        self.to_string()
    }
}

fn default_if_notfound<T: Default>(e: azure_core::Error) -> Result<T, Error> {
    match e.kind() {
        azure_core::error::ErrorKind::HttpResponse {
            status: azure_core::StatusCode::NotFound,
            ..
        } => Ok(Default::default()),
        _ => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::make_store_tests!(async {
        let ssss_host = std::env::var("SSSS_HOST").expect("SSSS_HOST must be set");
        Client::connect(&Authority::try_from(ssss_host).unwrap(), Environment::Dev)
            .await
            .unwrap()
    });
}
