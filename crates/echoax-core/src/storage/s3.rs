use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;

use crate::error::{EchoAccessError, Result};
use crate::storage::traits::CloudBackend;

fn map_sdk_err<E: std::error::Error + Send + Sync + 'static>(
    op: &'static str,
    err: SdkError<E>,
) -> EchoAccessError {
    EchoAccessError::Storage(format!("S3 {op}: {err}"))
}

fn is_not_found<E>(err: &SdkError<E>) -> bool {
    let msg = err.to_string();
    msg.contains("NotFound")
        || msg.contains("404")
        || msg.contains("NoSuchKey")
        || msg.contains("Not Found")
}

/// S3-compatible backend (Aliyun OSS, MinIO, AWS S3) using a custom endpoint URL.
pub struct S3Backend {
    client: Client,
    bucket: String,
}

fn infer_signing_region(endpoint: &str) -> String {
    // e.g. https://bucket.oss-cn-beijing.aliyuncs.com → cn-beijing
    if let Some(idx) = endpoint.find("oss-") {
        let after = &endpoint[idx + "oss-".len()..];
        if let Some(dot) = after.find('.') {
            return after[..dot].to_string();
        }
    }
    "us-east-1".to_string()
}

impl S3Backend {
    /// Builds a client for `endpoint` (e.g. `https://echo-access-data.oss-cn-beijing.aliyuncs.com`),
    /// `bucket`, and static access keys. Signing region is inferred from Aliyun-style endpoints
    /// (`oss-<region>.aliyuncs.com`); otherwise defaults to `us-east-1`.
    pub fn new(
        endpoint: impl Into<String>,
        bucket: impl Into<String>,
        access_key_id: impl Into<String>,
        access_key_secret: impl Into<String>,
    ) -> Self {
        let endpoint = endpoint.into();
        let bucket = bucket.into();
        let access_key_id = access_key_id.into();
        let access_key_secret = access_key_secret.into();
        let region = infer_signing_region(&endpoint);

        let credentials = Credentials::new(
            access_key_id,
            access_key_secret,
            None,
            None,
            "echoax-static",
        );

        let config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(region))
            .credentials_provider(credentials)
            .endpoint_url(endpoint)
            .force_path_style(false)
            .build();

        let client = Client::from_conf(config);
        Self { client, bucket }
    }

    /// Bucket name configured for this backend.
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

#[async_trait::async_trait]
impl CloudBackend for S3Backend {
    async fn upload(&self, key: &str, data: &[u8]) -> Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data.to_vec()))
            .send()
            .await
            .map_err(|e| map_sdk_err("put_object", e))?;
        Ok(())
    }

    async fn download(&self, key: &str) -> Result<Vec<u8>> {
        let out = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| map_sdk_err("get_object", e))?;
        let aggregated = out
            .body
            .collect()
            .await
            .map_err(|e| EchoAccessError::Storage(format!("S3 read body: {e}")))?;
        Ok(aggregated.into_bytes().to_vec())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| map_sdk_err("delete_object", e))?;
        Ok(())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        let mut continuation_token = None::<String>;

        loop {
            let resp = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix)
                .set_continuation_token(continuation_token.clone())
                .send()
                .await
                .map_err(|e| map_sdk_err("list_objects_v2", e))?;

            for obj in resp.contents() {
                if let Some(k) = obj.key() {
                    keys.push(k.to_string());
                }
            }

            continuation_token = resp.next_continuation_token().map(|t| t.to_string());
            if continuation_token.is_none() {
                break;
            }
        }

        Ok(keys)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) if is_not_found(&e) => Ok(false),
            Err(e) => Err(map_sdk_err("head_object", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::traits::CloudBackend;

    struct NoopBackend;

    #[async_trait::async_trait]
    impl CloudBackend for NoopBackend {
        async fn upload(&self, _key: &str, _data: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn download(&self, _key: &str) -> Result<Vec<u8>> {
            Ok(Vec::new())
        }

        async fn delete(&self, _key: &str) -> Result<()> {
            Ok(())
        }

        async fn list(&self, _prefix: &str) -> Result<Vec<String>> {
            Ok(Vec::new())
        }

        async fn exists(&self, _key: &str) -> Result<bool> {
            Ok(false)
        }
    }

    #[tokio::test]
    async fn cloud_backend_trait_is_object_safe() {
        let backend: Box<dyn CloudBackend> = Box::new(NoopBackend);
        assert!(backend.exists("k").await.unwrap() == false);
    }

    #[test]
    fn s3_backend_construction() {
        let b = S3Backend::new(
            "https://echo-access-data.oss-cn-beijing.aliyuncs.com",
            "my-bucket",
            "test-access-key-id",
            "test-secret",
        );
        assert_eq!(b.bucket(), "my-bucket");
    }

}
