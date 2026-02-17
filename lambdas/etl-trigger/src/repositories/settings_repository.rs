//! Settings repository — reads OrgSettings from DynamoDB for the ETL trigger.
//!
//! Used to resolve `auth_type: "default"` before pipeline construction.

use aws_sdk_dynamodb::types::AttributeValue;
use lambda_runtime::Error;
use onboard_you::ApiDispatcherConfig;
use serde::Deserialize;
use serde_dynamo::aws_sdk_dynamodb_1 as dynamo_serde;

/// Lightweight projection of the stored org settings — only the field the ETL trigger needs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredSettings {
    /// Full auth configuration — typed `ApiDispatcherConfig`.
    pub default_auth: ApiDispatcherConfig,
}

/// Fetch organisation settings by organizationId.
///
/// Returns `None` if no settings row exists for the organisation.
pub async fn get(
    dynamo: &aws_sdk_dynamodb::Client,
    table_name: &str,
    organization_id: &str,
) -> Result<Option<StoredSettings>, Error> {
    let result = dynamo
        .get_item()
        .table_name(table_name)
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

    let settings: StoredSettings = dynamo_serde::from_item(item)
        .map_err(|e| Error::from(format!("Failed to deserialize settings: {e}")))?;

    Ok(Some(settings))
}
