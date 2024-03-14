use ethers::types::{Address, Bytes};
use p384::elliptic_curve::JwkEcKey;
use serde::{Deserialize, Serialize};

use super::{Permit, WrappedKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityResponse {
    pub persistent: JwkEcKey,
    pub ephemeral: JwkEcKey,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcqRelIdentityRequest {
    #[serde(default)]
    pub duration: u64,
    pub authorization: Bytes,
    pub context: Bytes,
    pub permitter: Address,
    pub recipient: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcqRelIdentityResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit: Option<Permit>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetShareQuery {
    pub version: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareResponse {
    pub format: ShareResponseFormat,
    pub ss: WrappedSecretShare,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct WrappedSecretShare {
    pub index: u64,
    pub share: Bytes,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShareResponseFormat {
    Plain,
    EncAes256GcmSiv {
        #[serde(with = "hex::serde")]
        nonce: [u8; 12],
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetKeyQuery {
    pub version: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PutKeyRequest {
    pub key: WrappedKey,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyResponse {
    pub key: Bytes,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}
