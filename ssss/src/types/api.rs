use axum::http::header;
use axum_extra::headers;
use ethers::types::{Address, Bytes, Signature};
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
    #[serde(with = "hex::serde")]
    pub share: Vec<u8>,
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

pub struct SignatureHeader(pub Signature);

impl std::fmt::Display for SignatureHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0.to_vec()))
    }
}

static SIGNATURE_HEADER_NAME: header::HeaderName = header::HeaderName::from_static("signature");

impl headers::Header for SignatureHeader {
    fn name() -> &'static header::HeaderName {
        &SIGNATURE_HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i header::HeaderValue>,
    {
        let sig_hex = values.next().ok_or_else(headers::Error::invalid)?;
        let sig_bytes: Bytes = sig_hex
            .to_str()
            .ok()
            .and_then(|s| s.parse().ok())
            .ok_or_else(headers::Error::invalid)?;
        Ok(Self(
            (&*sig_bytes)
                .try_into()
                .map_err(|_| headers::Error::invalid())?,
        ))
    }

    fn encode<E: Extend<header::HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            header::HeaderValue::from_str(&self.to_string()).unwrap(),
        ));
    }
}

impl std::ops::Deref for SignatureHeader {
    type Target = Signature;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RequesterHeader(pub Address);

impl std::fmt::Display for RequesterHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

static REQUESTER_HEADER_NAME: header::HeaderName = header::HeaderName::from_static("requester");

impl headers::Header for RequesterHeader {
    fn name() -> &'static header::HeaderName {
        &REQUESTER_HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i header::HeaderValue>,
    {
        Ok(Self(
            values
                .next()
                .ok_or_else(headers::Error::invalid)?
                .to_str()
                .ok()
                .and_then(|s| s.parse().ok())
                .ok_or_else(headers::Error::invalid)?,
        ))
    }

    fn encode<E: Extend<header::HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            header::HeaderValue::from_str(&self.to_string()).unwrap(),
        ));
    }
}

impl std::ops::Deref for RequesterHeader {
    type Target = Address;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RequesterPublicKeyHeader(pub p384::PublicKey);

impl std::fmt::Display for RequesterPublicKeyHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0.to_sec1_bytes()))
    }
}

static REQUESTER_PUBKEY_HEADER_NAME: header::HeaderName =
    header::HeaderName::from_static("requester-pk");

impl headers::Header for RequesterPublicKeyHeader {
    fn name() -> &'static header::HeaderName {
        &REQUESTER_PUBKEY_HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i header::HeaderValue>,
    {
        let pk_hex = values.next().ok_or_else(headers::Error::invalid)?;
        let pk_bytes: Bytes = pk_hex
            .to_str()
            .ok()
            .and_then(|s| s.parse().ok())
            .ok_or_else(headers::Error::invalid)?;
        Ok(Self(
            p384::PublicKey::from_sec1_bytes(&pk_bytes).map_err(|_| headers::Error::invalid())?,
        ))
    }

    fn encode<E: Extend<header::HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            header::HeaderValue::from_str(&self.to_string()).unwrap(),
        ));
    }
}

impl std::ops::Deref for RequesterPublicKeyHeader {
    type Target = p384::PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
