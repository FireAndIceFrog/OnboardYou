//! Plan summary types — AI-generated plan summary for the pipeline editor.
//!
//! The plan summary is a **read-only projection** of the underlying manifest.
//! Each `PlanFeature` references the manifest action IDs it maps to, and each
//! manifest action carries a `disabled: bool` flag. Toggling a feature in
//! Normal mode sets `disabled` on its associated actions — no actions are ever
//! added or removed.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::SchemaGenerationStatus;

/// AI-generated plan summary cached on `PipelineConfig`.
///
/// Provides a human-readable description of the pipeline plus
/// toggleable features that map back to manifest actions.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanSummary {
    /// One-line headline (e.g. "Here's the plan to connect Workday to your App.")
    pub headline: String,
    /// Multi-sentence description of the pipeline
    pub description: String,
    /// Toggleable features — each references manifest action IDs
    pub features: Vec<PlanFeature>,
    /// Synthetic before/after preview data
    pub preview: PlanPreview,
    /// Current generation status (InProgress, Completed, Failed)
    pub generation_status: SchemaGenerationStatus,
}

/// A single feature card in the plan summary UI.
///
/// Each feature references one or more manifest actions via `action_ids`.
/// Toggling a feature flips the `disabled` flag on those actions.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanFeature {
    /// Unique identifier for this feature (e.g. "sync_start_dates")
    pub id: String,
    /// Icon name for the UI (e.g. "calendar", "users", "shield")
    pub icon: String,
    /// Human-readable label (e.g. "Sync Start Dates")
    pub label: String,
    /// Description of what this feature does
    pub description: String,
    /// References to the manifest action IDs this feature maps to
    pub action_ids: Vec<String>,
}

/// Synthetic before/after preview showing how data transforms.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanPreview {
    /// Label for the source side (e.g. "In Workday")
    pub source_label: String,
    /// Label for the target side (e.g. "In Your App")
    pub target_label: String,
    /// Sample record as it appears in the source system
    pub before: HashMap<String, String>,
    /// Sample record after pipeline transforms
    pub after: HashMap<String, String>,
    /// Warnings for destination fields that could not be mapped from the source.
    /// Each warning uses the sentinel value `__NEEDS_MAPPING__` in `after`
    /// and surfaces a human-readable message for user intervention.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<PreviewWarning>,
}

/// A warning for a destination field that has no matching source column.
///
/// The corresponding entry in `PlanPreview::after` will have the sentinel
/// value `__NEEDS_MAPPING__` to signal the UI that user action is required.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PreviewWarning {
    /// The destination field name that could not be mapped
    pub field: String,
    /// Human-readable explanation of why the mapping is missing
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_summary_round_trips() {
        let summary = PlanSummary {
            headline: "Here's the plan".into(),
            description: "A simple sync pipeline".into(),
            features: vec![PlanFeature {
                id: "active_only".into(),
                icon: "users".into(),
                label: "Active Employees Only".into(),
                description: "Only sync active employees".into(),
                action_ids: vec!["step_2".into()],
            }],
            preview: PlanPreview {
                source_label: "In Workday".into(),
                target_label: "In Your App".into(),
                before: [("name".into(), "Jane Doe".into())].into_iter().collect(),
                after: [("name".into(), "Jane Doe".into())].into_iter().collect(),
                warnings: vec![],
            },
            generation_status: SchemaGenerationStatus::Completed,
        };

        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: PlanSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.headline, summary.headline);
        assert_eq!(deserialized.features.len(), 1);
        assert_eq!(deserialized.features[0].id, "active_only");
    }

    #[test]
    fn plan_summary_with_all_statuses() {
        for status in [
            SchemaGenerationStatus::NotStarted,
            SchemaGenerationStatus::InProgress,
            SchemaGenerationStatus::Completed,
            SchemaGenerationStatus::Failed("oops".into()),
        ] {
            let summary = PlanSummary {
                headline: "h".into(),
                description: "d".into(),
                features: vec![],
                preview: PlanPreview {
                    source_label: "s".into(),
                    target_label: "t".into(),
                    before: HashMap::new(),
                    after: HashMap::new(),
                    warnings: vec![],
                },
                generation_status: status,
            };

            let json = serde_json::to_string(&summary).unwrap();
            let _: PlanSummary = serde_json::from_str(&json).unwrap();
        }
    }
}
