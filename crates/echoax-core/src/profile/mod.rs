//! Device profile types and TOML loading.

pub mod loader;
pub mod model;

pub use loader::{load_profile, validate_profile};
pub use model::{DeviceInfo, DeviceProfile, FieldOverride, SyncRule};
