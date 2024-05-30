use std::collections::HashMap;

use anyhow::anyhow;
use aws_sdk_dynamodb::{
    primitives::Blob,
    types::AttributeValue::{self, Bs, B, N, S},
};
use ethers::core::k256::ecdsa;
use futures_util::TryFutureExt as _;
use p384::pkcs8::DecodePublicKey as _;

use super::*;
use crate::utils::now;

#[derive(Clone)]
pub struct Backend {
    db: aws_sdk_dynamodb::Client,
    kms: aws_sdk_kms::Client,
    env: Environment,
    signer_address: tokio::sync::OnceCell<Address>,
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

impl Backend {
    pub async fn connect(env: Environment) -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::v2024_03_28()).await;
        let kms = aws_sdk_kms::Client::new(&config);
        let db = aws_sdk_dynamodb::Client::new(&config);
        Self {
            kms,
            db,
            env,
            signer_address: Default::default(),
        }
    }

    naming_fn!(secrets_table, "escrin-secrets");
    naming_fn!(verifiers_table, "escrin-verifiers");
    naming_fn!(kms_key, "alias/escrin-signer");

    async fn current_secret(
        &self,
        id: &impl ToAttributeValue,
        projection: &str,
    ) -> Result<Option<HashMap<String, AttributeValue>>, Error> {
        Ok(self
            .db
            .query()
            .table_name(self.secrets_table())
            .key_condition_expression("id = :id")
            .expression_attribute_values(":id", id.to_attribute_value())
            .projection_expression(projection)
            .scan_index_forward(false)
            .limit(1)
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .items
            .and_then(|items| items.into_iter().nth(0)))
    }

    async fn current_secret_version(
        &self,
        id: &impl ToAttributeValue,
    ) -> Result<Option<(ShareVersion, bool)>, Error> {
        Ok(self.current_secret(id, "version, expiry").await?.map(|v| {
            (
                unpack_u64("version", &v),
                try_unpack_u64("expiry", &v)
                    .map(|exp| exp > now())
                    .unwrap_or_default(),
            )
        }))
    }

    async fn put_secret(
        &self,
        id: &impl ToAttributeValue,
        version: u64,
        secret: Vec<u8>,
        extra_items: Option<HashMap<String, AttributeValue>>,
    ) -> Result<bool, Error> {
        let (current_version, is_pending) =
            self.current_secret_version(id).await?.unwrap_or_default();
        if is_pending || version != current_version + 1 {
            return Ok(false);
        }
        let mut items = extra_items.unwrap_or_default();
        items.insert("id".into(), id.to_attribute_value());
        items.insert("version".into(), N(version.to_string()));
        items.insert("secret".into(), B(Blob::new(secret)));
        let res = self
            .db
            .put_item()
            .table_name(self.secrets_table())
            .set_item(Some(items))
            .condition_expression("attribute_not_exists(id) AND attribute_not_exists(version)")
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from);
        match res {
            Ok(_) => Ok(true),
            Err(aws_sdk_dynamodb::Error::ConditionalCheckFailedException(_)) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_secret(
        &self,
        id: &impl ToAttributeValue,
        version: u64,
    ) -> Result<Option<(Vec<u8>, HashMap<String, AttributeValue>)>, Error> {
        let Some(mut res) = self
            .db
            .query()
            .table_name(self.secrets_table())
            .key_condition_expression("id = :id AND version = :version")
            .expression_attribute_values(":id", id.to_attribute_value())
            .expression_attribute_values(":version", N(version.to_string()))
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .items
            .unwrap_or_default()
            .into_iter()
            .nth(0)
            .and_then(|v| {
                let Some(expiry) = try_unpack_u64("expiry", &v) else {
                    return Some(v);
                };
                (expiry > now()).then_some(v)
            })
        else {
            return Ok(None);
        };
        let Some(secret) = try_unpack_blob("secret", &mut res) else {
            return Ok(None);
        };
        Ok(Some((secret.into_inner(), res)))
    }

    async fn delete_secret_version(
        &self,
        id: &impl ToAttributeValue,
        version: u64,
    ) -> Result<(), Error> {
        self.db
            .update_item()
            .table_name(self.secrets_table())
            .key("id", id.to_attribute_value())
            .key("version", N(version.to_string()))
            .update_expression("REMOVE secret, blinder")
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?;
        Ok(())
    }
}

