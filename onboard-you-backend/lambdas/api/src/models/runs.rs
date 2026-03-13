/// Response returned when a run is triggered.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct TriggerRunResponse {
    /// Confirmation message
    pub message: String,
}

/// Query parameters for listing runs.
#[derive(Debug, serde::Deserialize)]
pub struct ListRunsQuery {
    /// Page number (1-based, default 1).
    pub page: Option<i64>,
    /// Items per page (default 20, max 100).
    pub count_per_page: Option<i64>,
}
