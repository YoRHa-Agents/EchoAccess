//! Passphrase-based file encryption using the age `scrypt` recipient (rage-compatible).

use age::secrecy::SecretString;

use crate::error::{EchoAccessError, Result};

/// Encrypts `data` with a human passphrase (scrypt recipient). Output is binary age v1.
pub fn encrypt_file(data: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let secret = SecretString::from(passphrase.to_owned());
    let recipient = age::scrypt::Recipient::new(secret);
    age::encrypt(&recipient, data).map_err(|e| {
        EchoAccessError::Crypto(format!("age encrypt failed: {e}"))
    })
}

/// Decrypts an age ciphertext produced by [`encrypt_file`].
pub fn decrypt_file(encrypted: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let secret = SecretString::from(passphrase.to_owned());
    let identity = age::scrypt::Identity::new(secret);
    age::decrypt(&identity, encrypted).map_err(|e| {
        EchoAccessError::Crypto(format!("age decrypt failed: {e}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_round_trip() {
        let plain = b"echo access secret payload \x00\xff";
        let passphrase = "test passphrase for age round-trip";
        let ct = encrypt_file(plain, passphrase).unwrap();
        assert_ne!(ct.as_slice(), plain.as_slice());
        let out = decrypt_file(&ct, passphrase).unwrap();
        assert_eq!(out, plain);
    }

    #[test]
    fn decrypt_wrong_passphrase_fails() {
        let ct = encrypt_file(b"x", "right").unwrap();
        let err = decrypt_file(&ct, "wrong").unwrap_err();
        assert!(matches!(err, EchoAccessError::Crypto(_)));
    }
}
