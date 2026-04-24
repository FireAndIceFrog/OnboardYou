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
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ShowDataConfig {
    /// Resolved S3 object key for the output CSV — injected at runtime.
    ///
    /// Not present in user-authored manifests; set by the pipeline engine
    /// immediately before the actions are built.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub s3_key: Option<String>,
}

impl Default for ShowDataConfig {
    fn default() -> Self {
        Self { s3_key: None }
    }
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
