//! Cognito repository — authenticates users via the Cognito
//! `InitiateAuth` API using the `USER_PASSWORD_AUTH` flow.

use aws_sdk_cognitoidentityprovider::types::AuthFlowType;

use crate::models::{ApiError, AppState, LoginResponse};

/// Authenticate a user with email + password against Cognito.
///
/// Uses `InitiateAuth` (non-admin) with the `USER_PASSWORD_AUTH` flow,
/// which is enabled on the Cognito app client via
/// `ALLOW_USER_PASSWORD_AUTH`.
pub async fn authenticate(
    state: &AppState,
    email: &str,
    password: &str,
) -> Result<LoginResponse, ApiError> {
    let result = state
        .cognito
        .initiate_auth()
        .auth_flow(AuthFlowType::UserPasswordAuth)
        .client_id(&state.cognito_client_id)
        .auth_parameters("USERNAME", email)
        .auth_parameters("PASSWORD", password)
        .send()
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "Cognito InitiateAuth failed");
            ApiError::Unauthorized("Invalid email or password".into())
        })?;

    let auth = result.authentication_result().ok_or_else(|| {
        ApiError::Unauthorized("Authentication challenge required — not yet supported".into())
    })?;

    let id_token = auth
        .id_token()
        .ok_or_else(|| ApiError::Repository("Cognito did not return an id_token".into()))?;

    let access_token = auth
        .access_token()
        .ok_or_else(|| ApiError::Repository("Cognito did not return an access_token".into()))?;

    Ok(LoginResponse {
        id_token: id_token.to_string(),
        access_token: access_token.to_string(),
        refresh_token: auth.refresh_token().map(|s| s.to_string()),
        token_type: "Bearer".into(),
        expires_in: auth.expires_in(),
    })
}
