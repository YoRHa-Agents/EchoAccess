use std::collections::HashMap;

/// Named field override for programmatic use; TOML profiles use `field_overrides` maps on [`SyncRule`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct FieldOverride {
    pub field: String,
    pub value: String,
}

impl FieldOverride {
    pub fn new(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct DeviceProfile {
    pub device: DeviceInfo,
    pub sync_rules: Vec<SyncRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct DeviceInfo {
    pub os: String,
    pub role: String,
    pub hostname: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SyncRule {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub transforms: Vec<String>,
    #[serde(default)]
    pub masked_fields: Vec<String>,
    #[serde(default)]
    pub field_overrides: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_profile_deserialize() {
        let toml_str = r#"
[device]
os = "linux"
role = "server"
hostname = "srv-01"

[[sync_rules]]
source = "ssh/config"
target = "~/.ssh/config"
transforms = ["strip_gui"]
masked_fields = ["Host desktop-*"]
[sync_rules.field_overrides]
"user.email" = "ops@company.com"
"#;
        let profile: DeviceProfile = toml::from_str(toml_str).unwrap();
        assert_eq!(profile.device.hostname, "srv-01");
        assert_eq!(profile.device.os, "linux");
        assert_eq!(profile.sync_rules.len(), 1);
        assert_eq!(profile.sync_rules[0].transforms, vec!["strip_gui"]);
        assert_eq!(
            profile.sync_rules[0]
                .field_overrides
                .get("user.email")
                .map(String::as_str),
            Some("ops@company.com")
        );
    }

    #[test]
    fn device_profile_serialize_roundtrip() {
        let profile = DeviceProfile {
            device: DeviceInfo {
                os: "macos".into(),
                role: "dev".into(),
                hostname: "mac-1".into(),
            },
            sync_rules: vec![SyncRule {
                source: "git/config".into(),
                target: "~/.gitconfig".into(),
                transforms: vec![],
                masked_fields: vec![],
                field_overrides: HashMap::new(),
            }],
        };
        let serialized = toml::to_string(&profile).unwrap();
        let deserialized: DeviceProfile = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.device.hostname, "mac-1");
    }
}
