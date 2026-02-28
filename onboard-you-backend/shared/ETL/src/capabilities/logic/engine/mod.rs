//! Logic engine: Concrete domain-specific transformation implementations
//!
//! - identity_deduplicator: Column-major identity resolution using NID/Email
//! - masking: PII protection (SSN/Salary) based on residency rules
//! - scd_type_2: Effective dating logic for historical tracking

mod cellphone_sanitizer;
mod drop_column;
mod filter_by_value;
mod handle_diacritics;
mod identity_deduplicator;
mod iso_country_sanitizer;
mod masking;
mod regex_replace;
mod rename_column;
mod scd_type_2;

pub use cellphone_sanitizer::*;
pub use drop_column::*;
pub use filter_by_value::*;
pub use handle_diacritics::*;
pub use identity_deduplicator::*;
pub use iso_country_sanitizer::*;
pub use masking::*;
pub use regex_replace::*;
pub use rename_column::*;
pub use scd_type_2::*;
