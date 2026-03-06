use onboard_you_models::Manifest;
use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

/* ── Shared-model request type (mirrors the API's ConfigRequest) ── */

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // fields validated via deserialization, not read directly
pub struct ConfigRequest {
    pub name: String,
    pub image: Option<String>,
    pub cron: String,
    pub pipeline: Manifest,
}

/* ── Tool argument schemas ────────────────────────────────── */

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateConfigArgs {
    /// Customer company identifier (e.g. "acme-corp")
    pub customer_company_id: String,
    /// Pipeline configuration JSON — must include `name`, `cron`, and `pipeline` fields
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateConfigArgs {
    /// Customer company identifier
    pub customer_company_id: String,
    /// Pipeline configuration JSON to validate (only the `pipeline` field is used)
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SaveConfigArgs {
    /// Customer company identifier
    pub customer_company_id: String,
    /// Updated pipeline configuration JSON — must include `name`, `cron`, and `pipeline` fields
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FetchConfigArgs {
    /// Customer company identifier returned by `list_configs` (e.g. "acme-corp")
    pub customer_company_id: String,
}
