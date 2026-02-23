//! Settings repository — reads OrgSettings from DynamoDB for the ETL trigger.
//!
//! Used to resolve `auth_type: "default"` before pipeline construction.

use std::sync::Arc;

use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_runtime::Error;
use serde_dynamo::aws_sdk_dynamodb_1 as dynamo_serde;

use onboard_you::OrgSettings;

/// Repository trait used by the pipeline engine to fetch org settings.
#[async_trait]
pub trait ISettingsRepo: Send + Sync {
    async fn get(&self, organization_id: &str) -> Result<Option<OrgSettings>, Error>;
}

/// Dynamo-backed implementation of `ISettingsRepo`.
pub struct DynamoSettingsRepo {
    pub dynamo: aws_sdk_dynamodb::Client,
    pub table_name: String,
}

impl DynamoSettingsRepo {
    pub fn new(dynamo: aws_sdk_dynamodb::Client, table_name: String) -> Arc<Self> {
        Arc::new(Self { dynamo, table_name })
    }
}

#[async_trait]
impl ISettingsRepo for DynamoSettingsRepo {
    /// Fetch organisation settings by organizationId.
    ///
    /// Returns `None` if no settings row exists for the organisation.
    async fn get(&self, organization_id: &str) -> Result<Option<OrgSettings>, Error> {
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
            .map_err(|e| Error::from(format!("get_item (settings) failed: {e}")))?;

        let Some(item) = result.item else {
            return Ok(None);
        };

        let settings: OrgSettings = dynamo_serde::from_item(item)
            .map_err(|e| Error::from(format!("Failed to deserialize settings: {e}")))?;

        Ok(Some(settings))
    }
}
