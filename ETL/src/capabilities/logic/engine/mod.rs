//! Logic engine: Concrete domain-specific transformation implementations
//!
//! - identity_deduplicator: Column-major identity resolution using NID/Email
//! - identity_fuzzy_match: Probabilistic matching for high-fidelity record merging
//! - masking: PII protection (SSN/Salary) based on residency rules
//! - scd_type_2: Effective dating logic for historical tracking

mod identity_deduplicator;
mod identity_fuzzy_match;
mod masking;
mod scd_type_2;
mod rename_column;
mod drop_column;
mod regex_replace;
mod iso_country_sanitizer;

pub use identity_deduplicator::*;
pub use identity_fuzzy_match::*;
pub use masking::*;
pub use scd_type_2::*;
pub use rename_column::*;
pub use drop_column::*;
pub use regex_replace::*;
pub use iso_country_sanitizer::*;