impl Store for Backend {
    async fn put_share(&self, id: ShareId, ss: SecretShare) -> Result<bool, Error> {
        let mut extra_items = HashMap::with_capacity(4);
        extra_items.insert("blinder".into(), B(Blob::new((*ss.blinder).clone())));
        extra_items.insert("index".into(), N(ss.meta.index.to_string()));
        extra_items.insert(
            "commitments".into(),
            Bs(ss.meta.commitments.into_iter().map(Blob::new).collect()),
        );
        extra_items.insert(
            "expiry".into(),
            N((now() + PRE_COMMIT_EXPIRY.as_secs()).to_string()),
        );
        self.put_secret(&id, id.version, (*ss.share).clone(), Some(extra_items))
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
        let res = self
            .db
            .update_item()
            .table_name(self.secrets_table())
            .key("id", id.to_attribute_value())
            .key("version", N(id.version.to_string()))
            .condition_expression(
                "attribute_exists(id) AND attribute_exists(version) AND attribute_exists(secret)",
            )
            .update_expression("REMOVE expiry")
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from);
        match res {
            Ok(_) => Ok(true),
            Err(aws_sdk_dynamodb::Error::ConditionalCheckFailedException(_)) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_share(&self, id: ShareId) -> Result<Option<SecretShare>, Error> {
        let Some((share, mut items)) = self.get_secret(&id, id.version).await? else {
            return Ok(None);
        };
        if items.contains_key("expiry") {
            return Ok(None); // the share is uncommitted
        }
        Ok(Some(SecretShare {
            meta: unpack_secret_meta(&mut items),
            share: share.into(),
            blinder: unpack_blob("blinder", &mut items).into_inner().into(),
        }))
    }

    async fn get_current_share_version(
        &self,
        identity: IdentityLocator,
        name: String,
    ) -> Result<Option<(ShareVersion, bool)>, Error> {
        self.current_secret_version(&(&identity, name.as_str()))
            .await
    }

    async fn delete_share(&self, id: ShareId) -> Result<(), Error> {
        self.delete_secret_version(&id, id.version).await
    }

    async fn put_secret(&self, id: KeyId, key: WrappedKey) -> Result<bool, Error> {
        self.put_secret(&id, id.version, key.into_vec(), None).await
    }

    async fn get_secret(&self, id: KeyId) -> Result<Option<WrappedKey>, Error> {
        let Some((key, _)) = self.get_secret(&id, id.version).await? else {
            return Ok(None);
        };
        Ok(Some(key.into()))
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
        let res = self
            .db
            .put_item()
            .table_name(self.verifiers_table())
            .item("permitter", permitter.to_attribute_value())
            .item("identity", identity.to_attribute_value())
            .item("config", B(Blob::new(config)))
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from);
        match res {
            Ok(_) => Ok(()),
            Err(aws_sdk_dynamodb::Error::ConditionalCheckFailedException(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
    ) -> Result<Option<Vec<u8>>, Error> {
        Ok(self
            .db
            .query()
            .table_name(self.verifiers_table())
            .key_condition_expression("permitter = :permitter AND #i = :identity")
            .expression_attribute_names("#i", "identity")
            .expression_attribute_values(":permitter", permitter.to_attribute_value())
            .expression_attribute_values(":identity", identity.to_attribute_value())
            .projection_expression("config")
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .items
            .and_then(|items| items.into_iter().nth(0))
            .and_then(|mut res| {
                res.remove("config").map(|v| match v {
                    B(config) => config.into_inner(),
                    _ => panic!("expected blob config"),
                })
            }))
    }

    #[cfg(test)]
    async fn clear_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
    ) -> Result<(), Error> {
        self.db
            .delete_item()
            .table_name(self.verifiers_table())
            .key("permitter", permitter.to_attribute_value())
            .key("identity", identity.to_attribute_value())
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?;
        Ok(())
    }
}

impl Signer for Backend {
    async fn sign(&self, hash: H256) -> Result<Signature, Error> {
        let signer_addr_fut = self.signer_address();

        let sig_fut = self
            .kms
            .sign()
            .key_id(self.kms_key())
            .message_type(aws_sdk_kms::types::MessageType::Digest)
            .signing_algorithm(aws_sdk_kms::types::SigningAlgorithmSpec::EcdsaSha256)
            .message(Blob::new(hash.as_bytes().to_vec()))
            .send()
            .map_err(|e| Error::from(aws_sdk_kms::Error::from(e)))
            .and_then(|res| async {
                Ok(ecdsa::Signature::from_der(
                    &res.signature
                        .ok_or_else(|| anyhow!("failed to sign"))?
                        .into_inner(),
                )?)
            });

        let (signer_addr, sig) = tokio::try_join!(signer_addr_fut, sig_fut)?;

        Ok(signature_to_rsv(hash, signer_addr, sig))
    }

    async fn signer_address(&self) -> Result<Address, Error> {
        Ok(*self
            .signer_address
            .get_or_try_init(|| async {
                let pk_der = self
                    .kms
                    .get_public_key()
                    .key_id(self.kms_key())
                    .send()
                    .await
                    .map_err(aws_sdk_kms::Error::from)?
                    .public_key
                    .ok_or_else(|| anyhow!("failed to get public key"))?
                    .into_inner();
                let pk = ecdsa::VerifyingKey::from_public_key_der(&pk_der)
                    .map_err(|e| anyhow!("failed to parse public key: {e}"))?;
                Ok::<_, Error>(ethers::core::utils::public_key_to_address(&pk))
            })
            .await?)
    }
}

fn try_unpack_u64(key: &'static str, res: &HashMap<String, AttributeValue>) -> Option<u64> {
    res.get(key)?.as_n().ok()?.parse::<u64>().ok()
}

fn try_unpack_blob(key: &'static str, res: &mut HashMap<String, AttributeValue>) -> Option<Blob> {
    let B(b) = res.remove(key)? else {
        return None;
    };
    Some(b)
}

fn unpack_u64(key: &'static str, res: &HashMap<String, AttributeValue>) -> u64 {
    try_unpack_u64(key, res).expect(key)
}

fn unpack_blob(key: &'static str, res: &mut HashMap<String, AttributeValue>) -> Blob {
    try_unpack_blob(key, res).expect(key)
}

fn unpack_blobs(key: &'static str, res: &mut HashMap<String, AttributeValue>) -> Vec<Blob> {
    match res
        .remove(key)
        .unwrap_or_else(|| panic!("{key} not present"))
    {
        Bs(bs) => bs,
        _ => panic!("{key} not blob"),
    }
}

fn unpack_secret_meta(res: &mut HashMap<String, AttributeValue>) -> SecretShareMeta {
    SecretShareMeta {
        index: unpack_u64("index", res),
        commitments: unpack_blobs("commitments", res)
            .into_iter()
            .map(Blob::into_inner)
            .collect(),
    }
}

trait ToAttributeValue {
    fn to_attribute_value(&self) -> AttributeValue;
}

impl<T: ToKey> ToAttributeValue for T {
    fn to_attribute_value(&self) -> AttributeValue {
        S(self.to_key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::make_backend_tests!(Backend::connect(Environment::Dev));
}
