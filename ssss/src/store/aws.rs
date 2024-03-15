use std::collections::HashMap;

use aws_sdk_dynamodb::{
    primitives::Blob,
    types::{
        AttributeValue::{self, B, N, S},
        Put, TransactWriteItem,
    },
};

use super::*;

#[derive(Clone)]
pub struct Client {
    kms_client: aws_sdk_kms::Client,
    db_client: aws_sdk_dynamodb::Client,
    env: Environment,
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
    pub async fn connect(env: Environment) -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::v2023_11_09()).await;
        let kms_client = aws_sdk_kms::Client::new(&config);
        let db_client = aws_sdk_dynamodb::Client::new(&config);
        Self {
            kms_client,
            db_client,
            env,
        }
    }

    naming_fn!(shares_table, "escrin-shares");
    naming_fn!(keys_table, "escrin-keys");
    naming_fn!(permits_table, "escrin-permits");
    naming_fn!(nonces_table, "escrin-nonces");
    naming_fn!(verifiers_table, "escrin-verifiers");
    naming_fn!(chain_state_table, "escrin-chain-state");
    naming_fn!(kms_key, "alias/escrin-sek");

    async fn current_share_version(&self, id: ShareId) -> Result<Option<ShareVersion>, Error> {
        Ok(self
            .db_client
            .query()
            .table_name(self.shares_table())
            .key_condition_expression("id = :id")
            .expression_attribute_values(":id", id.identity.to_attribute_value())
            .projection_expression("version")
            .scan_index_forward(false)
            .limit(1)
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .items()
            .first()
            .map(|v| unpack_u64("version", v)))
    }

    async fn current_key_version(&self, id: &KeyId) -> Result<Option<KeyVersion>, Error> {
        Ok(self
            .db_client
            .query()
            .table_name(self.keys_table())
            .key_condition_expression("id = :id")
            .expression_attribute_values(":id", id.to_attribute_value())
            .projection_expression("version")
            .scan_index_forward(false)
            .limit(1)
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .items()
            .first()
            .map(|v| unpack_u64("version", v)))
    }
}

impl Store for Client {
    async fn put_share(&self, id: ShareId, ss: SecretShare) -> Result<bool, Error> {
        let current_version = self.current_share_version(id).await?.unwrap_or_default();
        if id.version != current_version + 1 {
            return Ok(false);
        }

        let enc_share = self
            .kms_client
            .encrypt()
            .key_id(self.kms_key())
            .plaintext(Blob::new((*ss.share).clone()))
            .encryption_context("share", id.to_encryption_context())
            .send()
            .await
            .map_err(aws_sdk_kms::Error::from)?
            .ciphertext_blob
            .unwrap();

        let res = self
            .db_client
            .put_item()
            .table_name(self.shares_table())
            .item("id", id.to_attribute_value())
            .item("version", N(id.version.to_string()))
            .item("index", N(ss.index.to_string()))
            .item("share", B(enc_share))
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

    async fn get_share(&self, id: ShareId) -> Result<Option<SecretShare>, Error> {
        let Some((index, enc_share)) = self
            .db_client
            .query()
            .table_name(self.shares_table())
            .key_condition_expression("id = :id AND version = :version")
            .expression_attribute_values(":id", id.to_attribute_value())
            .expression_attribute_values(":version", N(id.version.to_string()))
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .items
            .unwrap_or_default()
            .into_iter()
            .nth(0)
            .map(|mut v| {
                let index = unpack_u64("index", &v);
                let enc_share = unpack_blob("share", &mut v);
                (index, enc_share)
            })
        else {
            return Ok(None);
        };

        let share = self
            .kms_client
            .decrypt()
            .key_id(self.kms_key())
            .ciphertext_blob(enc_share)
            .encryption_context("share", id.to_encryption_context())
            .send()
            .await
            .map_err(aws_sdk_kms::Error::from)?
            .plaintext
            .unwrap()
            .into_inner();

        Ok(Some(SecretShare {
            index,
            share: share.into(),
        }))
    }

    async fn delete_share(&self, id: ShareId) -> Result<(), Error> {
        self.db_client
            .delete_item()
            .table_name(self.shares_table())
            .key("id", id.to_attribute_value())
            .key("version", N(id.version.to_string()))
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)
            .unwrap();
        Ok(())
    }

