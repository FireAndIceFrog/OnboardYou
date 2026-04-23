use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{Error, Result};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the `GenericIngestionConnector` pipeline action.
///
/// Accepts **any file type** (PDF, XML, CSV, images, etc.).  Non-CSV files are
/// converted to CSV asynchronously via AWS Textract before the ETL pipeline
/// runs.  The connector reads the pre-converted CSV from S3 using the same
/// `CSV_UPLOAD_BUCKET` as `CsvHrisConnector`.
///
/// # JSON config (user-facing)
///
/// ```json
/// {
///   "filename": "employees.pdf",
///   "columns": ["employee_id", "first_name", "last_name"],
///   "table_index": 0
/// }
/// ```
///
/// | Field         | Type       | Required | Description                                                       |
/// |---------------|------------|----------|-------------------------------------------------------------------|
/// | `filename`    | string     | **yes**  | Original upload filename — any extension supported               |
/// | `columns`     | [string]   | no       | Override column headers. If absent, the CSV header row is used    |
/// | `table_index` | number     | no       | Which Textract table to use (0-based, default 0). Only applies    |
/// |               |            |          | when a multi-table document is converted via Textract             |
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct GenericIngestionConnectorConfig {
    /// Original upload filename (e.g. `"employees.pdf"`, `"roster.xml"`).
    ///
    /// The connector resolves the converted CSV path by stripping the file
    /// extension and appending `.csv`:
    /// `{org_id}/{company_id}/{stem}.csv` on `CSV_UPLOAD_BUCKET`.
    pub filename: String,

    /// User-defined column headers.
    ///
    /// When provided, these override the header row in the converted CSV.
    /// Must exactly match the number of columns produced by Textract (or the
    /// CSV header count for native CSV uploads).  When `None`, column names
    /// are taken directly from the first row of the CSV.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub columns: Option<Vec<String>>,

    /// Zero-based index of the Textract table to extract.
    ///
    /// Relevant only for multi-table documents (PDFs, etc.).  Defaults to `0`
    /// (the first table found by Textract).  Ignored when the upload is
    /// already a CSV — no Textract call is made for CSV files.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub table_index: Option<usize>,

    /// Resolved S3 object key for the *converted* CSV — injected at runtime.
    ///
    /// Set by the pipeline engine after the file has been converted.  Not
    /// part of the user-facing config; omitted from serialised manifests.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[schema(ignore)]
    pub resolved_s3_key: Option<String>,
}

impl GenericIngestionConnectorConfig {
    /// Build from the raw `serde_json::Value` stored in `ActionConfig.config`.
    pub fn from_json(value: &serde_json::Value) -> Result<Self> {
        serde_json::from_value(value.clone()).map_err(|e| {
            Error::ConfigurationError(format!(
                "GenericIngestionConnector config parse error: {e}"
            ))
        })
    }

    /// Derive the S3 key for the *converted* CSV from the upload filename.
    ///
    /// Strips the original file extension and replaces it with `.csv`.
    /// For a CSV upload the stem is unchanged, so the key resolves to the
    /// same path `CsvHrisConnector` would use.
    pub fn resolve_s3_key(&mut self, organization_id: &str, customer_company_id: &str) {
        let stem = stem_of(&self.filename);
        self.resolved_s3_key = Some(format!(
            "{}/{}/{}.csv",
            organization_id, customer_company_id, stem
        ));
    }

    /// Return the resolved S3 key, or error if unresolved.
    pub fn s3_key(&self) -> Result<&str> {
        self.resolved_s3_key.as_deref().ok_or_else(|| {
            Error::ConfigurationError(
                "GenericIngestionConnector: s3_key not resolved — pipeline engine must call \
                 resolve_s3_key() before execution"
                    .into(),
            )
        })
    }

    /// Effective table index (defaults to 0 when not specified).
    pub fn effective_table_index(&self) -> usize {
        self.table_index.unwrap_or(0)
    }
}

