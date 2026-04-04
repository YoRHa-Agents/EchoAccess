use std::path::Path;

use crate::error::Result;
use crate::permission::policy::PermissionPolicy;

/// Stub: Windows ACL integration is not implemented yet.
pub fn apply_permission(path: &Path, _policy: PermissionPolicy) -> Result<()> {
    tracing::warn!("Windows ACL not yet implemented for {:?}", path);
    Ok(())
}

/// Stub: treats permissions as satisfied until ACL checks exist.
pub fn verify_permission(path: &Path, _policy: PermissionPolicy) -> Result<bool> {
    tracing::warn!("Windows ACL verify not yet implemented for {:?}", path);
    Ok(true)
}
