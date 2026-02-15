//! Core domain types and business interfaces
//!
//! - **traits**: The Contract — core traits (OnboardingAction) that capabilities must implement
//! - **engine**: Concrete data structures — RosterContext, Manifest, Errors

pub mod engine;
pub mod traits;

// Re-export for ergonomic imports: `use crate::domain::{...}`
pub use engine::{ActionConfig, ActionType, Error, FieldMetadata, Manifest, Result, RosterContext};
pub use traits::OnboardingAction;

// Re-export the ColumnCalculator trait so consumers can reach it from `crate::domain`
pub use crate::capabilities::logic::traits::ColumnCalculator;
