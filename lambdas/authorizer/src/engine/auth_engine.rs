//! Auth engine — business logic for token authorization.
//!
//! In dev mode (`AUTH_DEV_MODE=true`) every request is allowed with a
//! placeholder identity.  In production mode the engine validates a
//! Cognito JWT and extracts the `custom:organizationId` claim.

use std::sync::Arc;

use crate::dependancies::Dependancies;
use crate::models::{AuthConfig, AuthError, AuthEvent, AuthResponse, Claims};
use async_trait::async_trait;

#[async_trait]
pub trait IAuthEngine: Send + Sync {
    async fn authorize(
        &self,
        state: &Dependancies,
        event: &AuthEvent,
    ) -> Result<AuthResponse, AuthError>;
}

pub struct AuthEngine {
    cfg: Arc<AuthConfig>,
}

//------------------------------------ Public methods ------------------------------------------------
#[async_trait]
impl IAuthEngine for AuthEngine {
    /// Authorize an incoming API Gateway event.
    ///
    /// Returns an `AuthResponse` (Allow or Deny) that API Gateway turns into
    /// an IAM policy.
    async fn authorize(
        &self,
        state: &Dependancies,
        event: &AuthEvent,
    ) -> Result<AuthResponse, AuthError> {
        let method_arn = event.method_arn.as_deref().unwrap_or("*");

        // ── Dev mode — allow everything ─────────────────────────
        if self.cfg.dev_mode {
            tracing::warn!("AUTH_DEV_MODE enabled — all requests allowed");
            return Ok(AuthResponse::allow("dev-user", "dev-org", method_arn));
        }

        // ── Production — validate Cognito JWT ───────────────────
        let token = event
            .authorization_token
            .as_deref()
            .and_then(|t| t.strip_prefix("Bearer "))
            .ok_or(AuthError::MissingToken)?;

        let claims = self.get_claims(state, token).await?;

        let sub = claims.sub.unwrap_or_else(|| "unknown".to_string());
        let organization_id = claims.organization_id.ok_or_else(|| {
            AuthError::InvalidToken("Token missing custom:organizationId claim".into())
        })?;

        tracing::info!(sub = %sub, organization_id = %organization_id, "Token validated");

        Ok(AuthResponse::allow(&sub, &organization_id, method_arn))
    }
}

//------------------------------------ Private methods ------------------------------------------------
impl AuthEngine {
    /// Constructor for the auth engine, taking dependancies as input.
    pub fn new(cfg: Arc<AuthConfig>) -> Arc<Self> {
        Arc::new(Self { cfg })
    }

    async fn get_claims(&self, state: &Dependancies, token: &str) -> Result<Claims, AuthError> {
        // ── Production — validate Cognito JWT ───────────────────
        let user_pool_id =
            self.cfg.user_pool_id.as_deref().ok_or_else(|| {
                AuthError::InvalidToken("COGNITO_USER_POOL_ID not configured".into())
            })?;

        let client_id =
            self.cfg.client_id.as_deref().ok_or_else(|| {
                AuthError::InvalidToken("COGNITO_CLIENT_ID not configured".into())
            })?;

        let issuer = format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            self.cfg.aws_region, user_pool_id
        );

        // Fetch the JWKS (cached at the HTTP layer across warm invocations)
        let jwks = state.jwks_repository.fetch_jwks(&issuer).await?;

        // Decode the JWT header to find the key id
        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| AuthError::InvalidToken(format!("Bad JWT header: {e}")))?;

        let kid = header
            .kid
            .ok_or_else(|| AuthError::InvalidToken("Token missing kid".into()))?;

        // Look up the matching key in the JWKS
        let jwk = jwks["keys"]
            .as_array()
            .and_then(|keys| keys.iter().find(|k| k["kid"].as_str() == Some(&kid)))
            .ok_or_else(|| AuthError::InvalidToken("No matching JWK found".into()))?;

        let n = jwk["n"]
            .as_str()
            .ok_or_else(|| AuthError::InvalidToken("JWK missing 'n'".into()))?;
        let e = jwk["e"]
            .as_str()
            .ok_or_else(|| AuthError::InvalidToken("JWK missing 'e'".into()))?;

        let decoding_key = jsonwebtoken::DecodingKey::from_rsa_components(n, e)
            .map_err(|e| AuthError::InvalidToken(format!("Bad RSA components: {e}")))?;

        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_issuer(&[&issuer]);
        validation.set_audience(&[client_id]);

        let token_data = jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| AuthError::InvalidToken(format!("JWT verification failed: {e}")))?;

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{AuthConfig, AuthError, AuthEvent},
        repositories::jwks_repository::IJwksRepository,
    };
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    struct MockJwksRepository {
        called: Mutex<Vec<String>>,
    }

    impl MockJwksRepository {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                called: Mutex::new(vec![]),
            })
        }

        fn called_issuers(&self) -> Vec<String> {
            self.called.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl IJwksRepository for MockJwksRepository {
        async fn fetch_jwks(&self, issuer: &str) -> Result<serde_json::Value, AuthError> {
            self.called.lock().unwrap().push(issuer.to_string());
            Err(AuthError::JwksError("mocked jwks failure".into()))
        }
    }

    #[tokio::test]
    async fn dev_mode_allows_everything() {
        let cfg = Arc::new(AuthConfig {
            dev_mode: true,
            user_pool_id: None,
            client_id: None,
            aws_region: "eu-west-1".into(),
        });

        let mut dependancies = Dependancies::new(cfg);
        dependancies.jwks_repository = MockJwksRepository::new();

        let event = AuthEvent {
            authorization_token: None,
            method_arn: Some("arn:aws:execute-api:eu-west-1:123:api/stage/GET/resource".into()),
        };

        let res = dependancies
            .auth_engine
            .authorize(&dependancies, &event)
            .await
            .expect("dev mode should allow");
        assert_eq!(res.principal_id, "dev-user");
        assert_eq!(res.context["organizationId"], json!("dev-org"));
    }

    #[tokio::test]
    async fn missing_token_returns_error_in_prod() {
        let cfg = Arc::new(AuthConfig {
            dev_mode: false,
            user_pool_id: None,
            client_id: None,
            aws_region: "eu-west-1".into(),
        });

        let mut dependancies = Dependancies::new(cfg);
        dependancies.jwks_repository = MockJwksRepository::new();

        let event = AuthEvent {
            authorization_token: None,
            method_arn: None,
        };

        let res = dependancies
            .auth_engine
            .authorize(&dependancies, &event)
            .await;
        assert!(matches!(res, Err(AuthError::MissingToken)));
    }

    #[tokio::test]
    async fn jwks_repository_is_called_and_error_propagates() {
        let jwks_repository = MockJwksRepository::new();

        let cfg = Arc::new(AuthConfig {
            dev_mode: false,
            user_pool_id: Some("userpool123".into()),
            client_id: Some("clientid".into()),
            aws_region: "eu-west-1".into(),
        });
        let mut dependancies = Dependancies::new(cfg);
        dependancies.jwks_repository = jwks_repository.clone();

        let event = AuthEvent {
            authorization_token: Some("Bearer dummy.token.value".into()),
            method_arn: None,
        };

        let res = dependancies
            .auth_engine
            .authorize(&dependancies, &event)
            .await;
        assert!(matches!(res, Err(AuthError::JwksError(_))));

        let called = jwks_repository.called_issuers();
        assert_eq!(called.len(), 1);
        assert_eq!(
            called[0],
            "https://cognito-idp.eu-west-1.amazonaws.com/userpool123"
        );
    }
}
