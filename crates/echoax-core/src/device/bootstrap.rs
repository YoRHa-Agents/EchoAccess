use crate::error::{EchoAccessError, Result};

pub async fn bootstrap_device(_host: &str, _binary_path: Option<&str>) -> Result<()> {
    // TODO: implement remote device bootstrap
    // 1. Detect target OS via SSH (uname -s)
    // 2. Upload echoax binary
    // 3. Create ~/.config/echoax/ directory
    // 4. Run echoax init
    Err(EchoAccessError::Network(
        "Device bootstrap not yet implemented".into(),
    ))
}
