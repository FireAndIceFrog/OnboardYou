//! Ingestion engine: Concrete data acquisition implementations

mod csv_hris_connector;
mod validator;

pub use csv_hris_connector::*;
pub use validator::*;
