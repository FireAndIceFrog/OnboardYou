//! CSV upload engine — coordinates presigned URL generation and column discovery.

use crate::models::{ApiError, AppState, CsvColumnsResponse, PresignedUploadResponse};
use crate::repositories::s3_repository;

/// Build the S3 object key from runtime context.
fn s3_key(organization_id: &str, customer_company_id: &str, filename: &str) -> String {
    format!("{organization_id}/{customer_company_id}/{filename}")
}

/// Generate a presigned PUT URL for a CSV upload.
pub async fn presigned_upload(
    state: &AppState,
    organization_id: &str,
    customer_company_id: &str,
    filename: &str,
) -> Result<PresignedUploadResponse, ApiError> {
    if filename.is_empty() {
        return Err(ApiError::Validation("filename must not be empty".into()));
    }

    // Sanitise: only allow simple filenames (no path traversal)
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(ApiError::Validation(
            "filename must not contain path separators or '..'".into(),
        ));
    }

    let key = s3_key(organization_id, customer_company_id, filename);

    let upload_url =
        s3_repository::presigned_put_url(&state.s3, &state.csv_upload_bucket, &key).await?;

    Ok(PresignedUploadResponse {
        upload_url,
        key,
        filename: filename.to_string(),
    })
}

/// Read the columns from an already-uploaded CSV in S3.
pub async fn read_columns(
    state: &AppState,
    organization_id: &str,
    customer_company_id: &str,
    filename: &str,
) -> Result<CsvColumnsResponse, ApiError> {
    let key = s3_key(organization_id, customer_company_id, filename);

    let columns =
        s3_repository::read_csv_headers(&state.s3, &state.csv_upload_bucket, &key).await?;

    Ok(CsvColumnsResponse {
        filename: filename.to_string(),
        columns,
    })
}
