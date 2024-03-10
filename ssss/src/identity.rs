use aes_gcm_siv::{Aes256GcmSiv, KeyInit as _};

#[derive(Clone, Copy)]
pub struct Identity {
    sk: p384::NonZeroScalar,
}

impl Identity {
    pub fn new(sk: p384::SecretKey) -> Self {
        let scalar = sk.to_nonzero_scalar();
        Self { sk: scalar }
    }

    pub fn derive_shared_cipher(&self, opk: p384::PublicKey) -> Aes256GcmSiv {
        let shared = p384::ecdh::diffie_hellman(self.sk, opk.as_affine());
        let hkdf = shared.extract::<sha2::Sha256>(Some(b"ssss_ecdh_aes-256-gcm-siv"));
        let mut aes_key = [0u8; 32];
        hkdf.expand(&[], &mut aes_key).unwrap();
        Aes256GcmSiv::new_from_slice(&aes_key).unwrap()
    }

    pub fn public_key(&self) -> p384::PublicKey {
        p384::PublicKey::from_secret_scalar(&self.sk)
    }
}
