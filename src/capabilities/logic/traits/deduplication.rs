//! Deduplication trait: Interface for identity resolution strategies

use crate::domain::{Result, RosterContext};

/// Trait for identity resolution / deduplication strategies.
///
/// Implementations decide how duplicate employee records are detected and merged.
pub trait Deduplicator: Send + Sync {
    /// Resolve duplicate records in the roster context
    fn deduplicate(&self, context: RosterContext) -> Result<RosterContext>;
}
