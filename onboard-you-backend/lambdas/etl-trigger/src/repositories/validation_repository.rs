//! Validation repository — propagates columns through the pipeline and computes schema diffs.

use async_trait::async_trait;
use lambda_runtime::Error;
use std::collections::HashMap;
use std::sync::Arc;

use onboard_you::ActionFactoryTrait;
use onboard_you_models::{ActionConfigPayload, DynamicEgressModel, Manifest, RosterContext};

/// Result of running pipeline validation.
pub struct ValidationResult {
    /// The final column names after propagation through all actions.
    pub final_columns: Vec<String>,
    /// Human-readable schema diff (mapped / unmapped source / unmapped target).
    pub schema_diff: String,
}

/// Repository trait for pipeline validation logic.
#[async_trait]
pub trait IValidationRepo: Send + Sync {
    /// Validate a manifest by propagating columns through the pipeline
    /// and computing a schema diff against the egress configuration.
    fn validate(&self, manifest: &Manifest) -> ValidationResult;
}

/// Concrete implementation backed by the ETL `ActionFactory`.
pub struct ValidationRepository {
    pub action_factory: Arc<dyn ActionFactoryTrait>,
}

impl ValidationRepository {
    pub fn new(action_factory: Arc<dyn ActionFactoryTrait>) -> Arc<Self> {
        Arc::new(Self { action_factory })
    }
}

#[async_trait]
impl IValidationRepo for ValidationRepository {
    fn validate(&self, manifest: &Manifest) -> ValidationResult {
        let (final_columns, schema_diff) = run_validation(&self.action_factory, manifest);
        ValidationResult {
            final_columns,
            schema_diff,
        }
    }
}

/// Run validation and extract final columns + schema diff.
///
/// Uses the ETL action factory to propagate columns through the pipeline.
/// Returns `(final_columns, schema_diff_description)` — gracefully returns
/// empty data if validation fails (the AI can still generate a generic plan).
fn run_validation(
    factory: &Arc<dyn ActionFactoryTrait>,
    manifest: &Manifest,
) -> (Vec<String>, String) {
    // Build actions
    let actions: Vec<_> = match manifest
        .actions
        .iter()
        .map(|ac| factory.create(ac))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!("Validation failed during plan gen: {e}");
            return (vec![], String::new());
        }
    };

    // Propagate columns
    let mut context = RosterContext::new(polars::prelude::LazyFrame::default());
    for action in &actions {
        match action.calculate_columns(context) {
            Ok(ctx) => context = ctx,
            Err(e) => {
                tracing::warn!("Column calculation failed: {e}");
                return (vec![], String::new());
            }
        }
    }

    // Extract final columns
    let final_columns: Vec<String> = context
        .data
        .collect_schema()
        .map(|schema| schema.iter_names().map(|n| n.to_string()).collect())
        .unwrap_or_default();

    // Compute schema diff description
    let schema_diff = manifest
        .actions
        .iter()
        .find_map(|ac| match &ac.config {
            ActionConfigPayload::ApiDispatcher(cfg) => {
                let egress_schema = cfg.get_schema();
                Some(format_schema_diff(&final_columns, &egress_schema))
            }
            _ => None,
        })
        .unwrap_or_default();

    (final_columns, schema_diff)
}

/// Format schema diff as a human-readable string for the AI prompt.
fn format_schema_diff(
    final_columns: &[String],
    egress_schema: &HashMap<String, String>,
) -> String {
    let mut lines = Vec::new();

    for col in final_columns {
        if let Some(target) = egress_schema.get(col) {
            lines.push(format!("  MAPPED: {col} → {target}"));
        } else {
            lines.push(format!("  UNMAPPED SOURCE: {col} (no destination mapping)"));
        }
    }

    let final_set: std::collections::HashSet<&str> =
        final_columns.iter().map(|s| s.as_str()).collect();
    for key in egress_schema.keys() {
        if !final_set.contains(key.as_str()) {
            lines.push(format!(
                "  UNMAPPED TARGET: {key} (no source column)"
            ));
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_schema_diff() {
        let columns = vec!["name".into(), "email".into(), "extra".into()];
        let egress: HashMap<String, String> = [
            ("name".into(), "fullName".into()),
            ("missing".into(), "missingTarget".into()),
        ]
        .into_iter()
        .collect();

        let diff = format_schema_diff(&columns, &egress);
        assert!(diff.contains("MAPPED: name → fullName"));
        assert!(diff.contains("UNMAPPED SOURCE: email"));
        assert!(diff.contains("UNMAPPED SOURCE: extra"));
        assert!(diff.contains("UNMAPPED TARGET: missing"));
    }
}
