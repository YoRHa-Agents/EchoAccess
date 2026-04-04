//! Sensitivity-driven file permission policies and platform apply/verify helpers.

mod policy;

#[cfg(unix)]
mod posix;
#[cfg(windows)]
mod windows;

pub use policy::{PermissionPolicy, Sensitivity};

#[cfg(unix)]
pub use posix::{apply_permission, verify_permission};
#[cfg(windows)]
pub use windows::{apply_permission, verify_permission};
