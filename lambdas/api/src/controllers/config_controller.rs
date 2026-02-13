//! HTTP handlers for pipeline configuration endpoints.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::engine;
use crate::models::{ApiError, AppState, ErrorResponse, PipelineConfig};
use engine::validation_engine::ValidationResult;

/// GET /{organizationId}/{customerCompanyId}/config
///
/// Retrieve the pipeline configuration for a given organization and customer company.
#[utoipa::path(
    get,
    path = "/{organization_id}/{customer_company_id}/config",
    tag = "Configuration",
    params(
        ("organization_id" = String, Path, description = "Unique identifier for the organization"),
        ("customer_company_id" = String, Path, description = "Unique identifier for the customer company"),
    ),
    responses(
        (status = 200, description = "Pipeline configuration found", body = PipelineConfig),
        (status = 404, description = "Configuration not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn get_config(
    State(state): State<AppState>,
    Path((organization_id, customer_company_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let config = engine::config_engine::get(&state, &organization_id, &customer_company_id).await?;
    Ok(Json(config))
}

/// POST /{organizationId}/{customerCompanyId}/config
///
/// Create a new pipeline configuration. Validates the pipeline manifest,
/// persists to DynamoDB, and creates an EventBridge schedule.
#[utoipa::path(
    post,
    path = "/{organization_id}/{customer_company_id}/config",
    tag = "Configuration",
    params(
        ("organization_id" = String, Path, description = "Unique identifier for the organization"),
        ("customer_company_id" = String, Path, description = "Unique identifier for the customer company"),
    ),
    request_body(
        content = PipelineConfig,
        description = "Pipeline configuration to create",
    ),
    responses(
        (status = 200, description = "Configuration created successfully", body = PipelineConfig),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn create_config(
    State(state): State<AppState>,
    Path((organization_id, customer_company_id)): Path<(String, String)>,
    Json(body): Json<PipelineConfig>,
) -> Result<impl IntoResponse, ApiError> {
    engine::validation_engine::validate_pipeline(&body.pipeline)?;
    let saved = engine::config_engine::upsert(&state, &organization_id, &customer_company_id, body).await?;
    Ok((StatusCode::OK, Json(saved)))
}

/// PUT /{organizationId}/{customerCompanyId}/config
///
/// Update an existing pipeline configuration. Validates the pipeline manifest,
/// persists to DynamoDB, and updates the EventBridge schedule.
#[utoipa::path(
    put,
    path = "/{organization_id}/{customer_company_id}/config",
    tag = "Configuration",
    params(
        ("organization_id" = String, Path, description = "Unique identifier for the organization"),
        ("customer_company_id" = String, Path, description = "Unique identifier for the customer company"),
    ),
    request_body(
        content = PipelineConfig,
        description = "Pipeline configuration to update",
    ),
    responses(
        (status = 200, description = "Configuration updated successfully", body = PipelineConfig),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn update_config(
    State(state): State<AppState>,
    Path((organization_id, customer_company_id)): Path<(String, String)>,
    Json(body): Json<PipelineConfig>,
) -> Result<impl IntoResponse, ApiError> {
    engine::validation_engine::validate_pipeline(&body.pipeline)?;
    let saved = engine::config_engine::upsert(&state, &organization_id, &customer_company_id, body).await?;
    Ok((StatusCode::OK, Json(saved)))
}

/// POST /{organizationId}/{customerCompanyId}/config/validate
///
/// Dry-run column propagation: parses the pipeline manifest, builds every
/// action, and folds `calculate_columns` through the chain. Returns the
/// column set after each step without executing any real transformations.
#[utoipa::path(
    post,
    path = "/{organization_id}/{customer_company_id}/config/validate",
    tag = "Validation",
    params(
        ("organization_id" = String, Path, description = "Unique identifier for the organization"),
        ("customer_company_id" = String, Path, description = "Unique identifier for the customer company"),
    ),
    request_body(
        content = PipelineConfig,
        description = "Pipeline configuration to validate (only the pipeline field is used)",
    ),
    responses(
        (status = 200, description = "Validation result with per-step column snapshots", body = ValidationResult),
        (status = 400, description = "Validation error — misconfigured action or schema conflict", body = ErrorResponse),
    )
)]
pub async fn validate_config(
    Path((_organization_id, _customer_company_id)): Path<(String, String)>,
    Json(body): Json<PipelineConfig>,
) -> Result<impl IntoResponse, ApiError> {
    let result = engine::validation_engine::validate_pipeline(&body.pipeline)?;
    Ok(Json(result))
}
