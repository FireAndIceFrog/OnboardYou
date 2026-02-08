//! Core domain types and business interfaces
//!
//! The Contract: Defines the core traits (OnboardingAction) and connector interfaces
//! that all capabilities must implement.

pub mod errors;
pub mod manifest;
pub mod roster;

pub use errors::{Error, Result};
pub use manifest::Manifest;
pub use roster::RosterContext;

/// Core trait for all onboarding actions
///
/// Implementors (capabilities) transform a RosterContext through the pipeline
pub trait OnboardingAction: Send + Sync {
    /// Unique identifier for this action
    fn id(&self) -> &str;

    /// Execute this action on the given roster context
    fn execute(&self, context: RosterContext) -> Result<RosterContext>;
}
