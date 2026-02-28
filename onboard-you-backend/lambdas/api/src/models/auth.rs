//! Authentication request / response DTOs.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Login request — email + password (plain credentials).
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// User email address (Cognito username).
    pub email: String,
    /// User password.
    pub password: String,
}

/// Successful login response — Cognito token set.
#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct LoginResponse {
    /// JWT ID token (contains custom claims such as `organizationId`).
    pub id_token: String,
    /// JWT access token.
    pub access_token: String,
    /// Refresh token (can be exchanged for new tokens).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Token type — always `"Bearer"`.
    pub token_type: String,
    /// Token lifetime in seconds.
    pub expires_in: i32,
}
