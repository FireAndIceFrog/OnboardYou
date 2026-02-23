//! Organization-level settings for login / authentication defaults.

use crate::ApiDispatcherConfig;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Per-organization settings stored in DynamoDB.
///
/// `default_auth` holds the full auth configuration that maps to a
/// concrete `ApiDispatcherConfig` variant (Bearer, OAuth, OAuth2).
///
/// **Bearer example:**
/// ```json
/// {
///   "organizationId": "acme-corp",
///   "defaultAuth": {
///     "auth_type": "bearer",
///     "destination_url": "https://api.example.com/employees",
///     "token": "sk-live-abc123"
///   }
/// }
/// ```
///
/// **OAuth2 example:**
/// ```json
/// {
///   "organizationId": "acme-corp",
///   "defaultAuth": {
///     "auth_type": "oauth2",
///     "destination_url": "https://api.example.com/v2/employees",
///     "client_id": "app-12345",
///     "client_secret": "secret-value",
///     "token_url": "https://auth.example.com/oauth/token",
///     "scopes": ["employees.write"],
///     "grant_type": "client_credentials"
///   }
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrgSettings {
    /// Unique identifier for the organization (partition key)
    pub organization_id: String,

    /// Full auth configuration — typed to `ApiDispatcherConfig`.
    ///
    /// Must contain `"auth_type"` plus all fields required by the
    /// chosen strategy (bearer, oauth, oauth2).
    #[schema(value_type = Object)]
    pub default_auth: ApiDispatcherConfig,
}
