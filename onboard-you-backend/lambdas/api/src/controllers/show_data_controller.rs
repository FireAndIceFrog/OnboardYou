//! HTTP handlers for the "Show Data" output endpoint.
//!
//! Reads the CSV that was written by a `ShowData` pipeline action and
//! returns it as a JSON array of objects so the frontend can display it.

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use serde_json::Value;

use crate::models::{ApiError, Claims};
use crate::dependancies::Dependancies;

/// Response body for a single `ShowData` output file.
#[derive(Serialize, utoipa::ToSchema)]
pub struct ShowDataResponse {
    /// Column names derived from the CSV header row.
    pub columns: Vec<String>,
    /// Rows as JSON objects (column name → string value).
    pub rows: Vec<std::collections::HashMap<String, Value>>,
}

/// GET /config/{customer_company_id}/outputs/{action_id}
///
/// Reads the CSV produced by the named `ShowData` pipeline step and returns
/// it as a structured JSON payload.  Returns 404 if the output file does not
/// yet exist (i.e. the pipeline has not run successfully with that step).
#[utoipa::path(
    get,
    path = "/config/{customer_company_id}/outputs/{action_id}",
    tag = "Show Data",
    params(
        ("customer_company_id" = String, Path, description = "Customer company identifier"),
        ("action_id" = String, Path, description = "ID of the ShowData pipeline step"),
    ),
    responses(
        (status = 200, description = "CSV output as JSON", body = ShowDataResponse),
        (status = 401, description = "Unauthorized", body = crate::models::ErrorResponse),
        (status = 404, description = "Output not found", body = crate::models::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::models::ErrorResponse),
    )
)]
pub async fn get_show_data(
    State(state): State<Dependancies>,
    claims: Claims,
    Path((customer_company_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate inputs — no path traversal allowed.
    for segment in [&customer_company_id, &action_id] {
        if segment.contains('/') || segment.contains('\\') || segment.contains("..") {
            return Err(ApiError::Validation(
                "path segment must not contain separators or '..'".into(),
            ));
        }
    }

    let key = format!(
        "{}/{}/outputs/{}.csv",
        claims.organization_id, customer_company_id, action_id
    );

    let bytes = state.s3_repo.get_object_bytes(&key).await.map_err(|e| {
        // Surface a 404-style message when the file isn't there yet.
        match e {
            ApiError::Repository(ref msg) if msg.contains("NoSuchKey") || msg.contains("404") => {
                ApiError::NotFound(format!(
                    "Output for step '{action_id}' not found — has the pipeline run successfully?"
                ))
            }
            other => other,
        }
    })?;

    let text = String::from_utf8(bytes)
        .map_err(|_| ApiError::Repository("Output CSV contains invalid UTF-8".into()))?;

    let response = parse_csv_to_response(&text)?;
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// CSV → JSON helpers
// ---------------------------------------------------------------------------

fn parse_csv_to_response(csv: &str) -> Result<ShowDataResponse, ApiError> {
    let mut lines = csv.lines();

    let header_line = lines.next().ok_or_else(|| {
        ApiError::Repository("Output CSV is empty".into())
    })?;

    let columns: Vec<String> = parse_csv_line(header_line);

    if columns.is_empty() {
        return Err(ApiError::Repository("Output CSV has no columns".into()));
    }

    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let values = parse_csv_line(line);
        let mut row = std::collections::HashMap::new();
        for (i, col) in columns.iter().enumerate() {
            let val = values.get(i).cloned().unwrap_or_default();
            row.insert(col.clone(), Value::String(val));
        }
        rows.push(row);
    }

    Ok(ShowDataResponse { columns, rows })
}

/// Minimal RFC 4180 CSV line parser (handles quoted fields with embedded commas/newlines).
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if !in_quotes => {
                in_quotes = true;
            }
            '"' if in_quotes => {
                if chars.peek() == Some(&'"') {
                    // Escaped quote inside quoted field.
                    chars.next();
                    field.push('"');
                } else {
                    in_quotes = false;
                }
            }
            ',' if !in_quotes => {
                fields.push(field.trim().to_string());
                field = String::new();
            }
            other => field.push(other),
        }
    }
    fields.push(field.trim().to_string());
    fields
}
