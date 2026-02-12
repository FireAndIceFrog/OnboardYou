//! E2E Benchmark: Full pipeline through every action with timing metrics
//!
//! Generates CSVs at 1 000, 5 000 and 10 000 rows, runs the complete
//! action chain, and prints a stats table suitable for graphing.
//!
//! Run the benchmarks with:
//!   cargo test --release -p onboard_you --features benchmark -- --nocapture
//!
//! The heavy benchmarks (1k/5k/10k) are gated behind the `benchmark`
//! feature so they never run during normal `cargo test`.
//! The smoke test always runs.

mod common;

use common::mock_data::{full_pipeline_manifest, write_generated_csv};
use onboard_you::{ActionFactory, Manifest, PipelineRunner, RosterContext};
use polars::prelude::*;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Metrics captured for a single pipeline run.
#[derive(Debug)]
struct RunMetrics {
    row_count: usize,
    total_ms: f64,
    action_timings: Vec<(String, f64)>, // (action_id, millis)
    output_rows: usize,
    output_cols: usize,
    success: bool,
}

/// Execute the full pipeline on a generated CSV of `n` rows, returning metrics.
fn run_full_pipeline(n: usize) -> RunMetrics {
    // 1. Generate CSV to a temp file
    let (_tmp, csv_path) = write_generated_csv(n);
    let csv_str = csv_path.to_str().expect("path to str");

    // 2. Parse the manifest that chains every action
    let manifest_json = full_pipeline_manifest(csv_str);
    let manifest = Manifest::from_json(&manifest_json).expect("parse manifest");

    // 3. Resolve all actions via the factory
    let actions: Vec<_> = manifest
        .actions
        .iter()
        .map(|ac| ActionFactory::create(ac).expect(&format!("create action '{}'", ac.action_type)))
        .collect();

    // 4. Run the pipeline, timing each action individually
    let mut context = RosterContext::new(LazyFrame::default());
    let mut action_timings: Vec<(String, f64)> = Vec::with_capacity(actions.len());
    let total_start = Instant::now();

    let mut success = true;
    for (i, action) in actions.iter().enumerate() {
        let action_id = manifest.actions[i].id.clone();
        let action_type = manifest.actions[i].action_type.clone();

        let t0 = Instant::now();
        match action.execute(context) {
            Ok(ctx) => {
                let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
                action_timings.push((format!("{} ({})", action_id, action_type), elapsed));
                context = ctx;
            }
            Err(e) => {
                let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
                action_timings.push((format!("{} ({})", action_id, action_type), elapsed));
                eprintln!("  ✗ Action '{}' failed: {}", action_id, e);
                success = false;
                // Return partial metrics
                return RunMetrics {
                    row_count: n,
                    total_ms: total_start.elapsed().as_secs_f64() * 1000.0,
                    action_timings,
                    output_rows: 0,
                    output_cols: 0,
                    success,
                };
            }
        }
    }

    let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;

    // 5. Collect the final LazyFrame to measure output shape
    let df = context.data.collect().expect("collect final dataframe");
    let output_rows = df.height();
    let output_cols = df.width();

    RunMetrics {
        row_count: n,
        total_ms,
        action_timings,
        output_rows,
        output_cols,
        success,
    }
}

/// Pretty-print metrics for one run.
fn print_metrics(m: &RunMetrics) {
    let status = if m.success { "✓ SUCCESS" } else { "✗ FAILED" };
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!(
        "║  {} — {:>6} input rows → {:>6} output rows × {} cols",
        status, m.row_count, m.output_rows, m.output_cols,
    );
    println!(
        "║  Total pipeline time: {:.2} ms ({:.2} s)",
        m.total_ms,
        m.total_ms / 1000.0,
    );
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  {:<45} {:>10}", "Action", "Time (ms)");
    println!("║  {:<45} {:>10}", "─────────────────────────────────────────────", "─────────");

    for (name, ms) in &m.action_timings {
        let bar_len = (*ms / m.total_ms * 40.0).round() as usize;
        let bar: String = "█".repeat(bar_len.min(40));
        println!("║  {:<45} {:>8.2}  {}", name, ms, bar);
    }

    let accounted: f64 = m.action_timings.iter().map(|(_, ms)| ms).sum();
    let overhead = m.total_ms - accounted;
    println!("║  {:<45} {:>8.2}", "overhead (collect / framework)", overhead);
    println!("╚══════════════════════════════════════════════════════════════════╝");
}

