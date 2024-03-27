#![allow(unused)]
use std::sync::Arc;

use azure_core::Etag;
use azure_data_tables::prelude::*;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone)]
pub struct Client {
    env: Environment,

    secrets: Arc<azure_security_keyvault::SecretClient>,
    db: Arc<TableServiceClient>,
}

macro_rules! naming_fn {
    ($fn_name:ident, $prefix:literal) => {
        const fn $fn_name(&self) -> &'static str {
            match self.env {
                Environment::Dev => concat!($prefix, "-dev"),
                Environment::Prod => concat!($prefix, "-prod"),
            }
        }
    };
}

impl Client {
    pub async fn connect(
        account: String,
        host: &Authority,
        env: Environment,
    ) -> Result<Self, Error> {
        let creds = Arc::new(azure_identity::DefaultAzureCredential::default());
        let secrets = Arc::new(azure_security_keyvault::SecretClient::new(
            &format!("https://{env}.{host}.vault.azure.net"),
            creds.clone(),
        )?);
        let creds: Arc<dyn azure_core::auth::TokenCredential + 'static> = creds;
        let db = Arc::new(TableServiceClient::new(account, creds));

        Ok(Self { env, secrets, db })
    }

    naming_fn!(permits_table, "escrin-permits");
    naming_fn!(nonces_table, "escrin-nonces");
    naming_fn!(verifiers_table, "escrin-verifiers");
    naming_fn!(chain_state_table, "escrin-chain-state");

    async fn get_current_chain_state(
        &self,
        chain: ChainId,
    ) -> Result<Option<(Etag, ChainState)>, Error> {
        self.get_current(self.chain_state_table(), chain, None::<()>)
            .await
    }

    async fn get_current_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<Option<(Etag, Vec<u8>)>, Error> {
        self.get_current(self.verifiers_table(), permitter, Some(identity))
            .await
    }

    async fn get_current<T: serde::de::DeserializeOwned + Send + Sync>(
        &self,
        table: &'static str,
        partition_key: impl ToKey,
        row_key: Option<impl ToKey>,
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
        let mut share_versions = self
            .secrets
            .get_versions(id.identity.to_key())
            .into_stream();
        let mut current_version: u64 = 0;
        while let Some(versions) = share_versions.try_next().await? {
            current_version += versions.value.len() as u64;
        }
        if id.version != current_version + 1 {
            return Ok(false);
        }

        self.secrets
            .set(id.identity.to_key(), encode_ss(ss))
            .into_future()
            .await?;
        Ok(true)
    }

    async fn get_share(&self, id: ShareId) -> Result<Option<SecretShare>, Error> {
        let s = self
            .secrets
            .get(id.identity.to_key())
            .version(id.version.to_string())
            .into_future()
            .await?;
        if !s.attributes.enabled {
            return Ok(None);
        }
        decode_ss(s.value).map(Some)
    }

    async fn delete_share(&self, id: ShareId) -> Result<(), Error> {
        Ok(self
            .secrets
            .delete(id.identity.to_key())
            .into_future()
            .await?)
    }

    async fn put_key(&self, id: KeyId, key: WrappedKey) -> Result<bool, Error> {
        let mut key_versions = self.secrets.get_versions(id.to_key()).into_stream();
        let mut current_version: u64 = 0;
        while let Some(versions) = key_versions.try_next().await? {
            current_version += versions.value.len() as u64;
        }
        if id.version != current_version + 1 {
            return Ok(false);
        }

        self.secrets
            .set(id.identity.to_key(), hex::encode(&key.into_vec()))
            .into_future()
            .await?;
        Ok(true)
    }

    async fn get_key(&self, id: KeyId) -> Result<Option<WrappedKey>, Error> {
        let s = self
            .secrets
            .get(id.to_key())
            .version(id.version.to_string())
            .into_future()
            .await?;
        if !s.attributes.enabled {
            return Ok(None);
        }
        let key_hex = zeroize::Zeroizing::new(s.value);
        Ok(Some(hex::decode(&key_hex)?.into()))
    }

    async fn delete_key(&self, id: KeyId) -> Result<(), Error> {
        Ok(self
            .secrets
            .delete(id.identity.to_key())
            .into_future()
            .await?)
    }

