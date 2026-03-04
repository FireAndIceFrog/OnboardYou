//! Plan generation engine — orchestrates AI-powered pipeline plan creation.
//!
//! Delegates all work to repositories:
//! 1. Fetch the `PipelineConfig` via `IConfigRepo`
//! 2. Validate the pipeline via `IValidationRepo`
//! 3. Generate a plan summary via `ISchemaRepo` (which internally calls the LLM)
//! 4. Write back `PlanSummary` to DynamoDB

use std::sync::Arc;

use lambda_runtime::Error;

use crate::dependancies::Dependancies;

/// Run plan generation for the given organization + customer company.
pub async fn run(
    deps: Arc<Dependancies>,
    organization_id: &str,
    customer_company_id: &str,
    source_system: &str,
) -> Result<(), Error> {
    tracing::info!(
        %organization_id,
        %customer_company_id,
        %source_system,
        "Plan generation started"
    );

    // 1. Fetch config
    tracing::info!("Fetching pipeline config from DynamoDB");
    let mut config = deps
        .config_repo
        .get(organization_id, customer_company_id)
        .await?;
    tracing::info!("Pipeline config fetched successfully");

    // 2. Run validation + schema diff for context
    tracing::info!("Running pipeline validation");
    let validation = deps.validation_repo.validate(&config.pipeline);
    tracing::info!(
        final_columns_count = validation.final_columns.len(),
        schema_diff_len = validation.schema_diff.len(),
        "Validation complete"
    );

    // 3. Extract egress schema info
    let egress_schema = deps.schema_repo.extract_egress_schema(&config.pipeline);
    tracing::info!(
        egress_field_count = egress_schema.len(),
        "Egress schema extracted"
    );

    // 4. Generate plan summary (calls AI internally, falls back on failure)
    tracing::info!("Starting LLM plan generation");
    let plan_summary = deps
        .schema_repo
        .create_plan_summary(
            &deps,
            source_system,
            &validation.final_columns,
            &validation.schema_diff,
            &egress_schema,
        )
        .await;
    tracing::info!(
        generation_status = ?plan_summary.generation_status,
        feature_count = plan_summary.features.len(),
        "Plan summary generated"
    );

    // 5. Write back to DynamoDB
    tracing::info!("Writing plan summary to DynamoDB");
    config.plan_summary = Some(plan_summary);
    deps.config_repo.put(&config).await?;
    tracing::info!(%organization_id, %customer_company_id, "Plan summary written to DynamoDB");

    Ok(())
}
