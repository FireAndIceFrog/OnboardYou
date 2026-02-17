//! Inbound request body for upserting organization settings.
//!
//! Omits `organizationId` which the controller fills from Claims.

use serde::Deserialize;
use utoipa::ToSchema;

use super::OrgSettings;

/// Request body for `PUT /settings`.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SettingsRequest {
    /// Full auth configuration blob.
    pub default_auth: serde_json::Value,
}

impl SettingsRequest {
    /// Convert into a full `OrgSettings`, filling in server-side fields.
    pub fn into_settings(self) -> OrgSettings {
        OrgSettings {
            organization_id: String::new(),
            default_auth: self.default_auth,
        }
    }
}
