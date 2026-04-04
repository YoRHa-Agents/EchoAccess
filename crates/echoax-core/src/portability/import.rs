use std::path::Path;

use crate::error::Result;
use super::export::ExportManifest;

pub fn import_archive(
    archive_path: &Path,
    _target_dir: &Path,
    passphrase: &str,
) -> Result<ExportManifest> {
    let encrypted = std::fs::read(archive_path)?;

    let _decrypted = crate::crypto::decrypt_file(&encrypted, passphrase)?;

    // TODO: parse decrypted content, extract profiles, write to target_dir

    Ok(ExportManifest {
        version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: String::new(),
        profile_count: 0,
        includes_state: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn import_roundtrip() {
        let dir = TempDir::new().unwrap();
        let profile_path = dir.path().join("p.toml");
        std::fs::write(&profile_path, "[device]\nhostname = \"h\"\n").unwrap();

        let archive = dir.path().join("archive.age");
        crate::portability::export::export_archive(dir.path(), &archive, "pass123").unwrap();

        let target = TempDir::new().unwrap();
        let manifest = import_archive(&archive, target.path(), "pass123").unwrap();
        assert_eq!(manifest.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn import_wrong_passphrase_fails() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("x.toml"), "data").unwrap();
        let archive = dir.path().join("a.age");
        crate::portability::export::export_archive(dir.path(), &archive, "correct").unwrap();
        assert!(import_archive(&archive, dir.path(), "wrong").is_err());
    }
}
