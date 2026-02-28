//! Egress traits: Interfaces for outbound data delivery
//!
//! - **EgressRepository**: Core contract for all egress authentication and dispatch strategies

mod egress_repository;
pub mod dynamic_egress_model;

pub use crate::capabilities::egress::models::DispatchResponse;
pub use egress_repository::*;