//! Configuration model for the bearer / API-key egress strategy.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// How the credential is attached to outbound requests.
///
/// This is a pure discriminator — the actual header name or query-param key
/// lives in [`BearerRepoConfig::placement_key`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BearerPlacement {
    /// Standard `Authorization: Bearer <token>` header.
    AuthorizationHeader,
    /// Custom header whose name comes from `placement_key`.
    CustomHeader,
    /// Query parameter whose name comes from `placement_key`.
    QueryParam,
}

impl Default for BearerPlacement {
    fn default() -> Self {
        Self::AuthorizationHeader
    }
}

/// Configuration for the bearer / API-key strategy.
///
/// # JSON config (manifest)
///
/// ```json
/// {
///     "destination_url": "https://api.customer.com/employees",
///     "auth_type": "bearer",
///     "token": "sk-live-abc123",
///     "placement": "authorization_header"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BearerRepoConfig {
    /// Destination endpoint URL.
    pub destination_url: String,
    /// The static token / API key. `None` means no authentication.
    pub token: Option<String>,
    /// Where to place the token on the request.
    #[serde(default)]
    pub placement: BearerPlacement,
    /// Header name or query-param key (used with `CustomHeader` / `QueryParam`).
    /// Defaults to `"X-API-Key"` for custom headers, `"api_key"` for query params.
    pub placement_key: Option<String>,
    /// Extra static headers to attach (e.g. `Content-Type`).
    #[serde(default)]
    pub extra_headers: HashMap<String, String>,
}

impl BearerRepoConfig {
    /// Deserialise from the raw `serde_json::Value` stored in `ActionConfig.config`.
    pub fn from_json(value: &serde_json::Value) -> crate::domain::Result<Self> {
        serde_json::from_value(value.clone()).map_err(|e| {
            crate::domain::Error::ConfigurationError(format!(
                "Invalid BearerRepoConfig: {e}"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearer_config_from_json() {
        let json = serde_json::json!({
            "destination_url": "https://api.example.com/employees",
            "token": "sk-live-abc123",
            "placement": "authorization_header"
        });

        let config = BearerRepoConfig::from_json(&json).unwrap();
        assert_eq!(config.destination_url, "https://api.example.com/employees");
        assert_eq!(config.token.as_deref(), Some("sk-live-abc123"));
        assert_eq!(config.placement, BearerPlacement::AuthorizationHeader);
    }

    #[test]
    fn test_bearer_config_no_token() {
        let json = serde_json::json!({
            "destination_url": "https://api.example.com/open"
        });

        let config = BearerRepoConfig::from_json(&json).unwrap();
        assert!(config.token.is_none());
        assert_eq!(config.placement, BearerPlacement::AuthorizationHeader);
    }

    #[test]
    fn test_bearer_config_custom_header() {
        let json = serde_json::json!({
            "destination_url": "https://api.example.com/employees",
            "token": "key-abc",
            "placement": "custom_header",
            "placement_key": "X-API-Key"
        });

        let config = BearerRepoConfig::from_json(&json).unwrap();
        assert_eq!(config.placement, BearerPlacement::CustomHeader);
        assert_eq!(config.placement_key.as_deref(), Some("X-API-Key"));
    }

    #[test]
    fn test_bearer_config_query_param() {
        let json = serde_json::json!({
            "destination_url": "https://api.example.com/employees",
            "token": "key-abc",
            "placement": "query_param",
            "placement_key": "api_key"
        });

        let config = BearerRepoConfig::from_json(&json).unwrap();
        assert_eq!(config.placement, BearerPlacement::QueryParam);
        assert_eq!(config.placement_key.as_deref(), Some("api_key"));
    }

    #[test]
    fn test_bearer_config_extra_headers() {
        let json = serde_json::json!({
            "destination_url": "https://api.example.com/employees",
            "extra_headers": { "X-Custom": "value" }
        });

        let config = BearerRepoConfig::from_json(&json).unwrap();
        assert_eq!(config.extra_headers.get("X-Custom").unwrap(), "value");
    }
}
