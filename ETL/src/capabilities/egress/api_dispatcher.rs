//! HTTP/JSON delivery to client-facing destination APIs
//!
//! The `ApiDispatcher` is the `OnboardingAction` that sits at the end of
//! the pipeline. It delegates all real work to the `ApiEngine`, which
//! orchestrates authentication (Bearer / OAuth / OAuth2) and HTTP dispatch
//! with retries.
//!
//! The pipeline is sync (`OnboardingAction::execute`), but the engine's
//! internals are async — bridged via `tokio::runtime::Handle::block_on`.

use crate::capabilities::egress::engine::api_engine::ApiEngine;
use crate::capabilities::logic::traits::ColumnCalculator;
use crate::domain::{Error, OnboardingAction, Result, RosterContext};
use tracing::{info, warn};

/// API dispatcher for sending data to destination systems.
///
/// Wraps an `ApiEngine` that handles auth strategy selection, token
/// retrieval, payload delivery, and retry logic.
pub struct ApiDispatcher {
    engine: Option<ApiEngine>,
}

impl ApiDispatcher {
    /// Create an unconfigured dispatcher (engine will be built from manifest
    /// config at execution time).
    pub fn new() -> Self {
        Self { engine: None }
    }

    /// Create a dispatcher with a pre-built engine.
    pub fn with_engine(engine: ApiEngine) -> Self {
        Self {
            engine: Some(engine),
        }
    }

    /// Build the engine from manifest config JSON.
    pub fn from_action_config(value: &serde_json::Value) -> Result<Self> {
        let engine = ApiEngine::from_action_config(value)?;
        Ok(Self {
            engine: Some(engine),
        })
    }
}

impl Default for ApiDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ColumnCalculator for ApiDispatcher {
    fn calculate_columns(&self, context: RosterContext) -> Result<RosterContext> {
        // Egress does not alter the schema — pass through unchanged.
        Ok(context)
    }
}

impl OnboardingAction for ApiDispatcher {
    fn id(&self) -> &str {
        "api_dispatcher"
    }

    fn execute(&self, context: RosterContext) -> Result<RosterContext> {
        let engine = self.engine.as_ref().ok_or_else(|| {
            Error::ConfigurationError(
                "ApiDispatcher has no engine configured. \
                 Use from_action_config() or with_engine() before executing."
                    .into(),
            )
        })?;

        // 1. Collect the LazyFrame into a DataFrame
        let df = context
            .data
            .clone()
            .collect()
            .map_err(|e| Error::EgressError(format!("Failed to collect LazyFrame: {e}")))?;

        // 2. Serialize to JSON array of row objects
        let mut buf = Vec::new();
        serde_json::to_writer(&mut buf, &df.shape()).map_err(|e| {
            Error::SerializationError(e)
        })?;

        // TODO: Replace with proper row-level JSON serialization once
        // the Polars DataFrame → JSON helper is wired up. For now we
        // serialise the shape as a placeholder.
        let payload = String::from_utf8_lossy(&buf).to_string();

        info!(
            records = df.height(),
            columns = df.width(),
            "Dispatching data via ApiEngine"
        );

        // 3. Dispatch through the engine (sync boundary → async internals)
        let response = engine.dispatch(&payload)?;

        if response.status_code >= 400 {
            warn!(
                status_code = response.status_code,
                body = %response.body,
                "Destination returned error status"
            );
        } else {
            info!(
                status_code = response.status_code,
                records_sent = response.records_sent,
                "Dispatch successful"
            );
        }

        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::IntoLazy;

    #[test]
    fn test_api_dispatcher_id() {
        let action = ApiDispatcher::new();
        assert_eq!(action.id(), "api_dispatcher");
    }

    #[test]
    fn test_api_dispatcher_no_engine_errors() {
        let action = ApiDispatcher::new();
        let context = RosterContext::new(polars::prelude::df!("a" => [1]).unwrap().lazy());
        let result = action.execute(context);
        assert!(result.is_err());
    }

    #[test]
    fn test_api_dispatcher_from_config() {
        let json = serde_json::json!({
            "auth_type": "bearer",
            "destination_url": "https://api.example.com/employees",
            "token": "test-token"
        });

        let dispatcher = ApiDispatcher::from_action_config(&json);
        assert!(dispatcher.is_ok());
        assert_eq!(dispatcher.unwrap().id(), "api_dispatcher");
    }

    #[test]
    fn test_column_calculator_passthrough() {
        let action = ApiDispatcher::new();
        let context = RosterContext::new(polars::prelude::df!("a" => [1]).unwrap().lazy());
        let result = action.calculate_columns(context);
        assert!(result.is_ok());
    }
}
