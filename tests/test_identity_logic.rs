//! Verifies identity resolution engine accuracy with complex duplicates

mod common;

use onboard_you::{ActionFactory, ActionConfig, RosterContext};
use polars::prelude::*;

#[test]
fn test_identity_resolution_basic() {
    // Create test data with duplicate email records
    let df = df! {
        "employee_id" => &["001", "002", "003", "004"],
        "first_name"  => &["John", "John", "Jane", "Alice"],
        "last_name"   => &["Doe",  "Doe",  "Smith", "Jones"],
        "email"       => &["john@co.com", "john@co.com", "jane@co.com", "alice@co.com"],
        "salary"      => &[70_000i64, 72_000, 85_000, 92_000],
    }
    .expect("test df");

    // Resolve the deduplicator through the factory (integration!)
    let config = ActionConfig {
        id: "dedup".into(),
        action_type: "identity_deduplicator".into(),
        config: serde_json::json!({}),
    };
    let action = ActionFactory::create(&config).expect("create deduplicator");
    assert_eq!(action.id(), "identity_deduplicator");

    let ctx = RosterContext::new(df.lazy());
    let result = action.execute(ctx).expect("execute");
    let df = result.data.collect().expect("collect");

    // 001 and 002 share email → 002 is duplicate
    let is_dup: Vec<Option<bool>> = df
        .column("is_duplicate").unwrap()
        .bool().unwrap()
        .into_iter()
        .collect();
    assert_eq!(is_dup, vec![Some(false), Some(true), Some(false), Some(false)]);

    // canonical_id for 001 and 002 should both be "001"
    let canonical: Vec<Option<&str>> = df
        .column("canonical_id").unwrap()
        .str().unwrap()
        .into_iter()
        .collect();
    assert_eq!(canonical[0], Some("001"));
    assert_eq!(canonical[1], Some("001"));
}

#[test]
fn test_identity_fuzzy_match() {
    // "Jon Doe" is a fuzzy match to "John Doe"
    let df = df! {
        "employee_id" => &["001", "002", "003"],
        "first_name"  => &["John",  "Jon",   "Alice"],
        "last_name"   => &["Doe",   "Doe",   "Wonder"],
    }
    .expect("test df");

    let config = ActionConfig {
        id: "fuzzy".into(),
        action_type: "identity_fuzzy_match".into(),
        config: serde_json::json!({ "threshold": 0.80 }),
    };
    let action = ActionFactory::create(&config).expect("create fuzzy match");
    assert_eq!(action.id(), "identity_fuzzy_match");

    let ctx = RosterContext::new(df.lazy());
    let result = action.execute(ctx).expect("execute");
    let df = result.data.collect().expect("collect");

    let groups: Vec<Option<&str>> = df
        .column("match_group_id").unwrap()
        .str().unwrap()
        .into_iter()
        .collect();

    // John Doe and Jon Doe should be in the same group
    assert_eq!(groups[0], groups[1], "similar names should match");
    // Alice Wonder should be separate
    assert_ne!(groups[0], groups[2], "different names should not match");

    let confs: Vec<Option<f64>> = df
        .column("match_confidence").unwrap()
        .f64().unwrap()
        .into_iter()
        .collect();
    // Matched records should have confidence > threshold
    assert!(confs[0].unwrap() >= 0.80);
}
