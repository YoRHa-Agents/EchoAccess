use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncGroup {
    pub id: GroupId,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub path_prefixes: Vec<String>,
    #[serde(default)]
    pub include_globs: Vec<String>,
    #[serde(default)]
    pub exclude_globs: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl SyncGroup {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: GroupId(id.into()),
            name: name.into(),
            description: String::new(),
            path_prefixes: Vec::new(),
            include_globs: Vec::new(),
            exclude_globs: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn matches_path(&self, path: &str) -> bool {
        if !self.path_prefixes.is_empty() && !self.path_prefixes.iter().any(|p| path.starts_with(p))
        {
            return false;
        }

        if !self.exclude_globs.is_empty()
            && self.exclude_globs.iter().any(|g| glob_matches(g, path))
        {
            return false;
        }

        if !self.include_globs.is_empty() {
            return self.include_globs.iter().any(|g| glob_matches(g, path));
        }

        !self.path_prefixes.is_empty()
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

fn glob_matches(pattern: &str, path: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(ext) = pattern.strip_prefix("*.") {
        return path.ends_with(&format!(".{ext}"));
    }
    if let Some(prefix) = pattern.strip_suffix("/*") {
        return path.starts_with(prefix);
    }
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path.starts_with(prefix);
    }
    pattern == path
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupStore {
    groups: Vec<SyncGroup>,
}

impl GroupStore {
    pub fn new() -> Self {
        Self { groups: Vec::new() }
    }

    pub fn add(&mut self, group: SyncGroup) -> bool {
        if self.groups.iter().any(|g| g.id == group.id) {
            return false;
        }
        self.groups.push(group);
        true
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.groups.len();
        self.groups.retain(|g| g.id.0 != id);
        self.groups.len() < before
    }

    pub fn get(&self, id: &str) -> Option<&SyncGroup> {
        self.groups.iter().find(|g| g.id.0 == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut SyncGroup> {
        self.groups.iter_mut().find(|g| g.id.0 == id)
    }

    pub fn list(&self) -> &[SyncGroup] {
        &self.groups
    }

    pub fn list_by_tag(&self, tag: &str) -> Vec<&SyncGroup> {
        self.groups.iter().filter(|g| g.has_tag(tag)).collect()
    }

    pub fn resolve_paths<'a>(&self, group_id: &str, all_paths: &'a [String]) -> Vec<&'a String> {
        let group = match self.get(group_id) {
            Some(g) => g,
            None => return Vec::new(),
        };
        all_paths.iter().filter(|p| group.matches_path(p)).collect()
    }
}

impl Default for GroupStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_matches_prefix() {
        let mut g = SyncGroup::new("ssh", "SSH Configs");
        g.path_prefixes = vec!["ssh/".into()];
        assert!(g.matches_path("ssh/config"));
        assert!(g.matches_path("ssh/id_ed25519"));
        assert!(!g.matches_path("git/config"));
    }

    #[test]
    fn group_matches_glob() {
        let mut g = SyncGroup::new("toml-files", "All TOML");
        g.include_globs = vec!["*.toml".into()];
        assert!(g.matches_path("git/gitconfig.toml"));
        assert!(g.matches_path("config.toml"));
        assert!(!g.matches_path("script.sh"));
    }

    #[test]
    fn group_excludes() {
        let mut g = SyncGroup::new("shell", "Shell Configs");
        g.path_prefixes = vec!["shell/".into()];
        g.exclude_globs = vec!["*.bak".into()];
        assert!(g.matches_path("shell/aliases.sh"));
        assert!(!g.matches_path("shell/old.bak"));
    }

    #[test]
    fn group_tags() {
        let mut g = SyncGroup::new("dev", "Dev Tools");
        g.tags = vec!["development".into(), "tools".into()];
        assert!(g.has_tag("development"));
        assert!(g.has_tag("tools"));
        assert!(!g.has_tag("production"));
    }

    #[test]
    fn store_crud() {
        let mut store = GroupStore::new();
        let g = SyncGroup::new("ssh", "SSH");
        assert!(store.add(g.clone()));
        assert!(!store.add(g));
        assert_eq!(store.list().len(), 1);
        assert!(store.get("ssh").is_some());
        assert!(store.remove("ssh"));
        assert!(store.list().is_empty());
    }

    #[test]
    fn store_resolve_paths() {
        let mut store = GroupStore::new();
        let mut g = SyncGroup::new("ssh", "SSH");
        g.path_prefixes = vec!["ssh/".into()];
        store.add(g);

        let paths: Vec<String> = vec![
            "ssh/config".into(),
            "ssh/id_ed25519".into(),
            "git/config".into(),
        ];
        let resolved = store.resolve_paths("ssh", &paths);
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn store_list_by_tag() {
        let mut store = GroupStore::new();
        let mut g1 = SyncGroup::new("ssh", "SSH");
        g1.tags = vec!["security".into()];
        let mut g2 = SyncGroup::new("git", "Git");
        g2.tags = vec!["dev".into()];
        let mut g3 = SyncGroup::new("gpg", "GPG");
        g3.tags = vec!["security".into()];
        store.add(g1);
        store.add(g2);
        store.add(g3);
        assert_eq!(store.list_by_tag("security").len(), 2);
        assert_eq!(store.list_by_tag("dev").len(), 1);
    }
}
