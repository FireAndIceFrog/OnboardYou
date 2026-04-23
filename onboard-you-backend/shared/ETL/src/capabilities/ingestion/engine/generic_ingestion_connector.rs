//! GenericIngestionConnector: universal file ingestion via Textract-converted CSV
//!
//! Reads a **pre-converted CSV** from S3 and populates a `RosterContext` with
//! the parsed Polars LazyFrame.  The connector is *not* responsible for the
//! file conversion — that is performed asynchronously by the `file-converter`
//! Lambda before the ETL pipeline runs.
//!
//! ## Accepted file types
//!
//! Any file type (PDF, XML, images, etc.) can be uploaded. Non-CSV files are
//! converted to CSV by the `file-converter` Lambda using AWS Textract table
//! extraction.  CSV files are used directly without Textract.
//!
//! ## S3 key resolution
//!
//! The original upload filename (e.g. `"employees.pdf"`) is stored in the
//! config.  The connector resolves the *converted* CSV key as:
//! `{org_id}/{company_id}/{stem}.csv` where `{stem}` is the filename without
//! its extension.  For a CSV upload the stem is unchanged.
//!
//! ## Column headers
//!
//! When `config.columns` is set, those names are used as the column headers
//! (overriding whatever is in the CSV).  When absent, the CSV header row is
//! used as-is.
//!
//! ## Field ownership
//!
//! Every ingested column is tagged with `"GENERIC_INGESTION"` field-ownership
//! metadata so downstream logic actions can trace data provenance.

use crate::capabilities::ingestion::traits::HrisConnector;
use onboard_you_models::ColumnCalculator;
use onboard_you_models::{Error, OnboardingAction, Result, RosterContext};
use onboard_you_models::GenericIngestionConnectorConfig;
use polars::prelude::*;

// ---------------------------------------------------------------------------
// Source tag
// ---------------------------------------------------------------------------

const FIELD_SOURCE: &str = "GENERIC_INGESTION";

// ---------------------------------------------------------------------------
// Connector
// ---------------------------------------------------------------------------

/// HRIS connector that ingests employee data from any uploaded file.
///
/// The file must have been converted to CSV by the `file-converter` Lambda
/// before this action runs.  If the converted CSV is not present in S3 this
/// step returns a clear configuration error.
#[derive(Debug, Clone)]
pub struct GenericIngestionConnector {
    config: GenericIngestionConnectorConfig,
}

impl GenericIngestionConnector {
    /// Create a new connector from a pre-validated config.
    pub fn new(config: GenericIngestionConnectorConfig) -> Self {
        Self { config }
    }

    /// Construct from a deserialised config, applying validation.
    pub fn from_action_config(config: &GenericIngestionConnectorConfig) -> Result<Self> {
        let filename = config.filename.trim();

        if filename.is_empty() {
            return Err(Error::ConfigurationError(
                "GenericIngestionConnector requires a non-empty filename".into(),
            ));
        }

        if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
            return Err(Error::ConfigurationError(
                "GenericIngestionConnector filename must not contain path separators or '..'".into(),
            ));
        }

