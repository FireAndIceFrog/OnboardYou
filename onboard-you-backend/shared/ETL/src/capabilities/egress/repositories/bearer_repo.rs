//! Bearer repository: Covers static bearer tokens, custom API keys, and no-auth
//!
//! This is the simplest egress auth strategy. The token (if any) is provided
//! at construction time and returned verbatim on every `retrieve_token` call.
//! No refresh logic is needed — the caller is responsible for rotating the
//! value in the manifest / DynamoDB config when it expires.

use models::{BearerPlacement, BearerRepoConfig, DispatchResponse};
use crate::capabilities::egress::traits::EgressRepository;
use models::{Error, Result};

// ---------------------------------------------------------------------------
// Repository
// ---------------------------------------------------------------------------

/// Egress repository for static bearer tokens, API keys, and no-auth.
#[derive(Debug, Clone)]
pub struct BearerRepo {
    config: BearerRepoConfig,
}

impl BearerRepo {
    pub fn new(config: BearerRepoConfig) -> Self {
        Self { config }
    }

    pub fn from_action_config(config: &BearerRepoConfig) -> Result<Self> {
        Ok(Self::new(config.clone()))
    }
}

impl EgressRepository for BearerRepo {
    fn retrieve_token(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<String>>> + Send + '_>>
    {
        let token = self.config.token.clone();
        Box::pin(async move { Ok(token) })
    }

    fn send_data(
        &self,
        payload: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<DispatchResponse>> + Send + '_>>
    {
        let destination_url = self.config.destination_url.clone();
        let placement = self.config.placement.clone();
        let token = self.config.token.clone();
        let placement_key = self.config.placement_key.clone();
        let extra_headers = self.config.extra_headers.clone();
        let payload = payload.to_string();

        Box::pin(async move {
            let client = reqwest::Client::new();
            let mut request = client.post(&destination_url);

            // Attach authentication
            match (&placement, &token) {
                (BearerPlacement::AuthorizationHeader, Some(t)) => {
                    request = request.header("Authorization", format!("Bearer {t}"));
                }
                (BearerPlacement::CustomHeader, Some(t)) => {
                    let name = placement_key.as_deref().unwrap_or("X-API-Key");
                    request = request.header(name, t.as_str());
                }
                (BearerPlacement::QueryParam, Some(t)) => {
                    let param = placement_key.as_deref().unwrap_or("api_key");
                    request = request.query(&[(param, t.as_str())]);
                }
                (_, None) => { /* No auth */ }
            }

            // Attach extra headers
            for (key, value) in &extra_headers {
                request = request.header(key.as_str(), value.as_str());
            }

            // Send payload
            request = request
                .header("Content-Type", "application/json")
                .body(payload.clone());

            let response = request
                .send()
                .await
                .map_err(|e| Error::EgressError(format!("Bearer dispatch failed: {e}")))?;

            let status_code = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            let records_sent = serde_json::from_str::<Vec<serde_json::Value>>(&payload)
                .map(|v| v.len())
                .unwrap_or(1);

            Ok(DispatchResponse {
                status_code,
                body,
                records_sent,
            })
        })
    }
}
