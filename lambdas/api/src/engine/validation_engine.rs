//! Validation engine — dry-run column propagation using `CalculateColumns`.
//!
//! Parses the manifest, builds every action via the factory, and folds
//! `calculate_columns` through the pipeline without executing any real
//! transformations or touching external data sources.

use crate::models::ApiError;
use onboard_you::{ActionFactory, Manifest, RosterContext};
use polars::prelude::*;

/// Result of validating a single step in the pipeline.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StepValidation {
    /// Action id from the manifest.
    pub action_id: String,
    /// Action type (e.g. `csv_hris_connector`, `drop_column`).
    pub action_type: String,
    /// Columns present *after* this step completes.
    pub columns_after: Vec<String>,
}

/// Overall validation result for the entire pipeline.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationResult {
    /// Per-step column snapshots (in execution order).
    pub steps: Vec<StepValidation>,
    /// Final column set after the last step.
    pub final_columns: Vec<String>,
}

/// Validate a pipeline manifest by propagating columns through every step.
///
/// Returns the column set at each step, or an `ApiError` on the first failure.
pub fn validate_pipeline(pipeline_json: &serde_json::Value) -> Result<ValidationResult, ApiError> {
    let manifest: Manifest = serde_json::from_value(pipeline_json.clone()).map_err(|e| {
        ApiError::Validation(format!("Invalid pipeline manifest: {e}"))
    })?;

    if manifest.actions.is_empty() {
        return Ok(ValidationResult {
            steps: vec![],
            final_columns: vec![],
        });
    }

    // Build every action via the factory (validates config too)
    let actions: Vec<_> = manifest
        .actions
        .iter()
        .map(|ac| {
            ActionFactory::create(ac).map_err(|e| {
                ApiError::Validation(format!(
                    "Action '{}' (type '{}'): {e}",
                    ac.id, ac.action_type
                ))
            })
        })
        .collect::<Result<_, _>>()?;

    // Start with an empty context (no columns yet)
    let mut context = RosterContext::new(LazyFrame::default());
    let mut steps = Vec::with_capacity(actions.len());

    for (action, ac) in actions.iter().zip(manifest.actions.iter()) {
        context = action.calculate_columns(context).map_err(|e| {
            ApiError::Validation(format!(
                "Column calculation failed at '{}' (type '{}'): {e}",
                ac.id, ac.action_type
            ))
        })?;

        // Collect the schema from the lazy frame (cheap — no data)
        let schema = context.data.collect_schema().map_err(|e| {
            ApiError::Validation(format!(
                "Schema resolution failed at '{}': {e}",
                ac.id
            ))
        })?;

        let columns_after: Vec<String> = schema.iter_names().map(|n| n.to_string()).collect();

        steps.push(StepValidation {
            action_id: ac.id.clone(),
            action_type: ac.action_type.clone(),
            columns_after,
        });
    }

    let final_columns = steps
        .last()
        .map(|s| s.columns_after.clone())
        .unwrap_or_default();

    Ok(ValidationResult {
        steps,
        final_columns,
    })
}
