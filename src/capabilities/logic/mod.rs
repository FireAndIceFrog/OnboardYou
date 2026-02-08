//! Logic: Domain-specific data transformations
//!
//! Transformation capabilities:
//! - SCD Type 2: Effective dating for historical tracking
//! - Masking: PII protection based on residency rules
//! - Identity Resolution: Deduplication and fuzzy matching
//! - Field Cleaning: Data normalization

pub mod identity_deduplicator;
pub mod identity_fuzzy_match;
pub mod masking;
pub mod scd_type_2;

pub use identity_deduplicator::*;
pub use identity_fuzzy_match::*;
pub use masking::*;
pub use scd_type_2::*;
