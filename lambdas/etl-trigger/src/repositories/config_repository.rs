//! Config repository — reads PipelineConfig from DynamoDB using serde_dynamo.

use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_runtime::Error;
use serde_dynamo::aws_sdk_dynamodb_1 as dynamo_serde;
use std::sync::Arc;

use onboard_you::PipelineConfig;

/// Repository trait used by the pipeline engine to fetch pipeline configs.
#[async_trait]
pub trait IConfigRepo: Send + Sync {
    async fn get(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<PipelineConfig, Error>;
}

/// Dynamo-backed implementation of `IConfigRepo`.
pub struct DynamoConfigRepo {
    pub dynamo: aws_sdk_dynamodb::Client,
    pub table_name: String,
}

impl DynamoConfigRepo {
    pub fn new(dynamo: aws_sdk_dynamodb::Client, table_name: String) -> Arc<Self> {
        Arc::new(Self { dynamo, table_name })
    }
}

#[async_trait]
impl IConfigRepo for DynamoConfigRepo {
    /// Fetch a pipeline config by organization ID + customer company ID (composite key).
    async fn get(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<PipelineConfig, Error> {
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
            .map_err(|e| Error::from(format!("get_item failed: {e}")))?;

        let item = result.item.ok_or_else(|| {
            Error::from(format!(
                "No config found for org: {organization_id}, customer: {customer_company_id}"
            ))
        })?;

        let config: PipelineConfig = dynamo_serde::from_item(item)
            .map_err(|e| Error::from(format!("Failed to deserialize config: {e}")))?;

        Ok(config)
    }
}
