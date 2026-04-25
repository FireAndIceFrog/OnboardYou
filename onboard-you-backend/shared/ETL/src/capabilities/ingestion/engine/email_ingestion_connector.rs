//! EmailIngestionConnector: ingests an attachment staged by the email-ingestor Lambda.
//!
//! The `email-ingestor` Lambda parses inbound SES emails, extracts the first
//! attachment, stages it in `CSV_UPLOAD_BUCKET` (converting via Textract if
//! non-CSV), and triggers the ETL pipeline with a `filename_override` in the
//! `ScheduledEtlEvent`.
//!
//! The pipeline engine resolves that override into an S3 key and injects it
//! into this config before calling `execute`.  From there the connector
//! behaves identically to `GenericIngestionConnector` — it downloads the CSV
//! from S3 and returns a `LazyFrame`.
//!
//! ## Sender filtering
//!
//! `allowed_senders` in the config is the authoritative allowlist.  The
//! email-ingestor Lambda validates the sender *before* triggering the pipeline,
//! so this connector does not re-validate at execution time.  The config is
//! stored in the manifest for auditability.
//!
//! ## Field ownership
//!
//! Every ingested column is tagged with `"EMAIL_INGESTION"` field-ownership
//! metadata so downstream logic actions can trace data provenance.

use crate::capabilities::ingestion::engine::generic_ingestion_connector::GenericIngestionConnector;
use crate::capabilities::ingestion::traits::HrisConnector;
use onboard_you_models::ColumnCalculator;
use onboard_you_models::{Error, OnboardingAction, Result, RosterContext};
use onboard_you_models::EmailIngestionConnectorConfig;
use polars::prelude::*;

// ---------------------------------------------------------------------------
// Source tag
// ---------------------------------------------------------------------------

const FIELD_SOURCE: &str = "EMAIL_INGESTION";

// ---------------------------------------------------------------------------
// Connector
// ---------------------------------------------------------------------------

/// HRIS connector that ingests the email attachment staged in S3.
///
/// Requires `resolved_s3_key` to be set before `execute` is called.  The
/// pipeline engine does this via `EmailIngestionConnectorConfig::resolve_s3_key`.
#[derive(Debug, Clone)]
pub struct EmailIngestionConnector {
    config: EmailIngestionConnectorConfig,
}

impl EmailIngestionConnector {
    /// Create a new connector from a pre-validated config.
    pub fn new(config: EmailIngestionConnectorConfig) -> Self {
        Self { config }
    }

    /// Construct from a deserialised config, applying validation.
    pub fn from_action_config(config: &EmailIngestionConnectorConfig) -> Result<Self> {
        if config.allowed_senders.is_empty() {
            return Err(Error::ConfigurationError(
                "EmailIngestionConnector requires at least one entry in `allowed_senders`".into(),
            ));
        }

        for entry in &config.allowed_senders {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                return Err(Error::ConfigurationError(
                    "EmailIngestionConnector: `allowed_senders` must not contain empty entries".into(),
                ));
            }
            if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
                return Err(Error::ConfigurationError(format!(
                    "EmailIngestionConnector: invalid sender entry '{trimmed}'"
                )));
            }
        }

        Ok(Self::new(config.clone()))
    }

    /// Download the staged CSV from S3 and return raw bytes.
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
                            "EmailIngestionConnector: S3 GetObject failed for \
                             '{bucket}/{s3_key}': {e}\n\
                             Hint: ensure the email-ingestor staged the file before the ETL ran."
                        ))
                    })?;

                let bytes = resp
                    .body
                    .collect()
                    .await
                    .map_err(|e| {
                        Error::IngestionError(format!(
                            "EmailIngestionConnector: failed to read S3 body: {e}"
                        ))
                    })?;

                Ok(bytes.into_bytes().to_vec())
            })
        })
    }
}

impl HrisConnector for EmailIngestionConnector {
    fn fetch_data(&self) -> Result<LazyFrame> {
        let csv_bytes = self.download_from_s3()?;
        GenericIngestionConnector::parse_csv(&csv_bytes, self.config.columns.as_deref())
    }
}

