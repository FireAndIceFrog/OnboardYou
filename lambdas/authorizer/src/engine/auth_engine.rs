//! Auth engine — business logic for token authorization.
//!
//! In dev mode (`AUTH_DEV_MODE=true`) every request is allowed with a
//! placeholder identity.  In production mode the engine validates a
//! Cognito JWT and extracts the `custom:organizationId` claim.

use crate::models::{AuthError, AuthEvent, AuthResponse};
use crate::repositories::jwks_repository;

/// Configuration read once at cold-start.
pub struct AuthConfig {
    pub dev_mode: bool,
    pub user_pool_id: Option<String>,
    pub client_id: Option<String>,
    pub aws_region: String,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        Self {
            dev_mode: std::env::var("AUTH_DEV_MODE").unwrap_or_default() == "true",
            user_pool_id: std::env::var("COGNITO_USER_POOL_ID").ok(),
            client_id: std::env::var("COGNITO_CLIENT_ID").ok(),
            aws_region: std::env::var("AWS_REGION").unwrap_or_else(|_| "eu-west-1".into()),
        }
    }
}

/// Authorize an incoming API Gateway event.
///
/// Returns an `AuthResponse` (Allow or Deny) that API Gateway turns into
/// an IAM policy.
pub async fn authorize(cfg: &AuthConfig, event: &AuthEvent) -> Result<AuthResponse, AuthError> {
    let method_arn = event.method_arn.as_deref().unwrap_or("*");

    // ── Dev mode — allow everything ─────────────────────────
    if cfg.dev_mode {
        tracing::warn!("AUTH_DEV_MODE enabled — all requests allowed");
        return Ok(AuthResponse::allow("dev-user", "dev-org", method_arn));
    }

    // ── Production — validate Cognito JWT ───────────────────
    let token = event
        .authorization_token
        .as_deref()
        .and_then(|t| t.strip_prefix("Bearer "))
        .ok_or(AuthError::MissingToken)?;

    let user_pool_id = cfg
        .user_pool_id
        .as_deref()
        .ok_or_else(|| AuthError::InvalidToken("COGNITO_USER_POOL_ID not configured".into()))?;

    let client_id = cfg
        .client_id
        .as_deref()
        .ok_or_else(|| AuthError::InvalidToken("COGNITO_CLIENT_ID not configured".into()))?;

    let issuer = format!(
        "https://cognito-idp.{}.amazonaws.com/{}",
        cfg.aws_region, user_pool_id
    );

    // Fetch the JWKS (cached at the HTTP layer across warm invocations)
    let jwks = jwks_repository::fetch_jwks(&issuer).await?;

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

    let token_data = jsonwebtoken::decode::<std::collections::HashMap<String, serde_json::Value>>(
        token,
        &decoding_key,
        &validation,
    )
    .map_err(|e| AuthError::InvalidToken(format!("JWT verification failed: {e}")))?;

    let claims = token_data.claims;

    let sub = claims
        .get("sub")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let organization_id = claims
        .get("custom:organizationId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::InvalidToken("Token missing custom:organizationId claim".into()))?;

    tracing::info!(sub = %sub, organization_id = %organization_id, "Token validated");

    Ok(AuthResponse::allow(&sub, organization_id, method_arn))
}
