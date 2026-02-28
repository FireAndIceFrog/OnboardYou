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
use models::ApiDispatcherConfig;
use models::ColumnCalculator;
use models::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;
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

    /// Build the engine from a typed `ApiDispatcherConfig`.
    pub fn from_action_config(config: &ApiDispatcherConfig) -> Result<Self> {
        let engine = ApiEngine::from_action_config(config)?;
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

        // 2. Serialize to JSON: { "data": [ {col: value, …}, … ] }
        let payload = dataframe_to_json_payload(&df)?;

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

// ---------------------------------------------------------------------------
// DataFrame → JSON helper
// ---------------------------------------------------------------------------

/// Convert a Polars `DataFrame` into a JSON payload shaped as:
///
/// ```json
/// {
///   "data": [
///     {"column_a": "value1", "column_b": 42},
///     {"column_a": "value2", "column_b": 99}
///   ]
/// }
/// ```
///
/// Each row becomes a `serde_json::Map` keyed by column name.
fn dataframe_to_json_payload(df: &DataFrame) -> Result<String> {
    let col_names = df.get_column_names();
    let height = df.height();

    let mut rows: Vec<serde_json::Value> = Vec::with_capacity(height);

    for row_idx in 0..height {
        let mut map = serde_json::Map::with_capacity(col_names.len());

        for &name in &col_names {
            let series = df
                .column(name)
                .map_err(|e| Error::EgressError(format!("Column '{name}' not found: {e}")))?;
            let av = series.get(row_idx).map_err(|e| {
                Error::EgressError(format!(
                    "Failed to read row {row_idx}, column '{name}': {e}"
                ))
            })?;
            map.insert(name.to_string(), anyvalue_to_json(av));
        }

        rows.push(serde_json::Value::Object(map));
    }

    let envelope = serde_json::json!({ "data": rows });

    serde_json::to_string(&envelope).map_err(|e| Error::SerializationError(e))
}

/// Map a Polars `AnyValue` to a `serde_json::Value`.
fn anyvalue_to_json(av: AnyValue<'_>) -> serde_json::Value {
    match av {
        AnyValue::Null => serde_json::Value::Null,
        AnyValue::Boolean(b) => serde_json::Value::Bool(b),
        AnyValue::Int8(n) => serde_json::json!(n),
        AnyValue::Int16(n) => serde_json::json!(n),
        AnyValue::Int32(n) => serde_json::json!(n),
        AnyValue::Int64(n) => serde_json::json!(n),
        AnyValue::UInt8(n) => serde_json::json!(n),
        AnyValue::UInt16(n) => serde_json::json!(n),
        AnyValue::UInt32(n) => serde_json::json!(n),
        AnyValue::UInt64(n) => serde_json::json!(n),
        AnyValue::Float32(f) => serde_json::json!(f),
        AnyValue::Float64(f) => serde_json::json!(f),
        AnyValue::String(s) => serde_json::Value::String(s.to_string()),
        AnyValue::StringOwned(s) => serde_json::Value::String(s.to_string()),
        // Fall back to the Display impl for dates, datetimes, durations, etc.
        other => serde_json::Value::String(format!("{other}")),
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
        let cfg: ApiDispatcherConfig = serde_json::from_value(serde_json::json!({
            "auth_type": "bearer",
            "destination_url": "https://api.example.com/employees",
            "token": "test-token"
        }))
        .unwrap();

        let dispatcher = ApiDispatcher::from_action_config(&cfg);
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

    #[test]
    fn test_dataframe_to_json_single_row() {
        let df = df!(
            "employee_id" => ["E001"],
            "first_name"  => ["Alice"],
            "salary"      => [85_000i64]
        )
        .unwrap();

        let json_str = dataframe_to_json_payload(&df).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let data = parsed.get("data").unwrap().as_array().unwrap();
        assert_eq!(data.len(), 1);

        let row = &data[0];
        assert_eq!(row["employee_id"], "E001");
        assert_eq!(row["first_name"], "Alice");
        assert_eq!(row["salary"], 85_000);
    }

    #[test]
    fn test_dataframe_to_json_multiple_rows() {
        let df = df!(
            "id"   => ["E001", "E002", "E003"],
            "name" => ["Alice", "Bob", "Carol"],
            "active" => [true, false, true]
        )
        .unwrap();

        let json_str = dataframe_to_json_payload(&df).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let data = parsed.get("data").unwrap().as_array().unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data[1]["name"], "Bob");
        assert_eq!(data[2]["active"], true);
    }

    #[test]
    fn test_dataframe_to_json_null_handling() {
        let df = df!(
            "name"  => [Some("Alice"), None, Some("Carol")],
            "score" => [Some(100i64), Some(200i64), None]
        )
        .unwrap();

        let json_str = dataframe_to_json_payload(&df).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let data = parsed.get("data").unwrap().as_array().unwrap();
        assert!(data[1]["name"].is_null());
        assert!(data[2]["score"].is_null());
        assert_eq!(data[0]["name"], "Alice");
    }

    #[test]
    fn test_dataframe_to_json_empty() {
        let df = df!(
            "col_a" => Vec::<String>::new(),
            "col_b" => Vec::<i64>::new()
        )
        .unwrap();

        let json_str = dataframe_to_json_payload(&df).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let data = parsed.get("data").unwrap().as_array().unwrap();
        assert!(data.is_empty());
    }
}
