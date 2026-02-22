//! Config repository — trait + DynamoDB implementation for PipelineConfig persistence.
//!
//! Uses serde_dynamo to serialize/deserialize the entire struct in one shot.
//! No manual attribute mapping.

use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use serde_dynamo::aws_sdk_dynamodb_1 as dynamo_serde;

use crate::models::{ApiError, PipelineConfig};

/// Abstract persistence for pipeline configurations.
#[async_trait]
pub trait ConfigRepo: Send + Sync + 'static {
    async fn put(&self, config: &PipelineConfig) -> Result<(), ApiError>;
    async fn get(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<Option<PipelineConfig>, ApiError>;
    async fn list(&self, organization_id: &str) -> Result<Vec<PipelineConfig>, ApiError>;
    async fn delete(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<(), ApiError>;
}

/// DynamoDB-backed implementation.
pub struct DynamoConfigRepo {
    pub dynamo: aws_sdk_dynamodb::Client,
    pub table_name: String,
}

#[async_trait]
impl ConfigRepo for DynamoConfigRepo {
    async fn put(&self, config: &PipelineConfig) -> Result<(), ApiError> {
        let item = dynamo_serde::to_item(config)
            .map_err(|e| ApiError::Repository(format!("Failed to serialize config: {e}")))?;

        self.dynamo
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| ApiError::Repository(format!("put_item failed: {e}")))?;

        tracing::info!(
            organization_id = %config.organization_id,
            customer_company_id = %config.customer_company_id,
            "Config persisted"
        );
        Ok(())
    }

    async fn get(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<Option<PipelineConfig>, ApiError> {
        let result = self
            .dynamo
            .get_item()
            .table_name(&self.table_name)
            .key(
                "organizationId",
                AttributeValue::S(organization_id.to_string()),
            )
            .key(
                "customerCompanyId",
                AttributeValue::S(customer_company_id.to_string()),
            )
            .send()
            .await
            .map_err(|e| ApiError::Repository(format!("get_item failed: {e}")))?;

        let Some(item) = result.item else {
            return Ok(None);
        };

        let config: PipelineConfig = dynamo_serde::from_item(item)
            .map_err(|e| ApiError::Repository(format!("Failed to deserialize config: {e}")))?;

        Ok(Some(config))
    }

    async fn delete(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<(), ApiError> {
        self.dynamo
            .delete_item()
            .table_name(&self.table_name)
            .key(
                "organizationId",
                AttributeValue::S(organization_id.to_string()),
            )
            .key(
                "customerCompanyId",
                AttributeValue::S(customer_company_id.to_string()),
            )
            .send()
            .await
            .map_err(|e| ApiError::Repository(format!("delete_item failed: {e}")))?;

        tracing::info!(
            organization_id = %organization_id,
            customer_company_id = %customer_company_id,
            "Config deleted"
        );
        Ok(())
    }

    async fn list(&self, organization_id: &str) -> Result<Vec<PipelineConfig>, ApiError> {
        let result = self
            .dynamo
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("organizationId = :pk")
            .expression_attribute_values(":pk", AttributeValue::S(organization_id.to_string()))
            .send()
            .await
            .map_err(|e| ApiError::Repository(format!("query failed: {e}")))?;

        let items = result.items.unwrap_or_default();

        items
            .into_iter()
            .map(|item| {
                dynamo_serde::from_item(item)
                    .map_err(|e| {
                        ApiError::Repository(format!("Failed to deserialize config: {e}"))
                    })
            })
            .collect()
    }
}
