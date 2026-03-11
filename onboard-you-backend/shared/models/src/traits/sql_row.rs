//! `SqlRow!` — declarative macro that generates a `sqlx::FromRow` companion
//! struct (`{Name}Row`) and a `From<{Name}Row> for {Name}` impl.
//!
//! ## Field attributes (applied inside the struct body)
//!
//! | Attribute             | Row field type                        | From mapping       |
//! |-----------------------|---------------------------------------|--------------------|
//! | *(none)*              | same as original                      | `row.field`        |
//! | `#[json]`             | `sqlx::types::Json<OriginalType>`     | `row.field.0`      |
//! | `#[column(col)]`      | field renamed to `col`                | `field: row.col`   |
//! | `#[column(col)] #[json]` | renamed + Json-wrapped             | `field: row.col.0` |
//!
//! All other attributes (e.g. `#[serde(...)]`) are preserved on the original
//! struct and **stripped** from the generated Row.
//!
//! ## Example
//!
//! ```ignore
//! #[macro_rules_attribute::apply(crate::SqlRow!)]
//! #[derive(Clone, Debug, Serialize, Deserialize)]
//! pub struct PipelineConfig {
//!     pub name: String,
//!     #[json]
//!     pub pipeline: Manifest,
//! }
//! ```
//!
//! Generates:
//! ```ignore
//! #[derive(sqlx::FromRow)]
//! pub struct PipelineConfigRow {
//!     pub name: String,
//!     pub pipeline: sqlx::types::Json<Manifest>,
//! }
//!
//! impl From<PipelineConfigRow> for PipelineConfig {
//!     fn from(row: PipelineConfigRow) -> Self {
//!         Self { name: row.name, pipeline: row.pipeline.0 }
//!     }
//! }
//! ```

