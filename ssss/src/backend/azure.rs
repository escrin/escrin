use std::sync::Arc;

use azure_core::Etag;
use azure_data_tables::prelude::*;
use azure_security_keyvault::prelude::*;
use base64::prelude::*;
use ethers::core::k256::ecdsa;
use futures_util::{TryFutureExt as _, TryStreamExt as _};
use serde::{Deserialize, Serialize};

use super::*;
use crate::utils::now;

#[derive(Clone)]
pub struct Backend {
    secrets: Arc<SecretClient>,
    signer: Arc<KeyClient>,
    db: Arc<TableServiceClient>,
    signer_address: tokio::sync::OnceCell<Address>,
}

static SECRET_VERSIONS_TABLE: &str = "secretversions";
static VERIFIERS_TABLE: &str = "verifiers";
static KMS_KEY: &str = "escrin-signer";

impl Backend {
    pub async fn connect(host: &Authority, env: Environment) -> Result<Self, Error> {
        let unique_name = hex::encode(&<sha2::Sha256 as sha2::Digest>::digest(host.as_str())[0..8]);
        let sa_name = format!("{env}{unique_name}");
        let kv_url = format!("https://{env}{unique_name}.vault.azure.net");

        let creds = Arc::new(azure_identity::DefaultAzureCredential::default());
        let secrets = Arc::new(SecretClient::new(&kv_url, creds.clone())?);
        let signer = Arc::new(KeyClient::new(&kv_url, creds.clone())?);
        let creds: Arc<dyn azure_core::auth::TokenCredential + 'static> = creds;
        let db = Arc::new(TableServiceClient::new(sa_name, creds));

        Ok(Self {
            secrets,
            signer,
            db,
            signer_address: Default::default(),
        })
    }

    async fn get_current_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
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

    async fn get_secret_meta(
        &self,
        key: &impl ToKey,
        version: SecretVersion,
    ) -> Result<Option<SecretVersionEntity>, Error> {
        let version = match version {
            SecretVersion::Latest => None,
            SecretVersion::Numbered(n) => Some(InvSortableInt(n)),
        };
        self.get_current::<SecretVersionEntity>(SECRET_VERSIONS_TABLE, key, version.as_ref())
            .await
            .map(|res| res.map(|(_, e)| e))
    }

    async fn put_secret(
        &self,
        id: &impl ToKey,
        version: u64,
        secret: String,
        expiry: Option<u64>,
    ) -> Result<bool, Error> {
        let (current_version, is_pending) = self
            .get_secret_meta(id, SecretVersion::Latest)
            .await?
            .map(|m| (m.version.0, m.expiry.map(|e| e > now()).unwrap_or_default()))
            .unwrap_or_default();
        if version != current_version + 1 || is_pending {
            return Ok(false);
        }
        self.secrets.set(id.to_key(), secret).into_future().await?;
        let secret_entity = self.secrets.get(id.to_key()).into_future().await?;
        self.db
            .table_client(SECRET_VERSIONS_TABLE)
            .insert::<_, ()>(SecretVersionEntity {
                id: id.to_key(),
                version: InvSortableInt(version),
                guid: secret_entity.id.rsplit_once('/').unwrap().1.to_string(),
                expiry,
            })?
            .return_entity(false)
            .into_future()
            .await?;
        Ok(true)
    }

