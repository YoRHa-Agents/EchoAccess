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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStatus {
    Unresolved,
    Resolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictView {
    pub path: String,
    pub base_content: String,
    pub ours_content: String,
    pub theirs_content: String,
    pub merged_with_markers: Option<String>,
    pub conflict_count: usize,
    pub status: ConflictStatus,
}

impl ConflictView {
    pub fn from_entry(
        entry: &ConflictEntry,
        base: &str,
        ours: &str,
        theirs: &str,
        merged_with_markers: Option<String>,
        conflict_count: usize,
    ) -> Self {
        Self {
            path: entry.path.clone(),
            base_content: base.to_string(),
            ours_content: ours.to_string(),
            theirs_content: theirs.to_string(),
            merged_with_markers,
            conflict_count,
            status: if entry.resolved {
                ConflictStatus::Resolved
            } else {
                ConflictStatus::Unresolved
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Resolution {
    AcceptOurs,
    AcceptTheirs,
    AcceptBase,
    Custom(String),
}

impl Resolution {
    pub fn resolve_content(&self, base: &str, ours: &str, theirs: &str) -> String {
        match self {
            Resolution::AcceptOurs => ours.to_string(),
            Resolution::AcceptTheirs => theirs.to_string(),
            Resolution::AcceptBase => base.to_string(),
            Resolution::Custom(content) => content.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictStore {
    entries: Vec<ConflictEntry>,
}

impl ConflictStore {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, entry: ConflictEntry) {
        if let Some(existing) = self.entries.iter_mut().find(|e| e.path == entry.path) {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
    }

    pub fn get(&self, path: &str) -> Option<&ConflictEntry> {
        self.entries.iter().find(|e| e.path == path)
    }

    pub fn get_mut(&mut self, path: &str) -> Option<&mut ConflictEntry> {
        self.entries.iter_mut().find(|e| e.path == path)
    }

    pub fn list_unresolved(&self) -> Vec<&ConflictEntry> {
        self.entries.iter().filter(|e| !e.resolved).collect()
    }

    pub fn list_all(&self) -> &[ConflictEntry] {
        &self.entries
    }

    pub fn resolve(
        &mut self,
        path: &str,
        resolution: &Resolution,
        base: &str,
        ours: &str,
        theirs: &str,
    ) -> bool {
        if let Some(entry) = self.get_mut(path) {
            let content = resolution.resolve_content(base, ours, theirs);
            entry.resolve(content);
            true
        } else {
            false
        }
    }

    pub fn remove_resolved(&mut self) {
        self.entries.retain(|e| !e.resolved);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn unresolved_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.resolved).count()
    }
}

impl Default for ConflictStore {
    fn default() -> Self {
        Self::new()
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

    #[test]
    fn conflict_view_from_entry() {
        let entry = ConflictEntry::new("f.txt".into(), "b".into(), "o".into(), "t".into());
        let view = ConflictView::from_entry(&entry, "base", "ours", "theirs", None, 0);
        assert_eq!(view.path, "f.txt");
        assert_eq!(view.status, ConflictStatus::Unresolved);
        assert_eq!(view.base_content, "base");
    }

    #[test]
    fn resolution_accept_ours() {
        let r = Resolution::AcceptOurs;
        assert_eq!(r.resolve_content("base", "ours", "theirs"), "ours");
    }

    #[test]
    fn resolution_accept_theirs() {
        let r = Resolution::AcceptTheirs;
        assert_eq!(r.resolve_content("base", "ours", "theirs"), "theirs");
    }

    #[test]
    fn resolution_custom() {
        let r = Resolution::Custom("custom merge".into());
        assert_eq!(r.resolve_content("base", "ours", "theirs"), "custom merge");
    }

    #[test]
    fn conflict_store_crud() {
        let mut store = ConflictStore::new();
        store.add(ConflictEntry::new(
            "a.txt".into(),
            "b1".into(),
            "o1".into(),
            "t1".into(),
        ));
        store.add(ConflictEntry::new(
            "b.txt".into(),
            "b2".into(),
            "o2".into(),
            "t2".into(),
        ));

        assert_eq!(store.list_all().len(), 2);
        assert_eq!(store.unresolved_count(), 2);

        assert!(store.resolve("a.txt", &Resolution::AcceptOurs, "base", "ours", "theirs"));
        assert_eq!(store.unresolved_count(), 1);
        assert_eq!(store.list_unresolved().len(), 1);

        store.remove_resolved();
        assert_eq!(store.list_all().len(), 1);
    }

    #[test]
    fn conflict_store_upsert() {
        let mut store = ConflictStore::new();
        store.add(ConflictEntry::new(
            "a.txt".into(),
            "b1".into(),
            "o1".into(),
            "t1".into(),
        ));
        store.add(ConflictEntry::new(
            "a.txt".into(),
            "b2".into(),
            "o2".into(),
            "t2".into(),
        ));
        assert_eq!(store.list_all().len(), 1);
        assert_eq!(store.get("a.txt").unwrap().base_hash, "b2");
    }
}
