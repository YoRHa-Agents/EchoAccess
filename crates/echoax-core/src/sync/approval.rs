use crate::error::{EchoAccessError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSync {
    pub id: String,
    pub path: String,
    pub action: SyncAction,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SyncAction {
    Upload,
    Download,
    Delete,
}

pub struct ApprovalQueue {
    pending: Vec<PendingSync>,
}

impl ApprovalQueue {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    pub fn enqueue(&mut self, item: PendingSync) {
        self.pending.push(item);
    }

    pub fn approve(&mut self, id: &str) -> Result<PendingSync> {
        let pos = self
            .pending
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| EchoAccessError::Sync(format!("No pending item with id: {id}")))?;
        Ok(self.pending.remove(pos))
    }

    pub fn reject(&mut self, id: &str) -> Result<()> {
        let pos = self
            .pending
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| EchoAccessError::Sync(format!("No pending item with id: {id}")))?;
        self.pending.remove(pos);
        Ok(())
    }

    pub fn list_pending(&self) -> &[PendingSync] {
        &self.pending
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

impl Default for ApprovalQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_item(id: &str) -> PendingSync {
        PendingSync {
            id: id.into(),
            path: "/test/path".into(),
            action: SyncAction::Upload,
            timestamp: "2026-04-03T00:00:00Z".into(),
        }
    }

    #[test]
    fn enqueue_and_list() {
        let mut q = ApprovalQueue::new();
        assert!(q.is_empty());
        q.enqueue(sample_item("1"));
        q.enqueue(sample_item("2"));
        assert_eq!(q.list_pending().len(), 2);
    }

    #[test]
    fn approve_removes_item() {
        let mut q = ApprovalQueue::new();
        q.enqueue(sample_item("a"));
        let approved = q.approve("a").unwrap();
        assert_eq!(approved.id, "a");
        assert!(q.is_empty());
    }

    #[test]
    fn approve_nonexistent_returns_error() {
        let mut q = ApprovalQueue::new();
        assert!(q.approve("missing").is_err());
    }

    #[test]
    fn reject_removes_item() {
        let mut q = ApprovalQueue::new();
        q.enqueue(sample_item("r"));
        q.reject("r").unwrap();
        assert!(q.is_empty());
    }
}
