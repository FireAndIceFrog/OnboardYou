//! Request/response types for plan generation.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /config/{id}/generate-plan`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePlanRequest {
    /// Source system name — "Workday" or "CSV"
    pub source_system: String,
}

/// Response body for `POST /config/{id}/generate-plan` (202 Accepted).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GeneratePlanResponse {
    /// Current generation status
    pub status: String,
}
