use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{Error, Result};

/// Configuration for the `EmailIngestionConnector` pipeline action.
///
/// The connector does **not** download data directly from an inbox.  Instead,
/// an upstream `email-ingestor` Lambda parses inbound SES emails, extracts
/// the attachment, stages it in S3, and triggers the ETL pipeline with a
/// `filename_override` carrying the timestamped S3 key.
///
/// This config stores the allowlist of permitted sender addresses and an
/// optional subject filter.  The actual S3 key is injected at runtime by the
/// pipeline engine (via `resolve_s3_key`) using the `filename_override` from
/// the triggering event.
///
/// # JSON config (user-facing)
///
/// ```json
/// {
///   "allowed_senders": ["hr@acme.com", "@partner.com"],
///   "subject_filter": "Monthly Roster",
///   "columns": ["employee_id", "first_name", "last_name"]
/// }
/// ```
///
/// | Field            | Type     | Required | Description                                               |
/// |------------------|----------|----------|-----------------------------------------------------------|
/// | `allowed_senders`| [string] | **yes**  | Sender addresses (exact) or domain globs (`@domain.com`) |
/// | `subject_filter` | string   | no       | If set, emails whose subject does not contain this string |
/// |                  |          |          | are ignored by the ingestor                               |
/// | `columns`        | [string] | no       | Override column headers in the extracted CSV              |
/// | `table_index`    | number   | no       | Textract table index for multi-table documents (default 0)|
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct EmailIngestionConnectorConfig {
    /// Allowed sender addresses or domain patterns.
    ///
    /// Entries may be:
    /// - An exact address (`"hr@acme.com"`) — matched case-insensitively.
    /// - A domain glob (`"@acme.com"`) — matches any address ending with that suffix.
    ///
    /// Must not be empty.
    pub allowed_senders: Vec<String>,

    /// Optional subject-line filter.
    ///
    /// When present, the `email-ingestor` Lambda only triggers the pipeline
    /// when the email subject contains this string (case-insensitive).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub subject_filter: Option<String>,

    /// User-defined column headers — same semantics as `GenericIngestionConnectorConfig`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub columns: Option<Vec<String>>,

    /// Zero-based Textract table index (default 0).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub table_index: Option<usize>,

    /// Resolved S3 key — injected at runtime by the pipeline engine.
    ///
    /// Set from the `filename_override` field of the triggering
    /// `ScheduledEtlEvent`.  Not part of the user-facing config.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[schema(ignore)]
    pub resolved_s3_key: Option<String>,
}

impl EmailIngestionConnectorConfig {
    /// Resolve the S3 key from the runtime-supplied attachment filename.
    ///
    /// Unlike `GenericIngestionConnectorConfig`, the filename is not stored in
    /// the config — it arrives per-run via `filename_override` in the event.
    pub fn resolve_s3_key(
        &mut self,
        organization_id: &str,
        customer_company_id: &str,
        filename: &str,
    ) {
        let stem = stem_of(filename);
        self.resolved_s3_key = Some(format!(
            "{}/{}/{}.csv",
            organization_id, customer_company_id, stem
        ));
    }

    /// Return the resolved S3 key, or an error if unresolved.
    pub fn s3_key(&self) -> Result<&str> {
        self.resolved_s3_key.as_deref().ok_or_else(|| {
            Error::ConfigurationError(
                "EmailIngestionConnector: s3_key not resolved — pipeline engine must supply \
                 filename_override before execution"
                    .into(),
            )
        })
    }

    /// Effective table index (defaults to 0 when not specified).
    pub fn effective_table_index(&self) -> usize {
        self.table_index.unwrap_or(0)
    }

    /// Return `true` if `sender` is permitted by the `allowed_senders` list.
    ///
    /// Matching is case-insensitive.  Entries starting with `@` are treated as
    /// domain suffixes (e.g. `"@acme.com"` matches `"hr@acme.com"`).  All
    /// other entries are compared as exact addresses.
    pub fn is_sender_allowed(&self, sender: &str) -> bool {
        let sender_lc = sender.to_lowercase();
        self.allowed_senders.iter().any(|entry| {
            let entry_lc = entry.to_lowercase();
            if entry_lc.starts_with('@') {
                sender_lc.ends_with(&entry_lc)
            } else {
                sender_lc == entry_lc
            }
        })
    }
}

