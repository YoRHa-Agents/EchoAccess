use crate::error::Result;

pub struct SchedulerTrigger {
    interval_secs: u64,
    running: bool,
}

impl SchedulerTrigger {
    pub fn new(interval_secs: u64) -> Self {
        Self {
            interval_secs,
            running: false,
        }
    }

    pub fn interval_secs(&self) -> u64 {
        self.interval_secs
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub async fn start(&mut self) -> Result<()> {
        self.running = true;
        tracing::info!(
            "Scheduler started with {}s interval (stub)",
            self.interval_secs
        );
        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_construction() {
        let s = SchedulerTrigger::new(3600);
        assert_eq!(s.interval_secs(), 3600);
        assert!(!s.is_running());
    }
}
