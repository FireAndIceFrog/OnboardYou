//! Config repository — reads PipelineConfig from DynamoDB using serde_dynamo.

use aws_sdk_dynamodb::types::AttributeValue;
use lambda_runtime::Error;
use serde::Deserialize;
use serde_dynamo::aws_sdk_dynamodb_1 as dynamo_serde;

/// Lightweight projection of the stored config — only the fields the ETL trigger needs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredConfig {
    pub organization_id: String,
    pub pipeline: serde_json::Value,
}

/// Fetch a pipeline config by organization ID (full document deserialization).
pub async fn get(
    dynamo: &aws_sdk_dynamodb::Client,
    table_name: &str,
    organization_id: &str,
) -> Result<StoredConfig, Error> {
    let result = dynamo
        .get_item()
        .table_name(table_name)
        .key("organizationId", AttributeValue::S(organization_id.to_string()))
        .send()
        .await
        .map_err(|e| Error::from(format!("get_item failed: {e}")))?;

    let item = result
        .item
        .ok_or_else(|| Error::from(format!("No config found for org: {organization_id}")))?;

    let config: StoredConfig = dynamo_serde::from_item(item)
        .map_err(|e| Error::from(format!("Failed to deserialize config: {e}")))?;

    Ok(config)
}
