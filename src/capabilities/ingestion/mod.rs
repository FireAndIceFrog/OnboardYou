//! Ingestion: Data acquisition (Webhooks, API Polling, CSV parsing)

pub mod hris_connector;
pub mod validator;

pub use hris_connector::*;
pub use validator::*;
