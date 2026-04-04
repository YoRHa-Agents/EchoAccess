use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SyncStatus {
    Synced,
    Modified,
    New,
    Deleted,
    Conflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    pub path: String,
    pub hash: String,
    pub status: SyncStatus,
    pub last_modified: String,
}

impl FileState {
    pub fn new(path: String, hash: String) -> Self {
        Self {
            path,
            hash,
            status: SyncStatus::New,
            last_modified: String::new(),
        }
    }

    pub fn is_modified_since(&self, other: &Self) -> bool {
        self.hash != other.hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_file_state() {
        let fs = FileState::new("a.txt".into(), "abc123".into());
        assert_eq!(fs.status, SyncStatus::New);
        assert_eq!(fs.path, "a.txt");
    }

    #[test]
    fn modified_detection() {
        let a = FileState::new("a.txt".into(), "hash1".into());
        let b = FileState::new("a.txt".into(), "hash2".into());
        assert!(a.is_modified_since(&b));
        assert!(!a.is_modified_since(&a));
    }
}
