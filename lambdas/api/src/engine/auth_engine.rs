//! Auth engine — business logic for the login flow.
//!
//! Validates the incoming request and delegates to the Cognito
//! repository for credential verification.

use crate::dependancies::Dependancies;
use crate::models::{ApiError, LoginRequest, LoginResponse};
use crate::repositories::cognito_repository;

/// Authenticate a user with email + password.
///
/// Returns a token set on success, or an `ApiError` on failure.
pub async fn login(state: &Dependancies, req: &LoginRequest) -> Result<LoginResponse, ApiError> {
    // Basic input validation
    if req.email.trim().is_empty() {
        return Err(ApiError::Validation("email is required".into()));
    }
    if req.password.is_empty() {
        return Err(ApiError::Validation("password is required".into()));
    }

    let response = cognito_repository::authenticate(state, &req.email, &req.password).await?;

    tracing::info!(email = %req.email, "User authenticated successfully");

    Ok(response)
}