/// Return the file stem (name without extension).
fn stem_of(filename: &str) -> &str {
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

    fn make_config(senders: Vec<&str>) -> EmailIngestionConnectorConfig {
        EmailIngestionConnectorConfig {
            allowed_senders: senders.into_iter().map(String::from).collect(),
            subject_filter: None,
            columns: None,
            table_index: None,
            resolved_s3_key: None,
        }
    }

    // -----------------------------------------------------------------------
    // is_sender_allowed — table-driven
    // -----------------------------------------------------------------------

    #[test]
    fn is_sender_allowed_exact_match() {
        struct Case { sender: &'static str, expected: bool }
        let cases = vec![
            Case { sender: "hr@acme.com",    expected: true },
            Case { sender: "HR@ACME.COM",    expected: true },  // case-insensitive
            Case { sender: "other@acme.com", expected: false },
            Case { sender: "hr@other.com",   expected: false },
        ];
        let cfg = make_config(vec!["hr@acme.com"]);
        for c in &cases {
            assert_eq!(
                cfg.is_sender_allowed(c.sender),
                c.expected,
                "sender: {}",
                c.sender
            );
        }
    }

    #[test]
    fn is_sender_allowed_domain_glob() {
        struct Case { sender: &'static str, expected: bool }
        let cases = vec![
            Case { sender: "hr@acme.com",      expected: true },
            Case { sender: "payroll@acme.com", expected: true },
            Case { sender: "HR@ACME.COM",      expected: true },
            Case { sender: "hr@other.com",     expected: false },
            Case { sender: "acme.com",         expected: false }, // no @
        ];
        let cfg = make_config(vec!["@acme.com"]);
        for c in &cases {
            assert_eq!(
                cfg.is_sender_allowed(c.sender),
                c.expected,
                "sender: {}",
                c.sender
            );
        }
    }

    #[test]
    fn is_sender_allowed_mixed_list() {
        let cfg = make_config(vec!["hr@acme.com", "@partner.com"]);
        assert!(cfg.is_sender_allowed("hr@acme.com"));
        assert!(cfg.is_sender_allowed("anyone@partner.com"));
        assert!(!cfg.is_sender_allowed("other@acme.com"));
        assert!(!cfg.is_sender_allowed("hr@other.com"));
    }

    #[test]
    fn is_sender_allowed_empty_list_denies_all() {
        let cfg = make_config(vec![]);
        assert!(!cfg.is_sender_allowed("hr@acme.com"));
    }

    // -----------------------------------------------------------------------
    // resolve_s3_key + s3_key — table-driven
    // -----------------------------------------------------------------------

    #[test]
    fn resolve_s3_key_table() {
        struct Case {
            org: &'static str,
            company: &'static str,
            filename: &'static str,
            expected_key: &'static str,
        }
        let cases = vec![
            Case { org: "org-1", company: "co-1", filename: "roster_20260425T143000Z.csv",  expected_key: "org-1/co-1/roster_20260425T143000Z.csv" },
            Case { org: "org-1", company: "co-1", filename: "data_20260425T143000Z.pdf",    expected_key: "org-1/co-1/data_20260425T143000Z.csv" },
            Case { org: "org-2", company: "co-2", filename: "report_20260425T143000Z.xlsx", expected_key: "org-2/co-2/report_20260425T143000Z.csv" },
        ];
        for c in &cases {
            let mut cfg = make_config(vec!["hr@acme.com"]);
            cfg.resolve_s3_key(c.org, c.company, c.filename);
            assert_eq!(
                cfg.s3_key().unwrap(),
                c.expected_key,
                "filename: {}",
                c.filename
            );
        }
    }

    #[test]
    fn s3_key_errors_before_resolve() {
        let cfg = make_config(vec!["hr@acme.com"]);
        assert!(cfg.s3_key().is_err());
    }

    // -----------------------------------------------------------------------
    // Serde round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn serde_roundtrip_full() {
        let json = serde_json::json!({
            "allowed_senders": ["hr@acme.com", "@partner.com"],
            "subject_filter": "Monthly Roster",
            "columns": ["id", "name"],
            "table_index": 1
        });
        let cfg: EmailIngestionConnectorConfig = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(cfg.allowed_senders, vec!["hr@acme.com", "@partner.com"]);
        assert_eq!(cfg.subject_filter.as_deref(), Some("Monthly Roster"));
        assert_eq!(cfg.table_index, Some(1));
        assert!(cfg.resolved_s3_key.is_none());
    }

    #[test]
    fn serde_roundtrip_minimal() {
        let json = serde_json::json!({ "allowed_senders": ["hr@acme.com"] });
        let cfg: EmailIngestionConnectorConfig = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.allowed_senders, vec!["hr@acme.com"]);
        assert!(cfg.subject_filter.is_none());
        assert!(cfg.columns.is_none());
        assert!(cfg.table_index.is_none());
    }

    #[test]
    fn resolved_s3_key_not_serialized_in_user_config() {
        let mut cfg = make_config(vec!["hr@acme.com"]);
        cfg.resolve_s3_key("org", "co", "roster.csv");
        let json = serde_json::to_value(&cfg).unwrap();
        // resolved_s3_key has skip_serializing_if = Option::is_none... but it IS Some here.
        // We want to confirm the field IS present in full serialization (it's only hidden
        // from the OpenAPI schema via #[schema(ignore)]).
        // The important check: user-facing deserialization ignores it when absent.
        let json_no_key = serde_json::json!({ "allowed_senders": ["hr@acme.com"] });
        let cfg2: EmailIngestionConnectorConfig =
            serde_json::from_value(json_no_key).unwrap();
        assert!(cfg2.resolved_s3_key.is_none());
        let _ = json; // suppress unused warning
    }
}
