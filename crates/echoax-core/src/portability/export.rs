use std::path::Path;

use crate::error::{EchoAccessError, Result};

#[derive(Debug, Clone)]
pub struct ExportManifest {
    pub version: String,
    pub created_at: String,
    pub profile_count: usize,
    pub includes_state: bool,
}

pub fn export_archive(
    config_dir: &Path,
    output_path: &Path,
    passphrase: &str,
) -> Result<ExportManifest> {
    if !config_dir.exists() {
        return Err(EchoAccessError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Config directory not found: {}", config_dir.display()),
        )));
    }

    let profiles: Vec<_> = std::fs::read_dir(config_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "toml")
        })
        .collect();

    let mut archive_data = Vec::new();
    for entry in &profiles {
        let content = std::fs::read(entry.path())?;
        archive_data.extend_from_slice(&content);
        archive_data.push(b'\n');
    }

    let encrypted = crate::crypto::encrypt_file(&archive_data, passphrase)?;
    std::fs::write(output_path, &encrypted)?;

    Ok(ExportManifest {
        version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: String::new(),
        profile_count: profiles.len(),
        includes_state: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn export_nonexistent_dir_fails() {
        let result = export_archive(Path::new("/nonexistent"), Path::new("/tmp/out"), "pass");
        assert!(result.is_err());
    }
}
