//! Manifest: Versioned, declarative pipeline configuration
//!
//! Defines the schema for JSON/YAML pipeline configs that determine execution order

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Version of the manifest schema
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Manifest {
    /// Schema version (e.g. "1.0")
    pub version: String,
    /// Ordered list of pipeline actions
    pub actions: Vec<ActionConfig>,
}

/// Configuration for a single action in the pipeline
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ActionConfig {
    /// Unique identifier for this pipeline step
    pub id: String,
    /// Factory key — selects the Rust implementation (e.g. "csv_hris_connector", "scd_type_2")
    pub action_type: String,
    /// Action-specific configuration (shape depends on action_type)
    pub config: serde_json::Value,
}

impl Manifest {
    /// Parse a manifest from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize manifest to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_deserialization() {
        let json = r#"{
            "version": "1.0",
            "actions": [
                {
                    "id": "ingest_hris",
                    "action_type": "ingestion",
                    "config": {}
                }
            ]
        }"#;

        let manifest = Manifest::from_json(json).expect("Failed to parse manifest");
        assert_eq!(manifest.version, "1.0");
        assert_eq!(manifest.actions.len(), 1);
    }
}
