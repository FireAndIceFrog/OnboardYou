use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use utoipa::ToSchema;

/// JSON error envelope returned to clients.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Human-readable error description
    pub error: String,
}

/// Typed API errors — auto-mapped to HTTP status codes via IntoResponse.
#[derive(Debug)]
pub enum ApiError {
    Unauthorized(String),
    NotFound(String),
    Validation(String),
    Repository(String),
    Conflict(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            ApiError::NotFound(id) => (
                StatusCode::NOT_FOUND,
                format!("Config not found for org: {id}"),
            ),
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Repository(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
        };

        tracing::error!(error = %message, status = %status);
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