    async fn put_key(&self, id: KeyId, key: WrappedKey) -> Result<bool, Error> {
        let current_version = self.current_key_version(&id).await?.unwrap_or_default();
        if id.version != current_version + 1 {
            return Ok(false);
        }

        let enc_key = self
            .kms_client
            .encrypt()
            .key_id(self.kms_key())
            .plaintext(Blob::new(key.into_vec()))
            .encryption_context("key", id.to_encryption_context())
            .send()
            .await
            .map_err(aws_sdk_kms::Error::from)?
            .ciphertext_blob
            .unwrap();

        let res = self
            .db_client
            .put_item()
            .table_name(self.keys_table())
            .item("id", id.to_attribute_value())
            .item("version", N(id.version.to_string()))
            .item("key", B(enc_key))
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

    async fn get_key(&self, id: KeyId) -> Result<Option<WrappedKey>, Error> {
        let Some(enc_key) = self
            .db_client
            .query()
            .table_name(self.keys_table())
            .key_condition_expression("id = :id AND version = :version")
            .expression_attribute_values(":id", id.to_attribute_value())
            .expression_attribute_values(":version", N(id.version.to_string()))
            .projection_expression("#k")
            .expression_attribute_names("#k", "key")
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .items
            .unwrap_or_default()
            .into_iter()
            .nth(0)
            .map(|mut v| unpack_blob("key", &mut v))
        else {
            return Ok(None);
        };

        Ok(Some(
            self.kms_client
                .decrypt()
                .key_id(self.kms_key())
                .ciphertext_blob(enc_key)
                .encryption_context("key", id.to_encryption_context())
                .send()
                .await
                .map_err(aws_sdk_kms::Error::from)?
                .plaintext
                .unwrap()
                .into_inner()
                .into(),
        ))
    }

    async fn delete_key(&self, id: KeyId) -> Result<(), Error> {
        self.db_client
            .delete_item()
            .table_name(self.keys_table())
            .key("id", id.to_attribute_value())
            .key("version", N(id.version.to_string()))
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)
            .unwrap();
        Ok(())
    }

