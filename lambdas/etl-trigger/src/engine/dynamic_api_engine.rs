//! Engine for the "DynamicApi" scheduled event.
//!
//! This module is intentionally small – the business logic lives in the
//! `run` function which is exercised by unit tests below using in‑memory
//! fake repositories.

use lambda_runtime::Error;
use onboard_you::ApiDispatcherConfig;
use std::sync::Arc;

use crate::dependancies::Dependancies;

/// Execute the dynamic‑api workflow for a single organisation/company pair.
///
/// The caller already knows which company triggered the event but we don’t use
/// that value here; it is logged purely for symmetry with other engines.
pub async fn run(
    deps: Arc<Dependancies>,
    organization_id: &str,
    customer_company_id: &str,

) -> Result<(), Error> {
    tracing::info!(%organization_id, %customer_company_id, "DynamicApi event received");
    //get settings model

    let settings = match deps.settings_repo.get(organization_id).await? {
        Some(s) => s,
        None => {
            tracing::warn!(%organization_id, "No settings found for org, using defaults");
            return Err(Error::from("No settings found for org"));
        }
    };
    let url = match settings.default_auth.clone() {
        ApiDispatcherConfig::Bearer(cfg) => cfg.output_schema_openapi_url,
        ApiDispatcherConfig::OAuth(cfg) => cfg.output_schema_openapi_url,
        ApiDispatcherConfig::OAuth2(cfg) => cfg.output_schema_openapi_url,
        ApiDispatcherConfig::Default => {
            tracing::warn!(%organization_id, "Default auth type found in settings, cannot proceed with Dynamic API workflow");
            return Err(Error::from("Default auth type found in settings"));
        },
    }.unwrap_or_default();

    tracing::info!(%organization_id, openapi_url = %url, "Fetching OpenAPI schema");
    //get openapi schema
    let openapi_json = deps.openapi_repo.fetch(&url.clone()).await?;

    tracing::info!(%organization_id, openapi_url = %url, "Fetched OpenAPI schema");

    // parse openapi schema and generate manifest

    
    let dynamic_api = deps.gh_models_repo.generate_dynamic_body(&openapi_json).await?;

    let modified_schema: ApiDispatcherConfig = match settings.default_auth.clone() {
        ApiDispatcherConfig::Bearer(cfg) => {
            let mut new_cfg = cfg.clone();
            new_cfg.output_schema = Some(dynamic_api.output_schema);
            new_cfg.output_schema_body_path = Some(dynamic_api.output_schema_body_path);
            onboard_you::ApiDispatcherConfig::Bearer(new_cfg)
        },
        ApiDispatcherConfig::OAuth(cfg) => {
            let mut new_cfg = cfg.clone();
            new_cfg.output_schema = Some(dynamic_api.output_schema);
            new_cfg.output_schema_body_path = Some(dynamic_api.output_schema_body_path);
            onboard_you::ApiDispatcherConfig::OAuth(new_cfg)
        },
        ApiDispatcherConfig::OAuth2(cfg) => {
            let mut new_cfg = cfg.clone();
            new_cfg.output_schema = Some(dynamic_api.output_schema);
            new_cfg.output_schema_body_path = Some(dynamic_api.output_schema_body_path);
            onboard_you::ApiDispatcherConfig::OAuth2(new_cfg)
        },
        _ => {
            tracing::warn!(%organization_id, "Default auth type found in settings, cannot proceed with Dynamic API workflow");
            return Err(Error::from("Default auth type found in settings"));
        },
    };

    let mut settings = settings.clone();
    settings.default_auth = modified_schema.clone();

    deps.settings_repo.save(&settings).await?;
    
    Ok(())
}