        let mut cleaned = config.clone();
        cleaned.filename = filename.to_string();
        Ok(Self::new(cleaned))
    }

    /// Download the converted CSV from S3 and return the raw bytes.
    ///
    /// Uses `block_in_place` to bridge async AWS SDK into the synchronous
    /// `OnboardingAction::execute` interface.
    fn download_from_s3(&self) -> Result<Vec<u8>> {
        let s3_key = self.config.s3_key()?;

        let bucket = std::env::var("CSV_UPLOAD_BUCKET").map_err(|_| {
            Error::ConfigurationError("CSV_UPLOAD_BUCKET environment variable is not set".into())
        })?;

        let handle = tokio::runtime::Handle::current();
        tokio::task::block_in_place(|| {
            handle.block_on(async {
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
                            "GenericIngestionConnector: S3 GetObject failed for \
                             '{bucket}/{s3_key}': {e}\n\
                             Hint: ensure the file has been converted before running the pipeline."
                        ))
                    })?;

                let bytes = resp
                    .body
                    .collect()
                    .await
                    .map_err(|e| {
                        Error::IngestionError(format!(
                            "GenericIngestionConnector: failed to read S3 body: {e}"
                        ))
                    })?;

                Ok(bytes.into_bytes().to_vec())
            })
        })
    }

    /// Parse a CSV byte slice into a LazyFrame.
    ///
    /// Forces every column to `DataType::String` to prevent type inference —
    /// downstream sanitizers expect string inputs.
    ///
    /// When `override_columns` is provided those names are applied to the
    /// columns in order, replacing whatever headers the CSV contains.
    fn parse_csv(csv_bytes: &[u8], override_columns: Option<&[String]>) -> Result<LazyFrame> {
        // 1. Read header row to discover column count / names.
        let header_df = CsvReadOptions::default()
            .with_n_rows(Some(0))
            .into_reader_with_file_handle(std::io::Cursor::new(csv_bytes))
            .finish()
            .map_err(|e| Error::IngestionError(format!("Failed to read CSV header: {e}")))?;

        let detected_names: Vec<PlSmallStr> = header_df
            .get_column_names()
            .into_iter()
            .cloned()
            .collect();

        // 2. Validate user-supplied column count if provided.
        if let Some(cols) = override_columns {
            if cols.len() != detected_names.len() {
                return Err(Error::ConfigurationError(format!(
                    "GenericIngestionConnector: `columns` has {} entries but the CSV has {} \
                     columns. They must match exactly.",
                    cols.len(),
                    detected_names.len(),
                )));
            }
        }

        // 3. Build the all-String read schema based on effective column names.
        let effective_names: Vec<&str> = match override_columns {
            Some(cols) => cols.iter().map(String::as_str).collect(),
            None => detected_names.iter().map(|s| s.as_str()).collect(),
        };

        let all_string_schema: SchemaRef = Arc::new(
            effective_names
                .iter()
                .map(|&name| Field::new(name.into(), DataType::String))
                .collect::<Schema>(),
        );

        // 4. Re-read the full CSV with the resolved schema.
        //    When column names are overridden we skip the original header row
        //    and inject the new names via the schema.
        let opts = if override_columns.is_some() {
            CsvReadOptions::default()
                .with_has_header(true) // skip original header row
                .with_schema_overwrite(Some(all_string_schema.clone()))
                .with_schema(Some(all_string_schema))
        } else {
            CsvReadOptions::default().with_schema_overwrite(Some(all_string_schema))
        };

        let df = opts
            .into_reader_with_file_handle(std::io::Cursor::new(csv_bytes))
            .finish()
            .map_err(|e| Error::IngestionError(format!("Failed to parse CSV: {e}")))?;

        Ok(df.lazy())
    }
}

impl HrisConnector for GenericIngestionConnector {
    fn fetch_data(&self) -> Result<LazyFrame> {
        let csv_bytes = self.download_from_s3()?;
        Self::parse_csv(&csv_bytes, self.config.columns.as_deref())
    }
}

impl ColumnCalculator for GenericIngestionConnector {
    fn calculate_columns(&self, context: RosterContext) -> Result<RosterContext> {
        // Schema propagation — no S3 access.
        //
        // When the user declared columns up-front use them. Otherwise we have
        // no schema information at validate time, so we return an empty frame
        // (the pipeline engine will still run validation with whatever columns
        // downstream actions declare as inputs).
        let columns: Vec<Column> = match &self.config.columns {
            Some(cols) => cols
                .iter()
                .map(|name| Column::new(name.into(), Vec::<&str>::new()))
                .collect(),
            None => vec![],
        };

        let empty_df = DataFrame::new(0, columns)
            .map_err(|e| Error::IngestionError(format!("Failed to build schema: {e}")))?;

        let mut ctx = RosterContext::with_deps(empty_df.lazy(), context.deps.clone());

        if let Some(cols) = &self.config.columns {
            for col_name in cols {
                ctx.set_field_source(col_name.clone(), FIELD_SOURCE.into());
            }
        }

        Ok(ctx)
    }
}

impl OnboardingAction for GenericIngestionConnector {
    fn id(&self) -> &str {
        "generic_ingestion_connector"
    }

    fn execute(&self, context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            filename = %self.config.filename,
            s3_key = ?self.config.resolved_s3_key,
            table_index = self.config.effective_table_index(),
            "GenericIngestionConnector: ingesting converted CSV from S3"
        );

        let mut lf = self.fetch_data()?;

        let schema = lf
            .collect_schema()
            .map_err(|e| Error::IngestionError(format!("Failed to collect schema: {e}")))?;

        let mut ctx = RosterContext::with_deps(lf, context.deps.clone());

        for field_name in schema.iter_names() {
            ctx.set_field_source(field_name.to_string(), FIELD_SOURCE.into());
        }

