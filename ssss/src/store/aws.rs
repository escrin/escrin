use std::collections::HashMap;

use aws_sdk_dynamodb::{
    primitives::Blob,
    types::AttributeValue::{self, B, N, S},
};
use rand::RngCore as _;

use super::*;
use crate::cli::Environment;

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
    naming_fn!(permits_table, "escrin-permits");
    naming_fn!(chain_state_table, "escrin-chain-state");
    naming_fn!(kms_key, "alias/escrin-sek");

    async fn current_share_version(&self, identity: IdentityLocator) -> Result<Option<u64>, Error> {
        Ok(self
            .db_client
            .query()
            .table_name(self.shares_table())
            .key_condition_expression("id = :id")
            .expression_attribute_values(":id", identity.into())
            .projection_expression("version")
            .scan_index_forward(false)
            .limit(1)
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .items()
            .first()
            .map(unpack_version))
    }
}

impl Store for Client {
    async fn create_share(&self, identity: IdentityLocator) -> Result<ShareId, Error> {
        let version = self.current_share_version(identity).await?.unwrap_or(0) + 1;
        let n_version = N(version.to_string());

        let mut share = vec![0u8; SHARE_SIZE];
        rand::thread_rng().fill_bytes(&mut share);

        let share_id = ShareId { identity, version };

        let enc_share = self
            .kms_client
            .encrypt()
            .key_id(self.kms_key())
            .plaintext(Blob::new(share))
            .encryption_context("share", share_id.to_key())
            .send()
            .await
            .map_err(aws_sdk_kms::Error::from)?
            .ciphertext_blob
            .unwrap();

        self.db_client
            .put_item()
            .table_name(self.shares_table())
            .item("id", identity.into())
            .item("version", n_version)
            .item("share", B(enc_share))
            .condition_expression("attribute_not_exists(id) AND attribute_not_exists(version)")
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?;

        Ok(share_id)
    }

    #[cfg(test)]
    async fn destroy_share(&self, share: ShareId) -> Result<(), Error> {
        self.db_client
            .delete_item()
            .table_name(self.shares_table())
            .key("id", share.identity.into())
            .key("version", N(share.version.to_string()))
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)
            .unwrap();
        Ok(())
    }

    async fn get_share(&self, share: ShareId) -> Result<Option<WrappedShare>, Error> {
        let maybe_enc_share = self
            .db_client
            .query()
            .table_name(self.shares_table())
            .key_condition_expression("id = :id AND version = :version")
            .expression_attribute_values(":id", share.identity.into())
            .expression_attribute_values(":version", N(share.version.to_string()))
            .projection_expression("#s")
            .expression_attribute_names("#s", "share")
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .items
            .unwrap_or_default()
            .into_iter()
            .nth(0)
            .map(|mut v| unpack_share(&mut v));

        let enc_share = match maybe_enc_share {
            Some(s) => s,
            None => return Ok(None),
        };

        Ok(Some(WrappedShare(
            self.kms_client
                .decrypt()
                .key_id(self.kms_key())
                .ciphertext_blob(enc_share)
                .encryption_context("share", share.to_key())
                .send()
                .await
                .map_err(aws_sdk_kms::Error::from)?
                .plaintext
                .unwrap()
                .into_inner(),
        )))
    }

    async fn create_permit(
        &self,
        share: ShareId,
        recipient: Address,
        expiry: u64,
    ) -> Result<Option<Permit>, Error> {
        let n_exp = N(expiry.to_string());
        let res = self
            .db_client
            .put_item()
            .table_name(self.permits_table())
            .item("share", share.into())
            .item("expiry", n_exp.clone())
            .item("recipient", address_to_value(&recipient))
            .condition_expression("attribute_not_exists(expiry) OR expiry < :expiry")
            .expression_attribute_values(":expiry", n_exp)
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from);
        match res {
            Ok(_) => Ok(Some(Permit { expiry })),
            Err(aws_sdk_dynamodb::Error::ConditionalCheckFailedException(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn read_permit(
        &self,
        share: ShareId,
        recipient: Address,
    ) -> Result<Option<Permit>, Error> {
        Ok(self
            .db_client
            .query()
            .table_name(self.permits_table())
            .key_condition_expression("#s = :share AND recipient = :recipient")
            .expression_attribute_names("#s", "share")
            .expression_attribute_values(":share", share.into())
            .expression_attribute_values(":recipient", address_to_value(&recipient))
            .projection_expression("expiry")
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .items()
            .first()
            .and_then(|v| {
                let expiry = unpack_u64("expiry", v);
                if expiry <= now() {
                    None
                } else {
                    Some(Permit { expiry })
                }
            }))
    }

    async fn delete_permit(&self, share: ShareId, recipient: Address) -> Result<(), Error> {
        self.db_client
            .delete_item()
            .table_name(self.permits_table())
            .key("share", share.into())
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
        let new_block = match block {
            Some(block) => block,
            None => return Ok(()),
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
}

impl From<IdentityLocator> for AttributeValue {
    fn from(identity: IdentityLocator) -> Self {
        S(identity.to_key())
    }
}

impl From<ShareId> for AttributeValue {
    fn from(share: ShareId) -> Self {
        S(share.to_key())
    }
}

fn unpack_version(res: &HashMap<String, AttributeValue>) -> u64 {
    res.get("version")
        .expect("version key")
        .as_n()
        .expect("numeric version")
        .parse::<u64>()
        .expect("parseable numeric version")
}

fn unpack_u64(key: &str, res: &HashMap<String, AttributeValue>) -> u64 {
    res.get(key)
        .expect(key)
        .as_n()
        .unwrap()
        .parse::<u64>()
        .unwrap()
}

fn unpack_share(res: &mut HashMap<String, AttributeValue>) -> Blob {
    match res.remove("share").expect("share present") {
        B(b) => b,
        _ => panic!("share not blob"),
    }
}

fn address_to_value(addr: &Address) -> AttributeValue {
    S(format!("{addr:#x}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::make_sstore_tests!(Client::connect(Environment::Dev).await);
}
