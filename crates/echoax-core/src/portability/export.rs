use std::path::Path;

use serde::Serialize;

use crate::error::{EchoAccessError, Result};
use crate::profile::DeviceProfile;

const REDACTED_VALUE: &str = "[REDACTED]";
const SECRET_FIELD_KEYWORDS: &[&str] = &[
    "password",
    "secret",
    "token",
    "credential",
    "private_key",
    "privatekey",
    "secret_key",
    "api_key",
    "apikey",
];

#[derive(Debug, Clone)]
pub struct ExportManifest {
    pub version: String,
    pub created_at: String,
    pub profile_count: usize,
    pub includes_state: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportProfilePreview {
    pub name: String,
    pub filename: String,
    pub hostname: String,
    pub os: String,
    pub role: String,
    pub rules_count: usize,
    pub redacted_fields: usize,
}

#[derive(Debug, Serialize)]
struct ArchivePayload {
    version: String,
    created_at: String,
    profiles: Vec<ArchiveProfile>,
}

#[derive(Debug, Serialize)]
struct ArchiveProfile {
    name: String,
    filename: String,
    hostname: String,
    os: String,
    role: String,
    redacted_fields: usize,
    content: String,
}

#[derive(Debug)]
struct PreparedExportProfile {
    summary: ExportProfilePreview,
    content: String,
}

pub fn preview_export_profiles(
    config_dir: &Path,
    filter: &str,
) -> Result<Vec<ExportProfilePreview>> {
    Ok(collect_export_profiles(config_dir, filter)?
        .into_iter()
        .map(|profile| profile.summary)
        .collect())
}

pub fn export_archive(
    config_dir: &Path,
    output_path: &Path,
    passphrase: &str,
) -> Result<ExportManifest> {
    export_archive_filtered(config_dir, output_path, passphrase, "")
}

pub fn export_archive_filtered(
    config_dir: &Path,
    output_path: &Path,
    passphrase: &str,
    filter: &str,
) -> Result<ExportManifest> {
    let prepared_profiles = collect_export_profiles(config_dir, filter)?;
    let created_at = export_timestamp();
    let archive_payload = ArchivePayload {
        version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: created_at.clone(),
        profiles: prepared_profiles
            .iter()
            .map(|profile| ArchiveProfile {
                name: profile.summary.name.clone(),
                filename: profile.summary.filename.clone(),
                hostname: profile.summary.hostname.clone(),
                os: profile.summary.os.clone(),
                role: profile.summary.role.clone(),
                redacted_fields: profile.summary.redacted_fields,
                content: profile.content.clone(),
            })
            .collect(),
    };

    let archive_data = serde_json::to_vec_pretty(&archive_payload).map_err(|e| {
        EchoAccessError::Serialization(format!("Failed to serialize export archive: {e}"))
    })?;
    let encrypted = crate::crypto::encrypt_file(&archive_data, passphrase)?;
    std::fs::write(output_path, &encrypted)?;

    Ok(ExportManifest {
        version: env!("CARGO_PKG_VERSION").to_string(),
        created_at,
        profile_count: prepared_profiles.len(),
        includes_state: false,
    })
}

fn collect_export_profiles(config_dir: &Path, filter: &str) -> Result<Vec<PreparedExportProfile>> {
    if !config_dir.exists() {
        return Err(EchoAccessError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Config directory not found: {}", config_dir.display()),
        )));
    }

    let normalized_filter = filter.trim().to_lowercase();
    let mut profile_paths: Vec<_> = std::fs::read_dir(config_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .collect();
    profile_paths.sort();

    let mut prepared = Vec::new();
    for path in profile_paths {
        let raw_content = std::fs::read_to_string(&path)?;
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_default();
        let filename = path
            .file_name()
            .map(|filename| filename.to_string_lossy().to_string())
            .unwrap_or_default();

        match crate::profile::load_profile(&path) {
            Ok(profile) => {
                if !matches_profile_filter(&name, &profile, &normalized_filter) {
                    continue;
                }

                let (redacted_profile, redacted_fields) = redact_profile(&profile);
                let content = toml::to_string_pretty(&redacted_profile).map_err(|e| {
                    EchoAccessError::Serialization(format!(
                        "Failed to serialize export profile '{}': {e}",
                        path.display()
                    ))
                })?;

                prepared.push(PreparedExportProfile {
                    summary: ExportProfilePreview {
                        name,
                        filename,
                        hostname: profile.device.hostname.clone(),
                        os: profile.device.os.clone(),
                        role: profile.device.role.clone(),
                        rules_count: profile.sync_rules.len(),
                        redacted_fields,
                    },
                    content,
                });
            }
            Err(_) => {
                if !matches_raw_profile_filter(&name, &raw_content, &normalized_filter) {
                    continue;
                }

                prepared.push(PreparedExportProfile {
                    summary: ExportProfilePreview {
                        name,
                        filename,
                        hostname: String::new(),
                        os: String::new(),
                        role: String::new(),
                        rules_count: 0,
                        redacted_fields: 0,
                    },
                    content: raw_content,
                });
            }
        }
    }

    Ok(prepared)
}

fn matches_profile_filter(name: &str, profile: &DeviceProfile, filter: &str) -> bool {
    if filter.is_empty() {
        return true;
    }

    if contains_filter(name, filter)
        || contains_filter(&profile.device.hostname, filter)
        || contains_filter(&profile.device.os, filter)
        || contains_filter(&profile.device.role, filter)
    {
        return true;
    }

    profile.sync_rules.iter().any(|rule| {
        contains_filter(&rule.source, filter)
            || contains_filter(&rule.target, filter)
            || rule
                .transforms
                .iter()
                .any(|transform| contains_filter(transform, filter))
            || rule
                .masked_fields
                .iter()
                .any(|masked| contains_filter(masked, filter))
            || rule
                .field_overrides
                .keys()
                .any(|field| contains_filter(field, filter))
    })
}