/// Main entry point — applied via `#[macro_rules_attribute::apply(crate::SqlRow!)]`.
///
/// Kicks off the accumulator which processes fields one at a time, then
/// emits both the original struct (with custom attrs stripped) and the
/// generated Row struct + From impl.
///
/// A single `__row` token is created here and threaded through all
/// recursive expansions so that every reference to the `From` parameter
/// shares the same hygiene context.
#[macro_export]
macro_rules! SqlRow {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::__sql_accumulate! {
            @process,
            attrs       = [$(#[$attr])*],
            vis         = [$vis],
            name        = $name,
            rvar        = [__row],
            pending     = [],
            orig_fields = [],
            row_fields  = [],
            from_fields = [],
            rest        = [$($body)*]
        }
    };
}

// ── Accumulator ───────────────────────────────────────────────
//
// Processes struct body tokens one chunk at a time, accumulating:
//   orig_fields — fields for the original struct (custom attrs stripped)
//   row_fields  — fields for the Row struct (Json wrappers, renames)
//   from_fields — field mappings for From<Row>
//   pending     — non-custom attributes waiting to be flushed to orig_fields
//   rvar        — a single captured ident used as the `from(rvar: Row)` param
//                 (avoids hygiene mismatch across recursive expansions)

#[doc(hidden)]
#[macro_export]
macro_rules! __sql_accumulate {
    // ── Base case: emit everything ──────────────────────────
    (
        @process,
        attrs       = [$($attr:tt)*],
        vis         = [$vis:vis],
        name        = $name:ident,
        rvar        = [$r:ident],
        pending     = [$($pa:tt)*],
        orig_fields = [$($of:tt)*],
        row_fields  = [$($rf:tt)*],
        from_fields = [$($ff:tt)*],
        rest        = []
    ) => {
        // 1. Original struct (custom attrs stripped, others preserved)
        $($attr)*
        $vis struct $name {
            $($of)*
        }

        // 2. Row struct + From impl (paste only for ident concatenation)
        ::paste::paste! {
            #[derive(::sqlx::FromRow)]
            pub struct [< $name Row >] {
                $($rf)*
            }

            impl From<[< $name Row >]> for $name {
                fn from($r: [< $name Row >]) -> Self {
                    Self {
                        $($ff)*
                    }
                }
            }
        }
    };

    // ── #[column(col)] #[json] field ────────────────────────
    (
        @process,
        attrs       = [$($attr:tt)*],
        vis         = [$svis:vis],
        name        = $name:ident,
        rvar        = [$r:ident],
        pending     = [$($pa:tt)*],
        orig_fields = [$($of:tt)*],
        row_fields  = [$($rf:tt)*],
        from_fields = [$($ff:tt)*],
        rest        = [# [column($col:ident)] # [json] $fvis:vis $field:ident : $ty:ty, $($tail:tt)*]
    ) => {
        $crate::__sql_accumulate! {
            @process,
            attrs       = [$($attr)*],
            vis         = [$svis],
            name        = $name,
            rvar        = [$r],
            pending     = [],
            orig_fields = [$($of)* $($pa)* $fvis $field : $ty,],
            row_fields  = [$($rf)* pub $col : ::sqlx::types::Json<$ty>,],
            from_fields = [$($ff)* $field : $r.$col.0,],
            rest        = [$($tail)*]
        }
    };

    // ── #[json] #[column(col)] field (reversed) ─────────────
    (
        @process,
        attrs       = [$($attr:tt)*],
        vis         = [$svis:vis],
        name        = $name:ident,
        rvar        = [$r:ident],
        pending     = [$($pa:tt)*],
        orig_fields = [$($of:tt)*],
        row_fields  = [$($rf:tt)*],
        from_fields = [$($ff:tt)*],
        rest        = [# [json] # [column($col:ident)] $fvis:vis $field:ident : $ty:ty, $($tail:tt)*]
    ) => {
        $crate::__sql_accumulate! {
            @process,
            attrs       = [$($attr)*],
            vis         = [$svis],
            name        = $name,
            rvar        = [$r],
            pending     = [],
            orig_fields = [$($of)* $($pa)* $fvis $field : $ty,],
            row_fields  = [$($rf)* pub $col : ::sqlx::types::Json<$ty>,],
            from_fields = [$($ff)* $field : $r.$col.0,],
            rest        = [$($tail)*]
        }
    };

    // ── #[column(col)] field (no json) ──────────────────────
    (
        @process,
        attrs       = [$($attr:tt)*],
        vis         = [$svis:vis],
        name        = $name:ident,
        rvar        = [$r:ident],
        pending     = [$($pa:tt)*],
        orig_fields = [$($of:tt)*],
        row_fields  = [$($rf:tt)*],
        from_fields = [$($ff:tt)*],
        rest        = [# [column($col:ident)] $fvis:vis $field:ident : $ty:ty, $($tail:tt)*]
    ) => {
        $crate::__sql_accumulate! {
            @process,
            attrs       = [$($attr)*],
            vis         = [$svis],
            name        = $name,
            rvar        = [$r],
            pending     = [],
            orig_fields = [$($of)* $($pa)* $fvis $field : $ty,],
            row_fields  = [$($rf)* pub $col : $ty,],
            from_fields = [$($ff)* $field : $r.$col,],
            rest        = [$($tail)*]
        }
    };

    // ── #[json] field (no column) ───────────────────────────
    (
        @process,
        attrs       = [$($attr:tt)*],
        vis         = [$svis:vis],
        name        = $name:ident,
        rvar        = [$r:ident],
        pending     = [$($pa:tt)*],
        orig_fields = [$($of:tt)*],
        row_fields  = [$($rf:tt)*],
        from_fields = [$($ff:tt)*],
        rest        = [# [json] $fvis:vis $field:ident : $ty:ty, $($tail:tt)*]
    ) => {
        $crate::__sql_accumulate! {
            @process,
            attrs       = [$($attr)*],
            vis         = [$svis],
            name        = $name,
            rvar        = [$r],
            pending     = [],
            orig_fields = [$($of)* $($pa)* $fvis $field : $ty,],
            row_fields  = [$($rf)* pub $field : ::sqlx::types::Json<$ty>,],
            from_fields = [$($ff)* $field : $r.$field.0,],
            rest        = [$($tail)*]
        }
    };

    // ── Any other attribute — push to pending ───────────────
    (
        @process,
        attrs       = [$($attr:tt)*],
        vis         = [$svis:vis],
        name        = $name:ident,
        rvar        = [$r:ident],
        pending     = [$($pa:tt)*],
        orig_fields = [$($of:tt)*],
        row_fields  = [$($rf:tt)*],
        from_fields = [$($ff:tt)*],
        rest        = [# [$($other:tt)*] $($tail:tt)*]
    ) => {
        $crate::__sql_accumulate! {
            @process,
            attrs       = [$($attr)*],
            vis         = [$svis],
            name        = $name,
            rvar        = [$r],
            pending     = [$($pa)* #[$($other)*]],
            orig_fields = [$($of)*],
            row_fields  = [$($rf)*],
            from_fields = [$($ff)*],
            rest        = [$($tail)*]
        }
    };

    // ── Plain field (with trailing comma) ────────────────────
    (
        @process,
        attrs       = [$($attr:tt)*],
        vis         = [$svis:vis],
        name        = $name:ident,
        rvar        = [$r:ident],
        pending     = [$($pa:tt)*],
        orig_fields = [$($of:tt)*],
        row_fields  = [$($rf:tt)*],
        from_fields = [$($ff:tt)*],
        rest        = [$fvis:vis $field:ident : $ty:ty, $($tail:tt)*]
    ) => {
        $crate::__sql_accumulate! {
            @process,
            attrs       = [$($attr)*],
            vis         = [$svis],
            name        = $name,
            rvar        = [$r],
            pending     = [],
            orig_fields = [$($of)* $($pa)* $fvis $field : $ty,],
            row_fields  = [$($rf)* pub $field : $ty,],
            from_fields = [$($ff)* $field : $r.$field,],
            rest        = [$($tail)*]
        }
    };

    // ── Trailing field (no comma — last field) ──────────────
    (
        @process,
        attrs       = [$($attr:tt)*],
        vis         = [$svis:vis],
        name        = $name:ident,
        rvar        = [$r:ident],
        pending     = [$($pa:tt)*],
        orig_fields = [$($of:tt)*],
        row_fields  = [$($rf:tt)*],
        from_fields = [$($ff:tt)*],
        rest        = [$fvis:vis $field:ident : $ty:ty]
    ) => {
        $crate::__sql_accumulate! {
            @process,
            attrs       = [$($attr)*],
            vis         = [$svis],
            name        = $name,
            rvar        = [$r],
            pending     = [],
            orig_fields = [$($of)* $($pa)* $fvis $field : $ty,],
            row_fields  = [$($rf)* pub $field : $ty,],
            from_fields = [$($ff)* $field : $r.$field,],
            rest        = []
        }
    };
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[macro_rules_attribute::apply(crate::SqlRow!)]
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct TestModel {
        pub name: String,
        pub value: i32,
    }

    #[test]
    fn row_converts_to_model() {
        let row = TestModelRow {
            name: "test".into(),
            value: 42,
        };
        let model: TestModel = row.into();
        assert_eq!(model.name, "test");
        assert_eq!(model.value, 42);
    }
}
