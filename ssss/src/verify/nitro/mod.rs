use std::{collections::HashMap, sync::LazyLock};

use ethers::{abi::AbiEncode as _, types::U256};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use tracing::warn;
use webpki::types::{CertificateDer, UnixTime};

use super::*;
use crate::sync::{PermitRequestEvent, PermitRequestKind};

static CA_CERT_DER: &[u8] = include_bytes!("./root.der");
static CA_CERT: LazyLock<CertificateDer<'static>> = LazyLock::new(|| CA_CERT_DER.into());
static ANCHORS: LazyLock<Vec<webpki::types::TrustAnchor<'static>>> =
    LazyLock::new(|| vec![webpki::anchor_from_trusted_cert(&CA_CERT).unwrap()]);

#[derive(Clone, Copy, Debug, Default)]
pub struct NitroEnclaveVerifier;

impl Verifier for NitroEnclaveVerifier {
    async fn verify(&self, req: PermitRequestEvent, policy_bytes: &[u8]) -> Option<Verification> {
        // TODO: filter identities that are not registered with this SSSS
        let PermitRequestEvent { authorization, .. } = req;

        let policy: Policy = ciborium::from_reader(policy_bytes).ok()?;

        let binding = ethers::core::utils::keccak256(
            (
                req.chain,
                req.permitter,
                req.identity.0,
                req.recipient,
                match req.kind {
                    PermitRequestKind::Revoke => U256::zero(),
                    PermitRequestKind::Grant { duration } => U256::from(duration),
                },
            )
                .encode(),
        );

        let (ud, pcrs) = Self::verify_attestation_document(&authorization, UnixTime::now())?;

        if ud.user_data.len() < binding.len() || ud.user_data[0..binding.len()] != binding {
            return None;
        }

        policy.pcrs.check(&pcrs)?;

        Some(Verification {
            nonce: ud.nonce,
            public_key: ud.public_key,
        })
    }
}

impl NitroEnclaveVerifier {
    fn verify_attestation_document(
        doc_bytes: &[u8],
        now: UnixTime,
    ) -> Option<(AttestationUserData, HashMap<usize, Pcr>)> {
        let sign1 = <coset::CoseSign1 as coset::CborSerializable>::from_slice(doc_bytes).ok()?;

        let doc: AttestationDocument = ciborium::from_reader(sign1.payload.as_deref()?).unwrap();

        if doc.digest != "SHA384" {
            warn!(
                "received unsupported digest in attestation document: {}",
                doc.digest
            );
            return None;
        }

        let cert_der = doc.certificate.into();
        let ee_cert = webpki::EndEntityCert::try_from(&cert_der).ok()?;
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
            .ok()?;

        sign1
            .verify_signature(&[], |sig, data| ee_cert.verify_signature(&ES384, data, sig))
            .ok()?;

        Some((
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
struct AttestationDocument {
    #[allow(unused)]
    module_id: serde::de::IgnoredAny,
    digest: String,
    #[allow(unused)]
    timestamp: serde::de::IgnoredAny,
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

#[derive(Serialize, Deserialize)]
struct Policy {
    version: u8,
    pcrs: PolicyPcrs,
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
    fn check(&self, pcr_map: &HashMap<usize, Pcr>) -> Option<()> {
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
                        if pcr_map.get(&$pcr)? != expected_pcr {
                            return None;
                        }
                    }
                )+
            }
        }
        check_pcrs!(0, 1, 2, 3, 4, 8);
        Some(())
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
        let attestaion_doc = std::fs::read(
            std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("../evm/test/identity/v1/permitters/att_doc_sample.bin"),
        )
        .unwrap();
        NitroEnclaveVerifier::verify_attestation_document(
            &attestaion_doc,
            UnixTime::since_unix_epoch(std::time::Duration::from_secs(1703101376)),
        )
        .unwrap();
    }
}
