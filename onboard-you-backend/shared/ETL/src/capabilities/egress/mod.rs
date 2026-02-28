//! Egress: Data delivery, authentication, and observability
//!
//! - **models**: Configuration and data types (AuthType, RetryPolicy, DispatchResponse, repo configs)
//! - **traits**: Repository interfaces (EgressRepository)
//! - **repositories**: Concrete auth + dispatch implementations (BearerRepo, OAuthRepo, OAuth2Repo)
//! - **engine**: Orchestration layer (ApiEngine — selects repo, applies retry policy)
//! - API Dispatcher: HTTP/JSON delivery to destination APIs (OnboardingAction wrapper)
//! - Observability: Request/response logging and RCA

pub mod api_dispatcher;
pub mod engine;
pub mod models;
pub mod observability;
pub mod repositories;
pub mod traits;

pub use api_dispatcher::*;
pub use engine::*;
pub use models::*;
pub use observability::*;
pub use repositories::*;
pub use traits::*;
