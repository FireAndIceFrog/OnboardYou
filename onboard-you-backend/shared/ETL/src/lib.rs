//! OnboardYou: Zero-persistence employee onboarding pipeline
//!
//! This library implements the Mediator Pattern for declarative, GDPR/HIPAA-compliant
//! employee data orchestration using Polars LazyFrames for in-memory processing.
//!
//! ## Architecture
//!
//! - **domain**: Core types and business interfaces (the Contract)
//! - **capabilities**: Functional logic steps (the Muscle) - ingestion, logic, egress
//! - **orchestration**: Pipeline assembly and execution (the Mediator)
//!
//! ## Core Execution Flow
//!
//! 1. **Ingest**: Receive data from HRIS sources
//! 2. **Validate**: Enforce data quality constraints
//! 3. **Transform**: Apply business logic (SCD Type 2, masking, deduplication)
//! 4. **Dispatch**: Send results to destination APIs
//! 5. **Observe**: Log and track execution

pub mod capabilities;
pub mod orchestration;
pub use orchestration::{ActionFactory, ActionFactoryTrait, StepError};