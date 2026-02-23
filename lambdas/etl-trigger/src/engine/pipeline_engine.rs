//! Pipeline engine — loads config, builds actions, runs the ETL pipeline.
//!
//! When a manifest action specifies `auth_type: "default"`, the engine
//! fetches the organisation's stored auth settings from the settings table
//! and injects them into the action config before factory construction.

use lambda_runtime::Error;
use std::sync::Arc;

use crate::dependancies::Dependancies;

use crate::models::PipelineResult;

/// Load config from DynamoDB, build the pipeline, and execute it.
pub async fn run(
    deps: Arc<Dependancies>,
    organization_id: &str,
    customer_company_id: &str,
) -> Result<PipelineResult, Error> {
    tracing::info!(%organization_id, %customer_company_id, "ETL trigger fired");

    // 1. Fetch config via injected repository
    let config = deps
        .config_repo
        .get(organization_id, customer_company_id)
        .await?;

    // 2. Deserialize the Manifest
    let mut manifest = config.pipeline;

    // 3. Resolve any "default" auth types from the settings table via injected repo
    manifest = deps.etl_repo.resolve_default_auth(&deps, &mut manifest, organization_id).await?;

    // 3b. Resolve CSV S3 keys from org_id / company_id / filename
    manifest = deps.etl_repo.resolve_csv_s3_keys(&mut manifest, organization_id, customer_company_id)?;

    deps.pipeline_repo.run_pipeline(&deps, manifest, organization_id, customer_company_id).await
}