use std::sync::LazyLock;

use webpki::types::{CertificateDer, UnixTime};

use crate::sync::PermitRequestEvent;

static CA_CERT_DER: &[u8] = include_bytes!("./root.der");
static CA_CERT: LazyLock<CertificateDer<'static>> = LazyLock::new(|| CA_CERT_DER.into());
static ANCHORS: LazyLock<Vec<webpki::types::TrustAnchor<'static>>> =
    LazyLock::new(|| vec![webpki::anchor_from_trusted_cert(&CA_CERT).unwrap()]);

pub async fn verify(action: PermitRequestEvent) -> Option<()> {
    // TODO: filter identities that are not registered with this SSSS
    let PermitRequestEvent {
        kind,
        identity,
        requester,
        recipient,
        context,
        authorization,
        ..
    } = action;

    verify_attestation(&authorization, UnixTime::now())
}

fn verify_attestation(doc_bytes: &[u8], now: UnixTime) -> Option<()> {
    let sign1 = <coset::CoseSign1 as coset::CborSerializable>::from_slice(doc_bytes).ok()?;

    let doc: AttestationDocument = ciborium::from_reader(sign1.payload.as_deref()?).ok()?;

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

    Some(())
}

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

#[derive(serde::Deserialize)]
struct AttestationDocument {
    module_id: String,
    digest: String,
    timestamp: u64,
    pcrs: std::collections::HashMap<usize, Vec<u8>>,
    certificate: Vec<u8>,
    cabundle: Vec<Vec<u8>>,
    public_key: Option<Vec<u8>>,
    user_data: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
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
        verify_attestation(
            &attestaion_doc,
            UnixTime::since_unix_epoch(std::time::Duration::from_secs(1703101376)),
        )
        .unwrap()
    }
}