/// Return the file stem (name without extension) for the given filename.
///
/// If the filename has no extension the full name is returned as-is.
fn stem_of(filename: &str) -> &str {
    // Only consider the last component so path separators (which are rejected
    // by validation) cannot affect the stem calculation.
    match filename.rfind('.') {
        Some(pos) if pos > 0 => &filename[..pos],
        _ => filename,
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // stem_of
    // -----------------------------------------------------------------------

    #[test]
    fn stem_of_strips_extension() {
        assert_eq!(stem_of("employees.pdf"), "employees");
    }

    #[test]
    fn stem_of_strips_csv_extension() {
        assert_eq!(stem_of("roster.csv"), "roster");
    }

    #[test]
    fn stem_of_no_extension() {
        assert_eq!(stem_of("employees"), "employees");
    }

    #[test]
    fn stem_of_multiple_dots_takes_last() {
        assert_eq!(stem_of("my.employees.v2.pdf"), "my.employees.v2");
    }

    #[test]
    fn stem_of_leading_dot_returns_full() {
        // `.gitignore` style — no meaningful stem, return as-is.
        assert_eq!(stem_of(".hidden"), ".hidden");
    }

    // -----------------------------------------------------------------------
    // from_json
    // -----------------------------------------------------------------------

    #[test]
    fn from_json_minimal() {
        let json = serde_json::json!({ "filename": "employees.pdf" });
        let cfg = GenericIngestionConnectorConfig::from_json(&json).unwrap();
        assert_eq!(cfg.filename, "employees.pdf");
        assert!(cfg.columns.is_none());
        assert!(cfg.table_index.is_none());
        assert!(cfg.resolved_s3_key.is_none());
    }

    #[test]
    fn from_json_full() {
        let json = serde_json::json!({
            "filename": "roster.pdf",
            "columns": ["id", "name", "email"],
            "table_index": 2
        });
        let cfg = GenericIngestionConnectorConfig::from_json(&json).unwrap();
        assert_eq!(cfg.filename, "roster.pdf");
        assert_eq!(cfg.columns.as_deref(), Some(&["id".to_string(), "name".to_string(), "email".to_string()][..]));
        assert_eq!(cfg.table_index, Some(2));
    }

    #[test]
    fn from_json_missing_filename_errors() {
        let json = serde_json::json!({ "columns": ["a"] });
        assert!(GenericIngestionConnectorConfig::from_json(&json).is_err());
    }

    // -----------------------------------------------------------------------
    // resolve_s3_key
    // -----------------------------------------------------------------------

    #[test]
    fn resolve_s3_key_converts_pdf_to_csv() {
        let mut cfg = GenericIngestionConnectorConfig {
            filename: "employees.pdf".into(),
            columns: None,
            table_index: None,
            resolved_s3_key: None,
        };
        cfg.resolve_s3_key("org-1", "company-1");
        assert_eq!(cfg.resolved_s3_key.as_deref(), Some("org-1/company-1/employees.csv"));
    }

    #[test]
    fn resolve_s3_key_csv_unchanged() {
        let mut cfg = GenericIngestionConnectorConfig {
            filename: "roster.csv".into(),
            columns: None,
            table_index: None,
            resolved_s3_key: None,
        };
        cfg.resolve_s3_key("org-2", "company-2");
        assert_eq!(cfg.resolved_s3_key.as_deref(), Some("org-2/company-2/roster.csv"));
    }

    #[test]
    fn resolve_s3_key_xml_to_csv() {
        let mut cfg = GenericIngestionConnectorConfig {
            filename: "data.xml".into(),
            columns: None,
            table_index: None,
            resolved_s3_key: None,
        };
        cfg.resolve_s3_key("org-3", "company-3");
        assert_eq!(cfg.resolved_s3_key.as_deref(), Some("org-3/company-3/data.csv"));
    }

    #[test]
    fn s3_key_errors_before_resolve() {
        let cfg = GenericIngestionConnectorConfig {
            filename: "data.pdf".into(),
            columns: None,
            table_index: None,
            resolved_s3_key: None,
        };
        assert!(cfg.s3_key().is_err());
    }

    #[test]
    fn s3_key_returns_key_after_resolve() {
        let mut cfg = GenericIngestionConnectorConfig {
            filename: "data.pdf".into(),
            columns: None,
            table_index: None,
            resolved_s3_key: None,
        };
        cfg.resolve_s3_key("org-1", "co-1");
        assert_eq!(cfg.s3_key().unwrap(), "org-1/co-1/data.csv");
    }

    // -----------------------------------------------------------------------
    // effective_table_index
    // -----------------------------------------------------------------------

    #[test]
    fn effective_table_index_defaults_to_zero() {
        let cfg = GenericIngestionConnectorConfig {
            filename: "file.pdf".into(),
            columns: None,
            table_index: None,
            resolved_s3_key: None,
        };
        assert_eq!(cfg.effective_table_index(), 0);
    }

    #[test]
    fn effective_table_index_uses_configured_value() {
        let cfg = GenericIngestionConnectorConfig {
            filename: "file.pdf".into(),
            columns: None,
            table_index: Some(3),
            resolved_s3_key: None,
        };
        assert_eq!(cfg.effective_table_index(), 3);
    }
}
