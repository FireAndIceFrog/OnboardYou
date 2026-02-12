//! The Loop: Sequentially executes Actions on the RosterContext

use crate::domain::{Manifest, OnboardingAction, Result, RosterContext};
use std::sync::Arc;

/// Pipeline runner that executes a sequence of actions.
///
/// Actions are executed in the order they appear in the `actions` vector,
/// which matches the order declared in the manifest.
pub struct PipelineRunner;

impl PipelineRunner {
    /// Execute a pipeline defined by a manifest.
    ///
    /// Each action receives the `RosterContext` produced by the previous step
    /// (fold pattern). The final context is returned.
    pub fn run(
        _manifest: &Manifest,
        actions: Vec<Arc<dyn OnboardingAction>>,
        mut context: RosterContext,
    ) -> Result<RosterContext> {
        for action in &actions {
            tracing::info!(action_id = action.id(), "PipelineRunner: executing action");
            context = action.execute(context)?;
        }
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::logic::traits::ColumnCalculator;
    use crate::domain::engine::manifest::ActionConfig;
    use polars::prelude::*;

    /// A trivial pass-through action for testing the runner.
    struct NoopAction;
    impl ColumnCalculator for NoopAction {
        fn calculate_columns(&self, ctx: RosterContext) -> Result<RosterContext> {
            Ok(ctx)
        }
    }
    impl OnboardingAction for NoopAction {
        fn id(&self) -> &str {
            "noop"
        }
        fn execute(&self, ctx: RosterContext) -> Result<RosterContext> {
            Ok(ctx)
        }
    }

    #[test]
    fn test_pipeline_runner_no_actions() {
        let manifest = Manifest {
            version: "1.0".into(),
            actions: vec![],
        };
        let ctx = RosterContext::new(LazyFrame::default());
        let result = PipelineRunner::run(&manifest, vec![], ctx).expect("run");
        assert!(result.field_metadata.is_empty());
    }

    #[test]
    fn test_pipeline_runner_single_action() {
        let manifest = Manifest {
            version: "1.0".into(),
            actions: vec![ActionConfig {
                id: "noop".into(),
                action_type: "noop".into(),
                config: serde_json::json!({}),
            }],
        };
        let ctx = RosterContext::new(LazyFrame::default());
        let actions: Vec<Arc<dyn OnboardingAction>> = vec![Arc::new(NoopAction)];
        let result = PipelineRunner::run(&manifest, actions, ctx).expect("run");
        assert!(result.field_metadata.is_empty());
    }
}
