//! Inbound request body for upserting organization settings.
//!
//! Omits `organizationId` which the controller fills from Claims.

use onboard_you_models::{ApiDispatcherConfig};
use serde::Deserialize;
use utoipa::ToSchema;

use onboard_you_models::OrgSettings;

/// Request body for `PUT /settings`.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SettingsRequest {
    /// Full auth configuration — typed to `ApiDispatcherConfig`.
    #[schema(value_type = Object)]
    pub default_auth: ApiDispatcherConfig,
}

impl SettingsRequest {
    /// Convert into a full `OrgSettings`, filling in server-side fields.
    pub fn into_settings(self) -> OrgSettings {
        OrgSettings {
            organization_id: String::new(),
            default_auth: self.default_auth
        }
    }
}
