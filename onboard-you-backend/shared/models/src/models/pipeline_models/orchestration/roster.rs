//! RosterContext: The central data structure wrapping Polars LazyFrame
//!
//! Tracks field ownership to identify if data was mastered by HRIS or modified by a Logic Action

use polars::prelude::*;
use std::collections::HashMap;

use crate::{ETLDependancies};

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
    data: LazyFrame,

    /// Metadata: Which field was mastered/modified by which capability
    field_metadata: HashMap<String, FieldMetadata>,

    pub deps: ETLDependancies,
}

impl RosterContext {
    /// Create a new RosterContext from a LazyFrame.
    ///
    /// Test-only convenience constructor that injects default dependencies.
    #[cfg(test)]
    pub fn new(data: LazyFrame) -> Self {
        Self::with_deps(data, ETLDependancies::default())
    }

    /// Create a new RosterContext with explicitly provided dependencies.
    pub fn with_deps(data: LazyFrame, deps: ETLDependancies) -> Self {
        Self {
            data,
            field_metadata: HashMap::new(),
            deps,
        }
    }
    pub fn get_data(&self) -> LazyFrame {
        self.data.clone()
    }

    pub fn set_data(&mut self, data: LazyFrame) {
        self.data = data;
    }

    pub fn field_metadata(&self) -> &HashMap<String, FieldMetadata> {
        &self.field_metadata
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
}

impl std::fmt::Debug for RosterContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RosterContext")
            .field("field_metadata", &self.field_metadata)
            .finish()
    }
}
