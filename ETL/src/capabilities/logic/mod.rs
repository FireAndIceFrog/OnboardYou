//! Logic: Domain-specific data transformations
//!
//! - **models**: Configuration types and shared primitives (SafeRegex, configs)
//! - **traits**: Transformation interfaces (Deduplicator, Masker)
//! - **engine**: Concrete implementations (SCDType2, PIIMasking, IdentityDeduplicator)

pub mod engine;
pub mod models;
pub mod traits;

pub use engine::*;
pub use models::*;
pub use traits::*;
