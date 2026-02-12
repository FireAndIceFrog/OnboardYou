//! Logic: Domain-specific data transformations
//!
//! - **traits**: Transformation interfaces (Deduplicator, Masker)
//! - **engine**: Concrete implementations (SCDType2, PIIMasking, IdentityDeduplicator)

pub mod engine;
pub mod traits;

pub use engine::*;
pub use traits::*;
