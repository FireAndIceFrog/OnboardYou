//! Schema repository — extracts egress schema and generates AI-powered plan summaries.

use async_trait::async_trait;
use gh_models::types::ChatMessage;
use lambda_runtime::Error;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::dependancies::Dependancies;
use onboard_you_models::{
    ActionConfigPayload, ActionType, DynamicEgressModel, Manifest, PlanFeature, PlanPreview,
    PlanPrompt, PlanSummary, SchemaGenerationStatus,
};

/// The AI model to use for plan generation.
const AI_MODEL: &str = "openai/gpt-4o";

/// Maximum number of generate → validate → retry cycles.
const MAX_ATTEMPTS: usize = 5;

/// Repository trait for schema extraction and AI plan generation.
#[async_trait]
pub trait ISchemaRepo: Send + Sync {
    /// Extract the egress schema (`HashMap<source_column, destination_field>`)
    /// from the manifest's ApiDispatcher action, if present.
    fn extract_egress_schema(&self, manifest: &Manifest) -> HashMap<String, String>;

    /// Generate a `PlanSummary` and the full AI-generated `Manifest` by
    /// calling the LLM with pipeline context.
    ///
    /// Returns an error if the AI fails to produce a valid manifest —
    /// a summary without a manifest is considered a hallucination.
    async fn create_plan_summary(
        &self,
        deps: &Dependancies,
        source_system: &str,
        final_columns: &[String],
        schema_diff: &str,
        egress_schema: &HashMap<String, String>,
    ) -> Result<(PlanSummary, Manifest), Error>;
}

/// Concrete implementation that delegates LLM calls to `deps.llm_repo`.
pub struct SchemaRepository;

impl SchemaRepository {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait]
impl ISchemaRepo for SchemaRepository {
    fn extract_egress_schema(&self, manifest: &Manifest) -> HashMap<String, String> {
        manifest
            .actions
            .iter()
            .find_map(|ac| match &ac.config {
                ActionConfigPayload::ApiDispatcher(cfg) => Some(cfg.get_schema()),
                _ => None,
            })
            .unwrap_or_default()
    }

    async fn create_plan_summary(
        &self,
        deps: &Dependancies,
        source_system: &str,
        final_columns: &[String],
        schema_diff: &str,
        egress_schema: &HashMap<String, String>,
    ) -> Result<(PlanSummary, Manifest), Error> {
        if egress_schema.is_empty() {
            tracing::warn!(
                source_system,
                "Egress schema is empty — preview 'after' fields will be unconstrained"
            );
        } else {
            tracing::info!(
                source_system,
                egress_field_count = egress_schema.len(),
                egress_fields = ?egress_schema.keys().collect::<Vec<_>>(),
                "Egress schema fields for plan generation"
            );
        }

        let prompt = PlanPrompt {
            source_system,
            final_columns,
            schema_diff,
            egress_schema,
        };
        tracing::info!("Building LLM prompt");
        let messages = prompt.generate_prompt();
        tracing::info!(
            system_prompt_len = messages.system.len(),
            user_prompt_len = messages.user.len(),
            "Prompt built — starting generate → validate → retry loop"
        );

        // Build initial conversation with system + user messages
        let mut conversation = vec![
            ChatMessage {
                role: "system".into(),
                content: messages.system.clone(),
            },
            ChatMessage {
                role: "user".into(),
                content: messages.user.clone(),
            },
        ];

        let expected_ingress = if source_system == "CSV" {
            ActionType::CsvHrisConnector
        } else {
            ActionType::WorkdayHrisConnector
        };

        let mut last_error = String::from("No attempts made");

        for attempt in 1..=MAX_ATTEMPTS {
            tracing::info!(attempt, max = MAX_ATTEMPTS, "Plan generation attempt");

            // Call LLM — retry on transient API errors instead of failing immediately
            let content = match deps
                .llm_repo
                .chat_completion(AI_MODEL, &conversation, 0.7, 4096, 1.0)
                .await
            {
                Ok(c) => c,
                Err(api_err) => {
                    last_error = format!("API error: {api_err}");
                    tracing::warn!(
                        attempt,
                        error = %api_err,
                        "LLM API call failed — will retry"
                    );
                    // Exponential backoff: 2s, 4s, 8s, 16s …
                    let backoff = Duration::from_secs(2u64.pow(attempt as u32));
                    sleep(backoff).await;
                    continue;
                }
            };

            tracing::info!(
                attempt,
                response_len = content.len(),
                "LLM response received"
            );
            tracing::debug!(llm_output = %content, "Raw LLM output");

            // Parse the response
            let (summary, manifest) = match parse_ai_response(&content) {
                Ok(result) => result,
                Err(parse_err) => {
                    last_error = format!("Parse error: {parse_err}");
                    tracing::warn!(attempt, error = %parse_err, "Failed to parse LLM response");
                    // Add the assistant's broken response + our feedback
                    conversation.push(ChatMessage {
                        role: "assistant".into(),
                        content: content.clone(),
                    });
                    conversation.push(ChatMessage {
                        role: "user".into(),
                        content: format!(
                            "Your response failed validation:\n{parse_err}\n\n\
                             Please fix these issues and return the corrected JSON. \
                             Remember: return ONLY valid JSON, no markdown."
                        ),
                    });
                    continue;
                }
            };

            // Validate
            let errors = validate_plan(
                &summary,
                &manifest,
                &expected_ingress,
                egress_schema,
            );

            if errors.is_empty() {
                tracing::info!(attempt, "Plan passed all validations");
                let summary = PlanSummary {
                    generation_status: SchemaGenerationStatus::Completed,
                    ..summary
                };
                return Ok((summary, manifest));
            }

            // Validation failed — feed errors back
            let error_report = errors.join("\n");
            last_error = error_report.clone();
            tracing::warn!(
                attempt,
                error_count = errors.len(),
                errors = %error_report,
                "Plan failed validation — retrying"
            );

            conversation.push(ChatMessage {
                role: "assistant".into(),
                content: content.clone(),
            });
            conversation.push(ChatMessage {
                role: "user".into(),
                content: format!(
                    "Your response failed validation with {} error(s):\n{}\n\n\
                     Please fix ALL of these issues and return the corrected JSON. \
                     Remember: return ONLY valid JSON, no markdown.",
                    errors.len(),
                    error_report,
                ),
            });
        }

        Err(Error::from(format!(
            "Plan generation failed after {MAX_ATTEMPTS} attempts. Last error: {last_error}"
        )))
    }
}

