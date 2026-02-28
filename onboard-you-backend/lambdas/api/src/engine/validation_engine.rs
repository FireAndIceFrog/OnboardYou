//! Validation engine — dry-run column propagation using `CalculateColumns`.
//!
//! Parses the manifest, builds every action via the factory, and folds
//! `calculate_columns` through the pipeline without executing any real
//! transformations or touching external data sources.

use crate::{
    dependancies::Dependancies,
    models::{ApiError, StepValidation, ValidationResult},
};
use onboard_you::{ActionFactoryTrait, Manifest, RosterContext};
use polars::prelude::*;

/// Validate a pipeline manifest by propagating columns through every step.
///
/// Returns the column set at each step, or an `ApiError` on the first failure.
pub fn validate_pipeline(
    deps: &Dependancies,
    pipeline_json: &Manifest,
) -> Result<ValidationResult, ApiError> {
    let manifest: Manifest = pipeline_json.clone();

    if manifest.actions.is_empty() {
        return Ok(ValidationResult {
            steps: vec![],
            final_columns: vec![],
        });
    }
    let action_factory = deps.etl_repo.create_action_factory();
    // Build every action via the factory (validates config too)
    let actions: Vec<_> = manifest
        .actions
        .iter()
        .map(|ac| {
            action_factory.create(ac).map_err(|e| {
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
            ApiError::Validation(format!("Schema resolution failed at '{}': {e}", ac.id))
        })?;

        let columns_after: Vec<String> = schema.iter_names().map(|n| n.to_string()).collect();

        steps.push(StepValidation {
            action_id: ac.id.clone(),
            action_type: ac.action_type.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependancies::{Dependancies, Env};

    #[tokio::test(flavor = "multi_thread")]
    async fn validate_empty_manifest_returns_empty() {
        let state = Dependancies::new(Env::default()).await;
        let manifest = Manifest {
            version: "1.0".into(),
            actions: vec![],
        };

        let res = validate_pipeline(&state, &manifest).expect("should succeed");
        assert!(res.steps.is_empty());
        assert!(res.final_columns.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn validate_propagates_csv_columns() {
        let state = Dependancies::new(Env::default()).await;

        let json = r#"{
            "version": "1.0",
            "actions": [
                { "id": "ingest", "action_type": "csv_hris_connector", "config": { "filename": "data.csv", "columns": ["a","b"] } }
            ]
        }"#;

        let manifest = Manifest::from_json(json).expect("parse manifest");

        let res = validate_pipeline(&state, &manifest).expect("validation should succeed");
        assert_eq!(res.steps.len(), 1);
        let step = &res.steps[0];
        assert_eq!(step.action_id, "ingest");
        assert_eq!(step.action_type, "csv_hris_connector");
        assert_eq!(step.columns_after, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(res.final_columns, vec!["a".to_string(), "b".to_string()]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn validate_rejects_bad_action_config() {
        let state = Dependancies::new(Env::default()).await;

        // Csv connector with empty columns -> factory.create should fail
        let json = r#"{
            "version": "1.0",
            "actions": [
                { "id": "ingest", "action_type": "csv_hris_connector", "config": { "filename": "data.csv", "columns": [] } }
            ]
        }"#;

        let manifest = Manifest::from_json(json).expect("parse manifest");

        let err = validate_pipeline(&state, &manifest).expect_err("should error");
        assert!(matches!(err, ApiError::Validation(_)));
    }
}
