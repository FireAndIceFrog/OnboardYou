//! JWKS repository — fetches the Cognito JSON Web Key Set.

use std::sync::Arc;

use crate::models::AuthError;
use async_trait::async_trait;
use serde_json::Value;

pub struct JwksRepository;

#[async_trait]
pub trait IJwksRepository: Send + Sync {
    async fn fetch_jwks(&self, issuer: &str) -> Result<Value, AuthError>;
}

#[async_trait]
impl IJwksRepository for JwksRepository {
    /// Fetch the JWKS document from the Cognito well-known endpoint.
    ///
    /// In a warm Lambda, the HTTP client's connection pool gives us implicit
    /// caching.  For explicit caching, a `OnceCell` layer can be added later.
    async fn fetch_jwks(&self, issuer: &str) -> Result<Value, AuthError> {
        let url = format!("{issuer}/.well-known/jwks.json");

        let jwks: Value = reqwest::get(&url)
            .await
            .map_err(|e| AuthError::JwksError(format!("HTTP request failed: {e}")))?
            .json()
            .await
            .map_err(|e| AuthError::JwksError(format!("Failed to parse JWKS JSON: {e}")))?;

        Ok(jwks)
    }
}

impl JwksRepository {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}
