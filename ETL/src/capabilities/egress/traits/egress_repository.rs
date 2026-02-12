//! Core trait for egress repositories (authentication + data dispatch)
//!
//! Every egress strategy (Bearer, OAuth, OAuth2) must implement this trait.
//! The `ApiEngine` delegates to a concrete `EgressRepository` at runtime,
//! allowing auth methods to be swapped via manifest config.

use crate::capabilities::egress::models::DispatchResponse;
use crate::domain::Result;

/// Core contract for all egress authentication and dispatch strategies.
///
/// Implementors handle two concerns:
///
/// 1. **Authentication** — obtain a valid token/credential for the destination API.
/// 2. **Data delivery** — serialise the payload and dispatch it to the destination.
///
/// Both methods are `async` because they perform network I/O. The pipeline's
/// sync boundary (`OnboardingAction::execute`) calls into these via
/// `tokio::runtime::Handle::block_on` inside the `ApiEngine`.
pub trait EgressRepository: Send + Sync {
    /// Retrieve a valid authentication token (or credential string) for the
    /// destination API.
    ///
    /// - **Bearer / API Key**: returns the static token verbatim.
    /// - **OAuth / OAuth2**: performs token exchange or refresh and returns the
    ///   access token.
    /// - **No auth**: returns `Ok(None)`.
    fn retrieve_token(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<String>>> + Send + '_>>;

    /// Serialise `payload` as JSON and dispatch it to the destination endpoint.
    ///
    /// The implementation must attach the token returned by [`retrieve_token`]
    /// to the outgoing request (header, query param, etc.).
    ///
    /// `payload` is the JSON string of the collected data (array of row objects).
    fn send_data(
        &self,
        payload: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<DispatchResponse>> + Send + '_>>;
}
