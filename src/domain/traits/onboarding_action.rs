//! Core trait for all onboarding actions
//!
//! Implementors (capabilities) transform a RosterContext through the pipeline

use crate::domain::engine::{RosterContext};
use crate::domain::engine::errors::Result;

/// Core trait for all onboarding actions
///
/// Implementors (capabilities) transform a RosterContext through the pipeline.
/// Each action receives the current context, transforms it, and returns the
/// updated context to be passed to the next action in the pipeline.
pub trait OnboardingAction: Send + Sync {
    /// Unique identifier for this action
    fn id(&self) -> &str;

    /// Execute this action on the given roster context
    fn execute(&self, context: RosterContext) -> Result<RosterContext>;
}
