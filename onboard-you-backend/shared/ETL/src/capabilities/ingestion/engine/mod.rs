//! Ingestion engine: Concrete data acquisition implementations

mod generic_ingestion_connector;
mod email_ingestion_connector;
mod workday_hris_connector;
mod sage_hr_connector;

pub use generic_ingestion_connector::*;
pub use email_ingestion_connector::*;
pub use workday_hris_connector::*;
pub use sage_hr_connector::*;
