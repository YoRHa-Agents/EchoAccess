//! Cryptographic primitives: KDF, session key handling, and age-based file encryption.

pub mod field_enc;
mod file_enc;
mod kdf;
mod session;

pub use file_enc::{decrypt_file, encrypt_file};
pub use kdf::{derive_key, generate_salt};
pub use session::SessionManager;