    async fn get_secret(
        &self,
        id: &impl ToKey,
        version: u64,
    ) -> Result<Option<(String, Option<u64> /* expiry */)>, Error> {
        let Some(m) = self
            .get_secret_meta(id, SecretVersion::Numbered(version))
            .await?
        else {
            return Ok(None);
        };
        if let Some(expiry) = m.expiry {
            if expiry != 0 && expiry <= now() {
                self.delete_secret_version(id, version).await?;
                return Ok(None);
            }
        }
        let res = self
            .secrets
            .get(id.to_key())
            .version(&m.guid)
            .into_future()
            .await;
        match res {
            Ok(s) => Ok(Some((s.value, m.expiry))),
            Err(e) => {
                if e.as_http_error().map(|e| e.status()) == Some(azure_core::StatusCode::Forbidden)
                {
                    Ok(None)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    async fn delete_secret_version(&self, id: &impl ToKey, version: u64) -> Result<(), Error> {
        let Some(m) = self
            .get_secret_meta(id, SecretVersion::Numbered(version))
            .await?
        else {
            return Ok(());
        };
        self.secrets
            .update(id.to_key())
            .version(m.guid)
            .enabled(false)
            .into_future()
            .await
            .or_else(default_if_notfound)
    }
}

impl Store for Backend {
    async fn put_share(&self, id: ShareId, ss: SecretShare) -> Result<bool, Error> {
        self.put_secret(
            &id,
            id.version,
            serde_json::to_string(&EncodableSecretShare::from(ss)).unwrap(),
            Some(now() + PRE_COMMIT_EXPIRY.as_secs()),
        )
        .await
    }

    async fn commit_share(&self, id: ShareId) -> Result<bool, Error> {
        if id.version > 1 {
            self.delete_share(ShareId {
                version: id.version - 1,
                ..id.clone()
            })
            .await?;
        }
        self.db
            .table_client(SECRET_VERSIONS_TABLE)
            .partition_key_client(id.to_key())
            .entity_client(InvSortableInt(id.version).to_key())
            .merge(
                serde_json::json!({
                    "expiry": 0
                }),
                IfMatchCondition::Any,
            )?
            .into_future()
            .await?;
        Ok(true)
    }

    async fn get_share(&self, id: ShareId) -> Result<Option<SecretShare>, Error> {
        let Some((s, expiry)) = self.get_secret(&id, id.version).await? else {
            return Ok(None);
        };
        if !matches!(expiry, Some(0) | None) {
            return Ok(None); // uncommitted
        }
        Ok(serde_json::from_str::<EncodableSecretShare>(&s)
            .ok()
            .map(Into::into))
    }

    async fn get_current_share_version(
        &self,
        identity: IdentityLocator,
        name: String,
    ) -> Result<Option<(ShareVersion, bool /* pending */)>, Error> {
        let Some(m) = self
            .get_secret_meta(&(&identity, name.as_str()), SecretVersion::Latest)
            .await?
        else {
            return Ok(None);
        };
        Ok(Some((
            m.version.0,
            m.expiry.map(|e| e > now()).unwrap_or_default(),
        )))
    }

    async fn delete_share(&self, id: ShareId) -> Result<(), Error> {
        self.delete_secret_version(&id, id.version).await
    }

    async fn put_secret(&self, id: KeyId, key: WrappedKey) -> Result<bool, Error> {
        self.put_secret(&id, id.version, hex::encode(&key), None)
            .await
    }

    async fn get_secret(&self, id: KeyId) -> Result<Option<WrappedKey>, Error> {
        let Some((k, _)) = self.get_secret(&id, id.version).await? else {
            return Ok(None);
        };
        Ok(Some(hex::decode(k)?.into()))
    }

    async fn delete_secret(&self, id: KeyId) -> Result<(), Error> {
        self.delete_secret_version(&id, id.version).await
    }

    async fn put_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
        config: Vec<u8>,
    ) -> Result<(), Error> {
        self.db
            .table_client(VERIFIERS_TABLE)
            .partition_key_client(permitter.to_key())
            .entity_client(identity.to_key())
            .insert_or_replace(VerifierEntity {
                permitter,
                identity,
                config,
            })?
            .into_future()
            .await?;
        Ok(())
    }

    async fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
    ) -> Result<Option<Vec<u8>>, Error> {
        Ok(self
            .get_current_verifier(permitter, identity)
            .await?
            .map(|(_, v)| v.config))
    }

    #[cfg(test)]
    async fn clear_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
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

impl Signer for Backend {
    async fn sign(&self, hash: H256) -> Result<Signature, Error> {
        let signer_addr_fut = self.signer_address();

        let sig_fut = self
            .signer
            .sign(
                KMS_KEY,
                SignatureAlgorithm::ES256K,
                BASE64_STANDARD.encode(hash),
            )
            .into_future()
            .map_err(Error::from)
            .and_then(|res| async move { Ok(ecdsa::Signature::from_slice(&res.signature)?) });

        let (signer_addr, sig) = tokio::try_join!(signer_addr_fut, sig_fut)?;

        Ok(signature_to_rsv(hash, signer_addr, sig))
    }

    async fn signer_address(&self) -> Result<Address, Error> {
        Ok(*self
            .signer_address
            .get_or_try_init(|| async {
                let pk_jwk = self.signer.get(KMS_KEY).into_future().await?.key;
                let mut pk_sec1_bytes = [0u8; 65];
                pk_sec1_bytes[0] = 0x04;
                pk_sec1_bytes[1..33].copy_from_slice(&pk_jwk.x.unwrap());
                pk_sec1_bytes[33..65].copy_from_slice(&pk_jwk.y.unwrap());
                let pk = ecdsa::VerifyingKey::from_sec1_bytes(&pk_sec1_bytes)?;
                Ok::<_, Error>(ethers::utils::public_key_to_address(&pk))
            })
            .await?)
    }
}

#[derive(Serialize, Deserialize)]
struct EncodableSecretShare {
    meta: SecretShareMeta,
    #[serde(with = "hex::serde")]
    share: Vec<u8>,
    #[serde(with = "hex::serde")]
    blinder: Vec<u8>,
}

impl From<SecretShare> for EncodableSecretShare {
    fn from(
        SecretShare {
            meta,
            share,
            blinder,
        }: SecretShare,
    ) -> Self {
        Self {
            meta,
            share: (*share).clone(),
            blinder: (*blinder).clone(),
        }
    }
}

impl From<EncodableSecretShare> for SecretShare {
    fn from(
        EncodableSecretShare {
            meta,
            share,
            blinder,
        }: EncodableSecretShare,
    ) -> Self {
        Self {
            meta,
            share: share.into(),
            blinder: blinder.into(),
        }
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
    #[serde(rename = "RowKey", with = "serde_key")]
    identity: IdentityLocator,
    #[serde(with = "hex::serde")]
    config: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SecretVersionEntity {
    #[serde(rename = "PartitionKey")]
    id: String,
    #[serde(rename = "RowKey")]
    version: InvSortableInt,
    guid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expiry: Option<u64>,
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

    crate::make_backend_tests!(async {
        let ssss_host = std::env::var("SSSS_HOST").unwrap_or("ssss.example.org".into());
        Backend::connect(&Authority::try_from(ssss_host).unwrap(), Environment::Dev)
            .await
            .unwrap()
    });
}
