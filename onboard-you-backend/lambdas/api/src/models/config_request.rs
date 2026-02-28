//! Inbound request body for creating / updating a pipeline config.
//!
//! Omits server-controlled fields (`organizationId`, `customerCompanyId`,
//! `lastEdited`) that the controller fills from Claims + Path.

use onboard_you_models::Manifest;
use serde::Deserialize;
use utoipa::ToSchema;

use onboard_you_models::PipelineConfig;

/// Request body for `POST /config/{id}` and `PUT /config/{id}`.
///
/// Clients send only the fields they own; the server stamps identity
/// and timestamps before persisting.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRequest {
    /// Name of the pipeline
    pub name: String,

    /// Optional image/icon for the pipeline
    pub image: Option<String>,

    /// EventBridge-compatible schedule expression (cron or rate)
    pub cron: String,

    /// The full ETL pipeline manifest
    pub pipeline: Manifest,
}

impl ConfigRequest {
    /// Convert into a full `PipelineConfig`, filling in server-side fields.
    pub fn into_config(self) -> PipelineConfig {
        PipelineConfig {
            name: self.name,
            image: self.image,
            cron: self.cron,
            organization_id: String::new(),
            customer_company_id: String::new(),
            last_edited: String::new(),
            pipeline: self.pipeline,
        }
    }
}
