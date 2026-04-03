use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum EchoAccessError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Cryptography error: {0}")]
    Crypto(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Profile error: {0}")]
    Profile(String),

    #[error("Permission error: {0}")]
    Permission(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Network error: {0}")]
    Network(String),
}

pub type Result<T> = std::result::Result<T, EchoAccessError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_io_error() {
        let inner = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = EchoAccessError::from(inner);
        let msg = format!("{err}");
        assert!(msg.contains("I/O error"), "got: {msg}");
        assert!(msg.contains("file missing"), "got: {msg}");
    }

    #[test]
    fn display_config_error() {
        let err = EchoAccessError::Config("bad key".into());
        assert_eq!(format!("{err}"), "Configuration error: bad key");
    }

    #[test]
    fn display_crypto_error() {
        let err = EchoAccessError::Crypto("decrypt failed".into());
        assert_eq!(format!("{err}"), "Cryptography error: decrypt failed");
    }

    #[test]
    fn display_storage_error() {
        let err = EchoAccessError::Storage("db locked".into());
        assert_eq!(format!("{err}"), "Storage error: db locked");
    }

    #[test]
    fn display_sync_error() {
        let err = EchoAccessError::Sync("conflict".into());
        assert_eq!(format!("{err}"), "Sync error: conflict");
    }

    #[test]
    fn display_profile_error() {
        let err = EchoAccessError::Profile("not found".into());
        assert_eq!(format!("{err}"), "Profile error: not found");
    }

    #[test]
    fn display_permission_error() {
        let err = EchoAccessError::Permission("denied".into());
        assert_eq!(format!("{err}"), "Permission error: denied");
    }

    #[test]
    fn display_serialization_error() {
        let err = EchoAccessError::Serialization("invalid json".into());
        assert_eq!(format!("{err}"), "Serialization error: invalid json");
    }

    #[test]
    fn display_network_error() {
        let err = EchoAccessError::Network("timeout".into());
        assert_eq!(format!("{err}"), "Network error: timeout");
    }

    #[test]
    fn io_error_converts_via_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err: EchoAccessError = io_err.into();
        assert!(matches!(err, EchoAccessError::Io(_)));
    }

    #[test]
    fn result_alias_works() {
        fn example() -> Result<u32> {
            Ok(42)
        }
        assert_eq!(example().unwrap(), 42);
    }
}
