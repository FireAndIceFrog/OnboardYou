//! Simple OpenAPI repository for the ETL trigger lambda.

use async_trait::async_trait;
use lambda_runtime::Error;
use std::sync::Arc;

// HTTP client used to pull the swagger/openapi JSON
use reqwest::Client;

/// Abstraction for fetching JSON from a URL.
#[async_trait]
pub trait OpenApiRepo: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<String, Error>;
}

/// Simplest HTTP-backed implementation.
///
/// In production this simply does a `GET` against the provided URL and
/// returns the response body as a string.  In tests we drive it through
/// `mockito` so we can exercise the network behaviour without leaving the
/// repo.
pub struct SimpleOpenApiRepo {
    client: Client,
}

impl SimpleOpenApiRepo {
    /// Create a new repo using the provided HTTP client.
    pub fn new(client: Client) -> Arc<Self> {
        Arc::new(Self { client })
    }
}

#[async_trait]
impl OpenApiRepo for SimpleOpenApiRepo {
    async fn fetch(&self, url: &str) -> Result<String, Error> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::from(format!("failed to GET openapi url {url}: {e}")))?;

        let text = resp
            .text()
            .await
            .map_err(|e| Error::from(format!("failed to read response body: {e}")))?;

        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // `mockito` provides an in‑process HTTP server we can control.  Dev
    // dependency declared in Cargo.toml.
    use mockito::mock;

    #[tokio::test]
    async fn test_fetch_returns_remote_json() {
        let m = mock("GET", "/swagger.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"hello":"world"}"#)
            .create();

        let client = reqwest::Client::new();
        let repo = SimpleOpenApiRepo::new(client);
        let url = format!("{}/swagger.json", &mockito::server_url());

        let res = repo.fetch(&url).await.unwrap();
        assert_eq!(res, r#"{"hello":"world"}"#);

        m.assert();
    }
}
