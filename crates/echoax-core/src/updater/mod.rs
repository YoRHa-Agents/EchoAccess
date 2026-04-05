//! Auto-update: GitHub release checks and binary replacement (network/install are stubs).

mod checker;
mod installer;

pub use checker::{
    check_for_updates, get_platform_target, parse_github_release, semver_update_available,
    UpdateInfo,
};
pub use installer::{archive_extension, binary_name, install_update};
