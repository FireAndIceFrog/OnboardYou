//! Pipeline engine — loads config, builds actions, runs the ETL pipeline.

use lambda_runtime::Error;
use onboard_you::{ActionFactory, Manifest, PipelineRunner, RosterContext};
use polars::prelude::LazyFrame;

use crate::models::PipelineResult;
use crate::repositories::config_repository;

/// Load config from DynamoDB, build the pipeline, and execute it.
pub async fn run(
    dynamo: &aws_sdk_dynamodb::Client,
    table_name: &str,
    organization_id: &str,
) -> Result<PipelineResult, Error> {
    tracing::info!(%organization_id, "ETL trigger fired");

    // 1. Fetch config
    let config = config_repository::get(dynamo, table_name, organization_id).await?;

    // 2. Deserialize the Manifest
    let manifest: Manifest = serde_json::from_value(config.pipeline)
        .map_err(|e| Error::from(format!("Failed to parse manifest: {e}")))?;

    // 3. Build actions from manifest via Factory
    let actions: Vec<_> = manifest
        .actions
        .iter()
        .map(ActionFactory::create)
        .collect::<onboard_you::Result<_>>()
        .map_err(|e| Error::from(format!("Failed to build actions: {e}")))?;

    // 4. Execute the pipeline
    let context = RosterContext::new(LazyFrame::default());

    match PipelineRunner::run(&manifest, actions, context) {
        Ok(result) => {
            let rows = result.data.clone().collect().map(|df: polars::prelude::DataFrame| df.height()).ok();
            tracing::info!(%organization_id, rows_processed = ?rows, "Pipeline completed");
            Ok(PipelineResult::success(organization_id, rows))
        }
        Err(e) => {
            tracing::error!(%organization_id, error = %e, "Pipeline failed");
            Ok(PipelineResult::failure(organization_id, e))
        }
    }
}
