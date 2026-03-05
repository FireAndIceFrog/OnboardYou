//! Plan generation engine — orchestrates AI-powered pipeline plan creation.
//!
//! Delegates all work to repositories:
//! 1. Fetch the `PipelineConfig` via `IConfigRepo`
//! 2. Derive source system from the pipeline's ingress connector
//! 3. Validate the pipeline via `IValidationRepo`
//! 4. Resolve egress schema (from pipeline or org settings)
//! 5. Generate plan summary + manifest via `ISchemaRepo` (which internally calls the LLM)
//! 6. Merge AI-generated manifest actions into the pipeline
//! 7. Write back to DynamoDB

use std::sync::Arc;

use lambda_runtime::Error;
use onboard_you_models::{
    ActionType, DynamicEgressModel, PlanPreview, PlanSummary, SchemaGenerationStatus,
};

use crate::dependancies::Dependancies;

/// Derive the source system label from the pipeline's first (ingress) action.
fn source_system_from_pipeline(pipeline: &onboard_you_models::Manifest) -> &'static str {
    match pipeline.actions.first().map(|a| &a.action_type) {
        Some(ActionType::WorkdayHrisConnector) => "Workday",
        _ => "CSV",
    }
}

/// Run plan generation for the given organization + customer company.
///
/// On success, writes the plan summary + manifest to DynamoDB.
/// On failure, writes `SchemaGenerationStatus::Failed(reason)` so the
/// frontend can show the error instead of timing out.
pub async fn run(
    deps: Arc<Dependancies>,
    organization_id: &str,
    customer_company_id: &str,
) -> Result<(), Error> {
    match run_inner(deps.clone(), organization_id, customer_company_id).await {
        Ok(()) => Ok(()),
        Err(e) => {
            tracing::error!(
                %organization_id,
                %customer_company_id,
                error = %e,
                "Plan generation failed — persisting Failed status"
            );
            // Best-effort: write Failed status so the frontend stops polling
            if let Ok(mut config) = deps.config_repo.get(organization_id, customer_company_id).await {
                config.plan_summary = Some(PlanSummary {
                    headline: String::new(),
                    description: String::new(),
                    features: vec![],
                    preview: PlanPreview {
                        source_label: String::new(),
                        target_label: String::new(),
                        before: Default::default(),
                        after: Default::default(),
                        warnings: vec![],
                    },
                    generation_status: SchemaGenerationStatus::Failed(e.to_string()),
                });
                if let Err(put_err) = deps.config_repo.put(&config).await {
                    tracing::error!(
                        error = %put_err,
                        "Failed to persist Failed status to DynamoDB"
                    );
                }
            }
            Err(e)
        }
    }
}

/// Inner implementation — all errors propagate via `?` and are caught by `run()`.
async fn run_inner(
    deps: Arc<Dependancies>,
    organization_id: &str,
    customer_company_id: &str,
) -> Result<(), Error> {
    tracing::info!(
        %organization_id,
        %customer_company_id,
        "Plan generation started"
    );

    // 1. Fetch config
    tracing::info!("Fetching pipeline config from DynamoDB");
    let mut config = deps
        .config_repo
        .get(organization_id, customer_company_id)
        .await?;
    tracing::info!("Pipeline config fetched successfully");

    // 2. Derive source system from the ingress connector
    let source_system = source_system_from_pipeline(&config.pipeline);
    tracing::info!(%source_system, "Source system derived from pipeline");

    // 3. Run validation + schema diff for context
    tracing::info!("Running pipeline validation");
    let validation = deps.validation_repo.validate(&config.pipeline);
    tracing::info!(
        final_columns_count = validation.final_columns.len(),
        schema_diff_len = validation.schema_diff.len(),
        "Validation complete"
    );

    // 4. Extract egress schema info — prefer pipeline actions, fall back to org settings
    let mut egress_schema = deps.schema_repo.extract_egress_schema(&config.pipeline);
    if egress_schema.is_empty() {
        tracing::info!("No ApiDispatcher in pipeline — checking org settings for egress schema");
        match deps.settings_repo.get(organization_id).await {
            Ok(Some(settings)) => {
                let org_schema = settings.default_auth.get_schema();
                if !org_schema.is_empty() {
                    tracing::info!(
                        field_count = org_schema.len(),
                        "Using egress schema from org settings"
                    );
                    egress_schema = org_schema;
                }
            }
            Ok(None) => {
                tracing::info!("No org settings found — egress schema will be empty");
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to fetch org settings — continuing without egress schema");
            }
        }
    }
    tracing::info!(
        egress_field_count = egress_schema.len(),
        "Egress schema resolved"
    );

    // 5. Generate plan summary + manifest (calls AI internally)
    tracing::info!("Starting LLM plan generation");
    let (plan_summary, manifest) = deps
        .schema_repo
        .create_plan_summary(
            &deps,
            source_system,
            &validation.final_columns,
            &validation.schema_diff,
            &egress_schema,
        )
        .await?;
    tracing::info!(
        generation_status = ?plan_summary.generation_status,
        feature_count = plan_summary.features.len(),
        manifest_action_count = manifest.actions.len(),
        "Plan summary + manifest generated"
    );

    // 6. Replace pipeline actions with the AI-generated manifest
    tracing::info!(
        action_count = manifest.actions.len(),
        "Replacing pipeline actions with AI-generated manifest"
    );
    config.pipeline = manifest;

    // 7. Write back to DynamoDB
    tracing::info!("Writing plan summary to DynamoDB");
    config.plan_summary = Some(plan_summary);
    deps.config_repo.put(&config).await?;
    tracing::info!(%organization_id, %customer_company_id, "Plan generation complete — config saved to DynamoDB");

    Ok(())
}
