//! Manifest: Versioned, declarative pipeline configuration
//!
//! Defines the schema for JSON/YAML pipeline configs that determine execution order

use serde::{Deserialize, Serialize};

/// Version of the manifest schema
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub actions: Vec<ActionConfig>,
}

/// Configuration for a single action in the pipeline
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionConfig {
    pub id: String,
    pub action_type: String, // "ingestion", "validation", "logic", "egress"
    pub config: serde_json::Value, // Action-specific configuration
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