fn contains_filter(value: &str, filter: &str) -> bool {
    value.to_lowercase().contains(filter)
}

fn matches_raw_profile_filter(name: &str, raw_content: &str, filter: &str) -> bool {
    if filter.is_empty() {
        return true;
    }

    contains_filter(name, filter) || contains_filter(raw_content, filter)
}

fn redact_profile(profile: &DeviceProfile) -> (DeviceProfile, usize) {
    let mut redacted = profile.clone();
    let mut redacted_fields = 0;

    for rule in &mut redacted.sync_rules {
        let masked_fields = rule.masked_fields.clone();
        for (field, value) in &mut rule.field_overrides {
            if should_redact_field(field, &masked_fields) && value != REDACTED_VALUE {
                *value = REDACTED_VALUE.to_string();
                redacted_fields += 1;
            }
        }
    }

    (redacted, redacted_fields)
}

fn should_redact_field(field: &str, masked_fields: &[String]) -> bool {
    let normalized_field = field.to_lowercase();
    if SECRET_FIELD_KEYWORDS
        .iter()
        .any(|keyword| normalized_field.contains(keyword))
    {
        return true;
    }

    masked_fields
        .iter()
        .any(|pattern| wildcard_matches(pattern, field))
}

fn wildcard_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let value = value.to_lowercase();

    if pattern == "*" {
        return true;
    }

    if !pattern.contains('*') {
        return pattern == value;
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    let mut remaining = value.as_str();

    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if index == 0 && !pattern.starts_with('*') {
            if !remaining.starts_with(part) {
                return false;
            }
            remaining = &remaining[part.len()..];
            continue;
        }

        match remaining.find(part) {
            Some(position) => {
                remaining = &remaining[position + part.len()..];
            }
            None => {
                return false;
            }
        }
    }

    if !pattern.ends_with('*') {
        if let Some(last_part) = parts.iter().rev().find(|part| !part.is_empty()) {
            return value.ends_with(last_part);
        }
    }

    true
}

fn export_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::decrypt_file;
    use tempfile::TempDir;

    #[test]
    fn export_creates_file() {
        let dir = TempDir::new().unwrap();
        let profile_path = dir.path().join("test.toml");
        std::fs::write(&profile_path, "[device]\nhostname = \"test\"\n").unwrap();

        let output = dir.path().join("export.echoax.age");
        let manifest = export_archive(dir.path(), &output, "test-pass").unwrap();
        assert_eq!(manifest.profile_count, 1);
        assert!(output.exists());
    }

    #[test]
    fn preview_filters_profiles_and_counts_redactions() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("alpha.toml"),
            r#"
[device]
os = "linux"
role = "server"
hostname = "srv-alpha"

[[sync_rules]]
source = "ssh/config.base"
target = "~/.ssh/config"
masked_fields = ["password", "api_key"]
[sync_rules.field_overrides]
password = "hunter2"
api_key = "top-secret"
user = "deploy"
"#,
        )
        .unwrap();
        std::fs::write(
            dir.path().join("beta.toml"),
            r#"
[device]
os = "macos"
role = "dev"
hostname = "mac-beta"

[[sync_rules]]
source = "git/config"
target = "~/.gitconfig"
"#,
        )
        .unwrap();

        let preview = preview_export_profiles(dir.path(), "alpha").unwrap();
        assert_eq!(preview.len(), 1);
        assert_eq!(preview[0].name, "alpha");
        assert_eq!(preview[0].hostname, "srv-alpha");
        assert_eq!(preview[0].redacted_fields, 2);
    }

    #[test]
    fn filtered_export_redacts_secrets() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("alpha.toml"),
            r#"
[device]
os = "linux"
role = "server"
hostname = "srv-alpha"

[[sync_rules]]
source = "ssh/config.base"
target = "~/.ssh/config"
masked_fields = ["password"]
[sync_rules.field_overrides]
password = "hunter2"
api_key = "top-secret"
user = "deploy"
"#,
        )
        .unwrap();
        std::fs::write(
            dir.path().join("beta.toml"),
            r#"
[device]
os = "linux"
role = "edge"
hostname = "srv-beta"

[[sync_rules]]
source = "shell/aliases.sh"
target = "~/.aliases"
"#,
        )
        .unwrap();

        let output = dir.path().join("filtered.echoax.age");
        let manifest = export_archive_filtered(dir.path(), &output, "test-pass", "alpha").unwrap();
        assert_eq!(manifest.profile_count, 1);

        let encrypted = std::fs::read(&output).unwrap();
        let decrypted = decrypt_file(&encrypted, "test-pass").unwrap();
        let archive = String::from_utf8(decrypted).unwrap();

        assert!(archive.contains("\"name\": \"alpha\""));
        assert!(archive.contains("srv-alpha"));
        assert!(!archive.contains("srv-beta"));
        assert!(!archive.contains("hunter2"));
        assert!(!archive.contains("top-secret"));
        assert!(archive.contains(REDACTED_VALUE));
        assert!(archive.contains("deploy"));
    }

    #[test]
    fn export_nonexistent_dir_fails() {
        let result = export_archive(Path::new("/nonexistent"), Path::new("/tmp/out"), "pass");
        assert!(result.is_err());
    }
}
