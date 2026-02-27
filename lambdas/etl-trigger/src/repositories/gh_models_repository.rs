//! GH Models repository trait and simple echo implementation.

use async_trait::async_trait;
use lambda_runtime::Error;
use std::sync::Arc;
use gh_models::{GHModels, types::ChatMessage};

use crate::models::OpenapiDynamicApiResponse;

/// Abstracts generating dynamic bodies using a GH models service.
#[async_trait]
pub trait GhModelsRepo: Send + Sync {
    /// Generate a dynamic body from an input string.
    async fn generate_dynamic_body(&self, input: &str) -> Result<OpenapiDynamicApiResponse, Error>;
}

/// A trivial implementation that just echoes back the input.
pub struct GhModelsRepoImpl {
    client: Arc<GHModels>,
}

impl GhModelsRepoImpl {
    pub fn new(client: Arc<GHModels>) -> Arc<Self> {
        Arc::new(Self { client })
    }
}

#[async_trait]
impl GhModelsRepo for GhModelsRepoImpl {
    async fn generate_dynamic_body(&self, input: &str) -> Result<OpenapiDynamicApiResponse, Error> {
        // conversation history we will send to the model; the first message is
        // a system prompt describing our goal. subsequent entries are appended
        // as we iterate.
        let mut history: Vec<gh_models::types::ChatMessage> = vec![
            gh_models::types::ChatMessage {
                role: "system".into(),
                content: "You are a Rust code assistant.  Given an OpenAPI schema
string from an external service, construct a JSON object representing an
`OpenapiDynamicApiResponse` struct.  The JSON must contain exactly two
fields: `output_schema` (a valid JSON value) and
`output_schema_body_path` (a string path such as `data.items`).  Your
response should contain nothing but the JSON object.  If the Rust program
fails to deserialize the JSON, we will give you an error message describing the failure; in that case, 
provide a corrected JSON.  Keep all previous attempts in the
conversation history."
                    .to_string(),
            },
        ];

        // initial user message contains the schema we were given
        history.push(ChatMessage {
            role: "user".into(),
            content: format!("Here is the OpenAPI schema:\n{}", input),
        });

        for attempt in 0..5 {
            let resp = self
                .client
                .chat_completion("openai/gpt-4o", &history, 1.0, 16384, 1.0)
                .await
                .map_err(|e| Error::from(e))?;
            
            let assistant_msg = resp
                .choices
                .get(0)
                .ok_or_else(|| Error::from("empty response from model"))?
                .message
                .clone();

            // add the assistant's reply to the conversation so that further
            // rounds include previous attempts
            history.push(assistant_msg.clone());

            // try to deserialize the content; if it succeeds we are done,
            // otherwise append an error message and loop again
            match serde_json::from_str::<OpenapiDynamicApiResponse>(&assistant_msg.content) {
                Ok(val) => return Ok(val),
                Err(err) => {

                    tracing::warn!(attempt = %attempt,"Failed to run ai over OpenAPI schema. Attempting to recover with error message: {}", err);
                    history.push(ChatMessage {
                        role: "user".into(),
                        content: format!(
                            "The JSON above failed to deserialize into
`OpenapiDynamicApiResponse`: {}.  Please provide a corrected JSON
object only.",
                            err
                        ),
                    });
                    // continue loop
                }
            }
        }
        Err(Error::from("Failed to generate valid JSON after 5 attempts"))
    }
}