use async_trait::async_trait;
use aws_sdk_textract::{
    types::{DocumentLocation, FeatureType, S3Object as TextractS3Obj},
    Client as TextractClient,
};
use std::sync::Arc;

/// Abstraction over AWS Textract document analysis.
#[async_trait]
pub trait ITextractRepo: Send + Sync {
    /// Run Textract on an object already in S3 and return the first table as CSV bytes.
    async fn convert_to_csv(
        &self,
        inbox_bucket: &str,
        s3_key: &str,
    ) -> Result<Vec<u8>, String>;
}

/// AWS Textract-backed implementation of [`ITextractRepo`].
pub struct TextractRepository {
    textract: TextractClient,
}

impl TextractRepository {
    pub fn new(textract: TextractClient) -> Arc<Self> {
        Arc::new(Self { textract })
    }
}

#[async_trait]
impl ITextractRepo for TextractRepository {
    async fn convert_to_csv(
        &self,
        inbox_bucket: &str,
        s3_key: &str,
    ) -> Result<Vec<u8>, String> {
        let job_id = self.start_job(inbox_bucket, s3_key).await?;
        let blocks = self.poll_until_done(&job_id).await?;
        Ok(crate::engine::textract_engine::blocks_to_csv(&blocks))
    }
}

impl TextractRepository {
    async fn start_job(&self, inbox_bucket: &str, s3_key: &str) -> Result<String, String> {
        let job = self
            .textract
            .start_document_analysis()
            .document_location(
                DocumentLocation::builder()
                    .s3_object(
                        TextractS3Obj::builder()
                            .bucket(inbox_bucket)
                            .name(s3_key)
                            .build(),
                    )
                    .build(),
            )
            .feature_types(FeatureType::Tables)
            .send()
            .await
            .map_err(|e| format!("Textract StartDocumentAnalysis failed: {e}"))?;

        job.job_id()
            .map(|id| id.to_string())
            .ok_or_else(|| "Textract returned no job ID".to_string())
    }

    async fn poll_until_done(
        &self,
        job_id: &str,
    ) -> Result<Vec<aws_sdk_textract::types::Block>, String> {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;

            let result = self
                .textract
                .get_document_analysis()
                .job_id(job_id)
                .send()
                .await
                .map_err(|e| format!("Textract GetDocumentAnalysis failed: {e}"))?;

            match result.job_status() {
                Some(s) if s.as_str() == "SUCCEEDED" => return Ok(result.blocks().to_vec()),
                Some(s) if s.as_str() == "FAILED" => {
                    return Err(format!("Textract job {job_id} failed"))
                }
                _ => tracing::info!(job_id, "Textract job still running…"),
            }
        }
    }
}
