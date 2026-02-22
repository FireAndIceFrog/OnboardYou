//! HTTP handlers for CSV upload endpoints.

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};

use crate::{dependancies::Dependancies, engine, models::{CsvColumnsResponse, CsvFileQuery, PresignedUploadResponse}};
use crate::models::{ApiError, Claims};

/// POST /config/{customer_company_id}/csv-upload?filename=employees.csv
///
/// Returns a presigned PUT URL that the frontend uses to upload the CSV
/// directly to S3.  The S3 key is `{org_id}/{company_id}/{filename}`.
#[utoipa::path(
    post,
    path = "/config/{customer_company_id}/csv-upload",
    tag = "CSV Upload",
    params(
        ("customer_company_id" = String, Path, description = "Customer company identifier"),
        CsvFileQuery,
    ),
    responses(
        (status = 200, description = "Presigned upload URL", body = PresignedUploadResponse),
        (status = 400, description = "Invalid filename", body = crate::models::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::models::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::models::ErrorResponse),
    )
)]
pub async fn csv_presigned_upload(
    State(state): State<Dependancies>,
    claims: Claims,
    Path(customer_company_id): Path<String>,
    Query(query): Query<CsvFileQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let resp = engine::csv_upload_engine::presigned_upload(
        &state,
        &claims.organization_id,
        &customer_company_id,
        &query.filename,
    )
    .await?;

    Ok(Json(resp))
}

/// GET /config/{customer_company_id}/csv-columns?filename=employees.csv
///
/// Reads the header row of an already-uploaded CSV and returns the column
/// names.  Call this after the frontend finishes the presigned PUT upload.
#[utoipa::path(
    get,
    path = "/config/{customer_company_id}/csv-columns",
    tag = "CSV Upload",
    params(
        ("customer_company_id" = String, Path, description = "Customer company identifier"),
        CsvFileQuery,
    ),
    responses(
        (status = 200, description = "CSV column names", body = CsvColumnsResponse),
        (status = 400, description = "Invalid CSV or missing file", body = crate::models::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::models::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::models::ErrorResponse),
    )
)]
pub async fn csv_columns(
    State(state): State<Dependancies>,
    claims: Claims,
    Path(customer_company_id): Path<String>,
    Query(query): Query<CsvFileQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let resp = engine::csv_upload_engine::read_columns(
        &state,
        &claims.organization_id,
        &customer_company_id,
        &query.filename,
    )
    .await?;

    Ok(Json(resp))
}