        tracing::info!(
            fields = schema.len(),
            "GenericIngestionConnector: ingested {} fields",
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
    use onboard_you_models::ETLDependancies;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn make_config(filename: &str) -> GenericIngestionConnectorConfig {
        GenericIngestionConnectorConfig {
            filename: filename.into(),
            columns: None,
            table_index: None,
            resolved_s3_key: Some(format!("org-1/co-1/{}", filename)),
        }
    }

    fn make_config_with_cols(filename: &str, cols: Vec<&str>) -> GenericIngestionConnectorConfig {
        GenericIngestionConnectorConfig {
            filename: filename.into(),
            columns: Some(cols.into_iter().map(String::from).collect()),
            table_index: None,
            resolved_s3_key: Some(format!("org-1/co-1/{}", filename)),
        }
    }

    fn simple_csv() -> &'static [u8] {
        b"id,name,email\n1,Alice,alice@example.com\n2,Bob,bob@example.com\n"
    }

    fn make_deps() -> ETLDependancies {
        ETLDependancies::default()
    }

    // -----------------------------------------------------------------------
    // id
    // -----------------------------------------------------------------------

    #[test]
    fn connector_id_is_correct() {
        let connector = GenericIngestionConnector::new(make_config("employees.pdf"));
        assert_eq!(connector.id(), "generic_ingestion_connector");
    }

    // -----------------------------------------------------------------------
    // from_action_config validation
    // -----------------------------------------------------------------------

    #[test]
    fn empty_filename_is_rejected() {
        let cfg = GenericIngestionConnectorConfig {
            filename: "".into(),
            columns: None,
            table_index: None,
            resolved_s3_key: None,
        };
        assert!(GenericIngestionConnector::from_action_config(&cfg).is_err());
    }

    #[test]
    fn whitespace_filename_is_rejected() {
        let cfg = GenericIngestionConnectorConfig {
            filename: "   ".into(),
            columns: None,
            table_index: None,
            resolved_s3_key: None,
        };
        assert!(GenericIngestionConnector::from_action_config(&cfg).is_err());
    }

    #[test]
    fn path_traversal_filename_is_rejected() {
        for name in &["../evil.pdf", "/etc/passwd", "a\\b.pdf"] {
            let cfg = GenericIngestionConnectorConfig {
                filename: (*name).into(),
                columns: None,
                table_index: None,
                resolved_s3_key: None,
            };
            assert!(
                GenericIngestionConnector::from_action_config(&cfg).is_err(),
                "expected rejection for filename: {name}"
            );
        }
    }

    #[test]
    fn valid_filename_is_accepted() {
        let cfg = GenericIngestionConnectorConfig {
            filename: "employees.pdf".into(),
            columns: None,
            table_index: None,
            resolved_s3_key: None,
        };
        assert!(GenericIngestionConnector::from_action_config(&cfg).is_ok());
    }

    #[test]
    fn filename_is_trimmed_on_construction() {
        let cfg = GenericIngestionConnectorConfig {
            filename: "  employees.pdf  ".into(),
            columns: None,
            table_index: None,
            resolved_s3_key: None,
        };
        let connector = GenericIngestionConnector::from_action_config(&cfg).unwrap();
        assert_eq!(connector.config.filename, "employees.pdf");
    }

    // -----------------------------------------------------------------------
    // parse_csv — no column override
    // -----------------------------------------------------------------------

    #[test]
    fn parse_csv_reads_header_row() {
        let lf = GenericIngestionConnector::parse_csv(simple_csv(), None).unwrap();
        let df = lf.collect().unwrap();
        let col_names: Vec<&str> = df.get_column_names().into_iter().map(|s| s.as_str()).collect();
        assert_eq!(col_names, vec!["id", "name", "email"]);
    }

    #[test]
    fn parse_csv_all_columns_are_strings() {
        let lf = GenericIngestionConnector::parse_csv(simple_csv(), None).unwrap();
        let df = lf.collect().unwrap();
        for col in df.columns() {
            assert_eq!(
                col.dtype() as &DataType,
                &DataType::String,
                "column '{}' should be String",
                col.name()
            );
        }
    }

    #[test]
    fn parse_csv_correct_row_count() {
        let lf = GenericIngestionConnector::parse_csv(simple_csv(), None).unwrap();
        let df = lf.collect().unwrap();
        assert_eq!(df.height(), 2);
    }

    // -----------------------------------------------------------------------
    // parse_csv — with column override
    // -----------------------------------------------------------------------

