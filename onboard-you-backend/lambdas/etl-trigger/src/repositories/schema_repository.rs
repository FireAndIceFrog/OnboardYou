//! Schema repository — extracts egress schema and generates AI-powered plan summaries.

use async_trait::async_trait;
use gh_models::types::ChatMessage;
use lambda_runtime::Error;
use std::collections::HashMap;
use std::sync::Arc;

use crate::dependancies::Dependancies;
use crate::repositories::llm_repository::ILlmRepo;
use onboard_you_models::{
    ActionConfigPayload, DynamicEgressModel, Manifest, PlanFeature, PlanPreview, PlanPrompt,
    PlanSummary, SchemaGenerationStatus,
};

/// The AI model to use for plan generation.
const AI_MODEL: &str = "openai/gpt-4o";

/// Repository trait for schema extraction and AI plan generation.
#[async_trait]
pub trait ISchemaRepo: Send + Sync {
    /// Extract the egress schema (`HashMap<source_column, destination_field>`)
    /// from the manifest's ApiDispatcher action, if present.
    fn extract_egress_schema(&self, manifest: &Manifest) -> HashMap<String, String>;

    /// Generate a `PlanSummary` by calling the LLM with pipeline context.
    ///
    /// Falls back to a deterministic plan if the AI call fails.
    async fn create_plan_summary(
        &self,
        deps: &Dependancies,
        source_system: &str,
        final_columns: &[String],
        schema_diff: &str,
        egress_schema: &HashMap<String, String>,
    ) -> PlanSummary;
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
    ) -> PlanSummary {
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
            "Prompt built — calling LLM"
        );

        match call_ai_and_parse(&deps.llm_repo, &messages).await {
            Ok(plan) => {
                tracing::info!(
                    headline = %plan.headline,
                    feature_count = plan.features.len(),
                    "LLM plan generation succeeded"
                );
                PlanSummary {
                    generation_status: SchemaGenerationStatus::Completed,
                    ..plan
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "LLM plan generation failed — using fallback");
                generate_fallback_plan(source_system, final_columns, egress_schema)
            }
        }
    }
}

// ── Private helpers ────────────────────────────────────────────────

/// Call the AI model via `ILlmRepo` and parse the response into a `PlanSummary`.
async fn call_ai_and_parse(
    llm_repo: &Arc<dyn ILlmRepo>,
    messages: &onboard_you_models::PromptMessages,
) -> Result<PlanSummary, Error> {
    let chat_messages = vec![
        ChatMessage {
            role: "system".into(),
            content: messages.system.clone(),
        },
        ChatMessage {
            role: "user".into(),
            content: messages.user.clone(),
        },
    ];

    tracing::info!(model = AI_MODEL, "Sending chat completion request to LLM");
    let content = llm_repo
        .chat_completion(AI_MODEL, &chat_messages, 0.7, 4096, 1.0)
        .await?;

    tracing::info!(response_len = content.len(), "LLM response received");
    tracing::info!(llm_output = %content, "Raw LLM output");

    // Strip markdown code fences if present
    let json_str = content
        .trim()
        .strip_prefix("```json")
        .or_else(|| content.trim().strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .unwrap_or(content.trim());

    // Parse the AI response
    tracing::info!("Parsing LLM response as JSON");
    let parsed: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
        tracing::error!(error = %e, json_snippet = &json_str[..json_str.len().min(500)], "JSON parse failed");
        Error::from(format!(
            "Failed to parse AI response as JSON: {e}\nResponse: {json_str}"
        ))
    })?;
    tracing::info!("LLM response parsed successfully");

    // Extract summary section
    let summary = parsed
        .get("summary")
        .ok_or_else(|| Error::from("AI response missing 'summary' key"))?;

    let headline = summary["headline"]
        .as_str()
        .unwrap_or("Here's the plan for your pipeline.")
        .to_string();
    let description = summary["description"]
        .as_str()
        .unwrap_or("An automated data sync pipeline.")
        .to_string();

    let features: Vec<PlanFeature> = summary
        .get("features")
        .and_then(|f| {
            serde_json::from_value(f.clone())
                .map_err(|e| tracing::warn!(error = %e, "Failed to parse features — using empty list"))
                .ok()
        })
        .unwrap_or_default();

    let preview: PlanPreview = summary
        .get("preview")
        .and_then(|p| {
            serde_json::from_value(p.clone())
                .map_err(|e| tracing::warn!(error = %e, "Failed to parse preview — using defaults"))
                .ok()
        })
        .unwrap_or_else(|| PlanPreview {
            source_label: "Source".into(),
            target_label: "Destination".into(),
            before: HashMap::new(),
            after: HashMap::new(),
        });

    Ok(PlanSummary {
        headline,
        description,
        features: features.clone(),
        preview,
        generation_status: SchemaGenerationStatus::Completed,
    })
    .inspect(|_| {
        tracing::info!(
            feature_count = features.len(),
            "PlanSummary serialization complete"
        );
    })
}

