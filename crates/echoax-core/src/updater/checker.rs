use semver::Version;

use crate::error::{EchoAccessError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub release_notes: String,
    pub has_update: bool,
}

/// Returns true if `latest_version` is strictly greater than `current_version` (semver).
pub fn semver_update_available(current_version: &str, latest_version: &str) -> Result<bool> {
    let current = Version::parse(current_version).map_err(|e| {
        EchoAccessError::Network(format!("invalid current semver '{current_version}': {e}"))
    })?;
    let latest = Version::parse(latest_version).map_err(|e| {
        EchoAccessError::Network(format!("invalid latest semver '{latest_version}': {e}"))
    })?;
    Ok(latest > current)
}

/// Checks GitHub Releases for a newer version. Stub: does not call the network.
pub async fn check_for_updates(current_version: &str, _github_repo: &str) -> Result<UpdateInfo> {
    Ok(UpdateInfo {
        current_version: current_version.to_string(),
        latest_version: current_version.to_string(),
        download_url: String::new(),
        release_notes: String::new(),
        has_update: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_info_construction() {
        let info = UpdateInfo {
            current_version: "1.0.0".into(),
            latest_version: "1.0.0".into(),
            download_url: String::new(),
            release_notes: String::new(),
            has_update: false,
        };
        assert!(!info.has_update);
        assert_eq!(info.current_version, "1.0.0");
    }

    #[test]
    fn semver_newer_when_patch_increases() {
        assert!(semver_update_available("1.0.0", "1.0.1").unwrap());
    }

    #[test]
    fn semver_not_newer_when_equal() {
        assert!(!semver_update_available("1.0.0", "1.0.0").unwrap());
    }

    #[test]
    fn semver_not_newer_when_older() {
        assert!(!semver_update_available("1.1.0", "1.0.9").unwrap());
    }

    #[test]
    fn semver_invalid_returns_network_error() {
        let err = semver_update_available("not-a-version", "1.0.0").unwrap_err();
        assert!(matches!(err, EchoAccessError::Network(_)));
    }

    #[tokio::test]
    async fn check_stub_returns_no_update() {
        let info = check_for_updates("2.3.4", "owner/repo").await.unwrap();
        assert!(!info.has_update);
        assert_eq!(info.current_version, "2.3.4");
        assert_eq!(info.latest_version, "2.3.4");
        assert!(info.download_url.is_empty());
    }
}
