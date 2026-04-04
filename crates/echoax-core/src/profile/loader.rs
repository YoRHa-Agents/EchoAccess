use std::path::Path;

use crate::error::Result;

use super::model::DeviceProfile;

pub fn load_profile(path: &Path) -> Result<DeviceProfile> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        crate::error::EchoAccessError::Profile(format!("Failed to read profile: {}", e))
    })?;
    let profile: DeviceProfile = toml::from_str(&content).map_err(|e| {
        crate::error::EchoAccessError::Profile(format!("Invalid profile TOML: {}", e))
    })?;
    validate_profile(&profile)?;
    Ok(profile)
}

pub fn validate_profile(profile: &DeviceProfile) -> Result<()> {
    if profile.device.hostname.is_empty() {
        return Err(crate::error::EchoAccessError::Profile(
            "hostname cannot be empty".into(),
        ));
    }
    if profile.sync_rules.is_empty() {
        return Err(crate::error::EchoAccessError::Profile(
            "at least one sync rule required".into(),
        ));
    }
    for rule in &profile.sync_rules {
        if rule.source.is_empty() || rule.target.is_empty() {
            return Err(crate::error::EchoAccessError::Profile(
                "sync rule source and target cannot be empty".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    fn valid_toml() -> &'static str {
        r#"
[device]
os = "linux"
role = "edge"
hostname = "node-1"

[[sync_rules]]
source = "/data/in"
target = "/data/out"
transforms = ["normalize"]
masked_fields = ["password"]
[sync_rules.field_overrides]
user = "default"
"#
    }

    #[test]
    fn load_valid_profile() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", valid_toml()).unwrap();
        let profile = load_profile(file.path()).unwrap();
        assert_eq!(profile.device.hostname, "node-1");
        assert_eq!(profile.sync_rules.len(), 1);
        assert_eq!(profile.sync_rules[0].source, "/data/in");
        assert_eq!(profile.sync_rules[0].target, "/data/out");
        assert_eq!(profile.sync_rules[0].transforms, vec!["normalize"]);
        assert_eq!(profile.sync_rules[0].masked_fields, vec!["password"]);
        assert_eq!(
            profile.sync_rules[0]
                .field_overrides
                .get("user")
                .map(String::as_str),
            Some("default")
        );
    }

    #[test]
    fn invalid_toml_returns_profile_error() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "this is not valid toml [[[").unwrap();
        let err = load_profile(file.path()).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("Invalid profile TOML") || msg.contains("Profile error"),
            "got: {msg}"
        );
    }

    #[test]
    fn empty_hostname_fails_validation() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[device]
os = "linux"
role = "edge"
hostname = ""

[[sync_rules]]
source = "a"
target = "b"
"#
        )
        .unwrap();
        let err = load_profile(file.path()).unwrap_err();
        assert!(
            format!("{err}").contains("hostname cannot be empty"),
            "got: {err}"
        );
    }

    #[test]
    fn empty_sync_rules_fails_validation() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
sync_rules = []

[device]
os = "linux"
role = "edge"
hostname = "h"
"#
        )
        .unwrap();
        let err = load_profile(file.path()).unwrap_err();
        assert!(
            format!("{err}").contains("at least one sync rule required"),
            "got: {err}"
        );
    }

    #[test]
    fn empty_rule_source_or_target_fails() {
        let profile_empty_source = r#"
[device]
os = "linux"
role = "edge"
hostname = "h"

[[sync_rules]]
source = ""
target = "b"
"#;
        let parsed: DeviceProfile = toml::from_str(profile_empty_source).unwrap();
        let err = validate_profile(&parsed).unwrap_err();
        assert!(format!("{err}").contains("source and target cannot be empty"));

        let profile_empty_target = r#"
[device]
os = "linux"
role = "edge"
hostname = "h"

[[sync_rules]]
source = "a"
target = ""
"#;
        let parsed: DeviceProfile = toml::from_str(profile_empty_target).unwrap();
        let err = validate_profile(&parsed).unwrap_err();
        assert!(format!("{err}").contains("source and target cannot be empty"));
    }

    #[test]
    fn field_overrides_roundtrip_in_toml() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[device]
os = "macos"
role = "dev"
hostname = "mac-1"

[[sync_rules]]
source = "s"
target = "t"

[sync_rules.field_overrides]
k1 = "v1"
k2 = "v2"
"#
        )
        .unwrap();
        let profile = load_profile(file.path()).unwrap();
        let overrides = &profile.sync_rules[0].field_overrides;
        assert_eq!(overrides.get("k1").map(String::as_str), Some("v1"));
        assert_eq!(overrides.get("k2").map(String::as_str), Some("v2"));
    }
}
