//! Pipeline run log: tracks every ETL execution with status, timing,
//! warnings, validation results, and error context.
//!
//! The `PipelineRun` struct maps 1:1 to the `pipeline_runs` table.
//! Warnings are accumulated during execution via `PipelineWarning` and
//! serialized as a JSON array into the `warnings` column.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::Manifest;

/// Result of validating a single step in the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StepValidation {
    /// Action id from the manifest.
    pub action_id: String,
    /// Action type (e.g. `csv_hris_connector`, `drop_column`).
    pub action_type: String,
    /// Columns present *after* this step completes.
    pub columns_after: Vec<String>,
}

/// Overall validation result for the entire pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationResult {
    /// Per-step column snapshots (in execution order).
    pub steps: Vec<StepValidation>,
    /// Final column set after the last step.
    pub final_columns: Vec<String>,
}

/// A single warning emitted by an action during pipeline execution.
///
/// Warnings are non-fatal — the pipeline continues but the client
/// should be informed so they can fix their source data.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PipelineWarning {
    /// Which action emitted the warning (e.g. `"cellphone_sanitizer"`).
    pub action_id: String,
    /// Human-readable message.
    pub message: String,
    /// How many rows were affected.
    pub count: usize,
    /// Optional extra detail (e.g. the un-parseable values).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Status of a pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Running,
    Success,
    Failed,
    ValidationFailed,
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunStatus::Running => write!(f, "running"),
            RunStatus::Success => write!(f, "success"),
            RunStatus::Failed => write!(f, "failed"),
            RunStatus::ValidationFailed => write!(f, "validation_failed"),
        }
    }
}

impl std::str::FromStr for RunStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "running" => Ok(RunStatus::Running),
            "success" => Ok(RunStatus::Success),
            "failed" => Ok(RunStatus::Failed),
            "validation_failed" => Ok(RunStatus::ValidationFailed),
            other => Err(format!("unknown run status: {other}")),
        }
    }
}

/// A pipeline run record as stored in the `pipeline_runs` table.
#[macro_rules_attribute::apply(crate::SqlRow!)]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PipelineRun {
    /// UUID identifying this specific run.
    pub id: String,
    /// Organization that owns the pipeline.
    pub organization_id: String,
    /// Customer company the pipeline ran for.
    pub customer_company_id: String,
    /// Current status of the run.
    pub status: String,
    /// When the run started (ISO 8601).
    pub started_at: String,
    /// When the run finished (ISO 8601), if it has.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    /// Number of rows successfully processed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows_processed: Option<i32>,
    /// Action ID currently being executed (updated during the run).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_action: Option<String>,
    /// Error message if the run failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Which action caused the failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_action_id: Option<String>,
    /// Row index where the error occurred (if determinable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_row: Option<i32>,
    /// Warnings accumulated during the run.
    #[json]
    pub warnings: Vec<PipelineWarning>,
    /// Validation result from the pre-run dry-run (steps + columns).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[json]
    pub validation_result: Option<ValidationResult>,
    /// Snapshot of the manifest at the time of the run.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[json]
    pub manifest_snapshot: Option<Manifest>,
}