// ── Private helpers ────────────────────────────────────────────────

/// Parse the raw LLM text into a `(PlanSummary, Manifest)`.
///
/// Strips markdown fences, extracts both `summary` and `manifest` keys,
/// and deserialises into the model types.
fn parse_ai_response(content: &str) -> Result<(PlanSummary, Manifest), String> {
    // Strip markdown code fences if present
    let json_str = content
        .trim()
        .strip_prefix("```json")
        .or_else(|| content.trim().strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .unwrap_or(content.trim());

    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON: {e}"))?;

    // ── Extract summary ────────────────────────────────────────
    let summary_val = parsed
        .get("summary")
        .ok_or("Response missing required 'summary' key")?;

    let headline = summary_val["headline"]
        .as_str()
        .unwrap_or("Here's the plan for your pipeline.")
        .to_string();
    let description = summary_val["description"]
        .as_str()
        .unwrap_or("An automated data sync pipeline.")
        .to_string();

    let features: Vec<PlanFeature> = summary_val
        .get("features")
        .and_then(|f| serde_json::from_value(f.clone()).ok())
        .unwrap_or_default();

    let preview: PlanPreview = summary_val
        .get("preview")
        .and_then(|p| serde_json::from_value(p.clone()).ok())
        .unwrap_or_else(|| PlanPreview {
            source_label: "Source".into(),
            target_label: "Destination".into(),
            before: HashMap::new(),
            after: HashMap::new(),
        });

    // ── Extract manifest ───────────────────────────────────────
    let manifest_val = parsed
        .get("manifest")
        .ok_or("Response missing required 'manifest' key — a summary without a manifest is invalid")?;

    let manifest: Manifest = serde_json::from_value(manifest_val.clone())
        .map_err(|e| format!("Failed to deserialise manifest: {e}"))?;

    if manifest.actions.is_empty() {
        return Err("Manifest has zero actions — at minimum ingress + egress are required".into());
    }

    let summary = PlanSummary {
        headline,
        description,
        features,
        preview,
        generation_status: SchemaGenerationStatus::Completed,
    };

    Ok((summary, manifest))
}

/// Validate a parsed plan for structural correctness.
///
/// Returns a `Vec<String>` of human-readable errors (empty = valid).
fn validate_plan(
    summary: &PlanSummary,
    manifest: &Manifest,
    expected_ingress: &ActionType,
    egress_schema: &HashMap<String, String>,
) -> Vec<String> {
    let mut errors = Vec::new();

    let action_ids: Vec<&str> = manifest.actions.iter().map(|a| a.id.as_str()).collect();

    // 1. First action must be the expected ingress connector
    if let Some(first) = manifest.actions.first() {
        if &first.action_type != expected_ingress {
            errors.push(format!(
                "First action must be '{}' but got '{}'",
                expected_ingress, first.action_type
            ));
        }
    }

    // 2. Last action must be api_dispatcher (egress)
    if let Some(last) = manifest.actions.last() {
        if last.action_type != ActionType::ApiDispatcher {
            errors.push(format!(
                "Last action must be 'api_dispatcher' but got '{}'",
                last.action_type
            ));
        }
    }

    // 3. Action IDs must be unique
    let mut seen_ids = std::collections::HashSet::new();
    for action in &manifest.actions {
        if !seen_ids.insert(&action.id) {
            errors.push(format!("Duplicate action id: '{}'", action.id));
        }
    }

    // 4. All summary feature actionIds must reference real manifest action IDs
    for feature in &summary.features {
        for action_id in &feature.action_ids {
            if !action_ids.contains(&action_id.as_str()) {
                errors.push(format!(
                    "Feature '{}' references action_id '{}' which does not exist in the manifest. \
                     Valid action ids are: [{}]",
                    feature.id,
                    action_id,
                    action_ids.join(", "),
                ));
            }
        }
    }

    // 5. Preview "after" keys must be ONLY egress schema destination fields (the keys)
    if !egress_schema.is_empty() {
        let egress_destination_fields: std::collections::HashSet<&str> =
            egress_schema.keys().map(|k| k.as_str()).collect();
        for key in summary.preview.after.keys() {
            if !egress_destination_fields.contains(key.as_str()) {
                errors.push(format!(
                    "Preview 'after' contains field '{}' which is not a destination field in the EGRESS SCHEMA. \
                     Only these destination fields are allowed: [{}]",
                    key,
                    egress_destination_fields.iter().copied().collect::<Vec<_>>().join(", "),
                ));
            }
        }
    }

    // 6. Egress schema fields must be accounted for in the api_dispatcher config
    if !egress_schema.is_empty() {
        if let Some(egress_action) = manifest.actions.iter().find(|a| a.action_type == ActionType::ApiDispatcher) {
            if let ActionConfigPayload::ApiDispatcher(cfg) = &egress_action.config {
                let dispatcher_schema = cfg.get_schema();
                for (field_name, _field_type) in egress_schema {
                    if !dispatcher_schema.contains_key(field_name) {
                        errors.push(format!(
                            "Egress schema field '{field_name}' is not accounted for in the api_dispatcher schema. \
                             The api_dispatcher must include all destination fields from the egress schema."
                        ));
                    }
                }
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_egress_schema_empty_manifest() {
        let repo = SchemaRepository;
        let manifest = Manifest {
            version: "1.0".into(),
            actions: vec![],
        };
        let schema = repo.extract_egress_schema(&manifest);
        assert!(schema.is_empty());
    }

    #[test]
    fn test_parse_ai_response_valid() {
        let json = r#"{
            "manifest": {
                "version": "1.0",
                "actions": [
                    { "id": "step_1", "action_type": "workday_hris_connector", "disabled": false, "config": { "tenant_url": "https://example.com", "tenant_id": "t1", "username": "u", "password": "p", "worker_count_limit": 200, "response_group": { "include_personal_information": true, "include_employment_information": true, "include_compensation": false, "include_organizations": false, "include_roles": false } } },
                    { "id": "step_2", "action_type": "api_dispatcher", "disabled": false, "config": { "auth_type": "bearer", "destination_url": "https://api.example.com", "token": "tok", "schema": { "name": "fullName" } } }
                ]
            },
            "summary": {
                "headline": "Test plan",
                "description": "A test pipeline",
                "features": [
                    { "id": "f1", "icon": "zap", "label": "Sync", "description": "syncs data", "actionIds": ["step_1", "step_2"] }
                ],
                "preview": { "sourceLabel": "Source", "targetLabel": "Target", "before": {}, "after": {} }
            }
        }"#;

        let (summary, manifest) = parse_ai_response(json).expect("should parse");
        assert_eq!(summary.headline, "Test plan");
        assert_eq!(manifest.actions.len(), 2);
    }

    #[test]
    fn test_parse_ai_response_missing_manifest() {
        let json = r#"{ "summary": { "headline": "x", "description": "y", "features": [], "preview": { "sourceLabel": "S", "targetLabel": "T", "before": {}, "after": {} } } }"#;
        let err = parse_ai_response(json).unwrap_err();
        assert!(err.contains("manifest"), "Error should mention manifest: {err}");
    }

    #[test]
    fn test_validate_plan_valid() {
        let json = r#"{
            "manifest": {
                "version": "1.0",
                "actions": [
                    { "id": "step_1", "action_type": "workday_hris_connector", "disabled": false, "config": { "tenant_url": "https://example.com", "tenant_id": "t1", "username": "u", "password": "p", "worker_count_limit": 200, "response_group": { "include_personal_information": true, "include_employment_information": true, "include_compensation": false, "include_organizations": false, "include_roles": false } } },
                    { "id": "step_2", "action_type": "api_dispatcher", "disabled": false, "config": { "auth_type": "bearer", "destination_url": "https://api.example.com", "token": "tok", "schema": { "name": "fullName" } } }
                ]
            },
            "summary": {
                "headline": "Test",
                "description": "Desc",
                "features": [{ "id": "f1", "icon": "zap", "label": "L", "description": "D", "actionIds": ["step_1"] }],
                "preview": { "sourceLabel": "S", "targetLabel": "T", "before": {}, "after": {} }
            }
        }"#;

        let (summary, manifest) = parse_ai_response(json).unwrap();
        let errors = validate_plan(
            &summary,
            &manifest,
            &ActionType::WorkdayHrisConnector,
            &HashMap::new(),
        );
        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
    }

    #[test]
    fn test_validate_plan_wrong_ingress() {
        let json = r#"{
            "manifest": {
                "version": "1.0",
                "actions": [
                    { "id": "step_1", "action_type": "csv_hris_connector", "disabled": false, "config": { "filename": "data.csv", "columns": ["a"] } },
                    { "id": "step_2", "action_type": "api_dispatcher", "disabled": false, "config": { "auth_type": "bearer", "destination_url": "https://api.example.com", "token": "tok", "schema": {} } }
                ]
            },
            "summary": { "headline": "T", "description": "D", "features": [], "preview": { "sourceLabel": "S", "targetLabel": "T", "before": {}, "after": {} } }
        }"#;
        let (summary, manifest) = parse_ai_response(json).unwrap();
        let errors = validate_plan(&summary, &manifest, &ActionType::WorkdayHrisConnector, &HashMap::new());
        assert!(errors.iter().any(|e| e.contains("workday_hris_connector")), "Should flag wrong ingress: {errors:?}");
    }

    #[test]
    fn test_validate_plan_bad_feature_ref() {
        let json = r#"{
            "manifest": {
                "version": "1.0",
                "actions": [
                    { "id": "step_1", "action_type": "workday_hris_connector", "disabled": false, "config": { "tenant_url": "https://example.com", "tenant_id": "t1", "username": "u", "password": "p", "worker_count_limit": 200, "response_group": { "include_personal_information": true, "include_employment_information": true, "include_compensation": false, "include_organizations": false, "include_roles": false } } },
                    { "id": "step_2", "action_type": "api_dispatcher", "disabled": false, "config": { "auth_type": "bearer", "destination_url": "https://api.example.com", "token": "tok", "schema": {} } }
                ]
            },
            "summary": {
                "headline": "T", "description": "D",
                "features": [{ "id": "f1", "icon": "zap", "label": "L", "description": "D", "actionIds": ["step_99"] }],
                "preview": { "sourceLabel": "S", "targetLabel": "T", "before": {}, "after": {} }
            }
        }"#;
        let (summary, manifest) = parse_ai_response(json).unwrap();
        let errors = validate_plan(&summary, &manifest, &ActionType::WorkdayHrisConnector, &HashMap::new());
        assert!(errors.iter().any(|e| e.contains("step_99")), "Should flag dangling ref: {errors:?}");
    }
}
