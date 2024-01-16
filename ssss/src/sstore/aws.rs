use std::collections::HashMap;

use aws_sdk_dynamodb::{
    primitives::Blob,
    types::AttributeValue::{self, B, N, S},
};
use futures::future::TryFutureExt as _;
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

    naming_fn!(shares_table, "shares");
    naming_fn!(permits_table, "permits");
    naming_fn!(kms_key, "alias/escrin-sek");

    async fn current_share_version(&self, id: &IdentityId) -> Result<Option<u64>, Error> {
        Ok(self
            .db_client
            .query()
            .table_name(self.shares_table())
            .key_condition_expression("id = :id")
            .expression_attribute_values(":id", S(identity_id_to_key(id)))
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

impl ShareStore for Client {
    type Error = Error;

    async fn create_share(&self, identity: IdentityId) -> Result<ShareId, Error> {
        let id_s = identity_id_to_key(&identity);
        let s_id = S(id_s.clone());

        let version = self.current_share_version(&identity).await?.unwrap_or(0) + 1;
        let n_version = N(version.to_string());

        let mut share = vec![0u8; SHARE_SIZE];
        rand::thread_rng().fill_bytes(&mut share);

        let enc_share = self
            .kms_client
            .encrypt()
            .key_id(self.kms_key())
            .plaintext(Blob::new(share))
            .encryption_context("id", id_s)
            .encryption_context("version", version.to_string())
            .send()
            .await
            .map_err(aws_sdk_kms::Error::from)?
            .ciphertext_blob
            .unwrap();

        self.db_client
            .put_item()
            .table_name(self.shares_table())
            .item("id", s_id)
            .item("version", n_version)
            .item("share", B(enc_share))
            .condition_expression("attribute_not_exists(id) AND attribute_not_exists(version)")
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?;

        Ok(ShareId { identity, version })
    }

    #[cfg(test)]
    async fn destroy_share(&self, share: ShareId) -> Result<(), Self::Error> {
        self.db_client
            .delete_item()
            .table_name(self.shares_table())
            .key("id", S(identity_id_to_key(&share.identity)))
            .key("version", N(share.version.to_string()))
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)
            .unwrap();
        Ok(())
    }

    async fn get_share(&self, share: ShareId) -> Result<Option<WrappedShare>, Error> {
        let id_s = identity_id_to_key(&share.identity);
        let s_id = S(id_s.clone());

        let maybe_enc_share = self
            .db_client
            .query()
            .table_name(self.shares_table())
            .key_condition_expression("id = :id AND version = :version")
            .expression_attribute_values(":id", s_id.clone())
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
                .encryption_context("id", id_s)
                .encryption_context("version", share.version.to_string())
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
            .item("share", share_id_to_key(&share))
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
    ) -> Result<Option<Permit>, Self::Error> {
        Ok(self
            .db_client
            .query()
            .table_name(self.permits_table())
            .key_condition_expression("#s = :share AND recipient = :recipient")
            .expression_attribute_names("#s", "share")
            .expression_attribute_values(":share", share_id_to_key(&share))
            .expression_attribute_values(":recipient", address_to_value(&recipient))
            .projection_expression("expiry")
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?
            .items()
            .first()
            .and_then(|v| {
                let expiry = unpack_expiry(v);
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
            .key("share", share_id_to_key(&share))
            .key("recipient", address_to_value(&recipient))
            .send()
            .await
            .map_err(aws_sdk_dynamodb::Error::from)?;
        Ok(())
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

fn unpack_expiry(res: &HashMap<String, AttributeValue>) -> u64 {
    res.get("expiry")
        .expect("expiry value")
        .as_n()
        .expect("numeric expiry")
        .parse::<u64>()
        .expect("parseable numeric expiry")
}

fn unpack_share(res: &mut HashMap<String, AttributeValue>) -> Blob {
    match res.remove("share").expect("share present") {
        B(b) => b,
        _ => panic!("share not blob"),
    }
}

fn identity_id_to_key(id: &IdentityId) -> String {
    format!("{id:#x}")
}

fn address_to_value(addr: &Address) -> AttributeValue {
    S(format!("{addr:#x}"))
}

fn share_id_to_key(id: &ShareId) -> AttributeValue {
    S(format!(
        "{}-{}",
        identity_id_to_key(&id.identity),
        id.version
    ))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(#[from] aws_sdk_dynamodb::Error),

    #[error("kms error: {0}")]
    Kms(#[from] aws_sdk_kms::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::make_sstore_tests!(Client::connect(Environment::Dev).await);
}
