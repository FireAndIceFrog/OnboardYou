//! HTTP handlers for pipeline configuration endpoints.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::models::{ApiError, Claims, ConfigRequest, ErrorResponse, ListResponse};
use crate::{dependancies::Dependancies, engine, models::ValidationResult};
use onboard_you_models::PipelineConfig;

/// Query parameters for listing configs.
#[derive(Debug, serde::Deserialize)]
pub struct ListConfigsQuery {
    /// Page number (1-based, default 1).
    pub page: Option<i64>,
    /// Items per page (default 20, max 100).
    pub count_per_page: Option<i64>,
}

/// GET /config
///
/// List all pipeline configurations owned by the caller's organization.
/// The organization is resolved from the caller's JWT claims.
#[utoipa::path(
    get,
    path = "/config",
    tag = "Configuration",
    params(
        ("page" = Option<i64>, Query, description = "Page number (1-based)"),
        ("count_per_page" = Option<i64>, Query, description = "Items per page (default 20, max 100)"),
    ),
    responses(
        (status = 200, description = "Paginated list of pipeline configurations", body = ListResponse<PipelineConfig>),
        (status = 401, description = "Unauthorized — missing or invalid token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn list_configs(
    State(state): State<Dependancies>,
    claims: Claims,
    Query(params): Query<ListConfigsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let per_page = params.count_per_page.unwrap_or(20).min(100).max(1);
    let page = params.page.unwrap_or(1).max(1);
    let configs = engine::config_engine::list(&state, &claims.organization_id).await?;
    Ok(Json(ListResponse::from_vec(configs, page, per_page)))
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
    State(state): State<Dependancies>,
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
    State(state): State<Dependancies>,
    claims: Claims,
    Path(customer_company_id): Path<String>,
    Json(body): Json<ConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    engine::validation_engine::validate_pipeline(&state, &body.pipeline, None).await?;
    let config = body.into_config();
    let saved = engine::config_engine::upsert(
        &state,
        &claims.organization_id,
        &customer_company_id,
        config,
    )
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
    State(state): State<Dependancies>,
    claims: Claims,
    Path(customer_company_id): Path<String>,
    Json(body): Json<ConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    engine::validation_engine::validate_pipeline(&state, &body.pipeline, Some(claims.organization_id.clone())).await?;
    let config = body.into_config();
    let saved = engine::config_engine::upsert(
        &state,
        &claims.organization_id,
        &customer_company_id,
        config,
    )
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
    State(state): State<Dependancies>,
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
    State(state): State<Dependancies>,
    claims: Claims,
    Path(_customer_company_id): Path<String>,
    Json(body): Json<ConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let _ = &claims; // org scoping available if needed later
    let result = engine::validation_engine::validate_pipeline(&state, &body.pipeline, None).await?;
    Ok(Json(result))
}
