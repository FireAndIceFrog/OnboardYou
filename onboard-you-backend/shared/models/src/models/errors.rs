//! Unified error handling using thiserror
//!
//! Domain-specific error types for the onboarding pipeline

use thiserror::Error;

/// Result type for domain operations
pub type Result<T> = std::result::Result<T, Error>;

/// Domain error types
#[derive(Error, Debug)]
pub enum Error {
    #[error("Ingestion error: {0}")]
    IngestionError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Logic error: {0}")]
    LogicError(String),

    #[error("{0}")]
    EgressError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Polars error: {0}")]
    PolarsError(#[from] polars::error::PolarsError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