    #[test]
    fn parse_csv_applies_override_columns() {
        let override_cols = vec!["emp_id".to_string(), "full_name".to_string(), "contact".to_string()];
        let lf = GenericIngestionConnector::parse_csv(simple_csv(), Some(&override_cols)).unwrap();
        let df = lf.collect().unwrap();
        let col_names: Vec<&str> = df.get_column_names().into_iter().map(|s| s.as_str()).collect();
        assert_eq!(col_names, vec!["emp_id", "full_name", "contact"]);
    }

    #[test]
    fn parse_csv_override_column_count_mismatch_errors() {
        // CSV has 3 columns but we supply 2 override names — must fail.
        let bad_override = vec!["a".to_string(), "b".to_string()];
        let result = GenericIngestionConnector::parse_csv(simple_csv(), Some(&bad_override));
        match result {
            Ok(_) => panic!("expected an error but got Ok"),
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("3") && err_msg.contains("2"),
                    "error message should mention column counts, got: {err_msg}"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // calculate_columns
    // -----------------------------------------------------------------------

    #[test]
    fn calculate_columns_with_declared_cols_returns_schema() {
        let cfg = make_config_with_cols("employees.pdf", vec!["id", "name", "email"]);
        let connector = GenericIngestionConnector::new(cfg);
        let initial = RosterContext::with_deps(LazyFrame::default(), make_deps());
        let ctx = connector.calculate_columns(initial).unwrap();

        let schema = ctx.get_data().collect_schema().unwrap();
        let names: Vec<&str> = schema.iter_names().map(|n| n.as_str()).collect();
        assert_eq!(names, vec!["id", "name", "email"]);
    }

    #[test]
    fn calculate_columns_stamps_field_source() {
        let cfg = make_config_with_cols("file.pdf", vec!["col_a", "col_b"]);
        let connector = GenericIngestionConnector::new(cfg);
        let initial = RosterContext::with_deps(LazyFrame::default(), make_deps());
        let ctx = connector.calculate_columns(initial).unwrap();

        assert_eq!(
            ctx.field_metadata().get("col_a").map(|m| m.source.as_str()),
            Some("GENERIC_INGESTION")
        );
        assert_eq!(
            ctx.field_metadata().get("col_b").map(|m| m.source.as_str()),
            Some("GENERIC_INGESTION")
        );
    }

    #[test]
    fn calculate_columns_without_declared_cols_returns_empty_schema() {
        let cfg = make_config("employees.pdf");
        let connector = GenericIngestionConnector::new(cfg);
        let initial = RosterContext::with_deps(LazyFrame::default(), make_deps());
        let ctx = connector.calculate_columns(initial).unwrap();

        let schema = ctx.get_data().collect_schema().unwrap();
        assert_eq!(schema.len(), 0);
    }

    // -----------------------------------------------------------------------
    // ActionConfigPayload deserialization roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn action_config_payload_deserializes_correctly() {
        use onboard_you_models::{ActionConfig, ActionConfigPayload, ActionType};

        let json = serde_json::json!({
            "id": "ingest",
            "action_type": "generic_ingestion_connector",
            "config": {
                "filename": "employees.pdf",
                "columns": ["id", "name"],
                "table_index": 1
            }
        });

        let action_config: ActionConfig = serde_json::from_value(json).unwrap();

        assert_eq!(action_config.action_type, ActionType::GenericIngestionConnector);

        match action_config.config {
            ActionConfigPayload::GenericIngestionConnector(cfg) => {
                assert_eq!(cfg.filename, "employees.pdf");
                assert_eq!(cfg.columns.as_deref(), Some(&["id".to_string(), "name".to_string()][..]));
                assert_eq!(cfg.table_index, Some(1));
            }
            other => panic!("Unexpected payload variant: {other:?}"),
        }
    }

    #[test]
    fn action_config_payload_deserializes_minimal_config() {
        use onboard_you_models::{ActionConfig, ActionConfigPayload, ActionType};

        let json = serde_json::json!({
            "id": "ingest",
            "action_type": "generic_ingestion_connector",
            "config": {
                "filename": "data.xml"
            }
        });

        let action_config: ActionConfig = serde_json::from_value(json).unwrap();
        assert_eq!(action_config.action_type, ActionType::GenericIngestionConnector);

        match action_config.config {
            ActionConfigPayload::GenericIngestionConnector(cfg) => {
                assert_eq!(cfg.filename, "data.xml");
                assert!(cfg.columns.is_none());
                assert!(cfg.table_index.is_none());
            }
            other => panic!("Unexpected payload variant: {other:?}"),
        }
    }
}
