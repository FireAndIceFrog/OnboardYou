//! The Loop: Sequentially executes Actions on the RosterContext

use crate::domain::{Manifest, OnboardingAction, Result, RosterContext};
use std::sync::Arc;

/// Pipeline runner that executes a sequence of actions
pub struct PipelineRunner;

impl PipelineRunner {
    /// Execute a pipeline defined by a manifest
    pub fn run(
        manifest: &Manifest,
        actions: Vec<Arc<dyn OnboardingAction>>,
        mut context: RosterContext,
    ) -> Result<RosterContext> {
        // TODO: Implement pipeline execution
        // - Iterate through actions in manifest order
        // - For each action, find matching Arc<dyn OnboardingAction> by ID
        // - Execute action.execute(context), updating context
        // - Return final context
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_runner() {
        // TODO: Implement test using test manifest and mock actions
    }
}
