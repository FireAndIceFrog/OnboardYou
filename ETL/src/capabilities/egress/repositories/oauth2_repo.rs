//! OAuth2 repository: Client Credentials & Authorization Code (OIDC) flows
//!
//! Supports the two most common OAuth2 grant types for machine-to-machine
//! and delegated-user scenarios:
//!
//! - **Client Credentials** — the dispatcher authenticates as itself.
//! - **Authorization Code / OIDC** — uses a pre-obtained refresh token to
//!   acquire short-lived access tokens on behalf of a user/org.
//!
//! Token caching and proactive refresh happen internally so the `ApiEngine`
//! does not need to manage token lifecycle.
//!
//! # Future: DynamoDB / KeyVault integration
//!
//! `client_secret` and `refresh_token` will be encrypted per-org in DynamoDB
//! and decrypted at construction time via a `SecretProvider` trait.

use crate::capabilities::egress::models::{DispatchResponse, OAuth2GrantType, OAuth2RepoConfig};
use crate::capabilities::egress::traits::EgressRepository;
use crate::domain::{Error, Result};
use std::sync::Mutex;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Internal token cache
// ---------------------------------------------------------------------------

/// Cached access token with expiry tracking.
#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    /// When this token expires (wall-clock `Instant`).
    expires_at: Option<Instant>,
    /// Updated refresh token (some providers rotate it).
    refresh_token: Option<String>,
}

impl CachedToken {
    /// Returns `true` if the token is still valid (with a 30-second safety margin).
    fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(exp) => Instant::now() + std::time::Duration::from_secs(30) < exp,
            None => true, // No expiry info — assume valid until a 401 says otherwise.
        }
    }
}

// ---------------------------------------------------------------------------
// Repository
// ---------------------------------------------------------------------------

/// Egress repository implementing OAuth2 token exchange and data dispatch.
pub struct OAuth2Repo {
    config: OAuth2RepoConfig,
    /// Internal token cache protected by a mutex for interior mutability
    /// across the sync/async boundary.
    token_cache: Mutex<Option<CachedToken>>,
}

impl OAuth2Repo {
    pub fn new(config: OAuth2RepoConfig) -> Self {
        Self {
            config,
            token_cache: Mutex::new(None),
        }
    }

    pub fn from_action_config(config: &OAuth2RepoConfig) -> Result<Self> {
        Ok(Self::new(config.clone()))
    }

    /// Perform the token exchange with the authorization server.
    ///
    /// Called internally when the cache is empty or expired.
    async fn exchange_token(&self) -> Result<CachedToken> {
        let client = reqwest::Client::new();

        let mut params: Vec<(&str, String)> = vec![
            ("client_id", self.config.client_id.clone()),
            ("client_secret", self.config.client_secret.clone()),
        ];

        if !self.config.scopes.is_empty() {
            params.push(("scope", self.config.scopes.join(" ")));
        }

        match &self.config.grant_type {
            OAuth2GrantType::ClientCredentials => {
                params.push(("grant_type", "client_credentials".into()));
            }
            OAuth2GrantType::AuthorizationCode => {
                let initial_refresh = self.config.refresh_token.as_ref()
                    .ok_or_else(|| Error::ConfigurationError(
                        "authorization_code grant requires a 'refresh_token'".into(),
                    ))?;

                // Use the latest refresh token (may have been rotated).
                let current_refresh = self
                    .token_cache
                    .lock()
                    .ok()
                    .and_then(|cache| {
                        cache
                            .as_ref()
                            .and_then(|c| c.refresh_token.clone())
                    })
                    .unwrap_or_else(|| initial_refresh.clone());

                params.push(("grant_type", "refresh_token".into()));
                params.push(("refresh_token", current_refresh));
            }
        }

        let response = client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::EgressError(format!("OAuth2 token exchange failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::EgressError(format!(
                "OAuth2 token endpoint returned {status}: {body}"
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::EgressError(format!("Failed to parse token response: {e}")))?;

        let access_token = body
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::EgressError("Token response missing 'access_token'".into())
            })?
            .to_string();

        let expires_at = body.get("expires_in").and_then(|v| v.as_u64()).map(|secs| {
            Instant::now() + std::time::Duration::from_secs(secs)
        });

        let refresh_token = body
            .get("refresh_token")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(CachedToken {
            access_token,
            expires_at,
            refresh_token,
        })
    }

    /// Get a valid access token, refreshing if necessary.
    async fn get_valid_token(&self) -> Result<String> {
        // Check cache first.
        if let Ok(cache) = self.token_cache.lock() {
            if let Some(ref cached) = *cache {
                if cached.is_valid() {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        // Cache miss or expired — exchange.
        let new_token = self.exchange_token().await?;
        let access_token = new_token.access_token.clone();

        if let Ok(mut cache) = self.token_cache.lock() {
            *cache = Some(new_token);
        }

        Ok(access_token)
    }
}

impl EgressRepository for OAuth2Repo {
    fn retrieve_token(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<String>>> + Send + '_>>
    {
        Box::pin(async move {
            let token = self.get_valid_token().await?;
            Ok(Some(token))
        })
    }

    fn send_data(
        &self,
        payload: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<DispatchResponse>> + Send + '_>,
    > {
        let destination_url = self.config.destination_url.clone();
        let payload = payload.to_string();

        Box::pin(async move {
            let token = self.get_valid_token().await?;

            let client = reqwest::Client::new();
            let response = client
                .post(&destination_url)
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(payload.clone())
                .send()
                .await
                .map_err(|e| Error::EgressError(format!("OAuth2 dispatch failed: {e}")))?;

            let status_code = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            let records_sent = serde_json::from_str::<Vec<serde_json::Value>>(&payload)
                .map(|v| v.len())
                .unwrap_or(1);

            Ok(DispatchResponse {
                status_code,
                body,
                records_sent,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_token_validity() {
        let valid_token = CachedToken {
            access_token: "abc".into(),
            expires_at: Some(Instant::now() + std::time::Duration::from_secs(3600)),
            refresh_token: None,
        };
        assert!(valid_token.is_valid());

        let expired_token = CachedToken {
            access_token: "abc".into(),
            expires_at: Some(Instant::now() - std::time::Duration::from_secs(1)),
            refresh_token: None,
        };
        assert!(!expired_token.is_valid());
    }
}
