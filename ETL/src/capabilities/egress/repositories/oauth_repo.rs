//! OAuth repository: OAuth 1.0a authentication flow
//!
//! Handles the three-legged OAuth 1.0a signing process used by some legacy
//! APIs. The consumer key/secret and access token/secret are stored in config
//! (fetched from DynamoDB / KeyVault at startup). Each outgoing request is
//! signed with an HMAC-SHA1 signature per the OAuth 1.0a spec.
//!
//! # Future: DynamoDB / KeyVault integration
//!
//! Secrets (`consumer_secret`, `token_secret`) will be encrypted per-org in
//! DynamoDB and decrypted at construction time via a `SecretProvider` trait.

use crate::capabilities::egress::models::{DispatchResponse, OAuthRepoConfig};
use crate::capabilities::egress::traits::EgressRepository;
use crate::domain::{Error, Result};

// ---------------------------------------------------------------------------
// Repository
// ---------------------------------------------------------------------------

/// Egress repository implementing OAuth 1.0a request signing.
#[derive(Debug, Clone)]
pub struct OAuthRepo {
    config: OAuthRepoConfig,
}

impl OAuthRepo {
    pub fn new(config: OAuthRepoConfig) -> Self {
        Self { config }
    }

    pub fn from_action_config(value: &serde_json::Value) -> Result<Self> {
        let config = OAuthRepoConfig::from_json(value)?;
        Ok(Self::new(config))
    }

    /// Build the OAuth 1.0a `Authorization` header value.
    ///
    /// TODO: Implement full HMAC-SHA1 signature base string generation.
    /// For now returns a placeholder that captures the structure.
    fn build_authorization_header(&self, _method: &str, _url: &str) -> String {
        // In production this will:
        // 1. Generate nonce + timestamp
        // 2. Build signature base string (method, url, sorted params)
        // 3. Sign with HMAC-SHA1 using consumer_secret & token_secret
        // 4. Return the full Authorization header value
        format!(
            "OAuth oauth_consumer_key=\"{}\", oauth_token=\"{}\", \
             oauth_signature_method=\"HMAC-SHA1\", oauth_signature=\"TODO\", \
             oauth_timestamp=\"0\", oauth_nonce=\"TODO\", oauth_version=\"1.0\"",
            self.config.consumer_key, self.config.access_token
        )
    }
}

impl EgressRepository for OAuthRepo {
    fn retrieve_token(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<String>>> + Send + '_>>
    {
        // OAuth 1.0a doesn't use bearer tokens — auth is per-request signing.
        // Return the access_token for observability / logging purposes.
        let token = self.config.access_token.clone();
        Box::pin(async move { Ok(Some(token)) })
    }

    fn send_data(
        &self,
        payload: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<DispatchResponse>> + Send + '_>,
    > {
        let destination_url = self.config.destination_url.clone();
        let auth_header = self.build_authorization_header("POST", &destination_url);
        let payload = payload.to_string();

        Box::pin(async move {
            let client = reqwest::Client::new();
            let response = client
                .post(&destination_url)
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(payload.clone())
                .send()
                .await
                .map_err(|e| Error::EgressError(format!("OAuth dispatch failed: {e}")))?;

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

