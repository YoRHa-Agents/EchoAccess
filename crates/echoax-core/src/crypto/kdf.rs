use crate::error::{EchoAccessError, Result};

pub fn derive_key(password: &[u8], salt: &[u8]) -> Result<[u8; 32]> {
    use argon2::Argon2;
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password, salt, &mut key)
        .map_err(|e| EchoAccessError::Crypto(format!("KDF failed: {e}")))?;
    Ok(key)
}

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
        let key = derive_key(b"test-password", &salt).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn same_input_same_output() {
        let salt = [1u8; 16];
        let k1 = derive_key(b"pass", &salt).unwrap();
        let k2 = derive_key(b"pass", &salt).unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn different_password_different_key() {
        let salt = [2u8; 16];
        let k1 = derive_key(b"pass1", &salt).unwrap();
        let k2 = derive_key(b"pass2", &salt).unwrap();
        assert_ne!(k1, k2);
    }

    #[test]
    fn different_salt_different_key() {
        let k1 = derive_key(b"pass", &[1u8; 16]).unwrap();
        let k2 = derive_key(b"pass", &[2u8; 16]).unwrap();
        assert_ne!(k1, k2);
    }
}
