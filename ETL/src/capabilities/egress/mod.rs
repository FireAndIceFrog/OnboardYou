//! Egress: Data delivery and observability
//!
//! - API Dispatcher: HTTP/JSON delivery to destination APIs
//! - Observability: Request/response logging and RCA

pub mod api_dispatcher;
pub mod observability;

pub use api_dispatcher::*;
pub use observability::*;
