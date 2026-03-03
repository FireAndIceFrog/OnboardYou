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
    let mut config = deps
        .config_repo
        .get(organization_id, customer_company_id)
        .await?;

    // 2. Run validation + schema diff for context
    let validation = deps.validation_repo.validate(&config.pipeline);

    // 3. Extract egress schema info
    let egress_schema = deps.schema_repo.extract_egress_schema(&config.pipeline);

    // 4. Generate plan summary (calls AI internally, falls back on failure)
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

    // 5. Write back to DynamoDB
    config.plan_summary = Some(plan_summary);
    deps.config_repo.put(&config).await?;
    tracing::info!(%organization_id, %customer_company_id, "Plan summary written to DynamoDB");

    Ok(())
}
