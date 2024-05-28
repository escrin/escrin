use std::collections::{HashMap, HashSet};

use anyhow::anyhow;
use ethers::{abi::AbiEncode as _, types::H256};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use webpki::types::{CertificateDer, UnixTime};

use super::*;

static CA_CERT_DER: &[u8] = include_bytes!("./root.der");
static CA_CERT: Lazy<CertificateDer<'static>> = Lazy::new(|| CA_CERT_DER.into());
static ANCHORS: Lazy<Vec<webpki::types::TrustAnchor<'static>>> =
    Lazy::new(|| vec![webpki::anchor_from_trusted_cert(&CA_CERT).unwrap()]);

#[derive(Clone, Copy, Debug, Default)]
pub struct NitroEnclaveVerifier;

impl Verifier for NitroEnclaveVerifier {
    async fn verify(
        &self,
        raw_policy: serde_json::Value,
        req: RequestKind,
        identity: IdentityLocator,
        recipient: Address,
        authorization: &[u8],
        _context: &[u8],
        relayer: Option<Address>,
    ) -> Result<Verification, Error> {
        let policy: Policy = serde_json::from_value(raw_policy)
            .map_err(|e| Error::PolicyDecode(anyhow::Error::from(e)))?;

        if policy.version != 1 {
            return Err(Error::PolicyDecode(anyhow::anyhow!(
                "unsupported NE policy version {}",
                policy.version
            )));
        }

        if !policy.relayers.is_empty()
            && !relayer
                .map(|r| policy.relayers.contains(&r))
                .unwrap_or_default()
        {
            return Err(Error::Unauthorized("not a trusted relayer".into()));
        }

        let (ud, pcrs) =
            Self::verify_attestation_document(authorization, policy.max_age, UnixTime::now())?;
        let binding = (ud.user_data.len() >= H256::len_bytes())
            .then(|| &ud.user_data[0..H256::len_bytes()])
            .ok_or(Error::InvalidBinding)?;

        let expected_binding = ethers::core::utils::keccak256(
            (
                identity.chain,
                identity.registry,
                identity.id.0,
                recipient,
                matches!(req, RequestKind::Grant { .. }),
            )
                .encode(),
        );

        if binding != expected_binding {
            return Err(Error::BindingMismatch(SmallVec::from_buf(expected_binding)));
        }

        policy.pcrs.check(&pcrs)?;

        Ok(Verification {
            nonce: ud.nonce,
            public_key: ud.public_key,
            duration: match req {
                RequestKind::Grant { duration } => Some(duration.min(policy.max_duration)),
                RequestKind::Revoke => None,
            },
        })
    }
}

impl NitroEnclaveVerifier {
    fn verify_attestation_document(
        doc_bytes: &[u8],
        max_age: u64,
        now: UnixTime,
    ) -> Result<(AttestationUserData, HashMap<usize, Pcr>), Error> {
        let sign1 = <coset::CoseSign1 as coset::CborSerializable>::from_slice(doc_bytes)
            .map_err(|e| Error::AttestationDecode(anyhow::Error::from(e)))?;

        let doc: AttestationDocument = ciborium::de::from_reader_with_recursion_limit(
            sign1
                .payload
                .as_deref()
                .ok_or_else(|| Error::AttestationDecode(anyhow!("missing Sign1 payload")))?,
            5,
        )
        .unwrap();

        let doc_timestamp_s = doc.timestamp / 1000;
        if doc_timestamp_s > (now.as_secs() + 5) {
            return Err(Error::Timing("attestation doc not yet valid".into()));
        }
        if now.as_secs() - doc_timestamp_s >= max_age {
            return Err(Error::Timing("attestation doc expired".into()));
        }

        if doc.digest != "SHA384" {
            return Err(Error::AttestationDecode(anyhow!(
                "unsupported attesation digest: {}",
                doc.digest
            )));
        }

        let cert_der = doc.certificate.into();
        let ee_cert = webpki::EndEntityCert::try_from(&cert_der)
            .map_err(|e| Error::AttestationDecode(anyhow::Error::from(e)))?;
        ee_cert
            .verify_for_usage(
                webpki::ALL_VERIFICATION_ALGS,
                &ANCHORS,
                &doc.cabundle
                    .into_iter()
                    .skip(1)
                    .map(CertificateDer::from)
                    .collect::<Vec<_>>(),
                now,
                webpki::KeyUsage::server_auth(),
                None, // TODO: support CRL
                None,
            )
            .map_err(|e| Error::AttestationDecode(anyhow::Error::from(e)))?;

        sign1
            .verify_signature(&[], |sig, data| ee_cert.verify_signature(&ES384, data, sig))
            .map_err(|e| Error::AttestationDecode(anyhow::Error::from(e)))?;

        Ok((
            AttestationUserData {
                user_data: doc.user_data,
                public_key: doc.public_key,
                nonce: doc.nonce,
            },
            doc.pcrs,
        ))
    }
}

