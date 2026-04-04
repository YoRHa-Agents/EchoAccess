use crate::error::{EchoAccessError, Result};
use crate::storage::traits::CloudBackend;

pub struct S3Backend {
    endpoint: String,
    bucket: String,
    prefix: String,
}

impl S3Backend {
    pub fn new(endpoint: String, bucket: String, prefix: String) -> Self {
        Self {
            endpoint,
            bucket,
            prefix,
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    fn full_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }
}

#[async_trait::async_trait]
impl CloudBackend for S3Backend {
    async fn upload(&self, key: &str, _data: &[u8]) -> Result<()> {
        let _full = self.full_key(key);
        // TODO: implement with reqwest or aws-sdk-s3 (behind feature flag)
        Err(EchoAccessError::Storage(
            "S3 upload: awaiting SDK integration".into(),
        ))
    }

    async fn download(&self, key: &str) -> Result<Vec<u8>> {
        let _full = self.full_key(key);
        Err(EchoAccessError::Storage(
            "S3 download: awaiting SDK integration".into(),
        ))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let _full = self.full_key(key);
        Err(EchoAccessError::Storage(
            "S3 delete: awaiting SDK integration".into(),
        ))
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let _full = self.full_key(prefix);
        Err(EchoAccessError::Storage(
            "S3 list: awaiting SDK integration".into(),
        ))
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let _full = self.full_key(key);
        Err(EchoAccessError::Storage(
            "S3 exists: awaiting SDK integration".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s3_backend_construction() {
        let backend = S3Backend::new(
            "https://oss-cn-beijing.aliyuncs.com".into(),
            "echo-access-data".into(),
            "user/".into(),
        );
        assert_eq!(backend.endpoint(), "https://oss-cn-beijing.aliyuncs.com");
        assert_eq!(backend.bucket(), "echo-access-data");
    }

    #[test]
    fn full_key_includes_prefix() {
        let backend = S3Backend::new(String::new(), String::new(), "pfx/".into());
        assert_eq!(backend.full_key("file.txt"), "pfx/file.txt");
    }

    #[tokio::test]
    async fn methods_return_awaiting_integration() {
        let backend = S3Backend::new(String::new(), String::new(), String::new());
        assert!(backend.upload("k", b"v").await.is_err());
        assert!(backend.download("k").await.is_err());
    }
}
