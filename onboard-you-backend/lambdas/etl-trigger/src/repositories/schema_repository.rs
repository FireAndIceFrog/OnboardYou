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

        let (plan, manifest) = call_ai_and_parse(&deps.llm_repo, &messages).await?;

        tracing::info!(
            headline = %plan.headline,
            feature_count = plan.features.len(),
            manifest_action_count = manifest.actions.len(),
            "LLM plan generation succeeded"
        );

        let summary = PlanSummary {
            generation_status: SchemaGenerationStatus::Completed,
            ..plan
        };
        Ok((summary, manifest))
    }
}

// ── Private helpers ────────────────────────────────────────────────

/// Call the AI model via `ILlmRepo` and parse the response into a `PlanSummary`
/// plus the AI-generated `Manifest` (pipeline actions).
async fn call_ai_and_parse(
    llm_repo: &Arc<dyn ILlmRepo>,
    messages: &onboard_you_models::PromptMessages,
) -> Result<(PlanSummary, Manifest), Error> {
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

    // Extract the manifest (pipeline actions) from the AI response.
    // A summary without a manifest is a hallucination — fail hard.
    let manifest_value = parsed
        .get("manifest")
        .ok_or_else(|| {
            tracing::error!("AI response missing 'manifest' key — this is a hallucination");
            Error::from("AI response missing required 'manifest' key")
        })?;

    let manifest: Manifest = serde_json::from_value(manifest_value.clone()).map_err(|e| {
        tracing::error!(error = %e, "Failed to parse manifest from AI response");
        Error::from(format!("Failed to parse manifest from AI response: {e}"))
    })?;

    if manifest.actions.is_empty() {
        tracing::error!("AI returned a manifest with zero actions — hallucination");
        return Err(Error::from("AI returned an empty manifest (zero actions)"));
    }

    tracing::info!(
        feature_count = features.len(),
        manifest_action_count = manifest.actions.len(),
        "PlanSummary + Manifest parsed from LLM response"
    );

    Ok((
        PlanSummary {
            headline,
            description,
            features,
            preview,
            generation_status: SchemaGenerationStatus::Completed,
        },
        manifest,
    ))
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
}
