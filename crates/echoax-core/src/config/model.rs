use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{EchoAccessError, Result};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub session: SessionConfig,
    #[serde(default)]
    pub trigger: TriggerConfig,
    #[serde(default)]
    pub cloud: CloudConfig,
    #[serde(default)]
    pub update: UpdateConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_language() -> String {
    "en".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            language: default_language(),
            theme: default_theme(),
            auto_start: false,
            log_level: default_log_level(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub auto_lock: bool,
}

fn default_timeout_secs() -> u64 {
    300
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout_secs(),
            auto_lock: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default)]
    pub on_login: bool,
}

fn default_hotkey() -> String {
    "Ctrl+Shift+E".to_string()
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            hotkey: default_hotkey(),
            on_login: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default = "default_sync_interval_secs")]
    pub sync_interval_secs: u64,
}

fn default_sync_interval_secs() -> u64 {
    60
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: String::new(),
            sync_interval_secs: default_sync_interval_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    #[serde(default = "default_true")]
    pub auto_check: bool,
    #[serde(default = "default_check_interval_hours")]
    pub check_interval_hours: u64,
    #[serde(default)]
    pub channel: String,
}

fn default_true() -> bool {
    true
}

fn default_check_interval_hours() -> u64 {
    24
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_check: default_true(),
            check_interval_hours: default_check_interval_hours(),
            channel: String::new(),
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(EchoAccessError::Io)?;
        Self::from_toml_str(&content)
    }

    pub fn from_toml_str(s: &str) -> Result<Self> {
        toml::from_str(s).map_err(|e| EchoAccessError::Config(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_full_config() {
        let toml_str = r#"
[general]
language = "zh"
theme = "light"
auto_start = true
log_level = "debug"

[session]
timeout_secs = 600
auto_lock = true

[trigger]
hotkey = "Ctrl+E"
on_login = true

[cloud]
enabled = true
endpoint = "https://api.example.com"
sync_interval_secs = 120

[update]
auto_check = false
channel = "beta"
check_interval_hours = 48
"#;
        let cfg = AppConfig::from_toml_str(toml_str).unwrap();
        assert_eq!(cfg.general.language, "zh");
        assert_eq!(cfg.general.theme, "light");
        assert!(cfg.general.auto_start);
        assert_eq!(cfg.general.log_level, "debug");
        assert_eq!(cfg.session.timeout_secs, 600);
        assert!(cfg.session.auto_lock);
        assert_eq!(cfg.trigger.hotkey, "Ctrl+E");
        assert!(cfg.trigger.on_login);
        assert!(cfg.cloud.enabled);
        assert_eq!(cfg.cloud.endpoint, "https://api.example.com");
        assert_eq!(cfg.cloud.sync_interval_secs, 120);
        assert!(!cfg.update.auto_check);
        assert_eq!(cfg.update.channel, "beta");
        assert_eq!(cfg.update.check_interval_hours, 48);
    }

    #[test]
    fn deserialize_empty_uses_defaults() {
        let cfg = AppConfig::from_toml_str("").unwrap();
        assert_eq!(cfg.general.language, "en");
        assert_eq!(cfg.general.theme, "dark");
        assert!(!cfg.general.auto_start);
        assert_eq!(cfg.general.log_level, "info");
        assert_eq!(cfg.session.timeout_secs, 300);
        assert!(!cfg.session.auto_lock);
        assert_eq!(cfg.trigger.hotkey, "Ctrl+Shift+E");
        assert!(!cfg.trigger.on_login);
        assert!(!cfg.cloud.enabled);
        assert_eq!(cfg.cloud.sync_interval_secs, 60);
        assert!(cfg.update.auto_check);
        assert_eq!(cfg.update.check_interval_hours, 24);
    }

    #[test]
    fn deserialize_partial_config() {
        let toml_str = r#"
[general]
language = "fr"
"#;
        let cfg = AppConfig::from_toml_str(toml_str).unwrap();
        assert_eq!(cfg.general.language, "fr");
        assert!(!cfg.general.auto_start);
        assert_eq!(cfg.session.timeout_secs, 300);
    }

    #[test]
    fn invalid_toml_returns_config_error() {
        let result = AppConfig::from_toml_str("not valid [[[toml");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EchoAccessError::Config(_)));
    }

    #[test]
    fn load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
[general]
language = "de"
"#,
        )
        .unwrap();
        let cfg = AppConfig::load(&path).unwrap();
        assert_eq!(cfg.general.language, "de");
    }

    #[test]
    fn load_missing_file_returns_io_error() {
        let result = AppConfig::load(Path::new("/nonexistent/path/config.toml"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EchoAccessError::Io(_)));
    }

    #[test]
    fn roundtrip_serialize_deserialize() {
        let cfg = AppConfig::from_toml_str("").unwrap();
        let serialized = toml::to_string(&cfg).unwrap();
        let cfg2 = AppConfig::from_toml_str(&serialized).unwrap();
        assert_eq!(cfg.general.language, cfg2.general.language);
        assert_eq!(cfg.session.timeout_secs, cfg2.session.timeout_secs);
    }

    #[test]
    fn general_config_default_theme_is_dark() {
        let g = GeneralConfig::default();
        assert_eq!(g.theme, "dark");
    }

    #[test]
    fn general_config_serializes_theme_to_toml() {
        let g = GeneralConfig {
            language: "en".to_string(),
            theme: "light".to_string(),
            auto_start: false,
            log_level: "info".to_string(),
        };
        let serialized = toml::to_string(&g).unwrap();
        assert!(
            serialized.contains("theme = \"light\""),
            "expected theme in TOML: {serialized}"
        );
    }

    #[test]
    fn general_config_deserializes_theme_from_toml() {
        let toml_str = r#"
language = "en"
theme = "light"
auto_start = false
log_level = "info"
"#;
        let g: GeneralConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(g.theme, "light");
    }

    #[test]
    fn general_config_deserialize_without_theme_defaults_to_dark() {
        let toml_str = r#"
language = "fr"
auto_start = false
log_level = "info"
"#;
        let g: GeneralConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(g.theme, "dark");
    }

    #[test]
    fn app_config_roundtrip_preserves_theme() {
        let mut cfg = AppConfig::from_toml_str("").unwrap();
        cfg.general.theme = "light".to_string();
        let serialized = toml::to_string(&cfg).unwrap();
        let cfg2 = AppConfig::from_toml_str(&serialized).unwrap();
        assert_eq!(cfg2.general.theme, "light");
    }
}
