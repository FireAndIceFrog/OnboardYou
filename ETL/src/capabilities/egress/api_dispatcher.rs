//! HTTP/JSON delivery to client-facing destination APIs

use crate::domain::{OnboardingAction, Result, RosterContext};

/// API dispatcher for sending data to destination systems
pub struct ApiDispatcher;

impl OnboardingAction for ApiDispatcher {
    fn id(&self) -> &str {
        "api_dispatcher"
    }

    fn execute(&self, context: RosterContext) -> Result<RosterContext> {
        // TODO: Implement API dispatch logic
        // - Collect the LazyFrame
        // - Serialize to JSON
        // - Send to configured destination endpoints
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_dispatcher_id() {
        let action = ApiDispatcher;
        assert_eq!(action.id(), "api_dispatcher");
    }
}
