use std::io::{Read, Write};
use std::iter;

use age::secrecy::SecretString;

use crate::error::{EchoAccessError, Result};

pub fn encrypt_file(data: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let encryptor = age::Encryptor::with_user_passphrase(passphrase.into());
    let mut encrypted = vec![];
    let mut writer = encryptor
        .wrap_output(&mut encrypted)
        .map_err(|e| EchoAccessError::Crypto(format!("Encryption init failed: {e}")))?;
    writer
        .write_all(data)
        .map_err(|e| EchoAccessError::Crypto(format!("Encryption write failed: {e}")))?;
    writer
        .finish()
        .map_err(|e| EchoAccessError::Crypto(format!("Encryption finish failed: {e}")))?;
    Ok(encrypted)
}

pub fn decrypt_file(encrypted: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let decryptor = age::Decryptor::new_buffered(encrypted)
        .map_err(|e| EchoAccessError::Crypto(format!("Decryption init failed: {e}")))?;
    if !decryptor.is_scrypt() {
        return Err(EchoAccessError::Crypto(
            "Expected passphrase-encrypted data".into(),
        ));
    }

    let identity = age::scrypt::Identity::new(SecretString::from(passphrase.to_owned()));
    let mut reader = decryptor
        .decrypt(iter::once(&identity as &dyn age::Identity))
        .map_err(|e| EchoAccessError::Crypto(format!("Decryption failed: {e}")))?;

    let mut decrypted = vec![];
    reader
        .read_to_end(&mut decrypted)
        .map_err(|e| EchoAccessError::Crypto(format!("Decryption read failed: {e}")))?;
    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let data = b"Hello, EchoAccess!";
        let passphrase = "test-passphrase-123";
        let encrypted = encrypt_file(data, passphrase).unwrap();
        assert_ne!(encrypted, data);
        let decrypted = decrypt_file(&encrypted, passphrase).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let data = b"secret data";
        let encrypted = encrypt_file(data, "correct-pass").unwrap();
        let result = decrypt_file(&encrypted, "wrong-pass");
        assert!(result.is_err());
    }

    #[test]
    fn empty_data_roundtrip() {
        let encrypted = encrypt_file(b"", "pass").unwrap();
        let decrypted = decrypt_file(&encrypted, "pass").unwrap();
        assert!(decrypted.is_empty());
    }
}