/// Print a CSV-style summary for easy copy-paste into a spreadsheet / graphing tool.
fn print_csv_summary(runs: &[RunMetrics]) {
    println!("\n── CSV Summary (for graphing) ──────────────────────────────────────");
    // Header: rows, total_ms, then each action
    if let Some(first) = runs.first() {
        let action_names: Vec<&str> = first
            .action_timings
            .iter()
            .map(|(n, _)| n.as_str())
            .collect();
        print!("rows,total_ms,rows_per_sec");
        for name in &action_names {
            print!(",{}", name);
        }
        println!();

        for m in runs {
            let rps = if m.total_ms > 0.0 {
                m.row_count as f64 / (m.total_ms / 1000.0)
            } else {
                0.0
            };
            print!("{},{:.2},{:.0}", m.row_count, m.total_ms, rps);
            for (_, ms) in &m.action_timings {
                print!(",{:.2}", ms);
            }
            println!();
        }
    }
    println!("────────────────────────────────────────────────────────────────────");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Smoke test: run the full pipeline on a small dataset and assert correctness.
#[test]
fn test_full_pipeline_smoke() {
    let m = run_full_pipeline(100);
    assert!(m.success, "pipeline should complete successfully");
    assert!(m.output_rows > 0, "should produce output rows");
    // We started with 10 columns, actions add/remove some
    assert!(m.output_cols > 5, "should retain most columns");
    print_metrics(&m);
}

/// Benchmark at 1 000 / 5 000 / 10 000 rows with full metrics.
///
/// This is a single `#[test]` so the three sizes run in sequence and
/// we can print a comparative summary at the end.
///
/// Gated behind `--features benchmark` so it doesn't slow down normal CI.
#[test]
#[cfg(feature = "benchmark")]
fn test_benchmark_1k_5k_10k() {
    let sizes = [1_000, 5_000, 10_000];
    let mut all_metrics: Vec<RunMetrics> = Vec::with_capacity(sizes.len());

    for &n in &sizes {
        println!("\n>>> Generating and running pipeline for {} rows …", n);
        let m = run_full_pipeline(n);
        assert!(
            m.success,
            "pipeline should succeed for {} rows",
            n
        );
        print_metrics(&m);
        all_metrics.push(m);
    }

    // Comparative summary
    print_csv_summary(&all_metrics);

    // Sanity: row counts should be non-decreasing
    for pair in all_metrics.windows(2) {
        assert!(
            pair[1].output_rows >= pair[0].output_rows,
            "more input rows should yield ≥ output rows"
        );
    }
}

/// Quick correctness checks on the generated CSV itself.
#[test]
#[cfg(feature = "benchmark")]
fn test_csv_generator_correctness() {
    let csv = common::mock_data::generate_hris_csv(500);
    let lines: Vec<&str> = csv.lines().collect();
    // Header + 500 data rows
    assert_eq!(lines.len(), 501, "should have header + 500 rows");

    // Verify header columns
    let header = lines[0];
    let expected_cols = [
        "employee_id",
        "first_name",
        "last_name",
        "email",
        "national_id",
        "ssn",
        "salary",
        "start_date",
        "country_raw",
        "mobile_phone",
    ];
    for col in &expected_cols {
        assert!(
            header.contains(col),
            "header should contain '{}'",
            col
        );
    }

    // Spot-check: first data row starts with E000001
    assert!(
        lines[1].starts_with("E000001,"),
        "first data row should start with E000001"
    );

    // Verify determinism: two calls produce identical output
    let csv2 = common::mock_data::generate_hris_csv(500);
    assert_eq!(csv, csv2, "generator should be deterministic");
}
