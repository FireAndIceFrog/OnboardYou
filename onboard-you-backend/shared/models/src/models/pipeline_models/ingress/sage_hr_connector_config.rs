use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Configuration for the Sage HR REST API connector.
///
/// # JSON config example
///
/// ```json
/// {
///   "subdomain": "acme",
///   "api_token": "<your-sage-hr-api-token>",
///   "include_team_history": true,
///   "include_employment_status_history": true,
///   "include_position_history": true
/// }
/// ```
///
/// | Field                              | Type   | Required | Description                                         |
/// |------------------------------------|--------|----------|-----------------------------------------------------|
/// | `subdomain`                        | string | **yes**  | Sage HR subdomain (e.g. `acme` → `acme.sage.hr`)    |
/// | `api_token`                        | string | **yes**  | API token provided by the user                       |
/// | `include_team_history`             | bool   | no       | Include team history (default false)                 |
/// | `include_employment_status_history`| bool   | no       | Include employment status history (default false)    |
/// | `include_position_history`         | bool   | no       | Include position history (default false)             |
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SageHrConfig {
    pub subdomain: String,
    pub api_token: String,
    #[serde(default)]
    pub include_team_history: bool,
    #[serde(default)]
    pub include_employment_status_history: bool,
    #[serde(default)]
    pub include_position_history: bool,
}

impl SageHrConfig {
    /// Build from the raw `serde_json::Value` stored in `ActionConfig.config`.
    pub fn from_json(value: &serde_json::Value) -> crate::Result<Self> {
        serde_json::from_value(value.clone()).map_err(|e| {
            crate::Error::ConfigurationError(format!("SageHrConnector config error: {}", e))
        })
    }

    /// Return the API token as-is. The user provides the token directly
    /// via the pipeline configuration.
    pub fn api_token(&self) -> &str {
        &self.api_token
    }

    /// Build the employees API endpoint URL.
    ///
    /// Pattern: `https://{subdomain}.sage.hr/api/employees`
    pub fn employees_endpoint(&self) -> String {
        format!(
            "https://{}.sage.hr/api/employees",
            self.subdomain.trim_matches(|c: char| c == '.' || c.is_whitespace()),
        )
    }

    /// Build query parameters for the employees endpoint.
    pub fn query_params(&self, page: u32) -> Vec<(&'static str, String)> {
        let mut params = vec![("page".into(), page.to_string())];
        if self.include_team_history {
            params.push(("team_history", "true".into()));
        }
        if self.include_employment_status_history {
            params.push(("employment_status_history", "true".into()));
        }
        if self.include_position_history {
            params.push(("position_history", "true".into()));
        }
        params
    }
}
