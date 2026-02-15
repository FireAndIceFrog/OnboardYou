//! Authentication type discriminator (serde-powered)

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// The authentication type configured for this egress destination.
///
/// Determined from `"auth_type"` in the manifest `ActionConfig.config` JSON.
///
/// Uses `#[serde(rename_all = "snake_case")]` so the JSON value is matched
/// case-insensitively and with underscores, plus `#[serde(alias)]` to accept
/// common synonyms:
///
/// | JSON value                              | Resolves to         |
/// |-----------------------------------------|---------------------|
/// | `"bearer"`, `"api_key"`, `"none"`       | `AuthType::Bearer`  |
/// | `"oauth"`, `"oauth1"`                   | `AuthType::OAuth`   |
/// | `"oauth2"`, `"oidc"`, `"openid"`        | `AuthType::OAuth2`  |
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    /// No auth / static bearer token / custom API key.
    #[serde(alias = "api_key", alias = "none")]
    Bearer,
    /// OAuth 1.0a signed requests.
    #[serde(rename = "oauth", alias = "oauth1")]
    OAuth,
    /// OAuth2 client credentials or authorization code flow.
    #[serde(rename = "oauth2", alias = "oidc", alias = "openid")]
    OAuth2,
}
