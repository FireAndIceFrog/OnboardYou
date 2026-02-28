//! Core trait for all onboarding actions
//!
//! Implementors (capabilities) transform a RosterContext through the pipeline

use crate::traits::ColumnCalculator;
use crate::models::errors::Result;
use crate::models::RosterContext;

/// Core trait for all onboarding actions
///
/// Implementors (capabilities) transform a RosterContext through the pipeline.
/// Each action receives the current context, transforms it, and returns the
/// updated context to be passed to the next action in the pipeline.
///
/// Every action must also implement [`ColumnCalculator`] so the pipeline can
/// derive the output schema of each step without executing it.
pub trait OnboardingAction: ColumnCalculator + Send + Sync {
    /// Unique identifier for this action
    fn id(&self) -> &str;

    /// Execute this action on the given roster context
    fn execute(&self, context: RosterContext) -> Result<RosterContext>;
}
