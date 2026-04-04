use std::path::PathBuf;
use std::sync::mpsc;

use crate::error::{EchoAccessError, Result};

pub struct WatcherTrigger {
    paths: Vec<PathBuf>,
    debounce_ms: u64,
}

impl WatcherTrigger {
    pub fn new(paths: Vec<PathBuf>, debounce_ms: u64) -> Self {
        Self { paths, debounce_ms }
    }

    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.paths
    }

    pub fn debounce_ms(&self) -> u64 {
        self.debounce_ms
    }

    pub fn start(&self) -> Result<mpsc::Receiver<PathBuf>> {
        let (tx, rx) = mpsc::channel();
        // TODO: integrate with notify crate for real file watching
        tracing::info!("File watcher started for {} paths (stub)", self.paths.len());
        drop(tx);
        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watcher_construction() {
        let w = WatcherTrigger::new(vec![PathBuf::from("/tmp")], 2000);
        assert_eq!(w.watched_paths().len(), 1);
        assert_eq!(w.debounce_ms(), 2000);
    }
}
