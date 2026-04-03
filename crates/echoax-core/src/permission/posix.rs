use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::error::{EchoAccessError, Result};
use crate::permission::policy::PermissionPolicy;

fn mode_for_policy(policy: PermissionPolicy) -> u32 {
    match policy {
        PermissionPolicy::OwnerOnly => 0o600,
        PermissionPolicy::OwnerRwOthersR => 0o644,
        PermissionPolicy::OwnerFull => 0o755,
        PermissionPolicy::OwnerDir => 0o700,
    }
}

/// Sets Unix permission bits on `path` according to `policy`.
pub fn apply_permission(path: &Path, policy: PermissionPolicy) -> Result<()> {
    let mode = mode_for_policy(policy);
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).map_err(|e| {
        EchoAccessError::Permission(format!("set_permissions {}: {e}", path.display()))
    })?;
    Ok(())
}

/// Returns `Ok(true)` if the permission bits of `path` match `policy` (masked with `0o777`).
pub fn verify_permission(path: &Path, policy: PermissionPolicy) -> Result<bool> {
    let expected = mode_for_policy(policy);
    let meta = std::fs::metadata(path).map_err(|e| {
        EchoAccessError::Permission(format!("metadata {}: {e}", path.display()))
    })?;
    let actual = meta.permissions().mode() & 0o777;
    Ok(actual == expected)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;

    use tempfile::tempdir;

    #[test]
    fn apply_and_verify_file_owner_only() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("secret");
        fs::write(&path, b"x").expect("write");
        apply_permission(&path, PermissionPolicy::OwnerOnly).expect("apply");
        assert!(verify_permission(&path, PermissionPolicy::OwnerOnly).expect("verify"));
    }

    #[test]
    fn verify_permission_false_when_mode_differs() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("pub");
        fs::write(&path, b"x").expect("write");
        apply_permission(&path, PermissionPolicy::OwnerRwOthersR).expect("apply");
        assert!(
            !verify_permission(&path, PermissionPolicy::OwnerOnly).expect("verify"),
            "0644 file should not match OwnerOnly"
        );
    }

    #[test]
    fn apply_and_verify_directory_owner_dir() {
        let dir = tempdir().expect("tempdir");
        let sub = dir.path().join("nested");
        fs::create_dir(&sub).expect("mkdir");
        apply_permission(&sub, PermissionPolicy::OwnerDir).expect("apply");
        assert!(verify_permission(&sub, PermissionPolicy::OwnerDir).expect("verify"));
    }

    #[test]
    fn apply_owner_full_on_file() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("script");
        fs::write(&path, b"#!").expect("write");
        apply_permission(&path, PermissionPolicy::OwnerFull).expect("apply");
        assert!(verify_permission(&path, PermissionPolicy::OwnerFull).expect("verify"));
    }
}
