//! CsvHrisConnector: CSV-based HRIS data ingestion
//!
//! Reads a CSV file path from the declarative manifest config and populates a
//! `RosterContext` with the parsed Polars LazyFrame. Every ingested column is
//! tagged with `HRIS_CONNECTOR` field-ownership metadata so downstream logic
//! actions can trace data provenance.

use crate::capabilities::ingestion::traits::HrisConnector;
use crate::domain::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration extracted from the manifest `ActionConfig.config` JSON.
///
/// Expected shape:
/// ```json
/// { "csv_path": "/data/employees.csv" }
/// ```
#[derive(Debug, Clone)]
pub struct CsvHrisConnectorConfig {
    pub csv_path: PathBuf,
}

impl CsvHrisConnectorConfig {
    /// Build from the raw `serde_json::Value` stored in `ActionConfig.config`.
    pub fn from_json(value: &serde_json::Value) -> Result<Self> {
        let csv_path = value
            .get("csv_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::ConfigurationError(
                    "CsvHrisConnector requires a 'csv_path' string in config".into(),
                )
            })?;

        Ok(Self {
            csv_path: PathBuf::from(csv_path),
        })
    }
}

// ---------------------------------------------------------------------------
// Connector
// ---------------------------------------------------------------------------

/// HRIS connector that ingests employee data from a local CSV file.
///
/// This is the primary *ingestion* `OnboardingAction`.  It:
///
/// 1. Reads the CSV into a Polars `LazyFrame`.
/// 2. Stamps every column with `HRIS_CONNECTOR` field-ownership metadata on the
///    `RosterContext`.
/// 3. Returns the enriched context for downstream pipeline steps.
#[derive(Debug, Clone)]
pub struct CsvHrisConnector {
    config: CsvHrisConnectorConfig,
}

impl CsvHrisConnector {
    /// Create a new connector from a pre-validated config.
    pub fn new(config: CsvHrisConnectorConfig) -> Self {
        Self { config }
    }

    /// Convenience constructor straight from manifest JSON.
    pub fn from_action_config(value: &serde_json::Value) -> Result<Self> {
        let config = CsvHrisConnectorConfig::from_json(value)?;
        Ok(Self::new(config))
    }
}

impl HrisConnector for CsvHrisConnector {
    fn fetch_data(&self) -> Result<LazyFrame> {
        let lf = LazyCsvReader::new(&self.config.csv_path)
            .with_has_header(true)
            .with_try_parse_dates(true)
            .finish()
            .map_err(|e| {
                Error::IngestionError(format!(
                    "Failed to read CSV '{}': {}",
                    self.config.csv_path.display(),
                    e
                ))
            })?;
        Ok(lf)
    }
}

impl OnboardingAction for CsvHrisConnector {
    fn id(&self) -> &str {
        "csv_hris_connector"
    }

    fn execute(&self, _context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            csv_path = %self.config.csv_path.display(),
            "CsvHrisConnector: ingesting CSV"
        );

        // 1. Parse CSV → LazyFrame
        let mut lf = self.fetch_data()?;

