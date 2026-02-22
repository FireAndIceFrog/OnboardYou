use utoipa::ToSchema;


/// Result of validating a single step in the pipeline.
#[derive(Debug, Clone, serde::Serialize, ToSchema)]
pub struct StepValidation {
    /// Action id from the manifest.
    pub action_id: String,
    /// Action type (e.g. `csv_hris_connector`, `drop_column`).
    pub action_type: String,
    /// Columns present *after* this step completes.
    pub columns_after: Vec<String>,
}

/// Overall validation result for the entire pipeline.
#[derive(Debug, Clone, serde::Serialize, ToSchema)]
pub struct ValidationResult {
    /// Per-step column snapshots (in execution order).
    pub steps: Vec<StepValidation>,
    /// Final column set after the last step.
    pub final_columns: Vec<String>,
}