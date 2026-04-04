use crate::error::Result;

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
}
