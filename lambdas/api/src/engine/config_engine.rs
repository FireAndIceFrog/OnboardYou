//! Config engine — business logic for pipeline configuration.
//!
//! Validates inputs, stamps server-controlled fields,
//! then delegates to repositories for persistence and scheduling.

use crate::models::{ApiError, AppState, PipelineConfig};
use crate::repositories::{config_repository, schedule_repository};

/// Fetch a pipeline config by organization ID and customer company ID.
pub async fn get(
    state: &AppState,
    organization_id: &str,
    customer_company_id: &str,
) -> Result<PipelineConfig, ApiError> {
    config_repository::get(state, organization_id, customer_company_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("{organization_id}/{customer_company_id}"))
        })
}

/// List all pipeline configs owned by an organization.
pub async fn list(
    state: &AppState,
    organization_id: &str,
) -> Result<Vec<PipelineConfig>, ApiError> {
    config_repository::list(state, organization_id).await
}

/// Validate, persist, and schedule a pipeline config.
pub async fn upsert(
    state: &AppState,
    organization_id: &str,
    customer_company_id: &str,
    mut config: PipelineConfig,
) -> Result<PipelineConfig, ApiError> {
    // Server-controlled fields
    config.organization_id = organization_id.to_string();
    config.customer_company_id = customer_company_id.to_string();
    config.last_edited = chrono::Utc::now().to_rfc3339();

    validate(&config)?;

    config_repository::put(state, &config).await?;
    schedule_repository::upsert(state, &config).await?;

    tracing::info!(
        organization_id = %config.organization_id,
        customer_company_id = %config.customer_company_id,
        "Config saved and schedule updated"
    );

    Ok(config)
}

fn validate(config: &PipelineConfig) -> Result<(), ApiError> {
    if config.cron.is_empty() {
        return Err(ApiError::Validation("cron field is required".into()));
    }

    Ok(())
}
