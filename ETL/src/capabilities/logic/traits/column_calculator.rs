//! Column calculation trait: Interface for computing output columns per step
//!
//! Every pipeline action (ingestion, logic, egress) must declare how it
//! transforms the column set.  This enables up-front validation and lineage
//! tracking without executing the full pipeline.

use crate::domain::engine::errors::Result;
use crate::domain::engine::roster::RosterContext;

/// Trait for computing the output column set of a pipeline step.
///
/// Implementors receive a `RosterContext` whose `data` field is an **empty
/// `LazyFrame`** carrying the schema (column names + types) from the previous
/// step.  They apply the same structural transformations they would in
/// `execute` (drop, rename, add columns) and return the resulting context.
///
/// # Design notes
///
/// * By operating on an empty `LazyFrame`, logic engines can re-use the
///   exact same Polars operations they perform in `execute` — Polars
///   propagates the schema through the query plan without materialising
///   any data.
/// * Ingestion steps typically ignore the incoming context and return a
///   fresh `RosterContext` reflecting the connector's schema.
/// * Egress steps usually pass the context through unchanged.
/// * `field_metadata` flows through alongside the schema so that
///   provenance tracking remains consistent.
pub trait ColumnCalculator: Send + Sync {
    /// Compute the output schema given a context carrying the previous
    /// step's (empty) schema.
    fn calculate_columns(&self, context: RosterContext) -> Result<RosterContext>;
}