    async fn create_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
        expiry: u64,
        nonce: Nonce,
    ) -> Result<Option<Permit>, Error> {
        let b_nonce = B(Blob::new(nonce));
        let n_exp = N(expiry.to_string());

        let nonce_used = self
            .db_client
            .get_item()
            .table_name(self.nonces_table())
            .key("nonce", b_nonce.clone())
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .item
            .and_then(|v| (unpack_u64("expiry", &v) > now()).then_some(()))
            .is_some();
        if nonce_used {
            return Ok(None);
        }

        macro_rules! add_put_nonce_items {
            ($inp:expr) => {
                $inp.table_name(self.nonces_table())
                    .item("nonce", b_nonce.clone())
                    .item("expiry", n_exp.clone())
            };
        }

        let res = self
            .db_client
            .transact_write_items()
            .transact_items(
                TransactWriteItem::builder()
                    .put(add_put_nonce_items!(Put::builder()).build()?)
                    .build(),
            )
            .transact_items(
                TransactWriteItem::builder()
                    .put(
                        Put::builder()
                            .table_name(self.permits_table())
                            .item("identity", identity.to_attribute_value())
                            .item("expiry", n_exp.clone())
                            .item("recipient", address_to_value(&recipient))
                            .condition_expression(
                                "attribute_not_exists(expiry) OR expiry < :expiry",
                            )
                            .expression_attribute_values(":expiry", n_exp.clone())
                            .build()?,
                    )
                    .build(),
            )
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from);
        match res {
            Ok(_) => Ok(Some(Permit { expiry })),
            Err(aws_sdk_dynamodb::Error::TransactionCanceledException(_)) => {
                // The current expiry is later than the provided one, but set the nonce anyway.
                add_put_nonce_items!(self.db_client.put_item())
                    .send()
                    .await
                    .map_err(aws_sdk_dynamodb::Error::from)?;
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn read_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
    ) -> Result<Option<Permit>, Error> {
        Ok(self
            .db_client
            .query()
            .table_name(self.permits_table())
            .key_condition_expression("#i = :identity AND recipient = :recipient")
            .expression_attribute_names("#i", "identity")
            .expression_attribute_values(":identity", identity.to_attribute_value())
            .expression_attribute_values(":recipient", address_to_value(&recipient))
            .projection_expression("expiry")
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .items()
            .first()
            .and_then(|v| {
                let expiry = unpack_u64("expiry", v);
                (expiry > now()).then_some(Permit { expiry })
            }))
    }

    async fn delete_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
    ) -> Result<(), Error> {
        self.db_client
            .delete_item()
            .table_name(self.permits_table())
            .key("identity", identity.to_attribute_value())
            .key("recipient", address_to_value(&recipient))
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?;
        Ok(())
    }

    async fn get_chain_state(&self, chain: u64) -> Result<Option<ChainState>, Error> {
        Ok(self
            .db_client
            .query()
            .table_name(self.chain_state_table())
            .key_condition_expression("chain = :chain")
            .expression_attribute_values(":chain", N(chain.to_string()))
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .items()
            .first()
            .map(|v| ChainState {
                block: unpack_u64("block", v),
            }))
    }

    async fn update_chain_state(&self, chain: u64, update: ChainStateUpdate) -> Result<(), Error> {
        let ChainStateUpdate { block } = update;
        let Some(new_block) = block else {
            return Ok(());
        };

        let n_block = N(new_block.to_string());
        let res = self
            .db_client
            .put_item()
            .table_name(self.chain_state_table())
            .item("chain", N(chain.to_string()))
            .item("block", n_block.clone())
            .condition_expression("attribute_not_exists(#b) OR #b < :block")
            .expression_attribute_names("#b", "block")
            .expression_attribute_values(":block", n_block)
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from);
        match res {
            Ok(_) => Ok(()),
            Err(aws_sdk_dynamodb::Error::ConditionalCheckFailedException(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[cfg(test)]
    async fn clear_chain_state(&self, chain: u64) -> Result<(), Error> {
        self.db_client
            .delete_item()
            .table_name(self.chain_state_table())
            .key("chain", N(chain.to_string()))
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?;
        Ok(())
    }

    async fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<Option<Vec<u8>>, Error> {
        Ok(self
            .db_client
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

    async fn update_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
        config: Vec<u8>,
        EventIndex { block, log_index }: EventIndex,
    ) -> Result<(), Error> {
        let n_block = N(block.to_string());
        let n_log_index = N(log_index.to_string());
        let res = self
            .db_client
            .put_item()
            .table_name(self.verifiers_table())
            .item("permitter", permitter.to_attribute_value())
            .item("identity", identity.to_attribute_value())
            .item("config", B(Blob::new(config)))
            .item("block", n_block.clone())
            .item("log_index", n_log_index.clone())
            .condition_expression(
                "attribute_not_exists(#b) OR #b < :block OR (#b = :block AND log_index < :li)",
            )
            .expression_attribute_names("#b", "block")
            .expression_attribute_values(":block", n_block)
            .expression_attribute_values(":li", n_log_index)
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from);
        match res {
            Ok(_) => Ok(()),
            Err(aws_sdk_dynamodb::Error::ConditionalCheckFailedException(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[cfg(test)]
    async fn clear_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<(), Error> {
        self.db_client
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

fn unpack_u64(key: &'static str, res: &HashMap<String, AttributeValue>) -> u64 {
    res.get(key)
        .expect(key)
        .as_n()
        .unwrap()
        .parse::<u64>()
        .unwrap()
}

fn unpack_blob(key: &'static str, res: &mut HashMap<String, AttributeValue>) -> Blob {
    match res
        .remove(key)
        .unwrap_or_else(|| panic!("{key} not present"))
    {
        B(b) => b,
        _ => panic!("{key} not blob"),
    }
}

fn address_to_value(addr: &Address) -> AttributeValue {
    S(format!("{addr:#x}"))
}

pub trait ToAttributeValue {
    fn to_key(&self) -> String;

    fn to_attribute_value(&self) -> AttributeValue {
        S(self.to_key())
    }
}

pub trait ToEncryptionContext {
    fn to_encryption_context(&self) -> String;
}

impl ToAttributeValue for IdentityId {
    fn to_key(&self) -> String {
        format!("{:#x}", self.0)
    }
}

impl ToAttributeValue for IdentityLocator {
    fn to_key(&self) -> String {
        let Self {
            chain,
            registry,
            id: IdentityId(identity),
        } = &self;
        format!("{chain}-{registry:#x}-{identity:#x}")
    }
}

impl ToAttributeValue for ShareId {
    fn to_key(&self) -> String {
        self.identity.to_key()
    }
}

impl ToEncryptionContext for ShareId {
    fn to_encryption_context(&self) -> String {
        let Self { identity, version } = &self;
        format!("{}-{version}", identity.to_key())
    }
}

impl ToAttributeValue for KeyId {
    fn to_key(&self) -> String {
        let Self {
            name,
            identity,
            version: _,
        } = &self;
        format!("{}-{name}", identity.to_key())
    }
}

impl ToEncryptionContext for KeyId {
    fn to_encryption_context(&self) -> String {
        let Self {
            name,
            identity,
            version,
        } = &self;
        format!("{}-{name}-{version}", identity.to_key())
    }
}

impl ToAttributeValue for PermitterLocator {
    fn to_key(&self) -> String {
        let Self { chain, permitter } = &self;
        format!("{chain}-{permitter:#x}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::make_store_tests!(Client::connect(Environment::Dev).await);
}
