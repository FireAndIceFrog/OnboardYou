//! RosterContext: The central data structure wrapping Polars LazyFrame
//!
//! Tracks field ownership to identify if data was mastered by HRIS or modified by a Logic Action

use polars::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::run_log::PipelineWarning;

/// Shared collector that Polars `.map()` closures can push warnings into.
///
/// Because LazyFrames defer execution, closures inside `.map()` run at
/// collection time — long after `execute()` returns. This `Arc<Mutex<_>>`
/// lets the closures write warnings that are later drained by the pipeline
/// runner and merged into the `RosterContext.warnings` vec.
pub type WarningCollector = Arc<Mutex<Vec<PipelineWarning>>>;

/// Metadata tracking field ownership and source of truth
#[derive(Clone, Debug)]
pub struct FieldMetadata {
    pub source: String, // "HRIS_CONNECTOR" | "LOGIC_ACTION" | etc
    pub modified_by: Option<String>,
    pub timestamp: Option<String>,
}

/// The RosterContext: Wraps a Polars LazyFrame + field ownership metadata
///
/// This is the "pass-through" data structure that moves through the pipeline,
/// accumulating transformations without persistence.
#[derive(Clone)]
pub struct RosterContext {
    /// The actual data: lazy evaluation for efficiency
    pub data: LazyFrame,

    /// Metadata: Which field was mastered/modified by which capability
    pub field_metadata: HashMap<String, FieldMetadata>,

    /// Warnings accumulated during pipeline execution (non-fatal issues).
    pub warnings: Vec<PipelineWarning>,

    /// Shared collector for warnings emitted inside Polars `.map()` closures.
    /// Cloned into closures; drained after `.collect()` by the pipeline runner.
    pub warning_collector: WarningCollector,
}

impl RosterContext {
    /// Create a new RosterContext from a LazyFrame
    pub fn new(data: LazyFrame) -> Self {
        Self {
            data,
            field_metadata: HashMap::new(),
            warnings: Vec::new(),
            warning_collector: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record field ownership metadata
    pub fn set_field_source(&mut self, field: String, source: String) {
        self.field_metadata.insert(
            field,
            FieldMetadata {
                source,
                modified_by: None,
                timestamp: None,
            },
        );
    }

    /// Update field metadata with modification tracking
    pub fn mark_field_modified(&mut self, field: String, modified_by: String) {
        if let Some(metadata) = self.field_metadata.get_mut(&field) {
            metadata.modified_by = Some(modified_by);
        }
    }

    /// Record a non-fatal warning that will be surfaced in the run log.
    pub fn warn(&mut self, action_id: &str, message: String, count: usize, detail: Option<String>) {
        self.warnings.push(PipelineWarning {
            action_id: action_id.to_string(),
            message,
            count,
            detail,
        });
    }

    /// Drain deferred warnings from the shared collector into `self.warnings`.
    ///
    /// Call this after the LazyFrame has been collected (materialised) so that
    /// any warnings emitted inside Polars `.map()` closures are captured.
    pub fn drain_deferred_warnings(&mut self) {
        if let Ok(mut guard) = self.warning_collector.lock() {
            self.warnings.append(&mut *guard);
        }
    }
}

impl std::fmt::Debug for RosterContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RosterContext")
            .field("field_metadata", &self.field_metadata)
            .finish()
    }
}
