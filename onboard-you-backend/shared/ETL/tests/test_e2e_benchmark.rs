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

use common::mock_data::{full_pipeline_manifest, generate_hris_csv};
use onboard_you::{ActionFactory, ActionFactoryTrait };
use models::{Manifest, RosterContext};
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
    rss_delta_bytes: usize,
    peak_rss_bytes: usize,
}

/// Read current RSS (Resident Set Size) in bytes from `/proc/self/status`.
/// Returns 0 on non-Linux platforms or if the read fails.
fn get_rss_bytes() -> usize {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("VmRSS:"))?
                    .split_whitespace()
                    .nth(1)?
                    .parse::<usize>()
                    .ok()
            })
            .map(|kb| kb * 1024)
            .unwrap_or(0)
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}

/// Execute the full pipeline on a generated CSV of `n` rows, returning metrics.
///
/// The first manifest action (`csv_hris_connector`) would try to download from
/// S3, which is unavailable in tests.  Instead we:
///   1. Generate the CSV in-memory and read it directly into a Polars DataFrame.
///   2. Build a `RosterContext` from that DataFrame.
///   3. Parse the manifest, but only run actions[1..] (skipping the connector).
fn run_full_pipeline(n: usize) -> RunMetrics {
    // 1. Generate CSV in-memory → Polars DataFrame (no temp file / no S3)
    let csv_bytes = generate_hris_csv(n);
    let cursor = std::io::Cursor::new(csv_bytes.as_bytes());
    let df = CsvReader::new(cursor)
        .finish()
        .expect("parse generated CSV into DataFrame");

    // 2. Build a RosterContext seeded with the generated data
    let initial_context = RosterContext::new(df.lazy());

    // 3. Parse the manifest — the connector config must be valid JSON even
    //    though we never call its execute().
    let generated_columns: &[&str] = &[
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
    let manifest_json = full_pipeline_manifest("data.csv", generated_columns);
    let manifest = Manifest::from_json(&manifest_json).expect("parse manifest");

    // 4. Resolve all actions via the factory, but skip the first (csv_hris_connector)
    let actions: Vec<_> = manifest
        .actions
        .iter()
        .skip(1)
        .map(|ac| {
            ActionFactory::new()
                .create(ac)
                .expect(&format!("create action '{}'", ac.action_type))
        })
        .collect();

    // 5. Run the pipeline, timing each action individually
    let rss_before = get_rss_bytes();
    let mut context = initial_context;
    let mut action_timings: Vec<(String, f64)> = Vec::with_capacity(actions.len());
    let total_start = Instant::now();

    let mut success = true;
    for (i, action) in actions.iter().enumerate() {
        // Index into manifest.actions offset by 1 since we skipped the connector
        let action_cfg = &manifest.actions[i + 1];
        let action_id = action_cfg.id.clone();
        let action_type = action_cfg.action_type.clone();

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
                let rss_after = get_rss_bytes();
                return RunMetrics {
                    row_count: n,
                    total_ms: total_start.elapsed().as_secs_f64() * 1000.0,
                    action_timings,
                    output_rows: 0,
                    output_cols: 0,
                    success,
                    rss_delta_bytes: rss_after.saturating_sub(rss_before),
                    peak_rss_bytes: rss_after,
                };
            }
        }
    }

    let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;

    // 5. Collect the final LazyFrame to measure output shape
    let df = context.data.collect().expect("collect final dataframe");
    let output_rows = df.height();
    let output_cols = df.width();
    let rss_after = get_rss_bytes();

    RunMetrics {
        row_count: n,
        total_ms,
        action_timings,
        output_rows,
        output_cols,
        success,
        rss_delta_bytes: rss_after.saturating_sub(rss_before),
        peak_rss_bytes: rss_after,
    }
}

/// Pretty-print metrics for one run.
fn print_metrics(m: &RunMetrics) {
    let status = if m.success {
        "✓ SUCCESS"
    } else {
        "✗ FAILED"
    };
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
    println!(
        "║  Memory: RSS Δ {:.2} MB — peak RSS {:.2} MB",
        m.rss_delta_bytes as f64 / 1_048_576.0,
        m.peak_rss_bytes as f64 / 1_048_576.0,
    );
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  {:<45} {:>10}", "Action", "Time (ms)");
    println!(
        "║  {:<45} {:>10}",
        "─────────────────────────────────────────────", "─────────"
    );

    for (name, ms) in &m.action_timings {
        let bar_len = (*ms / m.total_ms * 40.0).round() as usize;
        let bar: String = "█".repeat(bar_len.min(40));
        println!("║  {:<45} {:>8.2}  {}", name, ms, bar);
    }

    let accounted: f64 = m.action_timings.iter().map(|(_, ms)| ms).sum();
    let overhead = m.total_ms - accounted;
    println!(
        "║  {:<45} {:>8.2}",
        "overhead (collect / framework)", overhead
    );
    println!("╚══════════════════════════════════════════════════════════════════╝");
}

