use aes_gcm_siv::{Aes256GcmSiv, KeyInit as _};

#[derive(Clone, Copy)]
pub struct Identity {
    sk: p384::NonZeroScalar,
}

pub static DEAL_SHARES_DOMAIN_SEP: &[u8] = b"deal-shares";
pub static GET_SHARE_DOMAIN_SEP: &[u8] = b"get-share";

impl Identity {
    pub fn persistent(sk: p384::SecretKey) -> Self {
        let scalar = sk.to_nonzero_scalar();
        Self { sk: scalar }
    }

    pub fn ephemeral() -> Self {
        Self {
            sk: p384::NonZeroScalar::random(&mut rand::thread_rng()),
        }
    }

    pub fn derive_shared_cipher(&self, opk: p384::PublicKey, hkdf_info: &[u8]) -> Aes256GcmSiv {
        derive_shared_cipher(&self.sk, &opk, hkdf_info)
    }

    pub fn public_key(&self) -> p384::PublicKey {
        p384::PublicKey::from_secret_scalar(&self.sk)
    }
}

pub fn derive_shared_cipher(
    sk: &p384::NonZeroScalar,
    opk: &p384::PublicKey,
    hkdf_info: &[u8],
) -> Aes256GcmSiv {
    let shared = p384::ecdh::diffie_hellman(sk, opk.as_affine());
    let hkdf = shared.extract::<sha2::Sha256>(Some(b"ssss_ecdh_aes-256-gcm-siv"));
    let mut aes_key = [0u8; 32];
    hkdf.expand(hkdf_info, &mut aes_key).unwrap();
    Aes256GcmSiv::new_from_slice(&aes_key).unwrap()
}
