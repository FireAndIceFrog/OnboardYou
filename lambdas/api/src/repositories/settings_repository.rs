//! Settings repository — DynamoDB persistence for OrgSettings.
//!
//! Uses serde_dynamo to serialize/deserialize the entire struct in one shot.
//! The table uses `organizationId` as the sole partition key (no sort key).

use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use serde_dynamo::aws_sdk_dynamodb_1 as dynamo_serde;

use crate::models::{ApiError, OrgSettings};

#[async_trait]
pub trait SettingsRepo: Send + Sync {
    async fn put(&self, settings: &OrgSettings) -> Result<(), ApiError>;
    async fn get(
        &self,
        organization_id: &str,
    ) -> Result<Option<OrgSettings>, ApiError>;
}

/// DynamoDB-backed implementation.
pub struct DynamoSettingsRepo {
    pub dynamo: aws_sdk_dynamodb::Client,
    pub table_name: String,
}

#[async_trait]
impl SettingsRepo for DynamoSettingsRepo {
    /// Persist an OrgSettings document to DynamoDB (full document).
    async fn put(&self, settings: &OrgSettings) -> Result<(), ApiError> {
        let item = dynamo_serde::to_item(settings)
            .map_err(|e| ApiError::Repository(format!("Failed to serialize settings: {e}")))?;

        self
            .dynamo
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| ApiError::Repository(format!("put_item failed: {e}")))?;

        tracing::info!(
            organization_id = %settings.organization_id,
            "Settings persisted"
        );
        Ok(())
    }

    /// Retrieve OrgSettings by organizationId (PK).
    async fn get(
        &self,
        organization_id: &str,
    ) -> Result<Option<OrgSettings>, ApiError> {
        let result = self
            .dynamo
            .get_item()
            .table_name(&self.table_name)
            .key(
                "organizationId",
                AttributeValue::S(organization_id.to_string()),
            )
            .send()
            .await
            .map_err(|e| ApiError::Repository(format!("get_item failed: {e}")))?;

        let Some(item) = result.item else {
            return Ok(None);
        };

        let settings: OrgSettings = dynamo_serde::from_item(item)
            .map_err(|e| ApiError::Repository(format!("Failed to deserialize settings: {e}")))?;

        Ok(Some(settings))
    }
}