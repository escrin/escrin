use axum::http::header;
use axum_extra::headers;
use ethers::{
    core::k256::{self, elliptic_curve::sec1::FromEncodedPoint as _},
    types::{Address, Bytes, Signature},
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use super::{SsssPermit, WrappedKey};

pub static PEDERSEN_VSS_BLINDER_GENERATOR: Lazy<k256::ProjectivePoint> = Lazy::new(|| {
    let generator: k256::EncodedPoint =
        "036f579b345d53115deb10137c9fdc633ed4abddfe8bd2ac36f3e5351bccf37808"
            .parse()
            .unwrap();
    k256::ProjectivePoint::from_encoded_point(&generator).unwrap()
});

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentityResponse {
    pub ephemeral: EphemeralKey,
    pub signer: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EphemeralKey {
    pub key_id: String,
    pub pk: p384::PublicKey,
    pub expiry: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetPolicyRequest {
    pub permitter: Address,
    pub policy: Box<serde_json::value::RawValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcqRelIdentityRequest {
    pub permitter: Address,
    pub recipient: Address,
    pub base_block: u64,
    #[serde(default)]
    pub duration: Option<u64>,
    #[serde(default)]
    pub authorization: Bytes,
    #[serde(default)]
    pub context: Bytes,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermitResponse {
    pub permit: SsssPermit,
    pub signer: Address,
    pub signature: Signature,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetShareQuery {
    pub version: u64,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub enum MaybeEncryptedRequest<T> {
    Encrypted(EncryptedPayload),
    Plain(T),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub format: EncryptedPayloadFormat,
    pub payload: Bytes,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum EncryptedPayloadFormat {
    P384EcdhAes256GcmSiv {
        curve: CurveP384,
        pk: p384::PublicKey,
        #[serde(with = "hex::serde")]
        nonce: [u8; 12],
        #[serde(default, skip_serializing_if = "String::is_empty")]
        recipient_key_id: String,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct CurveP384;

impl Serialize for CurveP384 {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        "P-384".serialize(s)
    }
}

impl<'de> Deserialize<'de> for CurveP384 {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if s != "P-384" {
            Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&s),
                &"P-384",
            ))
        } else {
            Ok(Self)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareBody {
    pub share: SecretShare,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct SecretShare {
    #[serde(flatten)]
    pub meta: super::SecretShareMeta,
    pub share: Bytes,
    pub blinder: Bytes,
}

impl From<crate::types::SecretShare> for SecretShare {
    fn from(ss: crate::types::SecretShare) -> Self {
        Self {
            meta: ss.meta,
            share: (*ss.share).clone().into(),
            blinder: (*ss.blinder).clone().into(),
        }
    }
}

impl From<SecretShare> for crate::types::SecretShare {
    fn from(ss: SecretShare) -> Self {
        Self {
            meta: ss.meta,
            share: Vec::from(ss.share.0).into(),
            blinder: Vec::from(ss.blinder.0).into(),
        }
    }
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
