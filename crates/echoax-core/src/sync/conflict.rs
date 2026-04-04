use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictEntry {
    pub path: String,
    pub base_hash: String,
    pub ours_hash: String,
    pub theirs_hash: String,
    pub merged_content: Option<String>,
    pub resolved: bool,
}

impl ConflictEntry {
    pub fn new(path: String, base_hash: String, ours_hash: String, theirs_hash: String) -> Self {
        Self {
            path,
            base_hash,
            ours_hash,
            theirs_hash,
            merged_content: None,
            resolved: false,
        }
    }

    pub fn resolve(&mut self, content: String) {
        self.merged_content = Some(content);
        self.resolved = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_conflict_is_unresolved() {
        let c = ConflictEntry::new("f.txt".into(), "b".into(), "o".into(), "t".into());
        assert!(!c.resolved);
        assert!(c.merged_content.is_none());
    }

    #[test]
    fn resolve_marks_done() {
        let mut c = ConflictEntry::new("f.txt".into(), "b".into(), "o".into(), "t".into());
        c.resolve("merged".into());
        assert!(c.resolved);
        assert_eq!(c.merged_content.as_deref(), Some("merged"));
    }
}