#[derive(Deserialize)]
#[deny(unused)]
struct AttestationDocument {
    #[allow(unused)]
    module_id: serde::de::IgnoredAny,
    digest: String,
    /// The timestamp at which the document was generated in milliseconds.
    timestamp: u64,
    pcrs: HashMap<usize, Pcr>,
    certificate: Vec<u8>,
    cabundle: Vec<Vec<u8>>,
    #[serde(default)]
    public_key: Vec<u8>,
    #[serde(default)]
    user_data: Vec<u8>,
    #[serde(default)]
    nonce: Vec<u8>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct AttestationUserData {
    public_key: Vec<u8>,
    user_data: Vec<u8>,
    nonce: Vec<u8>,
}

impl std::fmt::Debug for AttestationUserData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttestationUserData")
            .field("public_key", &hex::encode(&self.public_key))
            .field("user_data", &hex::encode(&self.user_data))
            .field("nonce", &hex::encode(&self.nonce))
            .finish()
    }
}

#[derive(Deserialize)]
#[forbid(unused)]
struct Policy {
    version: u8,
    pcrs: PolicyPcrs,
    #[serde(default)]
    max_duration: u64,
    #[serde(default)]
    relayers: HashSet<Address>,
    /// Attestation max age (seconds)
    #[serde(default = "default_max_age")]
    max_age: u64,
}

fn default_max_age() -> u64 {
    15 * 60 // 15 minutes
}

#[derive(Serialize, Deserialize)]
struct PolicyPcrs {
    /// A contiguous measure of the contents of the image file, without the section data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pcr0: Option<Pcr>,
    /// A contiguous measurement of the kernel and boot ramfs data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pcr1: Option<Pcr>,
    /// A contiguous, in-order measurement of the user applications, without the boot ramfs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pcr2: Option<Pcr>,
    /// A measurement of the IAM role assigned to the parent instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pcr3: Option<Pcr>,
    /// A measurement of the ID of the parent instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pcr4: Option<Pcr>,
    /// A measurement of the signing certificate specified for the enclave image file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pcr8: Option<Pcr>,
}

impl PolicyPcrs {
    fn check(&self, pcr_map: &HashMap<usize, Pcr>) -> Result<(), Error> {
        let PolicyPcrs {
            pcr0,
            pcr1,
            pcr2,
            pcr3,
            pcr4,
            pcr8,
        } = &self;
        macro_rules! check_pcrs {
            ($($pcr:literal),+ $(,)?) => {
                $(
                    if let Some(expected_pcr) = &paste::paste!([< pcr $pcr >]) {
                        if pcr_map.get(&$pcr).ok_or(Error::PcrMismatch($pcr))? != expected_pcr {
                            return Err(Error::PcrMismatch($pcr));
                        }
                    }
                )+
            }
        }
        check_pcrs!(0, 1, 2, 3, 4, 8);
        Ok(())
    }
}

type Pcr = SmallVec<[u8; 48]>;

#[derive(Clone, Copy, Debug)]
struct ES384;

impl webpki::types::SignatureVerificationAlgorithm for ES384 {
    fn public_key_alg_id(&self) -> webpki::types::AlgorithmIdentifier {
        webpki::alg_id::ECDSA_P384
    }

    fn signature_alg_id(&self) -> webpki::types::AlgorithmIdentifier {
        webpki::alg_id::ECDSA_SHA384
    }

    fn verify_signature(
        &self,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), webpki::types::InvalidSignature> {
        ring::signature::UnparsedPublicKey::new(
            &ring::signature::ECDSA_P384_SHA384_FIXED,
            public_key,
        )
        .verify(message, signature)
        .map_err(|_| webpki::types::InvalidSignature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_attestation() {
        let attestation_doc_path = std::env::var("NIX_NE_ATT_DOC")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
                    .join("../evm/test/identity/v1/permitters/att_doc_sample.bin")
            });
        let attestation_doc = std::fs::read(attestation_doc_path).unwrap();
        NitroEnclaveVerifier::verify_attestation_document(
            &attestation_doc,
            u64::max_value(),
            UnixTime::since_unix_epoch(std::time::Duration::from_secs(1703101376)),
        )
        .unwrap();
    }
}
