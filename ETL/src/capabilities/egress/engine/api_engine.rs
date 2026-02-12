//! ApiEngine: Orchestrates egress authentication and data dispatch
//!
//! The `ApiEngine` is the single entry point for the `ApiDispatcher`.
//! It owns a concrete `EgressRepository` (Bearer, OAuth, OAuth2) selected
//! at construction time from the manifest config, and exposes a synchronous
//! `dispatch` method that bridges the async internals via the tokio runtime.
//!
//! # Architecture
//!
//! ```text
//!  ApiDispatcher (OnboardingAction — sync)
//!       │
//!       ▼
//!  ApiEngine::dispatch()  ← sync boundary, uses tokio block_on
//!       │
//!       ├─ repo.retrieve_token()   (async)
//!       └─ repo.send_data()        (async)
//!       │
//!       ▼
//!  EgressRepository impl (BearerRepo | OAuthRepo | OAuth2Repo)
//! ```

use crate::capabilities::egress::models::{AuthType, DispatchResponse, RetryPolicy};
use crate::capabilities::egress::repositories::bearer_repo::BearerRepo;
use crate::capabilities::egress::repositories::oauth2_repo::OAuth2Repo;
use crate::capabilities::egress::repositories::oauth_repo::OAuthRepo;
use crate::capabilities::egress::traits::EgressRepository;
use crate::domain::{Error, Result};
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Orchestrator for egress authentication and data dispatch.
///
/// Constructed once per pipeline execution from the manifest config.
/// The `ApiDispatcher` delegates all real work here.
pub struct ApiEngine {
    /// The concrete repository handling auth + HTTP dispatch.
    repo: Box<dyn EgressRepository>,
    /// Retry policy for transient failures.
    retry_policy: RetryPolicy,
}

impl ApiEngine {
    /// Build an `ApiEngine` from the raw manifest `ActionConfig.config` JSON.
    ///
    /// Reads `"auth_type"` to select the right repository, then delegates
    /// config parsing to that repo's `from_action_config`.
    pub fn from_action_config(value: &serde_json::Value) -> Result<Self> {
        let auth_type: AuthType = value
            .get("auth_type")
            .map(|v| serde_json::from_value(v.clone()))
            .unwrap_or(Ok(AuthType::Bearer))
            .map_err(|e| {
                Error::ConfigurationError(format!(
                    "Invalid auth_type: {e}. Expected: bearer | api_key | none | oauth | oauth1 | oauth2 | oidc | openid"
                ))
            })?;

        let repo: Box<dyn EgressRepository> = match auth_type {
            AuthType::Bearer => Box::new(BearerRepo::from_action_config(value)?),
            AuthType::OAuth => Box::new(OAuthRepo::from_action_config(value)?),
            AuthType::OAuth2 => Box::new(OAuth2Repo::from_action_config(value)?),
        };

        Ok(Self {
            repo,
            retry_policy: RetryPolicy::default(),
        })
    }

    /// Override the default retry policy.
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Dispatch data to the configured destination.
    ///
    /// **This is the sync boundary.** Internally spawns async work via tokio.
    /// Called from `ApiDispatcher::execute()`.
    pub fn dispatch(&self, payload: &str) -> Result<DispatchResponse> {
        // Use the existing tokio runtime (Lambda / pipeline already runs inside one).
        let handle = tokio::runtime::Handle::try_current().map_err(|_| {
            Error::EgressError(
                "ApiEngine::dispatch requires a running tokio runtime".into(),
            )
        })?;

        handle.block_on(self.dispatch_with_retry(payload))
    }

    /// Internal async dispatch with retry loop.
    async fn dispatch_with_retry(&self, payload: &str) -> Result<DispatchResponse> {
        let mut last_error: Option<Error> = None;

        for attempt in 1..=self.retry_policy.max_attempts {
            match self.repo.send_data(payload).await {
                Ok(response) => {
                    if self
                        .retry_policy
                        .retryable_status_codes
                        .contains(&response.status_code)
                        && attempt < self.retry_policy.max_attempts
                    {
                        let backoff = self.retry_policy.initial_backoff_ms * 2u64.pow(attempt - 1);
                        warn!(
                            attempt,
                            status_code = response.status_code,
                            backoff_ms = backoff,
                            "Retryable status code, backing off"
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(backoff)).await;
                        last_error = Some(Error::EgressError(format!(
                            "Retryable status {}: {}",
                            response.status_code, response.body
                        )));
                        continue;
                    }

                    info!(
                        attempt,
                        status_code = response.status_code,
                        records_sent = response.records_sent,
                        "Egress dispatch completed"
                    );
                    return Ok(response);
                }
                Err(e) => {
                    if attempt < self.retry_policy.max_attempts {
                        let backoff = self.retry_policy.initial_backoff_ms * 2u64.pow(attempt - 1);
                        warn!(
                            attempt,
                            backoff_ms = backoff,
                            error = %e,
                            "Dispatch failed, retrying"
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(backoff)).await;
                    }
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::EgressError("Dispatch failed: no attempts made".into())
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_type_serde_parsing() {
        let de = |s: &str| -> AuthType {
            serde_json::from_value(serde_json::Value::String(s.into())).unwrap()
        };

        assert_eq!(de("bearer"), AuthType::Bearer);
        assert_eq!(de("api_key"), AuthType::Bearer);
        assert_eq!(de("none"), AuthType::Bearer);
        assert_eq!(de("oauth"), AuthType::OAuth);
        assert_eq!(de("oauth1"), AuthType::OAuth);
        assert_eq!(de("oauth2"), AuthType::OAuth2);
        assert_eq!(de("oidc"), AuthType::OAuth2);
        assert_eq!(de("openid"), AuthType::OAuth2);

        let bad: std::result::Result<AuthType, _> =
            serde_json::from_value(serde_json::Value::String("unknown".into()));
        assert!(bad.is_err());
    }

    #[test]
    fn test_engine_from_bearer_config() {
        let json = serde_json::json!({
            "auth_type": "bearer",
            "destination_url": "https://api.example.com/employees",
            "token": "sk-live-abc123"
        });

        let engine = ApiEngine::from_action_config(&json);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_engine_from_oauth2_config() {
        let json = serde_json::json!({
            "auth_type": "oauth2",
            "destination_url": "https://api.example.com/v2/employees",
            "client_id": "app-12345",
            "client_secret": "secret-value",
            "token_url": "https://auth.example.com/oauth/token",
            "scopes": ["employees.write"],
            "grant_type": "client_credentials"
        });

        let engine = ApiEngine::from_action_config(&json);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_default_retry_policy() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert!(policy.retryable_status_codes.contains(&429));
        assert!(policy.retryable_status_codes.contains(&503));
    }
}
