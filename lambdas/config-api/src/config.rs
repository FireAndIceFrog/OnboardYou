//! Shared configuration types for the Config API Lambda

use serde::{Deserialize, Serialize};

/// The pipeline config as stored in DynamoDB and exchanged via the API.
///
/// ```json
/// {
///   "cron": "rate(1 hour)",
///   "organizationId": "acme-corp",
///   "lastEdited": "2026-02-09T12:00:00Z",
///   "pipeline": { "version": "1.0", "actions": [...] }
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineConfig {
    /// EventBridge-compatible schedule expression (cron or rate)
    /// e.g. "cron(0 12 * * ? *)" or "rate(1 hour)"
    pub cron: String,

    /// Unique identifier for the organization
    pub organization_id: String,

    /// ISO 8601 timestamp of last edit — set by the server
    pub last_edited: String,

    /// The full ETL pipeline manifest (passed through to the ETL Lambda)
    pub pipeline: serde_json::Value,
}

/// Shared AWS client state, cloned per-request.
#[derive(Clone)]
pub struct AppState {
    pub dynamo: aws_sdk_dynamodb::Client,
    pub scheduler: aws_sdk_scheduler::Client,
    pub table_name: String,
    pub etl_lambda_arn: String,
    pub scheduler_role_arn: String,
}
