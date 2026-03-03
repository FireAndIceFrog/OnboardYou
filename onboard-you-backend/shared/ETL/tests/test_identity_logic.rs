//! Verifies identity resolution engine accuracy with complex duplicates

mod common;

use onboard_you::*;
use onboard_you_models::{ActionConfig, ActionConfigPayload, ActionType, RosterContext};
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
        action_type: ActionType::IdentityDeduplicator,
        config: ActionConfigPayload::IdentityDeduplicator(
            serde_json::from_value(serde_json::json!({ "columns": ["email"] })).unwrap(),
        ),
        disabled: false,
    };
    let action = ActionFactory::new()
        .create(&config)
        .expect("create deduplicator");
    assert_eq!(action.id(), "identity_deduplicator");

    let ctx = RosterContext::new(df.lazy());
    let result = action.execute(ctx).expect("execute");
    let df = result.data.collect().expect("collect");

    // 001 and 002 share email → 002 is duplicate
    let is_dup: Vec<Option<bool>> = df
        .column("is_duplicate")
        .unwrap()
        .bool()
        .unwrap()
        .into_iter()
        .collect();
    assert_eq!(
        is_dup,
        vec![Some(false), Some(true), Some(false), Some(false)]
    );

    // canonical_id for 001 and 002 should both be "001"
    let canonical: Vec<Option<&str>> = df
        .column("canonical_id")
        .unwrap()
        .str()
        .unwrap()
        .into_iter()
        .collect();
    assert_eq!(canonical[0], Some("001"));
    assert_eq!(canonical[1], Some("001"));
}