        // 2. Discover column names so we can stamp field-ownership metadata.
        //    We use the schema of the LazyFrame (cheap — no full collect).
        let schema = lf.collect_schema().map_err(|e| {
            Error::IngestionError(format!("Failed to collect schema: {}", e))
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
    use std::io::Write;

    /// Helper: write CSV content to a temporary file, return its path.
    fn write_temp_csv(content: &str) -> (tempfile::NamedTempFile, PathBuf) {
        let mut tmp = tempfile::NamedTempFile::new().expect("create temp file");
        tmp.write_all(content.as_bytes())
            .expect("write csv content");
        let path = tmp.path().to_path_buf();
        (tmp, path)
    }

    const TEST_CSV: &str = "\
employee_id,first_name,last_name,email,ssn,salary,start_date
001,John,Doe,john.doe@example.com,123-45-6789,75000,2024-01-01
002,Jane,Smith,jane.smith@example.com,987-65-4321,85000,2024-02-15
003,Alice,Johnson,alice.j@example.com,555-12-3456,92000,2024-03-10
";

    #[test]
    fn test_csv_connector_id() {
        let connector = CsvHrisConnector::new(CsvHrisConnectorConfig {
            csv_path: PathBuf::from("/dev/null"),
        });
        assert_eq!(connector.id(), "csv_hris_connector");
    }

    #[test]
    fn test_config_from_json_valid() {
        let json = serde_json::json!({ "csv_path": "/tmp/test.csv" });
        let config = CsvHrisConnectorConfig::from_json(&json).unwrap();
        assert_eq!(config.csv_path, PathBuf::from("/tmp/test.csv"));
    }

    #[test]
    fn test_config_from_json_missing_path() {
        let json = serde_json::json!({});
        let result = CsvHrisConnectorConfig::from_json(&json);
        assert!(result.is_err());
    }

    #[test]
    fn test_csv_ingestion_populates_lazyframe() {
        let (_tmp, path) = write_temp_csv(TEST_CSV);

        let connector = CsvHrisConnector::new(CsvHrisConnectorConfig {
            csv_path: path,
        });

        // Use an empty initial context (ingestion replaces it)
        let initial = RosterContext::new(LazyFrame::default());
        let ctx = connector.execute(initial).expect("execute should succeed");

        // Collect and verify row count
        let df = ctx.data.collect().expect("collect");
        assert_eq!(df.height(), 3, "should have 3 rows");
        assert_eq!(df.width(), 7, "should have 7 columns");

        // Verify field-ownership metadata was set for every column
        assert_eq!(ctx.field_metadata.len(), 7);
        for (_field, meta) in &ctx.field_metadata {
            assert_eq!(meta.source, "HRIS_CONNECTOR");
            assert!(meta.modified_by.is_none());
        }
    }

    #[test]
    fn test_csv_ingestion_column_names() {
        let (_tmp, path) = write_temp_csv(TEST_CSV);

        let connector = CsvHrisConnector::new(CsvHrisConnectorConfig {
            csv_path: path,
        });

        let initial = RosterContext::new(LazyFrame::default());
        let ctx = connector.execute(initial).expect("execute should succeed");

        let expected_columns = [
            "employee_id",
            "first_name",
            "last_name",
            "email",
            "ssn",
            "salary",
            "start_date",
        ];

        for col in &expected_columns {
            assert!(
                ctx.field_metadata.contains_key(*col),
                "metadata should contain field '{}'",
                col
            );
        }
    }

    #[test]
    fn test_csv_ingestion_data_values() {
        let (_tmp, path) = write_temp_csv(TEST_CSV);

        let connector = CsvHrisConnector::new(CsvHrisConnectorConfig {
            csv_path: path,
        });

        let initial = RosterContext::new(LazyFrame::default());
        let ctx = connector.execute(initial).expect("execute should succeed");
        let df = ctx.data.collect().expect("collect");

        // Verify first row data
        let first_names = df.column("first_name").expect("first_name column");
        let first_name = first_names.str().expect("as str").get(0).unwrap();
        assert_eq!(first_name, "John");

        let emails = df.column("email").expect("email column");
        let email = emails.str().expect("as str").get(1).unwrap();
        assert_eq!(email, "jane.smith@example.com");
    }

    #[test]
    fn test_csv_ingestion_file_not_found() {
        let connector = CsvHrisConnector::new(CsvHrisConnectorConfig {
            csv_path: PathBuf::from("/nonexistent/path/does_not_exist.csv"),
        });

        let initial = RosterContext::new(LazyFrame::default());
        let result = connector.execute(initial);
        assert!(result.is_err(), "should error on missing file");
    }

    #[test]
    fn test_from_action_config() {
        let json = serde_json::json!({ "csv_path": "/tmp/data.csv" });
        let connector = CsvHrisConnector::from_action_config(&json).unwrap();
        assert_eq!(connector.config.csv_path, PathBuf::from("/tmp/data.csv"));
        assert_eq!(connector.id(), "csv_hris_connector");
    }
}
