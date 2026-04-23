//! AWS Textract repository — document analysis via the Textract API.
//!
//! The trait is thin: it returns normalised `TextractBlock`s so the
//! pure parsing logic in `onboard_you::capabilities::conversion::textract_parser`
//! can be tested independently of AWS.

use async_trait::async_trait;
use aws_sdk_textract::types::{BlockType, Document, FeatureType, RelationshipType, S3Object};

use onboard_you::capabilities::conversion::textract_parser::TextractBlock;

use crate::models::ApiError;

/// Trait for Textract document analysis — injectable for testing.
#[async_trait]
pub trait TextractRepo: Send + Sync {
    /// Call `AnalyzeDocument` on an S3 object and return a flat list of blocks.
    ///
    /// The caller is responsible for extracting tables via
    /// `textract_parser::extract_tables`.
    async fn analyze_s3_document(
        &self,
        key: &str,
    ) -> Result<Vec<TextractBlock>, ApiError>;
}

pub struct TextractRepository {
    pub textract: aws_sdk_textract::Client,
    pub bucket: String,
}

#[async_trait]
impl TextractRepo for TextractRepository {
    async fn analyze_s3_document(
        &self,
        key: &str,
    ) -> Result<Vec<TextractBlock>, ApiError> {
        let doc = Document::builder()
            .s3_object(
                S3Object::builder()
                    .bucket(&self.bucket)
                    .name(key)
                    .build(),
            )
            .build();

        let resp = self
            .textract
            .analyze_document()
            .document(doc)
            .feature_types(FeatureType::Tables)
            .send()
            .await
            .map_err(|e| {
                ApiError::Repository(format!(
                    "Textract AnalyzeDocument failed for s3://{}/{}: {e}",
                    self.bucket, key
                ))
            })?;

        let blocks = resp
            .blocks()
            .iter()
            .map(|b| {
                let child_ids = b
                    .relationships()
                    .iter()
                    .filter(|r| r.r#type() == Some(&RelationshipType::Child))
                    .flat_map(|r| r.ids().iter().cloned())
                    .collect();

                TextractBlock {
                    id: b.id().unwrap_or_default().to_string(),
                    block_type: match b.block_type() {
                        Some(BlockType::Table) => "TABLE".into(),
                        Some(BlockType::Cell) => "CELL".into(),
                        Some(BlockType::Word) => "WORD".into(),
                        Some(BlockType::Line) => "LINE".into(),
                        Some(BlockType::Page) => "PAGE".into(),
                        Some(other) => format!("{other:?}"),
                        None => String::new(),
                    },
                    text: b.text().map(ToOwned::to_owned),
                    row_index: b.row_index(),
                    column_index: b.column_index(),
                    child_ids,
                }
            })
            .collect();

        Ok(blocks)
    }
}
