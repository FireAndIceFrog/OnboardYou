//! Concrete, typed configuration for the API dispatcher action.
//!
//! Discriminated by `auth_type` in JSON — the same field that `AuthType`
//! already parses. The enum variants carry the per-strategy config:
//!
//! | `auth_type`                          | Variant   | Inner config       |
//! |--------------------------------------|-----------|--------------------|
//! | `"bearer"` / `"api_key"` / `"none"`  | `Bearer`  | `BearerRepoConfig` |
//! | `"oauth"` / `"oauth1"`               | `OAuth`   | `OAuthRepoConfig`  |
//! | `"oauth2"` / `"oidc"` / `"openid"`   | `OAuth2`  | `OAuth2RepoConfig` |
//! | `"default"`                          | `Default` | *(unit)*           |
//!
//! Custom `Serialize` / `Deserialize` impls flatten the inner config so
//! the JSON shape matches what `ApiEngine` already expects:
//!
//! ```json
//! {
//!   "auth_type": "bearer",
//!   "destination_url": "https://api.example.com/employees",
//!   "token": "sk-live-abc123"
//! }
//! ```

use super::{AuthType, BearerRepoConfig, OAuth2RepoConfig, OAuthRepoConfig};
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Fully-typed API dispatcher configuration.
///
/// The `Default` variant is a meta-type: the ETL trigger resolves it to
/// the organisation's stored settings **before** pipeline construction.
/// If it reaches `ApiEngine` unresolved, construction fails.
#[derive(Clone, Debug, ToSchema)]
pub enum ApiDispatcherConfig {
    /// No auth / static bearer token / custom API key.
    Bearer(BearerRepoConfig),
    /// OAuth 1.0a signed requests.
    OAuth(OAuthRepoConfig),
    /// OAuth2 client credentials or authorization code flow.
    OAuth2(OAuth2RepoConfig),
    /// Placeholder — resolved to a concrete type at runtime.
    Default,
}

impl ApiDispatcherConfig {
    /// Returns the `AuthType` discriminator for this config.
    pub fn auth_type(&self) -> AuthType {
        match self {
            Self::Bearer(_) => AuthType::Bearer,
            Self::OAuth(_) => AuthType::OAuth,
            Self::OAuth2(_) => AuthType::OAuth2,
            Self::Default => AuthType::Default,
        }
    }

    /// Returns `true` if this is the `Default` meta-type that still
    /// needs resolution.
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

// ---------------------------------------------------------------------------
// Custom Serialize — flattens inner config + injects `auth_type`
// ---------------------------------------------------------------------------

impl Serialize for ApiDispatcherConfig {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = match self {
            Self::Bearer(c) => serde_json::to_value(c),
            Self::OAuth(c) => serde_json::to_value(c),
            Self::OAuth2(c) => serde_json::to_value(c),
            Self::Default => Ok(serde_json::json!({})),
        }
        .map_err(serde::ser::Error::custom)?;

        let auth_str = match self {
            Self::Bearer(_) => "bearer",
            Self::OAuth(_) => "oauth",
            Self::OAuth2(_) => "oauth2",
            Self::Default => "default",
        };

        if let serde_json::Value::Object(ref mut obj) = map {
            obj.insert(
                "auth_type".to_string(),
                serde_json::Value::String(auth_str.to_string()),
            );
        }

        map.serialize(serializer)
    }
}

// ---------------------------------------------------------------------------
// Custom Deserialize — reads `auth_type`, then parses variant fields
// ---------------------------------------------------------------------------

impl<'de> Deserialize<'de> for ApiDispatcherConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        let auth_type: AuthType = value
            .get("auth_type")
            .map(|v| serde_json::from_value(v.clone()))
            .unwrap_or(Ok(AuthType::Bearer))
            .map_err(serde::de::Error::custom)?;

        match auth_type {
            AuthType::Bearer => {
                let config: BearerRepoConfig =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::Bearer(config))
            }
            AuthType::OAuth => {
                let config: OAuthRepoConfig =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::OAuth(config))
            }
            AuthType::OAuth2 => {
                let config: OAuth2RepoConfig =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::OAuth2(config))
            }
            AuthType::Default => Ok(Self::Default),
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearer_round_trip() {
        let json = serde_json::json!({
            "auth_type": "bearer",
            "destination_url": "https://api.example.com/employees",
            "token": "sk-live-abc123"
        });

        let config: ApiDispatcherConfig = serde_json::from_value(json).unwrap();
        assert!(matches!(config, ApiDispatcherConfig::Bearer(_)));
        assert_eq!(config.auth_type(), AuthType::Bearer);

        let re_serialized = serde_json::to_value(&config).unwrap();
        assert_eq!(re_serialized["auth_type"], "bearer");
        assert_eq!(
            re_serialized["destination_url"],
            "https://api.example.com/employees"
        );
    }

    #[test]
    fn test_oauth2_round_trip() {
        let json = serde_json::json!({
            "auth_type": "oauth2",
            "destination_url": "https://api.example.com/v2/employees",
            "client_id": "app-12345",
            "client_secret": "secret-value",
            "token_url": "https://auth.example.com/oauth/token",
            "scopes": ["employees.write"],
            "grant_type": "client_credentials"
        });

        let config: ApiDispatcherConfig = serde_json::from_value(json).unwrap();
        assert!(matches!(config, ApiDispatcherConfig::OAuth2(_)));
    }

    #[test]
    fn test_oauth1_alias() {
        let json = serde_json::json!({
            "auth_type": "oauth1",
            "destination_url": "https://legacy.example.com/api",
            "consumer_key": "ck_abc",
            "consumer_secret": "cs_secret",
            "access_token": "at_token",
            "token_secret": "ts_secret"
        });

        let config: ApiDispatcherConfig = serde_json::from_value(json).unwrap();
        assert!(matches!(config, ApiDispatcherConfig::OAuth(_)));
    }

    #[test]
    fn test_default_variant() {
        let json = serde_json::json!({ "auth_type": "default" });
        let config: ApiDispatcherConfig = serde_json::from_value(json).unwrap();
        assert!(config.is_default());
    }

    #[test]
    fn test_no_auth_type_defaults_to_bearer() {
        let json = serde_json::json!({
            "destination_url": "https://open.example.com/api"
        });
        let config: ApiDispatcherConfig = serde_json::from_value(json).unwrap();
        assert!(matches!(config, ApiDispatcherConfig::Bearer(_)));
    }
}