/// Print a merged summary table with one column per run size, including memory.
fn print_merged_summary(runs: &[RunMetrics]) {
    if runs.is_empty() {
        return;
    }

    let label_w: usize = 46;
    let col_w: usize = 14;
    let n = runs.len();
    let inner_w = label_w + 2 + n * (1 + col_w);

    // ── drawing helpers ─────────────────────────────────────────────────
    let h_line = |left: &str, mid: &str, right: &str, fill: &str| {
        print!("{}{}", left, fill.repeat(label_w + 2));
        for _ in 0..n {
            print!("{}{}", mid, fill.repeat(col_w));
        }
        println!("{}", right);
    };

    let row = |label: &str, values: Vec<String>| {
        print!("║  {:<width$}", label, width = label_w);
        for v in &values {
            print!("│ {:>width$} ", v, width = col_w - 2);
        }
        println!("║");
    };

    // ── title ───────────────────────────────────────────────────────────
    let title = format!(
        "Benchmark Summary — {} rows",
        runs.iter()
            .map(|r| r.row_count.to_string())
            .collect::<Vec<_>>()
            .join(" / ")
    );
    println!();
    h_line("╔", "═", "╗", "═");
    println!("║  {:<width$}║", title, width = inner_w - 2);
    h_line("╠", "╤", "╣", "═");

    // ── column headers ──────────────────────────────────────────────────
    row(
        "",
        runs.iter()
            .map(|r| format!("{} rows", r.row_count))
            .collect(),
    );
    h_line("╟", "┼", "╢", "─");

    // ── result overview ─────────────────────────────────────────────────
    row(
        "Status",
        runs.iter()
            .map(|r| {
                if r.success {
                    "✓ PASS".to_string()
                } else {
                    "✗ FAIL".to_string()
                }
            })
            .collect(),
    );
    row(
        "Output rows",
        runs.iter().map(|r| format!("{}", r.output_rows)).collect(),
    );
    row(
        "Output cols",
        runs.iter().map(|r| format!("{}", r.output_cols)).collect(),
    );
    h_line("╟", "┼", "╢", "─");

    // ── timing ──────────────────────────────────────────────────────────
    row(
        "Total time (ms)",
        runs.iter().map(|r| format!("{:.2}", r.total_ms)).collect(),
    );
    row(
        "Total time (s)",
        runs.iter()
            .map(|r| format!("{:.3}", r.total_ms / 1000.0))
            .collect(),
    );
    row(
        "Throughput (rows/sec)",
        runs.iter()
            .map(|r| {
                if r.total_ms > 0.0 {
                    format!("{:.0}", r.row_count as f64 / (r.total_ms / 1000.0))
                } else {
                    "—".to_string()
                }
            })
            .collect(),
    );
    h_line("╟", "┼", "╢", "─");

    // ── memory ──────────────────────────────────────────────────────────
    row(
        "Memory RSS Δ (MB)",
        runs.iter()
            .map(|r| format!("{:.2}", r.rss_delta_bytes as f64 / 1_048_576.0))
            .collect(),
    );
    row(
        "Peak RSS (MB)",
        runs.iter()
            .map(|r| format!("{:.2}", r.peak_rss_bytes as f64 / 1_048_576.0))
            .collect(),
    );
    h_line("╟", "┼", "╢", "─");

    // ── per-action timings (ms) ─────────────────────────────────────────
    if let Some(first) = runs.first() {
        for (i, (name, _)) in first.action_timings.iter().enumerate() {
            row(
                name,
                runs.iter()
                    .map(|r| {
                        r.action_timings
                            .get(i)
                            .map(|(_, ms)| format!("{:.2}", ms))
                            .unwrap_or_else(|| "—".to_string())
                    })
                    .collect(),
            );
        }
    }
    row(
        "overhead (collect / framework)",
        runs.iter()
            .map(|r| {
                let accounted: f64 = r.action_timings.iter().map(|(_, ms)| ms).sum();
                format!("{:.2}", r.total_ms - accounted)
            })
            .collect(),
    );
    h_line("╚", "╧", "╝", "═");

    // ── CSV (for graphing / spreadsheet) ────────────────────────────────
    println!("\n── CSV Summary ────────────────────────────────────────────────────");
    if let Some(first) = runs.first() {
        let action_names: Vec<&str> = first
            .action_timings
            .iter()
            .map(|(n, _)| n.as_str())
            .collect();
        print!("rows,total_ms,rows_per_sec,rss_delta_mb,peak_rss_mb");
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
            print!(
                "{},{:.2},{:.0},{:.2},{:.2}",
                m.row_count,
                m.total_ms,
                rps,
                m.rss_delta_bytes as f64 / 1_048_576.0,
                m.peak_rss_bytes as f64 / 1_048_576.0,
            );
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
    // Warm-up: run a tiny pipeline to force one-time initialisation
    // (Polars thread pool, regex cache, page faults) so it doesn't
    // inflate the RSS Δ of the first real run.
    println!(">>> Warm-up run (50 rows) …");
    let _ = run_full_pipeline(50);

    let sizes = [1_000, 5_000, 10_000];
    let mut all_metrics: Vec<RunMetrics> = Vec::with_capacity(sizes.len());

    for &n in &sizes {
        println!("\n>>> Generating and running pipeline for {} rows …", n);
        let m = run_full_pipeline(n);
        assert!(m.success, "pipeline should succeed for {} rows", n);
        all_metrics.push(m);
    }

    // Full run summary (merged table with columns per size + memory)
    print_merged_summary(&all_metrics);

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
        assert!(header.contains(col), "header should contain '{}'", col);
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
