//! LLM repository — wraps AI/LLM calls behind a testable trait.

use async_trait::async_trait;
use gh_models::types::ChatMessage;
use gh_models::GHModels;
use lambda_runtime::Error;
use reqwest::StatusCode;
use std::error::Error as StdError;
use std::sync::Arc;

/// Repository trait for LLM interactions.
///
/// Abstracting AI calls behind a trait allows the engine to be tested
/// with deterministic fakes — no network calls required.
#[async_trait]
pub trait ILlmRepo: Send + Sync {
    /// Send a chat completion request and return the raw text response.
    async fn chat_completion(
        &self,
        model: &str,
        messages: &[ChatMessage],
        temperature: f32,
        max_tokens: usize,
        top_p: f32,
    ) -> Result<String, Error>;
}

/// Concrete implementation backed by `GHModels`.
pub struct GHModelsLlmRepository {
    pub client: GHModels,
}

impl GHModelsLlmRepository {
    pub fn new(github_token: String) -> Arc<Self> {
        Arc::new(Self {
            client: GHModels::new(github_token),
        })
    }
}

#[async_trait]
impl ILlmRepo for GHModelsLlmRepository {
    async fn chat_completion(
        &self,
        model: &str,
        messages: &[ChatMessage],
        temperature: f32,
        max_tokens: usize,
        top_p: f32,
    ) -> Result<String, Error> {
        let response = self
            .client
            .chat_completion(model, messages, temperature, max_tokens, top_p)
            .await
            .map_err(|e: reqwest::Error| Error::from(format!("AI API call failed: {e} status = {status} source = {source}", 
                status = e.status().unwrap_or(StatusCode::NO_CONTENT),
                source = e.source().map(|s| s.to_string()).unwrap_or_else(|| "No additional error info".to_string())
            )))?;

        let content = response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| Error::from("AI returned empty response"))?;

        Ok(content)
    }
}
