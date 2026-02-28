//! Configuration model for OAuth 1.0a egress strategy.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::DynamicEgressModel;

/// Configuration for OAuth 1.0a signed requests.
///
/// # JSON config (manifest)
///
/// ```json
/// {
///     "destination_url": "https://legacy.customer.com/api/roster",
///     "auth_type": "oauth",
///     "consumer_key": "ck_abc123",
///     "consumer_secret": "cs_secret",
///     "access_token": "at_token",
///     "token_secret": "ts_secret"
/// }
/// ```
/// ```
#[macro_rules_attribute::apply(DynamicEgressModel!)]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OAuthRepoConfig {
    /// Destination endpoint URL.
    pub destination_url: String,
    /// OAuth 1.0a consumer key.
    pub consumer_key: String,
    /// OAuth 1.0a consumer secret.
    pub consumer_secret: String,
    /// OAuth 1.0a access token.
    pub access_token: String,
    /// OAuth 1.0a token secret.
    pub token_secret: String,
}

impl OAuthRepoConfig {
    /// Deserialise from the raw `serde_json::Value` stored in `ActionConfig.config`.
    pub fn from_json(value: &serde_json::Value) -> crate::Result<Self> {
        serde_json::from_value(value.clone()).map_err(|e| {
            crate::Error::ConfigurationError(format!("Invalid OAuthRepoConfig: {e}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_config_from_json() {
        let json = serde_json::json!({
            "destination_url": "https://legacy.example.com/api",
            "consumer_key": "ck_abc",
            "consumer_secret": "cs_secret",
            "access_token": "at_token",
            "token_secret": "ts_secret"
        });

        let config = OAuthRepoConfig::from_json(&json).unwrap();
        assert_eq!(config.destination_url, "https://legacy.example.com/api");
        assert_eq!(config.consumer_key, "ck_abc");
    }

    #[test]
    fn test_oauth_config_missing_field() {
        let json = serde_json::json!({
            "destination_url": "https://example.com"
        });

        let result = OAuthRepoConfig::from_json(&json);
        assert!(result.is_err());
    }

    #[test]
    fn test_oauth_config_ignores_extra_fields() {
        let json = serde_json::json!({
            "destination_url": "https://legacy.example.com/api",
            "auth_type": "oauth",
            "consumer_key": "ck_abc",
            "consumer_secret": "cs_secret",
            "access_token": "at_token",
            "token_secret": "ts_secret"
        });

        let config = OAuthRepoConfig::from_json(&json).unwrap();
        assert_eq!(config.consumer_key, "ck_abc");
    }
}
