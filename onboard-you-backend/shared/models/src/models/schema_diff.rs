//! Schema diff types — comparison of pipeline columns to egress schema.

use serde::Serialize;
use utoipa::ToSchema;

/// A single column mapping from the pipeline to the egress destination.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ColumnMapping {
    /// Column name in the pipeline (from `final_columns`)
    pub source_column: String,
    /// Destination field name in the egress schema
    pub target_field: String,
}

/// Result of diffing `final_columns` against the egress schema.
///
/// Shows which pipeline columns map to destination fields, and which are
/// unmatched on either side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct SchemaDiff {
    /// Columns present in the pipeline that have a mapping in the egress schema
    pub mapped: Vec<ColumnMapping>,
    /// Pipeline columns with no egress mapping
    pub unmapped_source: Vec<String>,
    /// Egress schema fields with no matching pipeline column
    pub unmapped_target: Vec<String>,
}
