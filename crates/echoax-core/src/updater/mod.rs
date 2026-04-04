//! Auto-update: GitHub release checks and binary replacement (network/install are stubs).

mod checker;
mod installer;

pub use checker::{check_for_updates, semver_update_available, UpdateInfo};
pub use installer::install_update;
