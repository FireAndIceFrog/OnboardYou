use onboard_you::Manifest;
use serde::{Deserialize, Serialize};

/// The pipeline config as stored in DynamoDB and exchanged via the API.
///
/// ```json
/// {
///   "name": "Acme Onboarding Pipeline",
///   "cron": "rate(1 hour)",
///   "organizationId": "acme-corp",
///   "lastEdited": "2026-02-09T12:00:00Z",
///   "pipeline": { "version": "1.0", "actions": [...] }
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineConfig {
    /// Name of the pipeline
    pub name: String,

    pub image: Option<String>,

    /// EventBridge-compatible schedule expression (cron or rate)
    pub cron: String,

    /// Unique identifier for the organization (partition key)
    pub organization_id: String,

    /// ISO 8601 timestamp of last edit — set by the server
    #[serde(default)]
    pub last_edited: String,

    /// The full ETL pipeline manifest (passed through to the ETL Lambda)
    pub pipeline: Manifest,
}
