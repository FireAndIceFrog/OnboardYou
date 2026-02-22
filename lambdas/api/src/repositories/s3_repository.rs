//! S3 operations for CSV upload management.
//!
//! Generates presigned PUT URLs for direct browser uploads and reads
//! uploaded CSV headers for column discovery.

use async_trait::async_trait;
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;

use crate::models::ApiError;

/// Default presigned URL TTL: 24 hours.
const PRESIGN_TTL: Duration = Duration::from_secs(60 * 60 * 24);
pub struct S3Repository {
    pub s3: aws_sdk_s3::Client,
    pub bucket: String,
}

#[async_trait]
pub trait S3Repo: Send + Sync {
    async fn presigned_put_url(&self, key: &str) -> Result<String, ApiError>;
    async fn read_csv_headers(&self, key: &str) -> Result<Vec<String>, ApiError>;
}

#[async_trait]
impl S3Repo for S3Repository {
    /// Generate a presigned PUT URL so the frontend can upload a CSV directly to S3.
    ///
    /// The S3 key follows the convention: `{organization_id}/{customer_company_id}/{filename}`.
    async fn presigned_put_url(
        &self,
        key: &str,
    ) -> Result<String, ApiError> {
        let presign_config = PresigningConfig::builder()
            .expires_in(PRESIGN_TTL)
            .build()
            .map_err(|e| ApiError::Repository(format!("Presigning config error: {e}")))?;

        let presign_output: aws_sdk_s3::presigning::PresignedRequest = self.s3
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type("text/csv")
            .presigned(presign_config)
            .await
            .map_err(|e| ApiError::Repository(format!("Failed to generate presigned URL: {e}")))?;

        Ok(presign_output.uri().to_string())
    }

    /// Download the first few KB of a CSV from S3 and extract the header row.
    ///
    /// Returns the column names parsed from the first line. This is called after
    /// the frontend successfully uploads the file.
    async fn read_csv_headers(
        &self,
        key: &str,
    ) -> Result<Vec<String>, ApiError> {
        // Fetch only the first 8KB — plenty for the header row.
        let resp = self.s3
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .range("bytes=0-8191")
            .send()
            .await
            .map_err(|e| {
                ApiError::Repository(format!(
                    "Failed to read CSV headers from s3://{}/{}: {e}",
                    self.bucket, key
                ))
            })?;

        let bytes = resp
            .body
            .collect()
            .await
            .map_err(|e| ApiError::Repository(format!("Failed to read S3 body: {e}")))?;

        let raw = bytes.into_bytes();
        let text = String::from_utf8_lossy(&raw);

        let first_line = text.lines().next().ok_or_else(|| {
            ApiError::Validation("Uploaded CSV is empty — no header row found".into())
        })?;

        let columns: Vec<String> = first_line
            .split(',')
            .map(|col| col.trim().trim_matches('"').to_string())
            .filter(|col| !col.is_empty())
            .collect();

        if columns.is_empty() {
            return Err(ApiError::Validation(
                "CSV header row contains no valid column names".into(),
            ));
        }

        Ok(columns)
    }
}