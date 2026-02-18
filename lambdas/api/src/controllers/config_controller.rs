//! HTTP handlers for pipeline configuration endpoints.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::engine;
use crate::models::{ApiError, AppState, Claims, ConfigRequest, ErrorResponse, PipelineConfig};
use engine::validation_engine::ValidationResult;

/// GET /config
///
/// List all pipeline configurations owned by the caller's organization.
/// The organization is resolved from the caller's JWT claims.
#[utoipa::path(
    get,
    path = "/config",
    tag = "Configuration",
    responses(
        (status = 200, description = "List of pipeline configurations", body = Vec<PipelineConfig>),
        (status = 401, description = "Unauthorized — missing or invalid token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn list_configs(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<impl IntoResponse, ApiError> {
    let configs = engine::config_engine::list(&state, &claims.organization_id).await?;
    Ok(Json(configs))
}

/// GET /config/{customerCompanyId}
///
/// Retrieve the pipeline configuration for a given customer company.
/// The organization is resolved from the caller's JWT claims.
#[utoipa::path(
    get,
    path = "/config/{customer_company_id}",
    tag = "Configuration",
    params(
        ("customer_company_id" = String, Path, description = "Unique identifier for the customer company"),
    ),
    responses(
        (status = 200, description = "Pipeline configuration found", body = PipelineConfig),
        (status = 401, description = "Unauthorized — missing or invalid token", body = ErrorResponse),
        (status = 404, description = "Configuration not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn get_config(
    State(state): State<AppState>,
    claims: Claims,
    Path(customer_company_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let config =
        engine::config_engine::get(&state, &claims.organization_id, &customer_company_id).await?;
    Ok(Json(config))
}

/// POST /config/{customerCompanyId}
///
/// Create a new pipeline configuration. Validates the pipeline manifest,
/// persists to DynamoDB, and creates an EventBridge schedule.
/// The organization is resolved from the caller's JWT claims.
#[utoipa::path(
    post,
    path = "/config/{customer_company_id}",
    tag = "Configuration",
    params(
        ("customer_company_id" = String, Path, description = "Unique identifier for the customer company"),
    ),
    request_body(
        content = ConfigRequest,
        description = "Pipeline configuration to create",
    ),
    responses(
        (status = 200, description = "Configuration created successfully", body = PipelineConfig),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized — missing or invalid token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn create_config(
    State(state): State<AppState>,
    claims: Claims,
    Path(customer_company_id): Path<String>,
    Json(body): Json<ConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    engine::validation_engine::validate_pipeline(&body.pipeline)?;
    let config = body.into_config();
    let saved =
        engine::config_engine::upsert(&state, &claims.organization_id, &customer_company_id, config)
            .await?;
    Ok((StatusCode::OK, Json(saved)))
}

/// PUT /config/{customerCompanyId}
///
/// Update an existing pipeline configuration. Validates the pipeline manifest,
/// persists to DynamoDB, and updates the EventBridge schedule.
/// The organization is resolved from the caller's JWT claims.
#[utoipa::path(
    put,
    path = "/config/{customer_company_id}",
    tag = "Configuration",
    params(
        ("customer_company_id" = String, Path, description = "Unique identifier for the customer company"),
    ),
    request_body(
        content = ConfigRequest,
        description = "Pipeline configuration to update",
    ),
    responses(
        (status = 200, description = "Configuration updated successfully", body = PipelineConfig),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized — missing or invalid token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn update_config(
    State(state): State<AppState>,
    claims: Claims,
    Path(customer_company_id): Path<String>,
    Json(body): Json<ConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    engine::validation_engine::validate_pipeline(&body.pipeline)?;
    let config = body.into_config();
    let saved =
        engine::config_engine::upsert(&state, &claims.organization_id, &customer_company_id, config)
            .await?;
    Ok((StatusCode::OK, Json(saved)))
}

/// DELETE /config/{customerCompanyId}
///
/// Delete a pipeline configuration and its associated EventBridge schedule.
/// The organization is resolved from the caller's JWT claims.
#[utoipa::path(
    delete,
    path = "/config/{customer_company_id}",
    tag = "Configuration",
    params(
        ("customer_company_id" = String, Path, description = "Unique identifier for the customer company"),
    ),
    responses(
        (status = 204, description = "Configuration deleted successfully"),
        (status = 401, description = "Unauthorized — missing or invalid token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn delete_config(
    State(state): State<AppState>,
    claims: Claims,
    Path(customer_company_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    engine::config_engine::delete(&state, &claims.organization_id, &customer_company_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /config/{customerCompanyId}/validate
///
/// Dry-run column propagation: parses the pipeline manifest, builds every
/// action, and folds `calculate_columns` through the chain. Returns the
/// column set after each step without executing any real transformations.
#[utoipa::path(
    post,
    path = "/config/{customer_company_id}/validate",
    tag = "Validation",
    params(
        ("customer_company_id" = String, Path, description = "Unique identifier for the customer company"),
    ),
    request_body(
        content = ConfigRequest,
        description = "Pipeline configuration to validate (only the pipeline field is used)",
    ),
    responses(
        (status = 200, description = "Validation result with per-step column snapshots", body = ValidationResult),
        (status = 400, description = "Validation error — misconfigured action or schema conflict", body = ErrorResponse),
        (status = 401, description = "Unauthorized — missing or invalid token", body = ErrorResponse),
    )
)]
pub async fn validate_config(
    claims: Claims,
    Path(_customer_company_id): Path<String>,
    Json(body): Json<ConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let _ = &claims; // org scoping available if needed later
    let result = engine::validation_engine::validate_pipeline(&body.pipeline)?;
    Ok(Json(result))
}
