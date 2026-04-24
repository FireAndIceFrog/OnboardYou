use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Configuration for the `ShowData` pipeline action.
///
/// No user-facing fields are required — the S3 output key is derived at
/// runtime from the organisation, company and action IDs:
/// `{org_id}/{company_id}/outputs/{action_id}.csv`
///
/// Multiple `ShowData` steps in the same pipeline write to distinct files
/// because each action has a unique `id` within the manifest.
#[derive(Serialize, Deserialize, Debug, Clone, Default, ToSchema)]
pub struct ShowDataConfig {
    /// Resolved S3 object key for the output CSV — injected at runtime.
    ///
    /// Not present in user-authored manifests; set by the pipeline engine
    /// immediately before the actions are built.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub s3_key: Option<String>,
}

impl ShowDataConfig {
    /// Compute and store the output S3 key.
    ///
    /// Called by the pipeline engine before action instantiation so that
    /// each `ShowData` step gets its own distinct key.
    pub fn resolve_s3_key(&mut self, org_id: &str, company_id: &str, action_id: &str) {
        self.s3_key = Some(format!("{org_id}/{company_id}/outputs/{action_id}.csv"));
    }

    /// Return the resolved key, or an error if `resolve_s3_key` was never called.
    pub fn resolved_key(&self) -> crate::Result<&str> {
        self.s3_key
            .as_deref()
            .ok_or_else(|| crate::Error::ConfigurationError(
                "ShowData s3_key not resolved — resolve_s3_key() must be called before execution".into(),
            ))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── resolve_s3_key ────────────────────────────────────────────────────────

    #[test]
    fn resolve_s3_key_produces_expected_path() {
        let cases: &[(&str, &str, &str, &str)] = &[
            ("acme-corp", "company-42", "snapshot-1", "acme-corp/company-42/outputs/snapshot-1.csv"),
            ("org", "co", "step", "org/co/outputs/step.csv"),
            ("big-enterprise", "subsidiary", "data-preview", "big-enterprise/subsidiary/outputs/data-preview.csv"),
        ];

        for (org, company, action, expected) in cases {
            let mut cfg = ShowDataConfig::default();
            cfg.resolve_s3_key(org, company, action);
            assert_eq!(
                cfg.s3_key.as_deref(),
                Some(*expected),
                "org={org}, company={company}, action={action}",
            );
        }
    }

    // ── resolved_key ──────────────────────────────────────────────────────────

    #[test]
    fn resolved_key_errors_before_resolution() {
        let cfg = ShowDataConfig::default();
        assert!(cfg.resolved_key().is_err());
    }

    #[test]
    fn resolved_key_returns_key_after_resolution() {
        let mut cfg = ShowDataConfig::default();
        cfg.resolve_s3_key("org", "co", "step");
        assert_eq!(cfg.resolved_key().unwrap(), "org/co/outputs/step.csv");
    }

    // ── serde ─────────────────────────────────────────────────────────────────

    #[test]
    fn serde_omits_s3_key_when_none() {
        let cfg = ShowDataConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        assert!(!json.contains("s3_key"), "None s3_key must be skipped in JSON");
    }

    #[test]
    fn serde_deserializes_empty_object_to_none() {
        let cfg: ShowDataConfig = serde_json::from_str("{}").unwrap();
        assert!(cfg.s3_key.is_none());
    }

    #[test]
    fn serde_round_trips_resolved_key() {
        let mut cfg = ShowDataConfig::default();
        cfg.resolve_s3_key("org", "co", "step");
        let json = serde_json::to_string(&cfg).unwrap();
        let back: ShowDataConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.s3_key.as_deref(), Some("org/co/outputs/step.csv"));
    }
}
