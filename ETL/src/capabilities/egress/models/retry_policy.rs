//! Simple retry configuration for outbound HTTP requests.

use serde::{Deserialize, Serialize};

/// Retry policy applied by the `ApiEngine` around `EgressRepository::send_data`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_backoff_ms: u64,
    pub retryable_status_codes: Vec<u16>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 1_000,
            retryable_status_codes: vec![429, 502, 503, 504],
        }
    }
}
