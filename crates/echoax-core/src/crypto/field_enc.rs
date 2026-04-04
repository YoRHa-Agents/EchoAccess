use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};

use crate::error::{EchoAccessError, Result};

pub fn encrypt_field(key: &[u8; 32], field_path: &str, value: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| EchoAccessError::Crypto(format!("AES key init: {e}")))?;
    let nonce_bytes = derive_nonce(field_path);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let payload = Payload {
        msg: value,
        aad: field_path.as_bytes(),
    };
    cipher
        .encrypt(nonce, payload)
        .map_err(|e| EchoAccessError::Crypto(format!("Field encrypt failed: {e}")))
}

pub fn decrypt_field(key: &[u8; 32], field_path: &str, encrypted: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| EchoAccessError::Crypto(format!("AES key init: {e}")))?;
    let nonce_bytes = derive_nonce(field_path);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let payload = Payload {
        msg: encrypted,
        aad: field_path.as_bytes(),
    };
    cipher
        .decrypt(nonce, payload)
        .map_err(|e| EchoAccessError::Crypto(format!("Field decrypt failed: {e}")))
}

fn derive_nonce(field_path: &str) -> [u8; 12] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    field_path.hash(&mut hasher);
    let hash = hasher.finish();
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(&hash.to_le_bytes());
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [42u8; 32]
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = test_key();
        let value = b"secret_value";
        let enc = encrypt_field(&key, "user.password", value).unwrap();
        let dec = decrypt_field(&key, "user.password", &enc).unwrap();
        assert_eq!(dec, value);
    }

    #[test]
    fn wrong_key_fails() {
        let enc = encrypt_field(&test_key(), "path", b"data").unwrap();
        let wrong_key = [99u8; 32];
        assert!(decrypt_field(&wrong_key, "path", &enc).is_err());
    }

    #[test]
    fn aad_mismatch_fails() {
        let key = test_key();
        let enc = encrypt_field(&key, "path.a", b"data").unwrap();
        assert!(decrypt_field(&key, "path.b", &enc).is_err());
    }

    #[test]
    fn empty_value_roundtrip() {
        let key = test_key();
        let enc = encrypt_field(&key, "empty", b"").unwrap();
        let dec = decrypt_field(&key, "empty", &enc).unwrap();
        assert!(dec.is_empty());
    }
}