impl ColumnCalculator for EmailIngestionConnector {
    fn calculate_columns(&self, context: RosterContext) -> Result<RosterContext> {
        // Schema propagation — no S3 access at validate time.
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

impl OnboardingAction for EmailIngestionConnector {
    fn id(&self) -> &str {
        "email_ingestion_connector"
    }

    fn execute(&self, context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            allowed_senders = ?self.config.allowed_senders,
            subject_filter = ?self.config.subject_filter,
            s3_key = ?self.config.resolved_s3_key,
            "EmailIngestionConnector: ingesting staged attachment from S3"
        );

        let lf = self.fetch_data()?;

        let schema = lf
            .clone()
            .collect_schema()
            .map_err(|e| Error::IngestionError(format!("Failed to collect schema: {e}")))?;

        let mut ctx = RosterContext::with_deps(lf, context.deps.clone());

        for field_name in schema.iter_names() {
            ctx.set_field_source(field_name.to_string(), FIELD_SOURCE.into());
        }

        tracing::info!(
            fields = schema.len(),
            "EmailIngestionConnector: ingested {} fields",
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

    fn make_config(senders: Vec<&str>) -> EmailIngestionConnectorConfig {
        EmailIngestionConnectorConfig {
            allowed_senders: senders.into_iter().map(String::from).collect(),
            subject_filter: None,
            columns: None,
            table_index: None,
            resolved_s3_key: Some("org-1/co-1/roster_20260425T143000Z.csv".into()),
        }
    }

    fn make_config_with_cols(senders: Vec<&str>, cols: Vec<&str>) -> EmailIngestionConnectorConfig {
        EmailIngestionConnectorConfig {
            allowed_senders: senders.into_iter().map(String::from).collect(),
            subject_filter: None,
            columns: Some(cols.into_iter().map(String::from).collect()),
            table_index: None,
            resolved_s3_key: Some("org-1/co-1/roster_20260425T143000Z.csv".into()),
        }
    }

    fn make_deps() -> ETLDependancies {
        ETLDependancies::default()
    }

    // -----------------------------------------------------------------------
    // id
    // -----------------------------------------------------------------------

    #[test]
    fn connector_id_is_correct() {
        let connector = EmailIngestionConnector::new(make_config(vec!["hr@acme.com"]));
        assert_eq!(connector.id(), "email_ingestion_connector");
    }

    // -----------------------------------------------------------------------
    // from_action_config validation — table-driven
    // -----------------------------------------------------------------------

    #[test]
    fn from_action_config_validation() {
        struct Case { label: &'static str, senders: Vec<&'static str>, expect_ok: bool }
        let cases = vec![
            Case { label: "valid single sender",    senders: vec!["hr@acme.com"],     expect_ok: true  },
            Case { label: "valid domain glob",       senders: vec!["@partner.com"],    expect_ok: true  },
            Case { label: "multiple valid senders",  senders: vec!["hr@a.com", "@b.com"], expect_ok: true },
            Case { label: "empty list",              senders: vec![],                  expect_ok: false },
            Case { label: "blank entry",             senders: vec![""],               expect_ok: false },
            Case { label: "whitespace-only entry",   senders: vec!["   "],            expect_ok: false },
        ];

        for c in &cases {
            let cfg = EmailIngestionConnectorConfig {
                allowed_senders: c.senders.iter().map(|s| s.to_string()).collect(),
                subject_filter: None,
                columns: None,
                table_index: None,
                resolved_s3_key: None,
            };
            let result = EmailIngestionConnector::from_action_config(&cfg);
            assert_eq!(
                result.is_ok(),
                c.expect_ok,
                "case '{}': {:?}",
                c.label,
                result.err()
            );
        }
    }

    // -----------------------------------------------------------------------
    // calculate_columns
    // -----------------------------------------------------------------------

    #[test]
    fn calculate_columns_with_declared_cols_returns_schema() {
        let cfg = make_config_with_cols(vec!["hr@acme.com"], vec!["id", "name", "email"]);
        let connector = EmailIngestionConnector::new(cfg);
        let initial = RosterContext::with_deps(LazyFrame::default(), make_deps());
        let ctx = connector.calculate_columns(initial).unwrap();

        let schema = ctx.get_data().collect_schema().unwrap();
        let names: Vec<&str> = schema.iter_names().map(|n| n.as_str()).collect();
        assert_eq!(names, vec!["id", "name", "email"]);
    }

    #[test]
    fn calculate_columns_stamps_field_source() {
        let cfg = make_config_with_cols(vec!["hr@acme.com"], vec!["col_a", "col_b"]);
        let connector = EmailIngestionConnector::new(cfg);
        let initial = RosterContext::with_deps(LazyFrame::default(), make_deps());
        let ctx = connector.calculate_columns(initial).unwrap();

        assert_eq!(
            ctx.field_metadata().get("col_a").map(|m| m.source.as_str()),
            Some("EMAIL_INGESTION")
        );
        assert_eq!(
            ctx.field_metadata().get("col_b").map(|m| m.source.as_str()),
            Some("EMAIL_INGESTION")
        );
    }

    #[test]
    fn calculate_columns_without_declared_cols_returns_empty_schema() {
        let cfg = make_config(vec!["hr@acme.com"]);
        let connector = EmailIngestionConnector::new(cfg);
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
            "action_type": "email_ingestion_connector",
            "config": {
                "allowed_senders": ["hr@acme.com", "@partner.com"],
                "subject_filter": "Monthly Roster",
                "columns": ["id", "name"],
                "table_index": 0
            }
        });

        let action_config: ActionConfig = serde_json::from_value(json).unwrap();
        assert_eq!(action_config.action_type, ActionType::EmailIngestionConnector);

        match action_config.config {
            ActionConfigPayload::EmailIngestionConnector(cfg) => {
                assert_eq!(cfg.allowed_senders, vec!["hr@acme.com", "@partner.com"]);
                assert_eq!(cfg.subject_filter.as_deref(), Some("Monthly Roster"));
                assert_eq!(cfg.columns.as_deref(), Some(&["id".to_string(), "name".to_string()][..]));
                assert_eq!(cfg.table_index, Some(0));
            }
            other => panic!("Unexpected payload variant: {other:?}"),
        }
    }

    #[test]
    fn action_config_payload_deserializes_minimal() {
        use onboard_you_models::{ActionConfig, ActionConfigPayload, ActionType};

        let json = serde_json::json!({
            "id": "ingest",
            "action_type": "email_ingestion_connector",
            "config": {
                "allowed_senders": ["hr@acme.com"]
            }
        });

        let action_config: ActionConfig = serde_json::from_value(json).unwrap();
        assert_eq!(action_config.action_type, ActionType::EmailIngestionConnector);

        match action_config.config {
            ActionConfigPayload::EmailIngestionConnector(cfg) => {
                assert_eq!(cfg.allowed_senders, vec!["hr@acme.com"]);
                assert!(cfg.subject_filter.is_none());
                assert!(cfg.columns.is_none());
            }
            other => panic!("Unexpected payload variant: {other:?}"),
        }
    }
}
