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
pub mod observability;
pub mod repositories;
pub mod show_data;
pub mod traits;

pub use api_dispatcher::*;
pub use engine::*;
pub use observability::*;
pub use repositories::*;
pub use show_data::*;
pub use traits::*;
