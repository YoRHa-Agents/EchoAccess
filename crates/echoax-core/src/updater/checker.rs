use semver::Version;
use serde_json::Value;

use crate::error::{EchoAccessError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub checksum_url: String,
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

/// Detects the current platform's Rust target triple.
pub fn get_platform_target() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        (os, arch) => {
            tracing::warn!("Unsupported platform: {os}/{arch}");
            "unknown"
        }
    }
}

/// Parses a GitHub Releases API JSON response to extract update info.
/// `json_str` is the raw response from `https://api.github.com/repos/{owner}/{repo}/releases/latest`.
/// `current_version` is the running binary's version (e.g. "0.1.3").
///
/// Returns [`UpdateInfo`] with `download_url` set to the matching platform asset,
/// and `checksum_url` set to the corresponding `.sha256` file.
pub fn parse_github_release(json_str: &str, current_version: &str) -> Result<UpdateInfo> {
    let release: Value = serde_json::from_str(json_str)
        .map_err(|e| EchoAccessError::Network(format!("Failed to parse release JSON: {e}")))?;

    let tag = release["tag_name"]
        .as_str()
        .ok_or_else(|| EchoAccessError::Network("Missing tag_name in release".into()))?;

    // Strip leading 'v' from tag for version comparison
    let latest_version = tag.strip_prefix('v').unwrap_or(tag);

    let has_update = semver_update_available(current_version, latest_version)?;

    let release_notes = release["body"].as_str().unwrap_or("").to_string();

    let platform = get_platform_target();
    let mut download_url = String::new();
    let mut checksum_url = String::new();

    if let Some(assets) = release["assets"].as_array() {
        for asset in assets {
            if let Some(name) = asset["name"].as_str() {
                if let Some(url) = asset["browser_download_url"].as_str() {
                    if name.contains(platform) && !name.ends_with(".sha256") {
                        download_url = url.to_string();
                    }
                    if name.contains(platform) && name.ends_with(".sha256") {
                        checksum_url = url.to_string();
                    }
                }
            }
        }
    }

    Ok(UpdateInfo {
        current_version: current_version.to_string(),
        latest_version: latest_version.to_string(),
        download_url,
        checksum_url,
        release_notes,
        has_update,
    })
}

/// Checks GitHub Releases for a newer version. Stub: does not call the network;
/// callers with JSON should use [`parse_github_release`].
pub async fn check_for_updates(current_version: &str, _github_repo: &str) -> Result<UpdateInfo> {
    Ok(UpdateInfo {
        current_version: current_version.to_string(),
        latest_version: current_version.to_string(),
        download_url: String::new(),
        checksum_url: String::new(),
        release_notes: String::new(),
        has_update: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RELEASE_JSON: &str = r###"{
  "tag_name": "v0.2.0",
  "body": "## Changes\n- New feature X",
  "assets": [
    {"name": "echoax-v0.2.0-x86_64-unknown-linux-gnu.tar.gz", "browser_download_url": "https://github.com/YoRHa-Agents/EchoAccess/releases/download/v0.2.0/echoax-v0.2.0-x86_64-unknown-linux-gnu.tar.gz"},
    {"name": "echoax-v0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256", "browser_download_url": "https://github.com/YoRHa-Agents/EchoAccess/releases/download/v0.2.0/echoax-v0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256"},
    {"name": "echoax-v0.2.0-x86_64-apple-darwin.tar.gz", "browser_download_url": "https://github.com/YoRHa-Agents/EchoAccess/releases/download/v0.2.0/echoax-v0.2.0-x86_64-apple-darwin.tar.gz"},
    {"name": "echoax-v0.2.0-x86_64-apple-darwin.tar.gz.sha256", "browser_download_url": "https://github.com/YoRHa-Agents/EchoAccess/releases/download/v0.2.0/echoax-v0.2.0-x86_64-apple-darwin.tar.gz.sha256"}
  ]
}"###;

    #[test]
    fn update_info_construction() {
        let info = UpdateInfo {
            current_version: "1.0.0".into(),
            latest_version: "1.0.0".into(),
            download_url: String::new(),
            checksum_url: String::new(),
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
        assert!(info.checksum_url.is_empty());
    }

    #[test]
    fn get_platform_target_known_triple() {
        let expected = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
            ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
            ("macos", "x86_64") => "x86_64-apple-darwin",
            ("macos", "aarch64") => "aarch64-apple-darwin",
            ("windows", "x86_64") => "x86_64-pc-windows-msvc",
            _ => {
                assert_eq!(get_platform_target(), "unknown");
                return;
            }
        };
        assert_eq!(get_platform_target(), expected);
        assert_ne!(get_platform_target(), "unknown");
    }

    #[test]
    fn parse_github_release_valid() {
        let info = parse_github_release(SAMPLE_RELEASE_JSON, "0.1.0").unwrap();
        assert!(info.has_update);
        assert_eq!(info.latest_version, "0.2.0");
        assert_eq!(info.current_version, "0.1.0");
        assert!(info.release_notes.contains("New feature X"));

        match get_platform_target() {
            "x86_64-unknown-linux-gnu" => {
                assert_eq!(
                    info.download_url,
                    "https://github.com/YoRHa-Agents/EchoAccess/releases/download/v0.2.0/echoax-v0.2.0-x86_64-unknown-linux-gnu.tar.gz"
                );
                assert_eq!(
                    info.checksum_url,
                    "https://github.com/YoRHa-Agents/EchoAccess/releases/download/v0.2.0/echoax-v0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256"
                );
            }
            "x86_64-apple-darwin" => {
                assert!(info.download_url.contains("x86_64-apple-darwin"));
                assert!(info.checksum_url.ends_with(".sha256"));
            }
            _ => {
                assert!(info.download_url.is_empty());
                assert!(info.checksum_url.is_empty());
            }
        }
    }

    #[test]
    fn parse_github_release_same_version_no_update() {
        let info = parse_github_release(SAMPLE_RELEASE_JSON, "0.2.0").unwrap();
        assert!(!info.has_update);
        assert_eq!(info.latest_version, "0.2.0");
    }

    #[test]
    fn parse_github_release_platform_asset_missing() {
        let json = r#"{
  "tag_name": "v0.2.0",
  "body": "",
  "assets": [
    {"name": "echoax-v0.2.0-aarch64-unknown-linux-gnu.tar.gz", "browser_download_url": "https://example.com/a.tar.gz"},
    {"name": "echoax-v0.2.0-aarch64-unknown-linux-gnu.tar.gz.sha256", "browser_download_url": "https://example.com/a.sha256"}
  ]
}"#;
        // Only aarch64-linux assets; x86_64-linux / darwin x86_64 should not match.
        if get_platform_target() == "aarch64-unknown-linux-gnu" {
            let info = parse_github_release(json, "0.1.0").unwrap();
            assert!(!info.download_url.is_empty());
            return;
        }
        let info = parse_github_release(json, "0.1.0").unwrap();
        assert!(info.download_url.is_empty());
        assert!(info.checksum_url.is_empty());
    }

    #[test]
    fn parse_github_release_malformed_json_errors() {
        let err = parse_github_release("{ not valid json", "0.1.0").unwrap_err();
        assert!(matches!(err, EchoAccessError::Network(_)));
    }
}
