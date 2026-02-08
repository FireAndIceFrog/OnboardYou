//! Identity Resolution: Column-major identity resolution using NID/Email
//!
//! ## Algorithm
//!
//! 1. Build a **dedup key** per row:
//!    - If a `national_id` column exists and the value is non-null → use it.
//!    - Otherwise fall back to the `email` column.
//! 2. Within each dedup-key group, assign the *first* occurrence as the
//!    canonical record (`is_duplicate = false`) and tag subsequent rows
//!    (`is_duplicate = true`).
//! 3. A `canonical_id` column carries the `employee_id` of the canonical
//!    record so downstream actions can trace merges.

use crate::capabilities::logic::traits::Deduplicator;
use crate::domain::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;

/// Identity deduplication using column-major approach.
#[derive(Debug, Clone, Default)]
pub struct IdentityDeduplicator;

impl IdentityDeduplicator {
    pub fn new() -> Self {
        Self
    }
}

impl Deduplicator for IdentityDeduplicator {
    fn deduplicate(&self, context: RosterContext) -> Result<RosterContext> {
        self.execute(context)
    }
}

impl OnboardingAction for IdentityDeduplicator {
    fn id(&self) -> &str {
        "identity_deduplicator"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!("IdentityDeduplicator: resolving duplicates");

        // Collect eagerly — dedup logic requires grouped iteration
        let lf = std::mem::replace(&mut context.data, LazyFrame::default());
        let df = lf.collect().map_err(|e| {
            Error::LogicError(format!("Failed to collect for dedup: {}", e))
        })?;

        let schema = df.schema();
        let has_national_id = schema.contains("national_id");
        let has_email = schema.contains("email");

        if !has_national_id && !has_email {
            tracing::warn!("IdentityDeduplicator: no national_id or email column found — skipping");
            context.data = df.lazy();
            return Ok(context);
        }

        let n = df.height();

        // Build dedup keys per row
        let national_ids = if has_national_id {
            Some(df.column("national_id").unwrap().str().unwrap().clone())
        } else {
            None
        };
        let emails = if has_email {
            Some(df.column("email").unwrap().str().unwrap().clone())
        } else {
            None
        };
        let employee_ids = df.column("employee_id").unwrap().str().unwrap();

        let dedup_keys: Vec<String> = (0..n)
            .map(|i| {
                // Prefer national_id if present and non-null
                if let Some(ref nids) = national_ids {
                    if let Some(nid) = nids.get(i) {
                        return nid.to_string();
                    }
                }
                // Fall back to email
                if let Some(ref em) = emails {
                    if let Some(email) = em.get(i) {
                        return email.to_string();
                    }
                }
                format!("__unknown_{}", i)
            })
            .collect();

        // Track first occurrence of each dedup key → canonical employee_id
        let mut first_occurrence: std::collections::HashMap<&str, &str> =
            std::collections::HashMap::new();
        let mut canonical_ids: Vec<String> = Vec::with_capacity(n);
        let mut is_duplicate: Vec<bool> = Vec::with_capacity(n);

        for i in 0..n {
            let key = dedup_keys[i].as_str();
            let emp_id = employee_ids.get(i).unwrap_or("unknown");

            if let Some(&canon) = first_occurrence.get(key) {
                // Duplicate
                canonical_ids.push(canon.to_string());
                is_duplicate.push(true);
            } else {
                // First occurrence
                first_occurrence.insert(
                    // Safety: dedup_keys lives for the whole loop
                    unsafe { &*(key as *const str) },
                    unsafe { &*(emp_id as *const str) },
                );
                canonical_ids.push(emp_id.to_string());
                is_duplicate.push(false);
            }
        }

        let canonical_col = Column::new("canonical_id".into(), canonical_ids);
        let is_dup_col = Column::new("is_duplicate".into(), is_duplicate);

        let df = df
            .hstack(&[canonical_col, is_dup_col])
            .map_err(|e| Error::LogicError(format!("Failed to append dedup columns: {}", e)))?;

        // Update metadata
        for col_name in ["canonical_id", "is_duplicate"] {
            context.set_field_source(col_name.to_string(), "LOGIC_ACTION".into());
            context.mark_field_modified(col_name.to_string(), "identity_deduplicator".into());
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

    fn test_df_with_email_dupes() -> DataFrame {
        df! {
            "employee_id" => &["001", "002", "003", "004"],
            "first_name"  => &["John", "John", "Jane", "Alice"],
            "email"       => &["john@co.com", "john@co.com", "jane@co.com", "alice@co.com"],
            "salary"      => &[70_000i64, 72_000, 85_000, 92_000],
        }
        .expect("test df")
    }

    fn test_df_with_national_id() -> DataFrame {
        df! {
            "employee_id" => &["001", "002", "003"],
            "national_id" => &[Some("NID-A"), None, Some("NID-A")],
            "email"       => &["john@co.com", "jane@co.com", "johnny@co.com"],
            "salary"      => &[70_000i64, 85_000, 71_000],
        }
        .expect("test df")
    }

    #[test]
    fn test_identity_deduplicator_id() {
        let action = IdentityDeduplicator::new();
        assert_eq!(action.id(), "identity_deduplicator");
    }

    #[test]
    fn test_dedup_by_email() {
        let ctx = RosterContext::new(test_df_with_email_dupes().lazy());
        let action = IdentityDeduplicator::new();
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        // 001 and 002 share the same email → 002 is a duplicate
        let is_dup: Vec<Option<bool>> = df
            .column("is_duplicate").unwrap()
            .bool().unwrap()
            .into_iter()
            .collect();

        // 001 = first occurrence (false), 002 = duplicate (true),
        // 003 = unique (false), 004 = unique (false)
        assert_eq!(is_dup, vec![Some(false), Some(true), Some(false), Some(false)]);
    }

    #[test]
    fn test_canonical_id_set() {
        let ctx = RosterContext::new(test_df_with_email_dupes().lazy());
        let action = IdentityDeduplicator::new();
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let canonical: Vec<Option<&str>> = df
            .column("canonical_id").unwrap()
            .str().unwrap()
            .into_iter()
            .collect();

        // Both 001 and 002 should point to canonical 001
        assert_eq!(canonical[0], Some("001"));
        assert_eq!(canonical[1], Some("001"));
        // 003 and 004 are their own canonical
        assert_eq!(canonical[2], Some("003"));
        assert_eq!(canonical[3], Some("004"));
    }

    #[test]
    fn test_dedup_prefers_national_id() {
        let ctx = RosterContext::new(test_df_with_national_id().lazy());
        let action = IdentityDeduplicator::new();
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let is_dup: Vec<Option<bool>> = df
            .column("is_duplicate").unwrap()
            .bool().unwrap()
            .into_iter()
            .collect();

        // 001 (NID-A) and 003 (NID-A) share a national_id → 003 is dup
        // 002 has null national_id, falls back to email jane@co.com → unique
        assert_eq!(is_dup[0], Some(false)); // 001
        assert_eq!(is_dup[1], Some(false)); // 002
        assert_eq!(is_dup[2], Some(true));  // 003
    }

    #[test]
    fn test_no_dedup_columns_skips() {
        let df = df! {
            "employee_id" => &["001"],
            "salary"      => &[50_000i64],
        }
        .unwrap();
        let ctx = RosterContext::new(df.lazy());
        let action = IdentityDeduplicator::new();
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        // Should pass through unchanged — no canonical_id or is_duplicate columns
        assert!(df.column("canonical_id").is_err());
        assert!(df.column("is_duplicate").is_err());
    }

    #[test]
    fn test_field_metadata() {
        let ctx = RosterContext::new(test_df_with_email_dupes().lazy());
        let action = IdentityDeduplicator::new();
        let result = action.execute(ctx).expect("execute");

        for col_name in ["canonical_id", "is_duplicate"] {
            let meta = result.field_metadata.get(col_name)
                .unwrap_or_else(|| panic!("metadata for '{}'", col_name));
            assert_eq!(meta.source, "LOGIC_ACTION");
            assert_eq!(meta.modified_by.as_deref(), Some("identity_deduplicator"));
        }
    }
}