    async fn create_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
        expiry: u64,
        nonce: Nonce,
    ) -> Result<Option<Permit>, Error> {
        let nonces = self.db.table_client(self.nonces_table());
        let nonce_key = nonce_to_key(&nonce);

        let nonce_used = nonces
            .query()
            .filter(format!("PartitionKey eq '{nonce_key}'"))
            .top(1)
            .into_stream::<Expiring<()>>()
            .try_next()
            .await?
            .and_then(|res| res.entities.into_iter().nth(0))
            .map(|e| e.expiry > now())
            .unwrap_or_default();
        if nonce_used {
            return Ok(None);
        }

        let nonce_res = nonces
            .insert::<_, ()>(Expiring {
                item: NonceEntity { nonce },
                expiry,
            })?
            .return_entity(false)
            .into_future()
            .await;

        let cleanup_nonce = || async {
            nonces
                .partition_key_client(&nonce_key)
                .entity_client("")
                .delete()
                .into_future()
                .await
                .ok();
        };

        if let Err(e) = nonce_res {
            cleanup_nonce().await;
            return Err(e.into());
        }

        match self.read_permit(identity, recipient).await {
            Ok(Some(_)) => return Ok(None),
            Ok(None) => {}
            Err(e) => {
                cleanup_nonce().await;
                return Err(e);
            }
        }

        let permit_res = self
            .db
            .table_client(self.permits_table())
            .insert::<_, ()>(Expiring {
                item: PermitEntity {
                    identity,
                    recipient,
                },
                expiry,
            })?
            .return_entity(false)
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
            .table_client(self.permits_table())
            .query()
            .filter(format!(
                "PartitionKey eq '{}' and RowKey eq '{}' and expiry gt {}",
                identity.to_key(),
                recipient.to_key(),
                now()
            ))
            .top(1)
            .into_stream::<Expiring<()>>()
            .try_next()
            .await?
            .and_then(|res| res.entities.into_iter().nth(0))
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
            .table_client(self.permits_table())
            .partition_key_client(identity.to_key())
            .entity_client(recipient.to_key())
            .delete()
            .into_future()
            .await?;
        Ok(())
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
        let current_chain_state = self.get_current_chain_state(chain).await?;
        // TODO: get the etag from the current state and do conditional update

        self.db
            .table_client(self.chain_state_table())
            .partition_key_client(chain.to_key())
            .entity_client("")
            .update(update, IfMatchCondition::Any)?
            .into_future()
            .await?;
        Ok(())
    }

    #[cfg(test)]
    async fn clear_chain_state(&self, chain: ChainId) -> Result<(), Error> {
        self.db
            .table_client(self.chain_state_table())
            .partition_key_client(chain.to_key())
            .entity_client(chain.to_key())
            .delete()
            .into_future()
            .await?;
        Ok(())
    }

    async fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<Option<Vec<u8>>, Error> {
        Ok(self
            .get_current_verifier(permitter, identity)
            .await?
            .map(|(_, v)| v))
    }

    async fn update_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
        config: Vec<u8>,
        EventIndex { block, log_index }: EventIndex,
    ) -> Result<(), Error> {
        let current_verifier = self.get_current_verifier(permitter, identity).await?;
        // TODO: get the etag from the current state and do conditional update
        // TODO: store block and log index for idempotence
        self.db
            .table_client(self.verifiers_table())
            .partition_key_client(permitter.to_key())
            .entity_client(identity.to_key())
            .update(
                VerifierEntity {
                    permitter,
                    identity,
                    config,
                },
                IfMatchCondition::Any,
            )?
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
            .table_client(self.verifiers_table())
            .partition_key_client(permitter.to_key())
            .entity_client(identity.to_key())
            .delete()
            .into_future()
            .await?;
        Ok(())
    }
}

fn nonce_to_key(nonce: &[u8]) -> String {
    hex::encode(nonce)
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Expiring<T> {
    #[serde(flatten)]
    item: T,
    expiry: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NonceEntity {
    #[serde(rename = "PartitionKey", with = "hex::serde")]
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
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::make_store_tests!(async {
        let ssss_host = std::env::var("SSSS_HOST").expect("SSSS_HOST must be set");
        Client::connect(
            "".into(),
            &Authority::try_from(ssss_host).unwrap(),
            Environment::Dev,
        )
        .await
        .unwrap()
    });
}
