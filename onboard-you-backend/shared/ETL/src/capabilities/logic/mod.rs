//! Logic: Domain-specific data transformations
//!
//! - **models**: Configuration types and shared primitives (SafeRegex, configs)
//! - **traits**: Transformation interfaces (Deduplicator, Masker)
//! - **engine**: Concrete implementations (SCDType2, PIIMasking, IdentityDeduplicator)

pub mod engine;

pub use engine::*;
