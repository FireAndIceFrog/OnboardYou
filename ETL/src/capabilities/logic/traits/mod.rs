//! Logic traits: Interfaces for domain-specific data transformations
//!
//! These traits define the contracts that logic engine implementations must fulfil.

mod deduplication;
mod masking;

pub use deduplication::*;
pub use masking::*;
