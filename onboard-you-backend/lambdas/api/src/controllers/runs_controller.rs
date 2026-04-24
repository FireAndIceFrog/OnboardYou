//! Run history controller — lists and retrieves pipeline run logs.

use axum::extract::{Path, Query, State};
use axum::Json;

use crate::dependancies::Dependancies;
use crate::models::{ApiError, Claims, ErrorResponse, ListResponse, ListRunsQuery, TriggerRunResponse};
use onboard_you_models::PipelineRun;

/// List recent pipeline runs for a customer company.
#[utoipa::path(
    get,
    path = "/config/{customer_company_id}/runs",
    params(
        ("customer_company_id" = String, Path, description = "Customer company ID"),
        ("page" = Option<i64>, Query, description = "Page number (1-based)"),
        ("count_per_page" = Option<i64>, Query, description = "Items per page (default 20, max 100)"),
    ),
    responses(
        (status = 200, description = "Paginated list of pipeline runs", body = ListResponse<PipelineRun>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer" = [])),
    tag = "Runs",
)]
pub async fn list_runs(
    State(deps): State<Dependancies>,
    claims: Claims,
    Path(customer_company_id): Path<String>,
    Query(params): Query<ListRunsQuery>,
) -> Result<Json<ListResponse<PipelineRun>>, ApiError> {
    let per_page = params.count_per_page.unwrap_or(20).clamp(1, 100);
    let page = params.page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let total = deps
        .run_history_repo
        .count_runs(&claims.organization_id, &customer_company_id)
        .await?;
    let runs = deps
        .run_history_repo
        .list_runs(&claims.organization_id, &customer_company_id, per_page, offset)
        .await?;

    Ok(Json(ListResponse::new(runs, total, page, per_page)))
}

/// Get a single pipeline run by ID.
#[utoipa::path(
    get,
    path = "/config/{customer_company_id}/runs/{run_id}",
    params(
        ("customer_company_id" = String, Path, description = "Customer company ID"),
        ("run_id" = String, Path, description = "Pipeline run ID"),
    ),
    responses(
        (status = 200, description = "Pipeline run details", body = PipelineRun),
        (status = 404, description = "Run not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer" = [])),
    tag = "Runs",
)]
pub async fn get_run(
    State(deps): State<Dependancies>,
    claims: Claims,
    Path((customer_company_id, run_id)): Path<(String, String)>,
) -> Result<Json<PipelineRun>, ApiError> {
    let run = deps
        .run_history_repo
        .get_run(&claims.organization_id, &run_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Run {run_id} for {customer_company_id}")))?;
    Ok(Json(run))
}

/// Trigger a pipeline run immediately via SQS.
#[utoipa::path(
    post,
    path = "/config/{customer_company_id}/runs/trigger",
    params(
        ("customer_company_id" = String, Path, description = "Customer company ID"),
    ),
    responses(
        (status = 202, description = "Run triggered", body = TriggerRunResponse),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "A run is already in progress", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Runs",
)]
pub async fn trigger_run(
    State(deps): State<Dependancies>,
    claims: Claims,
    Path(customer_company_id): Path<String>,
) -> Result<(axum::http::StatusCode, Json<TriggerRunResponse>), ApiError> {
    if deps
        .run_history_repo
        .has_active_run(&claims.organization_id, &customer_company_id)
        .await?
    {
        return Err(ApiError::Conflict(format!(
            "A pipeline run is already in progress for {customer_company_id}"
        )));
    }

    deps.trigger_repo
        .trigger_run(&claims.organization_id, &customer_company_id)
        .await?;

    Ok((
        axum::http::StatusCode::ACCEPTED,
        Json(TriggerRunResponse {
            message: format!("Pipeline run triggered for {customer_company_id}"),
        }),
    ))
}
