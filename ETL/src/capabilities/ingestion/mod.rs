//! Ingestion: Data acquisition (Webhooks, API Polling, CSV parsing)
//!
//! - **traits**: Connector interfaces (HrisConnector)
//! - **engine**: Concrete implementations (CsvHrisConnector, DataValidator)

pub mod engine;
pub mod traits;

pub use engine::*;
pub use traits::*;