/// Generate a deterministic fallback plan when AI fails.
fn generate_fallback_plan(
    source_system: &str,
    final_columns: &[String],
    egress_schema: &HashMap<String, String>,
) -> PlanSummary {
    let mut features = Vec::new();

    if !egress_schema.is_empty() {
        features.push(PlanFeature {
            id: "rename_fields".into(),
            icon: "columns".into(),
            label: "Map Fields to Destination".into(),
            description: format!(
                "Rename {} source columns to match your destination API.",
                egress_schema.len()
            ),
            action_ids: vec!["step_rename".into()],
        });
    }

    features.push(PlanFeature {
        id: "sync_data".into(),
        icon: "zap".into(),
        label: format!("Sync from {source_system}"),
        description: format!("Pull employee data from {source_system} and send to your app."),
        action_ids: vec!["step_ingress".into(), "step_egress".into()],
    });

    let mut before = HashMap::new();
    let mut after = HashMap::new();
    before.insert("name".into(), "Jane Doe".into());
    before.insert("email".into(), "jane.doe@example.com".into());
    after.insert("name".into(), "Jane Doe".into());
    after.insert("email".into(), "jane.doe@example.com".into());

    for (src, dst) in egress_schema.iter().take(3) {
        before.insert(src.clone(), format!("(sample {src})"));
        after.insert(dst.clone(), format!("(sample {dst})"));
    }

    PlanSummary {
        headline: format!("Here's the plan to connect {source_system} to your App."),
        description: format!(
            "We'll sync employee data from {source_system}, mapping {} fields to your destination.",
            if egress_schema.is_empty() {
                final_columns.len()
            } else {
                egress_schema.len()
            }
        ),
        features,
        preview: PlanPreview {
            source_label: format!("In {source_system}"),
            target_label: "In Your App".into(),
            before,
            after,
        },
        generation_status: SchemaGenerationStatus::Completed,
    }
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
    fn test_fallback_plan_generates_valid_summary() {
        let egress: HashMap<String, String> = [
            ("name".into(), "fullName".into()),
            ("email".into(), "workEmail".into()),
        ]
        .into_iter()
        .collect();

        let plan = generate_fallback_plan("Workday", &["name".into(), "email".into()], &egress);

        assert!(plan.headline.contains("Workday"));
        assert!(!plan.features.is_empty());
        assert!(matches!(
            plan.generation_status,
            SchemaGenerationStatus::Completed
        ));

        let json = serde_json::to_string(&plan).unwrap();
        let back: PlanSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(back.headline, plan.headline);
    }

    #[test]
    fn test_fallback_plan_with_empty_egress() {
        let plan = generate_fallback_plan("CSV", &["col_a".into()], &HashMap::new());
        assert!(plan.headline.contains("CSV"));
        assert!(!plan.features.is_empty());
    }
}
