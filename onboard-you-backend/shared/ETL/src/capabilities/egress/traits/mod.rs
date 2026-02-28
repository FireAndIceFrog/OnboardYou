//! Egress traits: Interfaces for outbound data delivery
//!
//! - **EgressRepository**: Core contract for all egress authentication and dispatch strategies

mod egress_repository;
pub mod dynamic_egress_model;

pub use onboard_you_models::DispatchResponse;
pub use egress_repository::*;
pub use dynamic_egress_model::*;