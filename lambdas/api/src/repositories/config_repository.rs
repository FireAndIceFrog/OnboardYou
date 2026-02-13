//! Config repository — DynamoDB persistence for PipelineConfig.
//!
//! Uses serde_dynamo to serialize/deserialize the entire struct in one shot.
//! No manual attribute mapping.

use aws_sdk_dynamodb::types::AttributeValue;
use serde_dynamo::aws_sdk_dynamodb_1 as dynamo_serde;

use crate::models::{ApiError, AppState, PipelineConfig};

/// Persist a PipelineConfig to DynamoDB (full document).
pub async fn put(state: &AppState, config: &PipelineConfig) -> Result<(), ApiError> {
    let item = dynamo_serde::to_item(config)
        .map_err(|e| ApiError::Repository(format!("Failed to serialize config: {e}")))?;

    state
        .dynamo
        .put_item()
        .table_name(&state.table_name)
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

/// Retrieve a PipelineConfig by organizationId (PK) + customerCompanyId (SK).
pub async fn get(
    state: &AppState,
    organization_id: &str,
    customer_company_id: &str,
) -> Result<Option<PipelineConfig>, ApiError> {
    let result = state
        .dynamo
        .get_item()
        .table_name(&state.table_name)
        .key("organizationId", AttributeValue::S(organization_id.to_string()))
        .key("customerCompanyId", AttributeValue::S(customer_company_id.to_string()))
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

/// List all PipelineConfigs for an organization (Query on PK).
pub async fn list(
    state: &AppState,
    organization_id: &str,
) -> Result<Vec<PipelineConfig>, ApiError> {
    let result = state
        .dynamo
        .query()
        .table_name(&state.table_name)
        .key_condition_expression("organizationId = :pk")
        .expression_attribute_values(
            ":pk",
            AttributeValue::S(organization_id.to_string()),
        )
        .send()
        .await
        .map_err(|e| ApiError::Repository(format!("query failed: {e}")))?;

    let items = result.items.unwrap_or_default();

    items
        .into_iter()
        .map(|item| {
            dynamo_serde::from_item(item)
                .map_err(|e| ApiError::Repository(format!("Failed to deserialize config: {e}")))
        })
        .collect()
}
