//! Configuration model for OAuth2 egress strategy (Client Credentials & Auth Code / OIDC).

use serde::{Deserialize, Serialize};

/// The OAuth2 grant type this repo instance should use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuth2GrantType {
    /// Machine-to-machine: `client_id` + `client_secret` → access token.
    ClientCredentials,
    /// Delegated identity: uses a stored `refresh_token` to rotate access tokens.
    AuthorizationCode,
}

impl Default for OAuth2GrantType {
    fn default() -> Self {
        Self::ClientCredentials
    }
}

/// Configuration for OAuth2-authenticated egress.
///
/// # JSON config (manifest)
///
/// ```json
/// {
///     "destination_url": "https://api.customer.com/v2/employees",
///     "auth_type": "oauth2",
///     "grant_type": "client_credentials",
///     "client_id": "app-12345",
///     "client_secret": "secret-value",
///     "token_url": "https://auth.customer.com/oauth/token",
///     "scopes": ["employees.write"]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2RepoConfig {
    /// Destination endpoint URL.
    pub destination_url: String,
    /// OAuth2 client identifier.
    pub client_id: String,
    /// OAuth2 client secret.
    pub client_secret: String,
    /// Token endpoint URL (the authorization server).
    pub token_url: String,
    /// Requested scopes.
    #[serde(default)]
    pub scopes: Vec<String>,
    /// Grant type variant.
    #[serde(default)]
    pub grant_type: OAuth2GrantType,
    /// Pre-obtained refresh token (required for `AuthorizationCode` grant).
    pub refresh_token: Option<String>,
}

impl OAuth2RepoConfig {
    /// Deserialise from the raw `serde_json::Value` stored in `ActionConfig.config`.
    pub fn from_json(value: &serde_json::Value) -> crate::domain::Result<Self> {
        serde_json::from_value(value.clone()).map_err(|e| {
            crate::domain::Error::ConfigurationError(format!(
                "Invalid OAuth2RepoConfig: {e}"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth2_client_credentials_config() {
        let json = serde_json::json!({
            "destination_url": "https://api.example.com/v2/employees",
            "client_id": "app-12345",
            "client_secret": "secret-value",
            "token_url": "https://auth.example.com/oauth/token",
            "scopes": ["employees.write"],
            "grant_type": "client_credentials"
        });

        let config = OAuth2RepoConfig::from_json(&json).unwrap();
        assert_eq!(config.client_id, "app-12345");
        assert_eq!(config.grant_type, OAuth2GrantType::ClientCredentials);
        assert!(config.refresh_token.is_none());
    }

    #[test]
    fn test_oauth2_auth_code_config() {
        let json = serde_json::json!({
            "destination_url": "https://api.example.com/v2/employees",
            "client_id": "app-12345",
            "client_secret": "secret-value",
            "token_url": "https://auth.example.com/oauth/token",
            "grant_type": "authorization_code",
            "refresh_token": "rt_initial_token"
        });

        let config = OAuth2RepoConfig::from_json(&json).unwrap();
        assert_eq!(config.grant_type, OAuth2GrantType::AuthorizationCode);
        assert_eq!(config.refresh_token.as_deref(), Some("rt_initial_token"));
    }

    #[test]
    fn test_oauth2_defaults_to_client_credentials() {
        let json = serde_json::json!({
            "destination_url": "https://api.example.com/v2/employees",
            "client_id": "app-12345",
            "client_secret": "secret-value",
            "token_url": "https://auth.example.com/oauth/token"
        });

        let config = OAuth2RepoConfig::from_json(&json).unwrap();
        assert_eq!(config.grant_type, OAuth2GrantType::ClientCredentials);
        assert!(config.scopes.is_empty());
    }

    #[test]
    fn test_oauth2_ignores_extra_fields() {
        let json = serde_json::json!({
            "destination_url": "https://api.example.com/v2/employees",
            "auth_type": "oauth2",
            "client_id": "app-12345",
            "client_secret": "secret-value",
            "token_url": "https://auth.example.com/oauth/token",
            "grant_type": "client_credentials"
        });

        let config = OAuth2RepoConfig::from_json(&json).unwrap();
        assert_eq!(config.client_id, "app-12345");
    }
}
