pub mod git;
mod s3;
mod sqlite;
mod traits;

pub use s3::S3Backend;
pub use sqlite::{DeviceRecord, SqliteStore, SyncVersionRecord};
pub use traits::CloudBackend;
