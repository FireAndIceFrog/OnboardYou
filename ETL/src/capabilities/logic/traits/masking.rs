//! Masking trait: Interface for PII protection strategies

use crate::domain::{Result, RosterContext};

/// Trait for PII masking strategies.
///
/// Implementations decide which fields to mask and how, based on regulatory
/// requirements and residency rules.
pub trait Masker: Send + Sync {
    /// Apply masking rules to the roster context
    fn mask(&self, context: RosterContext) -> Result<RosterContext>;
}
