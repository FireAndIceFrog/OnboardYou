//! Pipeline engine — loads config, builds actions, runs the ETL pipeline.
//!
//! When a manifest action specifies `auth_type: "default"`, the engine
//! fetches the organisation's stored auth settings from the settings table
//! and injects them into the action config before factory construction.

use lambda_runtime::Error;
use onboard_you::{ActionFactory, ActionFactoryTrait, ActionConfigPayload, Manifest, PipelineRunner, RosterContext};
use polars::prelude::LazyFrame;

use crate::models::PipelineResult;
use crate::repositories::{config_repository, settings_repository};

/// Load config from DynamoDB, build the pipeline, and execute it.
pub async fn run(
    dynamo: &aws_sdk_dynamodb::Client,
    table_name: &str,
    settings_table_name: &str,
    organization_id: &str,
    customer_company_id: &str,
) -> Result<PipelineResult, Error> {
    tracing::info!(%organization_id, %customer_company_id, "ETL trigger fired");

    // 1. Fetch config
    let config = config_repository::get(dynamo, table_name, organization_id, customer_company_id).await?;

    // 2. Deserialize the Manifest
    let mut manifest: Manifest = serde_json::from_value(config.pipeline)
        .map_err(|e| Error::from(format!("Failed to parse manifest: {e}")))?;

    // 3. Resolve any "default" auth types from the settings table
    resolve_default_auth(dynamo, settings_table_name, organization_id, &mut manifest).await?;

    // 3b. Resolve CSV S3 keys from org_id / company_id / filename
    resolve_csv_s3_keys(organization_id, customer_company_id, &mut manifest);
    let action_factory = ActionFactory::new();
    // 4. Build actions from manifest via Factory
    let actions: Vec<_> = manifest
        .actions
        .iter()
        .map(|ac| action_factory.create(ac))
        .collect::<onboard_you::Result<_>>()
        .map_err(|e| Error::from(format!("Failed to build actions: {e}")))?;

    // 5. Execute the pipeline
    let context = RosterContext::new(LazyFrame::default());

    match PipelineRunner::run(&manifest, actions, context) {
        Ok(result) => {
            let rows = result.data.clone().collect().map(|df: polars::prelude::DataFrame| df.height()).ok();
            tracing::info!(%organization_id, %customer_company_id, rows_processed = ?rows, "Pipeline completed");
            Ok(PipelineResult::success(organization_id, customer_company_id, rows))
        }
        Err(e) => {
            tracing::error!(%organization_id, %customer_company_id, error = %e, "Pipeline failed");
            Ok(PipelineResult::failure(organization_id, customer_company_id, e))
        }
    }
}

/// Scan the manifest for actions with `ApiDispatcher(Default)` and replace
/// their config with the organisation's stored default auth settings.
///
/// The settings lookup is lazy — only performed if at least one action
/// actually uses `"default"`.
async fn resolve_default_auth(
    dynamo: &aws_sdk_dynamodb::Client,
    settings_table_name: &str,
    organization_id: &str,
    manifest: &mut Manifest,
) -> Result<(), Error> {
    // Quick scan: does any action use "default"?
    let needs_resolution = manifest.actions.iter().any(|ac| {
        matches!(
            ac.config,
            ActionConfigPayload::ApiDispatcher(ref cfg) if cfg.is_default()
        )
    });

    if !needs_resolution {
        return Ok(());
    }

    // Fetch org settings
    let settings = settings_repository::get(dynamo, settings_table_name, organization_id)
        .await?
        .ok_or_else(|| {
            Error::from(format!(
                "auth_type 'default' used but no settings found for org: {organization_id}"
            ))
        })?;

    tracing::info!(%organization_id, "Resolved default auth from settings table");

    // Replace config for every action that uses "default"
    for action in &mut manifest.actions {
        let is_default = matches!(
            action.config,
            ActionConfigPayload::ApiDispatcher(ref cfg) if cfg.is_default()
        );

        if is_default {
            tracing::info!(
                action_id = %action.id,
                action_type = %action.action_type,
                "Replacing auth_type 'default' with org settings"
            );
            action.config = ActionConfigPayload::ApiDispatcher(settings.default_auth.clone());
        }
    }

    Ok(())
}

/// Inject the resolved S3 key (`{org_id}/{company_id}/{filename}`) into every
/// `CsvHrisConnector` action in the manifest.
///
/// This must run **before** the factory builds the actions so that
/// `download_from_s3` can find the correct S3 object at runtime.
fn resolve_csv_s3_keys(
    organization_id: &str,
    customer_company_id: &str,
    manifest: &mut Manifest,
) {
    for action in &mut manifest.actions {
        if let ActionConfigPayload::CsvHrisConnector(ref mut cfg) = action.config {
            cfg.resolve_s3_key(organization_id, customer_company_id);
            tracing::info!(
                action_id = %action.id,
                filename = %cfg.filename,
                s3_key = ?cfg.resolved_s3_key,
                "Resolved CSV S3 key"
            );
        }
    }
}
