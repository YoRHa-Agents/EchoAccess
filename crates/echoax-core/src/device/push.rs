use std::path::PathBuf;

use crate::error::{EchoAccessError, Result};

pub struct PushEntry {
    pub source: PathBuf,
    pub target: PathBuf,
    pub encrypted: bool,
}

pub async fn push_to_device(_host: &str, _files: &[PushEntry]) -> Result<()> {
    // TODO: implement actual SSH push via openssh crate
    Err(EchoAccessError::Network(
        "Device push not yet implemented".into(),
    ))
}
