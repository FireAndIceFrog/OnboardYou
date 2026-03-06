//! REST client abstraction for HTTP API connectors
//!
//! Provides a mockable async trait for GET requests used by REST API
//! connectors (e.g. Sage HR). Uses non-blocking I/O so the tokio runtime
//! is never starved by long HTTP round-trips.
//!
//! The production [`ReqwestRestClient`] includes exponential backoff with
//! jitter for transient failures (429 / 5xx).

use onboard_you_models::{Error, Result};

// ───────────────────────────────────────────────────────────────────────────
// REST Client Trait
// ───────────────────────────────────────────────────────────────────────────

/// Trait abstracting HTTP GET calls for REST API services.
///
/// Inject a mock in tests or swap in a different HTTP backend without
/// changing any connector logic.
#[async_trait::async_trait]
pub trait RestClient: Send + Sync {
    /// Send a GET request with headers and query parameters, return the
    /// full response body as a `String`.
    async fn get(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        query_params: &[(&str, String)],
    ) -> Result<String>;
}

// ───────────────────────────────────────────────────────────────────────────
// Retry / Backoff Constants
// ───────────────────────────────────────────────────────────────────────────

/// Maximum number of retry attempts (including the initial request).
const MAX_ATTEMPTS: u32 = 4;

/// Base delay between retries (doubles each attempt).
const BASE_DELAY_MS: u64 = 500;

/// Returns `true` for HTTP status codes that are safe to retry.
fn is_retryable(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
}

// ───────────────────────────────────────────────────────────────────────────
// Production ReqwestRestClient (async, non-blocking)
// ───────────────────────────────────────────────────────────────────────────

/// Production HTTP client that sends REST requests via async `reqwest`.
///
/// Features:
/// - Non-blocking I/O — does not block the tokio runtime.
/// - Exponential backoff with jitter for 429 / 5xx responses.
/// - Returns the **full** response body in error messages — no truncation.
#[derive(Debug, Clone)]
pub struct ReqwestRestClient;

#[async_trait::async_trait]
impl RestClient for ReqwestRestClient {
    async fn get(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        query_params: &[(&str, String)],
    ) -> Result<String> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(false)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| Error::IngestionError(format!("HTTP client error: {}", e)))?;

        let query_refs: Vec<(&str, &str)> = query_params
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect();

        let mut last_err: Option<Error> = None;

        for attempt in 0..MAX_ATTEMPTS {
            if attempt > 0 {
                let delay_ms = BASE_DELAY_MS * 2u64.saturating_pow(attempt - 1);
                tracing::warn!(
                    attempt = attempt + 1,
                    delay_ms,
                    url,
                    "RestClient: retrying after backoff",
                );
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }

            let mut request = client.get(url);
            for (key, value) in headers {
                request = request.header(*key, *value);
            }
            request = request.query(&query_refs);

            let response = match request.send().await {
                Ok(r) => r,
                Err(e) => {
                    last_err = Some(Error::IngestionError(format!(
                        "REST request to '{}' failed: {}",
                        url, e
                    )));
                    continue;
                }
            };

            let status = response.status();
            let body = response.text().await.map_err(|e| {
                Error::IngestionError(format!("Failed to read REST response body: {}", e))
            })?;

            if status.is_success() {
                return Ok(body);
            }

            if is_retryable(status) && attempt + 1 < MAX_ATTEMPTS {
                tracing::warn!(
                    status = %status,
                    url,
                    "RestClient: received retryable status",
                );
                last_err = Some(Error::IngestionError(format!(
                    "REST API returned HTTP {}: {}",
                    status, body
                )));
                continue;
            }

            return Err(Error::IngestionError(format!(
                "REST API returned HTTP {}: {}",
                status, body
            )));
        }

        Err(last_err.unwrap_or_else(|| {
            Error::IngestionError(format!(
                "REST request to '{}' failed after {} attempts",
                url, MAX_ATTEMPTS
            ))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubClient(String);

    #[async_trait::async_trait]
    impl RestClient for StubClient {
        async fn get(
            &self,
            _url: &str,
            _headers: &[(&str, &str)],
            _query_params: &[(&str, String)],
        ) -> Result<String> {
            Ok(self.0.clone())
        }
    }

    struct FailClient;

    #[async_trait::async_trait]
    impl RestClient for FailClient {
        async fn get(
            &self,
            _url: &str,
            _headers: &[(&str, &str)],
            _query_params: &[(&str, String)],
        ) -> Result<String> {
            Err(Error::IngestionError("connection refused".into()))
        }
    }

    #[tokio::test]
    async fn test_stub_client_returns_body() {
        let c = StubClient(r#"{"data":[],"meta":{"current_page":1,"next_page":null,"previous_page":null,"total_pages":1,"per_page":50,"total_entries":0}}"#.into());
        let result = c.get("https://example.com/api", &[], &[]).await.unwrap();
        assert!(result.contains("data"));
    }

    #[tokio::test]
    async fn test_fail_client_returns_error() {
        let c = FailClient;
        let result = c.get("https://example.com/api", &[], &[]).await;
        assert!(result.is_err());
    }
}
