//! The Resolver: Maps manifest string IDs to Capability instances

use crate::domain::{Error, OnboardingAction, Result};
use std::sync::Arc;

/// Factory for creating OnboardingAction instances from manifest IDs
pub struct ActionFactory;

impl ActionFactory {
    /// Create an action instance from a manifest action ID
    pub fn create_action(action_id: &str) -> Result<Arc<dyn OnboardingAction>> {
        // TODO: Implement factory logic
        // Map action_id strings to concrete OnboardingAction implementations
        // Return Arc<dyn OnboardingAction> for flexibility
        Err(Error::ConfigurationError(format!(
            "Unknown action type: {}",
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
}
