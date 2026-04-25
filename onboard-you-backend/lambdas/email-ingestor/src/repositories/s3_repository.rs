use async_trait::async_trait;
use aws_sdk_s3::Client as S3Client;
use std::sync::Arc;

/// Abstraction over S3 download/upload — implement this trait in tests to
/// avoid real AWS calls.
#[async_trait]
pub trait IS3Repo: Send + Sync {
    async fn download(&self, bucket: &str, key: &str) -> Result<Vec<u8>, String>;
    async fn upload(
        &self,
        bucket: &str,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), String>;
}

/// AWS S3-backed implementation of [`IS3Repo`].
pub struct S3Repository {
    s3: S3Client,
}

impl S3Repository {
    pub fn new(s3: S3Client) -> Arc<Self> {
        Arc::new(Self { s3 })
    }
}

#[async_trait]
impl IS3Repo for S3Repository {
    async fn download(&self, bucket: &str, key: &str) -> Result<Vec<u8>, String> {
        let resp = self
            .s3
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| format!("S3 GetObject failed ({bucket}/{key}): {e}"))?;

        let bytes = resp
            .body
            .collect()
            .await
            .map_err(|e| format!("Failed to read S3 body: {e}"))?;

        Ok(bytes.into_bytes().to_vec())
    }

    async fn upload(
        &self,
        bucket: &str,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), String> {
        self.s3
            .put_object()
            .bucket(bucket)
            .key(key)
            .content_type(content_type)
            .body(bytes.into())
            .send()
            .await
            .map_err(|e| format!("S3 PutObject failed ({bucket}/{key}): {e}"))?;
        Ok(())
    }
}

