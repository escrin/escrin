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
    db: aws_sdk_dynamodb::Client,
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
        let db_client = aws_sdk_dynamodb::Client::new(&config);
        Self { db: db_client, env }
    }

    naming_fn!(secrets_table, "escrin-secrets");
    naming_fn!(permits_table, "escrin-permits");
    naming_fn!(nonces_table, "escrin-nonces");
    naming_fn!(verifiers_table, "escrin-verifiers");
    naming_fn!(chain_state_table, "escrin-chain-state");

    async fn current_secret_version(
        &self,
        id: &impl ToAttributeValue,
    ) -> Result<Option<ShareVersion>, Error> {
        Ok(self
            .db
            .query()
            .table_name(self.secrets_table())
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

    async fn put_secret(
        &self,
        id: &impl ToAttributeValue,
        version: u64,
        secret: Vec<u8>,
        extra_items: Option<HashMap<String, AttributeValue>>,
    ) -> Result<bool, Error> {
        let current_version = self.current_secret_version(id).await?.unwrap_or_default();
        if version != current_version + 1 {
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
        else {
            return Ok(None);
        };
        let secret = unpack_blob("secret", &mut res).into_inner();
        Ok(Some((secret, res)))
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
            .update_expression("REMOVE secret")
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)
            .unwrap();
        Ok(())
    }
}

impl Store for Client {
    async fn put_share(&self, id: ShareId, ss: SecretShare) -> Result<bool, Error> {
        self.put_secret(
            &id,
            id.version,
            (*ss.share).clone(),
            Some(HashMap::from_iter(std::iter::once((
                "index".to_string(),
                N(ss.index.to_string()),
            )))),
        )
        .await
    }

    async fn get_share(&self, id: ShareId) -> Result<Option<SecretShare>, Error> {
        let Some((share, items)) = self.get_secret(&id, id.version).await? else {
            return Ok(None);
        };
        let index = unpack_u64("index", &items);
        Ok(Some(SecretShare {
            index,
            share: share.into(),
        }))
    }

    async fn delete_share_version(&self, id: ShareId) -> Result<(), Error> {
        self.delete_secret_version(&id, id.version).await
    }

    async fn put_key(&self, id: KeyId, key: WrappedKey) -> Result<bool, Error> {
        self.put_secret(&id, id.version, key.into_vec(), None).await
    }

    async fn get_key(&self, id: KeyId) -> Result<Option<WrappedKey>, Error> {
        let Some((key, _)) = self.get_secret(&id, id.version).await? else {
            return Ok(None);
        };
        Ok(Some(key.into()))
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
        let b_nonce = B(Blob::new(nonce));
        let n_exp = N(expiry.to_string());

        let nonce_used = self
            .db
            .get_item()
            .table_name(self.nonces_table())
            .key("identity", identity.to_attribute_value())
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
                    .item("identity", identity.to_attribute_value())
                    .item("nonce", b_nonce.clone())
                    .item("expiry", n_exp.clone())
            };
        }

        let res = self
            .db
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
                add_put_nonce_items!(self.db.put_item())
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
            .db
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
        self.db
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
            .db
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
            .db
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
        self.db
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
            .db
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

    crate::make_store_tests!(Client::connect(Environment::Dev));
}
