//! Logic traits: Interfaces for domain-specific data transformations
//!
//! These traits define the contracts that logic engine implementations must fulfil.

mod column_calculator;
mod deduplication;
mod masking;

pub use column_calculator::*;
pub use deduplication::*;
pub use masking::*;
