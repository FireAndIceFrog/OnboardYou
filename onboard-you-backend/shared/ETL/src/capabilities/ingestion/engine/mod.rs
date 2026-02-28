//! Ingestion engine: Concrete data acquisition implementations

mod csv_hris_connector;
mod workday_hris_connector;

pub use csv_hris_connector::*;
pub use workday_hris_connector::*;
