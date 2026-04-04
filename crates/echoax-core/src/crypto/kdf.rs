//! Argon2id key derivation for master key material.

use crate::error::Result;

/// Derives a 32-byte key from `password` and `salt` using Argon2's default parameters.
pub fn derive_key(password: &[u8], salt: &[u8]) -> Result<[u8; 32]> {
    use argon2::Argon2;

    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password, salt, &mut key)
        .map_err(|e| crate::error::EchoAccessError::Crypto(format!("KDF failed: {e}")))?;
    Ok(key)
}

/// Generates a random 16-byte salt suitable for Argon2.
pub fn generate_salt() -> [u8; 16] {
    use rand::RngCore;

    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_key_produces_32_bytes() {
        let salt = generate_salt();
        let key = derive_key(b"correct horse battery staple", &salt).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn derive_key_is_deterministic_for_same_inputs() {
        let salt = [7u8; 16];
        let a = derive_key(b"pw", &salt).unwrap();
        let b = derive_key(b"pw", &salt).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn derive_key_differs_with_different_salt() {
        let a = derive_key(b"pw", &[1u8; 16]).unwrap();
        let b = derive_key(b"pw", &[2u8; 16]).unwrap();
        assert_ne!(a, b);
    }
}
