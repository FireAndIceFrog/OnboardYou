//! HTTP handlers for authentication endpoints.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::engine;
use crate::models::{ErrorResponse, LoginRequest, LoginResponse};
use crate::dependancies::Dependancies;

/// POST /auth/login
///
/// Authenticate with email + password and receive Cognito tokens.
/// This endpoint does **not** require a JWT — it is the entry point
/// for obtaining one.
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Authentication",
    request_body(
        content = LoginRequest,
        description = "User credentials",
    ),
    responses(
        (status = 200, description = "Authentication successful — token set returned", body = LoginResponse),
        (status = 400, description = "Validation error (missing email or password)", body = ErrorResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn login(
    State(state): State<Dependancies>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, crate::models::ApiError> {
    let tokens = engine::auth_engine::login(&state, &body).await?;
    Ok((StatusCode::OK, Json(tokens)))
}
