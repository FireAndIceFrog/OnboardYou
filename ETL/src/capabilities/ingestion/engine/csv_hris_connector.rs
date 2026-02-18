//! CsvHrisConnector: S3-based CSV HRIS data ingestion
//!
//! Reads a CSV file from S3 (keyed by org / company) and populates a
//! `RosterContext` with the parsed Polars LazyFrame. Every ingested column is
//! tagged with `HRIS_CONNECTOR` field-ownership metadata so downstream logic
//! actions can trace data provenance.
//!
//! The config declares the **expected column names** up-front (set when the
//! CSV is first uploaded).  `calculate_columns` uses these declarations to
//! propagate the schema without reading any data, while `execute` reads the
//! actual CSV from S3 at runtime.
//!
//! ## S3 key resolution
//!
//! The user-facing config stores only `filename` (e.g. `"employees.csv"`).
//! At runtime the ETL trigger resolves the full S3 key as
//! `{organization_id}/{customer_company_id}/{filename}` and injects it via
//! [`CsvHrisConnectorConfig::resolve_s3_key`].  The bucket name is read from
//! the `CSV_UPLOAD_BUCKET` environment variable.

use crate::capabilities::ingestion::traits::HrisConnector;
use crate::capabilities::logic::traits::ColumnCalculator;
use crate::domain::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
            Error::ConfigurationError(format!(
                "CsvHrisConnector config parse error: {e}"
            ))
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
    fn s3_key(&self) -> Result<&str> {
        self.resolved_s3_key.as_deref().ok_or_else(|| {
            Error::ConfigurationError(
                "CsvHrisConnector: s3_key not resolved — pipeline engine must call \
                 resolve_s3_key() before execution"
                    .into(),
            )
        })
    }
}

// ---------------------------------------------------------------------------
// Connector
// ---------------------------------------------------------------------------

/// HRIS connector that ingests employee data from a CSV file stored in S3.
///
/// This is the primary *ingestion* `OnboardingAction`.  It:
///
/// 1. Downloads the CSV from S3 into an in-memory buffer.
/// 2. Reads the buffer into a Polars `LazyFrame`.
/// 3. Stamps every column with `HRIS_CONNECTOR` field-ownership metadata on the
///    `RosterContext`.
/// 4. Returns the enriched context for downstream pipeline steps.
#[derive(Debug, Clone)]
pub struct CsvHrisConnector {
    config: CsvHrisConnectorConfig,
}

impl CsvHrisConnector {
    /// Create a new connector from a pre-validated config.
    pub fn new(config: CsvHrisConnectorConfig) -> Self {
        Self { config }
    }

    /// Construct from a deserialised config.
    pub fn from_action_config(config: &CsvHrisConnectorConfig) -> Result<Self> {
        if config.columns.is_empty() {
            return Err(Error::ConfigurationError(
                "CsvHrisConnector requires at least one declared column".into(),
            ));
        }
        if config.filename.is_empty() {
            return Err(Error::ConfigurationError(
                "CsvHrisConnector requires a non-empty filename".into(),
            ));
        }
        Ok(Self::new(config.clone()))
    }

    /// Download the CSV from S3 and return the bytes.
    ///
    /// Uses a one-shot `tokio::Runtime` to bridge the async AWS SDK into
    /// the synchronous `OnboardingAction::execute` interface.
    ///
    /// The bucket name is read from the `CSV_UPLOAD_BUCKET` env var.
    fn download_from_s3(&self) -> Result<Vec<u8>> {
        let s3_key = self.config.s3_key()?;

        let bucket = std::env::var("CSV_UPLOAD_BUCKET").map_err(|_| {
            Error::ConfigurationError(
                "CSV_UPLOAD_BUCKET environment variable is not set".into(),
            )
        })?;

        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            Error::IngestionError(format!("Failed to create tokio runtime: {e}"))
        })?;

        rt.block_on(async {
            let aws_config =
                aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
            let client = aws_sdk_s3::Client::new(&aws_config);

            let resp = client
                .get_object()
                .bucket(&bucket)
                .key(s3_key)
                .send()
                .await
                .map_err(|e| {
                    Error::IngestionError(format!(
                        "S3 GetObject failed for '{}/{}': {e}",
                        bucket, s3_key
                    ))
                })?;

            let bytes = resp.body.collect().await.map_err(|e| {
                Error::IngestionError(format!("Failed to read S3 body: {e}"))
            })?;

            Ok(bytes.into_bytes().to_vec())
        })
    }
}

impl HrisConnector for CsvHrisConnector {
    fn fetch_data(&self) -> Result<LazyFrame> {
        let csv_bytes = self.download_from_s3()?;

        let cursor = std::io::Cursor::new(csv_bytes);
        let df = CsvReader::new(cursor)
            .finish()
            .map_err(|e| {
                Error::IngestionError(format!(
                    "Failed to parse CSV '{}': {e}",
                    self.config.filename
                ))
            })?;

        Ok(df.lazy())
    }
}

