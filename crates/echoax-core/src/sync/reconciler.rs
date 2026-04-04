use crate::error::Result;

use super::merge::{self, MergeResult};
use super::state::{FileState, SyncStatus};

pub struct SyncEngine;

impl SyncEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn compare_states(&self, source: &FileState, target: &FileState) -> SyncStatus {
        if source.hash == target.hash {
            SyncStatus::Synced
        } else {
            SyncStatus::Modified
        }
    }

    pub fn compute_diff(&self, source: &str, target: &str) -> String {
        merge::compute_diff(source, target)
    }

    pub fn three_way_merge(&self, base: &str, ours: &str, theirs: &str) -> Result<MergeResult> {
        merge::three_way_merge(base, ours, theirs)
    }
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_synced_states() {
        let engine = SyncEngine::new();
        let a = FileState::new("f".into(), "same".into());
        let b = FileState::new("f".into(), "same".into());
        assert_eq!(engine.compare_states(&a, &b), SyncStatus::Synced);
    }

    #[test]
    fn compare_modified_states() {
        let engine = SyncEngine::new();
        let a = FileState::new("f".into(), "hash1".into());
        let b = FileState::new("f".into(), "hash2".into());
        assert_eq!(engine.compare_states(&a, &b), SyncStatus::Modified);
    }
}
