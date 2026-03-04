//! Request/response types for plan generation.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /config/{id}/generate-plan`.
///
/// Currently empty — source system is derived from the pipeline's ingress
/// connector. Kept as a struct so future fields can be added without a
/// breaking API change.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePlanRequest {}

/// Response body for `POST /config/{id}/generate-plan` (202 Accepted).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GeneratePlanResponse {
    /// Current generation status
    pub status: String,
}
