//! Settings engine — business logic for organization settings.
//!
//! Validates inputs, stamps server-controlled fields,
//! then delegates to the settings repository for persistence.
//!
//! The `default_auth` field is validated by attempting to construct
//! an `ApiEngine` from it — the same code path used at pipeline
//! execution time. If the config is invalid, the save is rejected.

use crate::models::{ApiError, AppState, OrgSettings};
use crate::repositories::settings_repository;
use onboard_you::capabilities::egress::engine::api_engine::ApiEngine;

/// Fetch settings for an organization. Returns `NotFound` if no settings exist.
pub async fn get(state: &AppState, organization_id: &str) -> Result<OrgSettings, ApiError> {
    settings_repository::get(state, organization_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("{organization_id}")))
}

/// Validate and persist organization settings.
///
/// Validates that `default_auth` can be parsed by `ApiEngine::from_action_config`
/// before persisting — prevents storing broken configs.
pub async fn upsert(
    state: &AppState,
    organization_id: &str,
    mut settings: OrgSettings,
) -> Result<OrgSettings, ApiError> {
    // Server-controlled field — always use the JWT-derived org id
    settings.organization_id = organization_id.to_string();

    validate(&settings)?;

    settings_repository::put(state, &settings).await?;

    tracing::info!(
        organization_id = %settings.organization_id,
        "Settings saved"
    );

    Ok(settings)
}

/// Validate that `default_auth` contains a well-formed auth config
/// by running it through the same factory the pipeline uses.
fn validate(settings: &OrgSettings) -> Result<(), ApiError> {
    ApiEngine::from_action_config(&settings.default_auth).map_err(|e| {
        ApiError::Validation(format!("Invalid default_auth config: {e}"))
    })?;
    Ok(())
}
