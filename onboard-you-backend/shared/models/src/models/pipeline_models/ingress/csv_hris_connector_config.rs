use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{Error, Result};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration extracted from the manifest `ActionConfig.config` JSON.
///
/// # JSON config (user-facing)
///
/// ```json
/// {
///   "filename": "employees.csv",
///   "columns": ["employee_id", "first_name", "last_name", "email"]
/// }
/// ```
///
/// | Field      | Type     | Required | Description                                        |
/// |------------|----------|----------|----------------------------------------------------|
/// | `filename` | string   | **yes**  | CSV file name — the S3 key prefix is added at runtime |
/// | `columns`  | [string] | **yes**  | Declared column names from the CSV header           |
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct CsvHrisConnectorConfig {
    /// CSV file name only (e.g. `"employees.csv"`).
    ///
    /// The full S3 key `{org_id}/{company_id}/{filename}` is resolved at
    /// runtime by the ETL trigger before pipeline execution.
    pub filename: String,

    /// Declared column names — set when the CSV is first uploaded.
    ///
    /// Used by `calculate_columns` for schema propagation and by the
    /// validation engine to verify downstream column references.
    pub columns: Vec<String>,

    /// Resolved S3 object key — injected at runtime by the pipeline engine.
    ///
    /// Not part of the user-facing config. Skipped when `None` during
    /// serialisation so it never leaks into stored configs.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[schema(ignore)]
    pub resolved_s3_key: Option<String>,
}

impl CsvHrisConnectorConfig {
    /// Build from the raw `serde_json::Value` stored in `ActionConfig.config`.
    pub fn from_json(value: &serde_json::Value) -> Result<Self> {
        serde_json::from_value(value.clone()).map_err(|e| {
            Error::ConfigurationError(format!("CsvHrisConnector config parse error: {e}"))
        })
    }

    /// Resolve the full S3 key from runtime context.
    ///
    /// Called by the ETL trigger's manifest pre-processor before the factory
    /// builds the action.
    pub fn resolve_s3_key(&mut self, organization_id: &str, customer_company_id: &str) {
        self.resolved_s3_key = Some(format!(
            "{}/{}/{}",
            organization_id, customer_company_id, self.filename
        ));
    }

    /// Return the resolved S3 key, or error if unresolved.
    pub fn s3_key(&self) -> Result<&str> {
        self.resolved_s3_key.as_deref().ok_or_else(|| {
            Error::ConfigurationError(
                "CsvHrisConnector: s3_key not resolved — pipeline engine must call \
                 resolve_s3_key() before execution"
                    .into(),
            )
        })
    }
}