//! DynamoDB persistence layer for pipeline configs

use aws_sdk_dynamodb::types::AttributeValue;
use lambda_http::Error;

use crate::config::{AppState, PipelineConfig};

/// Persist a PipelineConfig to DynamoDB.
///
/// Table schema:
///   PK: organizationId (String)
///   Attributes: cron, lastEdited, pipeline (JSON string)
pub async fn put_config(state: &AppState, config: &PipelineConfig) -> Result<(), Error> {
    let pipeline_json = serde_json::to_string(&config.pipeline)?;

    state
        .dynamo
        .put_item()
        .table_name(&state.table_name)
        .item("organizationId", AttributeValue::S(config.organization_id.clone()))
        .item("cron", AttributeValue::S(config.cron.clone()))
        .item("lastEdited", AttributeValue::S(config.last_edited.clone()))
        .item("pipeline", AttributeValue::S(pipeline_json))
        .send()
        .await
        .map_err(|e| Error::from(format!("DynamoDB put_item failed: {e}")))?;

    tracing::info!(
        organization_id = %config.organization_id,
        "Config persisted to DynamoDB"
    );

    Ok(())
}

/// Retrieve a PipelineConfig from DynamoDB by organizationId.
pub async fn get_config(
    state: &AppState,
    organization_id: &str,
) -> Result<Option<PipelineConfig>, Error> {
    let result = state
        .dynamo
        .get_item()
        .table_name(&state.table_name)
        .key("organizationId", AttributeValue::S(organization_id.to_string()))
        .send()
        .await
        .map_err(|e| Error::from(format!("DynamoDB get_item failed: {e}")))?;

    match result.item {
        Some(item) => {
            let cron = item
                .get("cron")
                .and_then(|v| v.as_s().ok())
                .cloned()
                .unwrap_or_default();

            let last_edited = item
                .get("lastEdited")
                .and_then(|v| v.as_s().ok())
                .cloned()
                .unwrap_or_default();

            let pipeline_str = item
                .get("pipeline")
                .and_then(|v| v.as_s().ok())
                .cloned()
                .unwrap_or_else(|| "{}".to_string());

            let pipeline: serde_json::Value = serde_json::from_str(&pipeline_str)?;

            Ok(Some(PipelineConfig {
                cron,
                organization_id: organization_id.to_string(),
                last_edited,
                pipeline,
            }))
        }
        None => Ok(None),
    }
}
