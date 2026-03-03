//! Validation engine — dry-run column propagation using `CalculateColumns`.
//!
//! Parses the manifest, builds every action via the factory, and folds
//! `calculate_columns` through the pipeline without executing any real
//! transformations or touching external data sources.

use std::collections::HashMap;

use crate::{
    dependancies::Dependancies,
    models::{ApiError, StepValidation, ValidationResult},
};
use onboard_you_models::{ColumnMapping, DynamicEgressModel, SchemaDiff};
use onboard_you::ActionFactoryTrait;
use onboard_you_models::{Manifest, RosterContext};
use polars::prelude::*;

/// Compute the diff between pipeline `final_columns` and the egress schema.
///
/// The egress schema is a `HashMap<String, String>` where keys are pipeline
/// column names and values are destination field names. This function
/// categorises each column/field as mapped, unmapped-source, or unmapped-target.
pub fn compute_schema_diff(
    final_columns: &[String],
    egress_schema: &HashMap<String, String>,
) -> SchemaDiff {
    let mut mapped = Vec::new();
    let mut unmapped_source = Vec::new();

    for col in final_columns {
        if let Some(target) = egress_schema.get(col) {
            mapped.push(ColumnMapping {
                source_column: col.clone(),
                target_field: target.clone(),
            });
        } else {
            unmapped_source.push(col.clone());
        }
    }

    // Egress fields whose key doesn't appear in final_columns
    let final_set: std::collections::HashSet<&str> =
        final_columns.iter().map(|s| s.as_str()).collect();
    let mut unmapped_target: Vec<String> = egress_schema
        .keys()
        .filter(|k| !final_set.contains(k.as_str()))
        .cloned()
        .collect();
    unmapped_target.sort(); // deterministic ordering

    SchemaDiff {
        mapped,
        unmapped_source,
        unmapped_target,
    }
}

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
            schema_diff: None,
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

    // Compute schema diff if the manifest has an ApiDispatcher egress action
    let schema_diff = manifest
        .actions
        .iter()
        .find_map(|ac| match &ac.config {
            onboard_you_models::ActionConfigPayload::ApiDispatcher(cfg) => {
                Some(compute_schema_diff(&final_columns, &cfg.get_schema()))
            }
            _ => None,
        });

    Ok(ValidationResult {
        steps,
        final_columns,
        schema_diff,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependancies::{Dependancies, Env};

    // ---- compute_schema_diff unit tests ----

    #[test]
    fn schema_diff_all_mapped() {
        let final_columns = vec!["email".into(), "name".into()];
        let mut egress: HashMap<String, String> = HashMap::new();
        egress.insert("email".into(), "work_email".into());
        egress.insert("name".into(), "full_name".into());

        let diff = compute_schema_diff(&final_columns, &egress);

        assert_eq!(diff.mapped.len(), 2);
        assert!(diff.unmapped_source.is_empty());
        assert!(diff.unmapped_target.is_empty());
        // Verify mapping content
        assert!(diff
            .mapped
            .iter()
            .any(|m| m.source_column == "email" && m.target_field == "work_email"));
        assert!(diff
            .mapped
            .iter()
            .any(|m| m.source_column == "name" && m.target_field == "full_name"));
    }

    #[test]
    fn schema_diff_partial_mapping() {
        let final_columns = vec!["email".into(), "phone".into(), "dept".into()];
        let mut egress: HashMap<String, String> = HashMap::new();
        egress.insert("email".into(), "work_email".into());
        egress.insert("start_date".into(), "startDate".into());

        let diff = compute_schema_diff(&final_columns, &egress);

        assert_eq!(diff.mapped.len(), 1);
        assert_eq!(diff.mapped[0].source_column, "email");
        assert_eq!(diff.mapped[0].target_field, "work_email");
        assert_eq!(diff.unmapped_source, vec!["phone", "dept"]);
        assert_eq!(diff.unmapped_target, vec!["start_date"]);
    }

    #[test]
    fn schema_diff_empty_egress_schema() {
        let final_columns = vec!["a".into(), "b".into(), "c".into()];
        let egress: HashMap<String, String> = HashMap::new();

        let diff = compute_schema_diff(&final_columns, &egress);

        assert!(diff.mapped.is_empty());
        assert_eq!(diff.unmapped_source, vec!["a", "b", "c"]);
        assert!(diff.unmapped_target.is_empty());
    }

    #[test]
    fn schema_diff_empty_final_columns() {
        let final_columns: Vec<String> = vec![];
        let mut egress: HashMap<String, String> = HashMap::new();
        egress.insert("email".into(), "work_email".into());

        let diff = compute_schema_diff(&final_columns, &egress);

        assert!(diff.mapped.is_empty());
        assert!(diff.unmapped_source.is_empty());
        assert_eq!(diff.unmapped_target, vec!["email"]);
    }

    #[test]
    fn schema_diff_both_empty() {
        let diff = compute_schema_diff(&[], &HashMap::new());

        assert!(diff.mapped.is_empty());
        assert!(diff.unmapped_source.is_empty());
        assert!(diff.unmapped_target.is_empty());
    }

    // ---- validate_pipeline integration tests ----

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