impl ColumnCalculator for CsvHrisConnector {
    fn calculate_columns(&self, _context: RosterContext) -> Result<RosterContext> {
        // Build an empty DataFrame from the declared column names.
        // No S3 access — purely config-driven schema propagation.
        let columns: Vec<Column> = self
            .config
            .columns
            .iter()
            .map(|name| Column::new(name.into(), Vec::<&str>::new()))
            .collect();

        let empty_df = DataFrame::new(0, columns).map_err(|e| {
            Error::IngestionError(format!("Failed to build CSV schema: {e}"))
        })?;

        let mut ctx = RosterContext::new(empty_df.lazy());
        for col_name in &self.config.columns {
            ctx.set_field_source(col_name.clone(), "HRIS_CONNECTOR".into());
        }

        Ok(ctx)
    }
}

impl OnboardingAction for CsvHrisConnector {
    fn id(&self) -> &str {
        "csv_hris_connector"
    }

    fn execute(&self, _context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            filename = %self.config.filename,
            s3_key = ?self.config.resolved_s3_key,
            declared_columns = self.config.columns.len(),
            "CsvHrisConnector: ingesting CSV from S3"
        );

        // 1. Download + Parse CSV → LazyFrame
        let mut lf = self.fetch_data()?;

        // 2. Discover actual column names from the data
        let schema = lf.collect_schema().map_err(|e| {
            Error::IngestionError(format!("Failed to collect schema: {e}"))
        })?;

        // 3. Build the RosterContext
        let mut ctx = RosterContext::new(lf);

        for field_name in schema.iter_names() {
            ctx.set_field_source(field_name.to_string(), "HRIS_CONNECTOR".into());
        }

        tracing::info!(
            fields = schema.len(),
            "CsvHrisConnector: ingested {} fields",
            schema.len()
        );

        Ok(ctx)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> CsvHrisConnectorConfig {
        CsvHrisConnectorConfig {
            filename: "data.csv".into(),
            columns: vec![
                "employee_id".into(),
                "first_name".into(),
                "last_name".into(),
                "email".into(),
                "ssn".into(),
                "salary".into(),
                "start_date".into(),
            ],
            resolved_s3_key: None,
        }
    }

    #[test]
    fn test_csv_connector_id() {
        let connector = CsvHrisConnector::new(test_config());
        assert_eq!(connector.id(), "csv_hris_connector");
    }

    #[test]
    fn test_config_from_json() {
        let json = serde_json::json!({
            "filename": "data.csv",
            "columns": ["a", "b", "c"]
        });
        let config = CsvHrisConnectorConfig::from_json(&json).unwrap();
        assert_eq!(config.filename, "data.csv");
        assert_eq!(config.columns, vec!["a", "b", "c"]);
        assert!(config.resolved_s3_key.is_none());
    }

    #[test]
    fn test_config_from_json_missing_fields() {
        let json = serde_json::json!({});
        let result = CsvHrisConnectorConfig::from_json(&json);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_s3_key() {
        let mut config = test_config();
        config.resolve_s3_key("org-001", "acme-corp");
        assert_eq!(
            config.resolved_s3_key.as_deref(),
            Some("org-001/acme-corp/data.csv")
        );
    }

    #[test]
    fn test_s3_key_unresolved_returns_error() {
        let config = test_config();
        let connector = CsvHrisConnector::new(config);
        // s3_key() should fail because resolved_s3_key is None
        assert!(connector.config.s3_key().is_err());
    }

    #[test]
    fn test_from_action_config_empty_columns_rejected() {
        let config = CsvHrisConnectorConfig {
            filename: "data.csv".into(),
            columns: vec![],
            resolved_s3_key: None,
        };
        let result = CsvHrisConnector::from_action_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_action_config_empty_filename_rejected() {
        let config = CsvHrisConnectorConfig {
            filename: "".into(),
            columns: vec!["a".into()],
            resolved_s3_key: None,
        };
        let result = CsvHrisConnector::from_action_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolved_s3_key_not_serialised_when_none() {
        let config = test_config();
        let json = serde_json::to_value(&config).unwrap();
        assert!(!json.as_object().unwrap().contains_key("resolved_s3_key"));
    }

    #[test]
    fn test_calculate_columns_uses_declared_columns() {
        let connector = CsvHrisConnector::new(test_config());
        let initial = RosterContext::new(LazyFrame::default());
        let mut ctx = connector
            .calculate_columns(initial)
            .expect("calculate_columns should succeed");

        let schema = ctx.data.collect_schema().expect("schema");
        let col_names: Vec<String> = schema.iter_names().map(|n| n.to_string()).collect();
        assert_eq!(
            col_names,
            vec![
                "employee_id",
                "first_name",
                "last_name",
                "email",
                "ssn",
                "salary",
                "start_date"
            ]
        );

        // Every column should have HRIS_CONNECTOR metadata
        assert_eq!(ctx.field_metadata.len(), 7);
        for (_, meta) in &ctx.field_metadata {
            assert_eq!(meta.source, "HRIS_CONNECTOR");
        }
    }
}
