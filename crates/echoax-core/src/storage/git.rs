use std::path::PathBuf;

use crate::error::{EchoAccessError, Result};
use super::traits::CloudBackend;

pub struct GitBackend {
    repo_path: PathBuf,
}

impl GitBackend {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }

    pub fn repo_path(&self) -> &PathBuf {
        &self.repo_path
    }
}

#[async_trait::async_trait]
impl CloudBackend for GitBackend {
    async fn upload(&self, _key: &str, _data: &[u8]) -> Result<()> {
        Err(EchoAccessError::Storage(
            "Git backend upload not yet implemented".into(),
        ))
    }

    async fn download(&self, _key: &str) -> Result<Vec<u8>> {
        Err(EchoAccessError::Storage(
            "Git backend download not yet implemented".into(),
        ))
    }

    async fn delete(&self, _key: &str) -> Result<()> {
        Err(EchoAccessError::Storage(
            "Git backend delete not yet implemented".into(),
        ))
    }

    async fn list(&self, _prefix: &str) -> Result<Vec<String>> {
        Err(EchoAccessError::Storage(
            "Git backend list not yet implemented".into(),
        ))
    }

    async fn exists(&self, _key: &str) -> Result<bool> {
        Err(EchoAccessError::Storage(
            "Git backend exists not yet implemented".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_backend_construction() {
        let backend = GitBackend::new(PathBuf::from("/tmp/repo"));
        assert_eq!(backend.repo_path(), &PathBuf::from("/tmp/repo"));
    }

    #[tokio::test]
    async fn git_methods_return_not_implemented() {
        let backend = GitBackend::new(PathBuf::from("/tmp/repo"));
        assert!(backend.upload("key", b"data").await.is_err());
        assert!(backend.download("key").await.is_err());
        assert!(backend.delete("key").await.is_err());
        assert!(backend.list("prefix").await.is_err());
        assert!(backend.exists("key").await.is_err());
    }
}
