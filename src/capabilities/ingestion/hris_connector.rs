//! Generic trait for external HRIS systems (Workday, BambooHR, etc)

use crate::domain::{OnboardingAction, Result, RosterContext};

/// Generic trait for HRIS system connectors
pub trait HrisConnector: Send + Sync {
    /// Fetch data from the HRIS system
    fn fetch_data(&self) -> Result<Vec<u8>>;
}

/// Example HRIS connector implementation
pub struct DefaultHrisConnector;

impl OnboardingAction for DefaultHrisConnector {
    fn id(&self) -> &str {
        "hris_connector_default"
    }

    fn execute(&self, context: RosterContext) -> Result<RosterContext> {
        // TODO: Implement HRIS data ingestion
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_hris_connector() {
        let connector = DefaultHrisConnector;
        assert_eq!(connector.id(), "hris_connector_default");
    }
}
