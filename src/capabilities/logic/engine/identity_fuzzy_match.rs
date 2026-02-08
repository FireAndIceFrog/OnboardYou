//! Probabilistic matching for high-fidelity record merging
//!
//! ## Algorithm
//!
//! 1. Collect the DataFrame eagerly (fuzzy matching is inherently O(n²) over
//!    pairs, so we do it in Rust on the collected data rather than fighting
//!    Polars' lazy API).
//! 2. For every pair of rows, compute Levenshtein similarity on the
//!    concatenated `first_name + " " + last_name` string.
//! 3. If the similarity ≥ `threshold` (default 0.8), the records are linked
//!    into the same match group.
//! 4. Two new columns are appended:
//!    - `match_group_id`: string identifier for the fuzzy-match group.
//!    - `match_confidence`: the highest pairwise similarity score within the
//!      group (1.0 if unmatched / only member).
//!
//! Configurable via manifest JSON:
//! ```json
//! { "threshold": 0.85 }
//! ```

use crate::domain::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;

// ---------------------------------------------------------------------------
// Levenshtein helpers (pure Rust — no extra crate needed)
// ---------------------------------------------------------------------------

/// Classic Levenshtein edit distance.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();

    let mut matrix: Vec<Vec<usize>> = vec![vec![0; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[a_len][b_len]
}

/// Normalised similarity in \[0.0, 1.0\] (1.0 = identical).
fn similarity(a: &str, b: &str) -> f64 {
    let max_len = a.chars().count().max(b.chars().count());
    if max_len == 0 {
        return 1.0;
    }
    1.0 - (levenshtein_distance(a, b) as f64 / max_len as f64)
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for fuzzy matching.
#[derive(Debug, Clone)]
pub struct IdentityFuzzyMatchConfig {
    /// Minimum similarity (0.0–1.0) to consider two records a match.
    pub threshold: f64,
    /// Columns to concatenate for comparison (e.g. ["first_name", "last_name"]).
    pub columns: Vec<String>,
    /// The column holding the employee/entity identifier.
    pub employee_id_column: String,
}

impl Default for IdentityFuzzyMatchConfig {
    fn default() -> Self {
        Self {
            threshold: 0.80,
            columns: vec!["first_name".into(), "last_name".into()],
            employee_id_column: "employee_id".into(),
        }
    }
}

impl IdentityFuzzyMatchConfig {
    pub fn from_json(value: &serde_json::Value) -> Self {
        let threshold = value
            .get("threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.80);

        let columns = value
            .get("columns")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_else(|| vec!["first_name".into(), "last_name".into()]);

        let employee_id_column = value
            .get("employee_id_column")
            .and_then(|v| v.as_str())
            .unwrap_or("employee_id")
            .to_string();

        Self {
            threshold,
            columns,
            employee_id_column,
        }
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Fuzzy matching for probabilistic identity resolution.
#[derive(Debug, Clone)]
pub struct IdentityFuzzyMatch {
    config: IdentityFuzzyMatchConfig,
}

impl IdentityFuzzyMatch {
    pub fn new(config: IdentityFuzzyMatchConfig) -> Self {
        Self { config }
    }

    pub fn from_action_config(value: &serde_json::Value) -> Self {
        Self::new(IdentityFuzzyMatchConfig::from_json(value))
    }
}

impl Default for IdentityFuzzyMatch {
    fn default() -> Self {
        Self::new(IdentityFuzzyMatchConfig::default())
    }
}

impl OnboardingAction for IdentityFuzzyMatch {
    fn id(&self) -> &str {
        "identity_fuzzy_match"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            threshold = self.config.threshold,
            columns = ?self.config.columns,
            "IdentityFuzzyMatch: running"
        );

        // --- Collect eagerly (fuzzy matching needs random access) -----------
        let lf = std::mem::replace(&mut context.data, LazyFrame::default());
        let df = lf.collect().map_err(|e| {
            Error::LogicError(format!("Failed to collect for fuzzy match: {}", e))
        })?;

        let schema = df.schema();

        // Filter configured columns to those present in the data
        let available_columns: Vec<&String> = self
            .config
            .columns
            .iter()
            .filter(|c| schema.contains(c.as_str()))
            .collect();

        if available_columns.is_empty() {
            tracing::warn!(
                configured = ?self.config.columns,
                "IdentityFuzzyMatch: none of the configured columns found — skipping"
            );
            context.data = df.lazy();
            return Ok(context);
        }

        let n = df.height();

        // Pre-extract string chunked arrays for the configured columns
        let column_arrays: Vec<&StringChunked> = available_columns
            .iter()
            .map(|name| df.column(name.as_str()).unwrap().str().unwrap())
            .collect();

        // Build composite strings for comparison by concatenating configured columns
        let composite_strings: Vec<String> = (0..n)
            .map(|i| {
                let parts: Vec<&str> = column_arrays
                    .iter()
                    .map(|arr| arr.get(i).unwrap_or(""))
                    .collect();
                parts.join(" ").to_lowercase()
            })
            .collect();

        // --- Union-Find for grouping ------------------------------------
        let mut parent: Vec<usize> = (0..n).collect();
        let mut best_score: Vec<f64> = vec![1.0; n];

        fn find(parent: &mut Vec<usize>, i: usize) -> usize {
            if parent[i] != i {
                parent[i] = find(parent, parent[i]);
            }
            parent[i]
        }

        fn union(parent: &mut Vec<usize>, a: usize, b: usize) {
            let ra = find(parent, a);
            let rb = find(parent, b);
            if ra != rb {
                parent[rb] = ra;
            }
        }

        for i in 0..n {
            for j in (i + 1)..n {
                let sim = similarity(&composite_strings[i], &composite_strings[j]);
                if sim >= self.config.threshold {
                    union(&mut parent, i, j);
                    if sim > best_score[i] {
                        best_score[i] = sim;
                    }
                    if sim > best_score[j] {
                        best_score[j] = sim;
                    }
                }
            }
        }

        // Resolve final groups
        let employee_ids = df
            .column(self.config.employee_id_column.as_str())
            .map_err(|e| {
                Error::LogicError(format!(
                    "Employee ID column '{}' not found: {}",
                    self.config.employee_id_column, e
                ))
            })?
            .str()
            .unwrap();
        let group_ids: Vec<String> = (0..n)
            .map(|i| {
                let root = find(&mut parent, i);
                format!("grp_{}", employee_ids.get(root).unwrap_or("unknown"))
            })
            .collect();

        let confidences: Vec<f64> = (0..n)
            .map(|i| {
                let root = find(&mut parent, i);
                best_score[root]
            })
            .collect();

        // --- Append columns to DataFrame --------------------------------
        let group_series = Column::new("match_group_id".into(), group_ids);
        let conf_series = Column::new("match_confidence".into(), confidences);

        let df = df
            .hstack(&[group_series, conf_series])
            .map_err(|e| Error::LogicError(format!("Failed to append columns: {}", e)))?;

        for col_name in ["match_group_id", "match_confidence"] {
            context.set_field_source(col_name.to_string(), "LOGIC_ACTION".into());
            context.mark_field_modified(col_name.to_string(), "identity_fuzzy_match".into());
        }

        context.data = df.lazy();
        Ok(context)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert!((similarity("hello", "hello") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_levenshtein_one_edit() {
        assert_eq!(levenshtein_distance("kitten", "sitten"), 1);
    }

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert!((similarity("", "") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_identity_fuzzy_match_id() {
        let action = IdentityFuzzyMatch::default();
        assert_eq!(action.id(), "identity_fuzzy_match");
    }

    #[test]
    fn test_fuzzy_groups_similar_names() {
        // "Jon Doe" and "John Doe" should fuzzy-match with default threshold 0.8
        let df = df! {
            "employee_id" => &["001", "002", "003"],
            "first_name"  => &["John", "Jon",  "Alice"],
            "last_name"   => &["Doe",  "Doe",  "Wonder"],
        }
        .unwrap();

        let ctx = RosterContext::new(df.lazy());
        let action = IdentityFuzzyMatch::default();
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        assert!(df.column("match_group_id").is_ok());
        assert!(df.column("match_confidence").is_ok());

        let groups: Vec<Option<&str>> = df
            .column("match_group_id").unwrap()
            .str().unwrap()
            .into_iter()
            .collect();

        // 001 and 002 should be in the same group, 003 in its own
        assert_eq!(groups[0], groups[1], "John Doe and Jon Doe should match");
        assert_ne!(groups[0], groups[2], "Alice Wonder should be separate");
    }

    #[test]
    fn test_no_matches_with_high_threshold() {
        let df = df! {
            "employee_id" => &["001", "002"],
            "first_name"  => &["John", "Alice"],
            "last_name"   => &["Doe",  "Wonder"],
        }
        .unwrap();

        let config = IdentityFuzzyMatchConfig {
            threshold: 0.99,
            ..Default::default()
        };
        let ctx = RosterContext::new(df.lazy());
        let action = IdentityFuzzyMatch::new(config);
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let groups: Vec<Option<&str>> = df
            .column("match_group_id").unwrap()
            .str().unwrap()
            .into_iter()
            .collect();

        // Each record should be its own group
        assert_ne!(groups[0], groups[1]);
    }

    #[test]
    fn test_confidence_scores() {
        let df = df! {
            "employee_id" => &["001", "002"],
            "first_name"  => &["John", "John"],
            "last_name"   => &["Doe",  "Doe"],
        }
        .unwrap();

        let ctx = RosterContext::new(df.lazy());
        let action = IdentityFuzzyMatch::default();
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let confs: Vec<Option<f64>> = df
            .column("match_confidence").unwrap()
            .f64().unwrap()
            .into_iter()
            .collect();

        // Identical names → confidence = 1.0
        assert!((confs[0].unwrap() - 1.0).abs() < f64::EPSILON);
        assert!((confs[1].unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_no_configured_columns_skips() {
        let df = df! {
            "employee_id" => &["001"],
            "salary"      => &[50_000i64],
        }
        .unwrap();

        let ctx = RosterContext::new(df.lazy());
        let action = IdentityFuzzyMatch::default();
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        assert!(df.column("match_group_id").is_err());
    }

    #[test]
    fn test_field_metadata() {
        let df = df! {
            "employee_id" => &["001"],
            "first_name"  => &["John"],
            "last_name"   => &["Doe"],
        }
        .unwrap();

        let ctx = RosterContext::new(df.lazy());
        let action = IdentityFuzzyMatch::default();
        let result = action.execute(ctx).expect("execute");

        for col_name in ["match_group_id", "match_confidence"] {
            let meta = result.field_metadata.get(col_name)
                .unwrap_or_else(|| panic!("metadata for '{}'", col_name));
            assert_eq!(meta.source, "LOGIC_ACTION");
            assert_eq!(meta.modified_by.as_deref(), Some("identity_fuzzy_match"));
        }
    }

    #[test]
    fn test_from_action_config() {
        let json = serde_json::json!({ "threshold": 0.90 });
        let action = IdentityFuzzyMatch::from_action_config(&json);
        assert!((action.config.threshold - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn test_config_from_json_with_columns() {
        let json = serde_json::json!({
            "threshold": 0.85,
            "columns": ["given_name", "surname"],
            "employee_id_column": "emp_id"
        });
        let config = IdentityFuzzyMatchConfig::from_json(&json);
        assert!((config.threshold - 0.85).abs() < f64::EPSILON);
        assert_eq!(config.columns, vec!["given_name", "surname"]);
        assert_eq!(config.employee_id_column, "emp_id");
    }

    #[test]
    fn test_config_defaults() {
        let json = serde_json::json!({});
        let config = IdentityFuzzyMatchConfig::from_json(&json);
        assert_eq!(config.columns, vec!["first_name", "last_name"]);
        assert_eq!(config.employee_id_column, "employee_id");
    }

    #[test]
    fn test_custom_columns() {
        let df = df! {
            "id"          => &["001", "002", "003"],
            "given_name"  => &["John", "Jon",  "Alice"],
            "surname"     => &["Doe",  "Doe",  "Wonder"],
        }
        .unwrap();

        let config = IdentityFuzzyMatchConfig {
            threshold: 0.80,
            columns: vec!["given_name".into(), "surname".into()],
            employee_id_column: "id".into(),
        };
        let ctx = RosterContext::new(df.lazy());
        let action = IdentityFuzzyMatch::new(config);
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let groups: Vec<Option<&str>> = df
            .column("match_group_id").unwrap()
            .str().unwrap()
            .into_iter()
            .collect();

        assert_eq!(groups[0], groups[1], "John Doe and Jon Doe should match");
        assert_ne!(groups[0], groups[2], "Alice Wonder should be separate");
    }
}
