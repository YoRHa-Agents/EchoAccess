use crate::error::Result;

/// Returns the expected binary name for the current platform.
pub fn binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "echo_access.exe"
    } else {
        "echo_access"
    }
}

/// Returns the archive extension for the current platform.
pub fn archive_extension() -> &'static str {
    if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    }
}

/// Downloads a release artifact, verifies SHA-256, and replaces the running binary.
/// Stub: logs intent only; does not download or call `self_replace` yet.
pub async fn install_update(download_url: &str, checksum_url: &str) -> Result<()> {
    tracing::info!(
        download_url = %download_url,
        checksum_url = %checksum_url,
        "Update installation not yet fully implemented"
    );
    // TODO: implement actual download + self_replace when self_update is connected
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn install_stub_ok() {
        install_update("https://example.com/bin", "https://example.com/sha256")
            .await
            .unwrap();
    }

    #[test]
    fn binary_name_platform() {
        if cfg!(target_os = "windows") {
            assert_eq!(binary_name(), "echo_access.exe");
        } else {
            assert_eq!(binary_name(), "echo_access");
        }
    }

    #[test]
    fn archive_extension_platform() {
        if cfg!(target_os = "windows") {
            assert_eq!(archive_extension(), "zip");
        } else {
            assert_eq!(archive_extension(), "tar.gz");
        }
    }
}
