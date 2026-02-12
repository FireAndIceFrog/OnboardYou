//! Egress traits: Interfaces for outbound data delivery
//!
//! - **EgressRepository**: Core contract for all egress authentication and dispatch strategies

mod egress_repository;

pub use crate::capabilities::egress::models::DispatchResponse;
pub use egress_repository::*;
