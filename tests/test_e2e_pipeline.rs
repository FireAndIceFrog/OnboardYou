//! End-to-End: Verifies a full run from CSV ingestion through the pipeline

mod common;

use common::*;
use onboard_you::{ActionFactory, Manifest, PipelineRunner, RosterContext};
use polars::prelude::*;

#[test]
fn test_e2e_csv_ingestion_pipeline() {
    // 1. Write sample CSV to a temp file
    let (_tmp, csv_path) = write_sample_csv();

    // 2. Build a manifest that points the csv_hris_connector at our temp file
    let manifest_json = sample_csv_manifest(csv_path.to_str().unwrap());
    let manifest = Manifest::from_json(&manifest_json).expect("parse manifest");

    // 3. Resolve actions via the factory
    let actions: Vec<_> = manifest
        .actions
        .iter()
        .map(|ac| ActionFactory::create(ac).expect("create action"))
        .collect();

    // 4. Run the pipeline with an empty initial context
    let initial = RosterContext::new(LazyFrame::default());
    let result = PipelineRunner::run(&manifest, actions, initial).expect("pipeline run");

    // 5. Verify the output
    let df = result.data.collect().expect("collect");
    assert_eq!(df.height(), 3, "should have 3 employee rows");
    assert_eq!(df.width(), 7, "should have 7 columns");

    // Verify field-ownership metadata was stamped
    assert_eq!(result.field_metadata.len(), 7);
    for (_field, meta) in &result.field_metadata {
        assert_eq!(meta.source, "HRIS_CONNECTOR");
    }
}

#[test]
fn test_e2e_empty_manifest() {
    let manifest = Manifest::from_json(r#"{ "version": "1.0", "actions": [] }"#)
        .expect("parse");
    let initial = RosterContext::new(LazyFrame::default());
    let result = PipelineRunner::run(&manifest, vec![], initial).expect("run");
    // With no actions the context passes through unchanged
    assert!(result.field_metadata.is_empty());
}
