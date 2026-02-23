//! HTTP handlers for organization settings endpoints.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::dependancies::Dependancies;
use crate::engine;
use crate::models::{ApiError, Claims, ErrorResponse, SettingsRequest};
use onboard_you::OrgSettings;

/// GET /settings
///
/// Retrieve the settings for the caller's organization.
/// The organization is resolved from the caller's JWT claims.
#[utoipa::path(
    get,
    path = "/settings",
    tag = "Settings",
    responses(
        (status = 200, description = "Organization settings", body = OrgSettings),
        (status = 401, description = "Unauthorized — missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Settings not found for organization", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn get_settings(
    State(state): State<Dependancies>,
    claims: Claims,
) -> Result<impl IntoResponse, ApiError> {
    let settings = engine::settings_engine::get(&state, &claims.organization_id).await?;
    Ok(Json(settings))
}

/// PUT /settings
///
/// Create or update the settings for the caller's organization.
/// The organization is resolved from the caller's JWT claims.
#[utoipa::path(
    put,
    path = "/settings",
    tag = "Settings",
    request_body(
        content = SettingsRequest,
        description = "Organization settings to save",
    ),
    responses(
        (status = 200, description = "Settings saved successfully", body = OrgSettings),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized — missing or invalid token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn upsert_settings(
    State(state): State<Dependancies>,
    claims: Claims,
    Json(body): Json<SettingsRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let settings = body.into_settings();
    let saved = engine::settings_engine::upsert(&state, &claims.organization_id, settings).await?;
    Ok((StatusCode::OK, Json(saved)))
}
