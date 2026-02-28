use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ───────────────────────────────────────────────────────────────────────────
// Configuration
// ───────────────────────────────────────────────────────────────────────────

/// Workday API version targeted by this connector.
pub const WORKDAY_API_VERSION: &str = "v45.2";

/// Default Workday SOAP namespace for Human_Resources.
pub const WORKDAY_HR_NAMESPACE: &str = "urn:com.workday/bsvc/Human_Resources";

/// Configuration extracted from the manifest `ActionConfig.config` JSON.
///
/// # JSON config example
///
/// ```json
/// {
///   "tenant_url": "https://wd3-impl-services1.workday.com",
///   "tenant_id": "acme_corp",
///   "username": "ISU_Onboarding",
///   "password": "env:WORKDAY_PASSWORD",
///   "worker_count_limit": 200,
///   "response_group": {
///     "include_personal_information": true,
///     "include_employment_information": true,
///     "include_compensation": true,
///     "include_organizations": true,
///     "include_roles": false
///   }
/// }
/// ```
///
/// | Field                  | Type   | Required | Description                                         |
/// |------------------------|--------|----------|-----------------------------------------------------|
/// | `tenant_url`           | string | **yes**  | Workday tenant base URL                              |
/// | `tenant_id`            | string | **yes**  | Workday tenant identifier                            |
/// | `username`             | string | **yes**  | Integration System User (ISU) username               |
/// | `password`             | string | **yes**  | ISU password (prefix `env:` to read from env var)    |
/// | `worker_count_limit`   | u32    | no       | Max workers per page (default 200)                   |
/// | `response_group`       | object | no       | Sections to include in Get_Workers response          |
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WorkdayConfig {
    pub tenant_url: String,
    pub tenant_id: String,
    pub username: String,
    pub password: String,
    #[serde(default = "default_worker_count_limit")]
    pub worker_count_limit: u32,
    #[serde(default)]
    pub response_group: WorkdayResponseGroup,
}

fn default_worker_count_limit() -> u32 {
    200
}

/// Controls which data sections are included in the `Get_Workers` response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WorkdayResponseGroup {
    #[serde(default = "default_true")]
    pub include_personal_information: bool,
    #[serde(default = "default_true")]
    pub include_employment_information: bool,
    #[serde(default)]
    pub include_compensation: bool,
    #[serde(default)]
    pub include_organizations: bool,
    #[serde(default)]
    pub include_roles: bool,
}

fn default_true() -> bool {
    true
}

impl Default for WorkdayResponseGroup {
    fn default() -> Self {
        Self {
            include_personal_information: true,
            include_employment_information: true,
            include_compensation: false,
            include_organizations: false,
            include_roles: false,
        }
    }
}

impl WorkdayConfig {
    /// Build from the raw `serde_json::Value` stored in `ActionConfig.config`.
    pub fn from_json(value: &serde_json::Value) -> Result<Self> {
        serde_json::from_value(value.clone()).map_err(|e| {
            Error::ConfigurationError(format!("WorkdayHrisConnector config error: {}", e))
        })
    }

    /// Resolve the password — if it starts with `env:`, read from the
    /// environment variable named after the prefix.
    pub fn resolved_password(&self) -> Result<String> {
        if let Some(var_name) = self.password.strip_prefix("env:") {
            std::env::var(var_name).map_err(|_| {
                Error::ConfigurationError(format!(
                    "Environment variable '{}' not set for Workday password",
                    var_name
                ))
            })
        } else {
            Ok(self.password.clone())
        }
    }

    /// Build the full SOAP endpoint URL for the Human_Resources service.
    ///
    /// Pattern: `{tenant_url}/ccx/service/{tenant_id}/Human_Resources/{version}`
    pub fn soap_endpoint(&self) -> String {
        format!(
            "{}/ccx/service/{}/Human_Resources/{}",
            self.tenant_url.trim_end_matches('/'),
            self.tenant_id,
            WORKDAY_API_VERSION
        )
    }
}