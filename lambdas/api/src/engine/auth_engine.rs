//! Auth engine — business logic for the login flow.
//!
//! Validates the incoming request and delegates to the Cognito
//! repository for credential verification.

use crate::dependancies::Dependancies;
use crate::models::{ApiError, LoginRequest, LoginResponse};

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

    let response = state.auth_repo.authenticate(&req.email, &req.password).await?;

    tracing::info!(email = %req.email, "User authenticated successfully");

    Ok(response)
}


#[cfg(test)]
mod tests {
    use crate::models::{LoginRequest, LoginResponse, ApiError};
    use crate::dependancies::{Dependancies, Env};
    use crate::repositories::cognito_repository::AuthRepo;
    use async_trait::async_trait;
    use std::sync::Arc;

    struct FakeAuthRepo {
        success_resp: Option<Arc<LoginResponse>>,
        err_msg: Option<String>,
    }

    #[async_trait]
    impl AuthRepo for FakeAuthRepo {
        async fn authenticate(&self, _email: &str, _password: &str) -> Result<LoginResponse, ApiError> {
            if let Some(r) = &self.success_resp {
                Ok(LoginResponse {
                    id_token: r.id_token.clone(),
                    access_token: r.access_token.clone(),
                    refresh_token: r.refresh_token.clone(),
                    token_type: r.token_type.clone(),
                    expires_in: r.expires_in,
                })
            } else if let Some(msg) = &self.err_msg {
                Err(ApiError::Unauthorized(msg.clone()))
            } else {
                Err(ApiError::Unauthorized("no auth configured".into()))
            }
        }
    }

    async fn build_state_with_auth(success: Option<LoginResponse>, err_msg: Option<String>) -> Dependancies {
        let mut deps = Dependancies::new(Env::default()).await;
        deps.auth_repo = Arc::new(FakeAuthRepo { success_resp: success.map(Arc::new), err_msg });
        deps
    }

    #[tokio::test]
    async fn login_success_returns_tokens() {
        let resp = LoginResponse {
            id_token: "idtok".into(),
            access_token: "acctok".into(),
            refresh_token: None,
            token_type: "Bearer".into(),
            expires_in: 3600,
        };

        let state = build_state_with_auth(Some(resp.clone()), None).await;

        let req = LoginRequest { email: "user@example.com".into(), password: "secret".into() };

        let out = super::login(&state, &req).await.expect("login should succeed");
        assert_eq!(out.id_token, resp.id_token);
        assert_eq!(out.access_token, resp.access_token);
    }

    #[tokio::test]
    async fn login_validation_errors() {
        let state = build_state_with_auth(None, Some("no".into())).await;

        let req = LoginRequest { email: "   ".into(), password: "p".into() };
        let err = super::login(&state, &req).await.expect_err("should error on empty email");
        assert!(matches!(err, ApiError::Validation(_)));

        let req2 = LoginRequest { email: "a@b.com".into(), password: "".into() };
        let err2 = super::login(&state, &req2).await.expect_err("should error on empty password");
        assert!(matches!(err2, ApiError::Validation(_)));
    }

    #[tokio::test]
    async fn login_propagates_auth_repo_error() {
        let state = build_state_with_auth(None, Some("bad creds".into())).await;

        let req = LoginRequest { email: "user@example.com".into(), password: "wrong".into() };
        let err = super::login(&state, &req).await.expect_err("should propagate auth error");
        assert!(matches!(err, ApiError::Unauthorized(_)));
    }
}
