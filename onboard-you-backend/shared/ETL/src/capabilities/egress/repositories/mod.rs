//! Egress repositories: Concrete auth + dispatch implementations
//!
//! - **BearerRepo**: Static bearer tokens, API keys, no-auth
//! - **OAuthRepo**: OAuth 1.0a signed requests
//! - **OAuth2Repo**: OAuth2 Client Credentials & Authorization Code / OIDC

pub mod bearer_repo;
pub mod oauth2_repo;
pub mod oauth_repo;

pub use bearer_repo::*;
pub use oauth2_repo::*;
pub use oauth_repo::*;
