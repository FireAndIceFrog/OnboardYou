//! The Resolver: Maps manifest string IDs to Capability instances
//!
//! Uses the `ActionConfig` from the manifest to instantiate the correct
//! `OnboardingAction` implementation, forwarding action-specific JSON config.

use crate::capabilities::ingestion::engine::CsvHrisConnector;
use crate::domain::{Error, OnboardingAction, Result};
use crate::domain::engine::manifest::ActionConfig;
use std::sync::Arc;

/// Factory for creating OnboardingAction instances from manifest action configs.
pub struct ActionFactory;

impl ActionFactory {
    /// Create an action instance from a full `ActionConfig`.
    ///
    /// The `action_type` field selects the concrete implementation, while
    /// `config` is forwarded as-is to the implementation's constructor.
    pub fn create(action_config: &ActionConfig) -> Result<Arc<dyn OnboardingAction>> {
        match action_config.action_type.as_str() {
            "csv_hris_connector" => {
                let connector = CsvHrisConnector::from_action_config(&action_config.config)?;
                Ok(Arc::new(connector))
            }
            // Future action types go here:
            // "data_validator" => { ... }
            // "scd_type_2"     => { ... }
            // "api_dispatcher"  => { ... }
            other => Err(Error::ConfigurationError(format!(
                "Unknown action type: '{}'",
                other
            ))),
        }
    }

    /// Legacy helper — resolve by bare id (no config). Kept for backward compat.
    pub fn create_action(action_id: &str) -> Result<Arc<dyn OnboardingAction>> {
        Err(Error::ConfigurationError(format!(
            "Unknown action type: '{}'. Use ActionFactory::create(config) instead.",
            action_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_unknown_action() {
        let result = ActionFactory::create_action("unknown_action");
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_creates_csv_connector() {
        let config = ActionConfig {
            id: "ingest".into(),
            action_type: "csv_hris_connector".into(),
            config: serde_json::json!({ "csv_path": "/tmp/test.csv" }),
        };
        let action = ActionFactory::create(&config).expect("should create csv connector");
        assert_eq!(action.id(), "csv_hris_connector");
    }

    #[test]
    fn test_factory_rejects_unknown_type() {
        let config = ActionConfig {
            id: "bad".into(),
            action_type: "nope".into(),
            config: serde_json::json!({}),
        };
        assert!(ActionFactory::create(&config).is_err());
    }
}
