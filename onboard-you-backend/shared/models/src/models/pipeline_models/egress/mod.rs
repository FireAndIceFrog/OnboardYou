//! Egress models: Configuration and data types for egress dispatch
//!
//! All structs/enums that describe *what* rather than *how* live here:
//! - Auth type selection (`AuthType`)
//! - Retry policy (`RetryPolicy`)
//! - Dispatch response (`DispatchResponse`)
//! - Per-repo configs (`BearerRepoConfig`, `OAuthRepoConfig`, `OAuth2RepoConfig`)

mod api_dispatcher_config;
mod auth_type;
mod bearer_config;
mod dispatch_response;
mod oauth2_config;
mod oauth_config;
mod retry_policy;
mod show_data_config;

pub use api_dispatcher_config::*;
pub use auth_type::*;
pub use bearer_config::*;
pub use dispatch_response::*;
pub use oauth2_config::*;
pub use oauth_config::*;
pub use retry_policy::*;
pub use show_data_config::*;
