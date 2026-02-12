//! Response from a data dispatch operation.

use serde::Serialize;

/// Wraps enough information for observability / retry logic without
/// coupling the trait to a specific HTTP client.
#[derive(Debug, Clone, Serialize)]
pub struct DispatchResponse {
    /// HTTP status code returned by the destination.
    pub status_code: u16,
    /// Response body (may be truncated for logging).
    pub body: String,
    /// Number of records that were included in the request payload.
    pub records_sent: usize,
}
