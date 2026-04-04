use crate::error::Result;

pub struct ManualTrigger;

impl ManualTrigger {
    pub fn new() -> Self {
        Self
    }

    pub async fn trigger_sync(&self) -> Result<()> {
        tracing::info!("Manual sync triggered");
        Ok(())
    }
}

impl Default for ManualTrigger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn manual_trigger_works() {
        let trigger = ManualTrigger::new();
        trigger.trigger_sync().await.unwrap();
    }
}
