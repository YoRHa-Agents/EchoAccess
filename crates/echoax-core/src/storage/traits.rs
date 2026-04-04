use crate::error::Result;

/// S3-compatible object storage (Aliyun OSS, MinIO, AWS S3, etc.).
#[async_trait::async_trait]
pub trait CloudBackend: Send + Sync {
    async fn upload(&self, key: &str, data: &[u8]) -> Result<()>;
    async fn download(&self, key: &str) -> Result<Vec<u8>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;
    async fn exists(&self, key: &str) -> Result<bool>;
}
