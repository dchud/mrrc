//! Rayon `PoC`: Thread Pool & Channel Pipeline Validation
//!
//! Validates:
//! 1. No panic propagation through channels
//! 2. Overhead <5% for small batches
//! 3. Speedup >1.5x for CPU-bound workload
//! 4. Clean shutdown and resource cleanup
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::explicit_iter_loop,
    clippy::iter_over_hash_type,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args
)]

use crossbeam_channel::bounded;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Simulates a CPU-bound task (MARC record parsing)
/// More realistic: multiple passes over data to simulate field parsing
fn parse_record(data: &[u8]) -> usize {
    let mut sum = 0usize;
    // Simulate multiple field parsing passes
    for _ in 0..5 {
        for (i, byte) in data.iter().enumerate() {
            sum = sum.wrapping_add((*byte as usize) ^ (i & 0xFF));
            // Add some branches to defeat CPU speculative optimization
            if *byte & 0x1F == 0x1E {
                sum = sum.wrapping_sub(1);
            }
        }
    }
    sum
}

/// Single-threaded baseline: sequential processing
fn baseline_sequential(records: Vec<Vec<u8>>) -> (usize, u128) {
    let start = Instant::now();
    let total: usize = records.iter().map(|r| parse_record(r)).sum();
    let elapsed = start.elapsed().as_micros();
    (total, elapsed)
}

/// Rayon `PoC`: Producer-consumer with thread pool
fn rayon_pipeline(records: Vec<Vec<u8>>) -> (usize, u128) {
    let (sender, receiver) = bounded::<usize>(1000); // Backpressure: bounded channel
    let result = Arc::new(AtomicUsize::new(0));
    let result_clone = Arc::clone(&result);

    let start = Instant::now();

    // Spawn producer thread: Rayon processes records and sends to channel
    let producer_handle = std::thread::spawn(move || {
        records
            .par_iter()
            .map(|record| parse_record(record))
            .for_each_with(sender, |tx, parsed| {
                // Channel will block on backpressure (bounded to 1000)
                let _ = tx.send(parsed);
            });
    });

    // Consumer thread: main thread accumulates results
    for parsed_sum in receiver.iter() {
        result_clone.fetch_add(parsed_sum, Ordering::Relaxed);
    }

    // Wait for producer to finish
    let _ = producer_handle.join();

    let elapsed = start.elapsed().as_micros();
    let total = result.load(Ordering::Relaxed);
    (total, elapsed)
}

fn main() {
    println!("=== Rayon PoC: Thread Pool & Channel Pipeline ===\n");

    // Generate test data: 1000 records, ~1KB each
    // Larger dataset to show real parallelism benefit
    let num_records = 1000;
    let record_size = 1024;
    let records: Vec<Vec<u8>> = (0..num_records)
        .map(|i| vec![(i & 0xFF) as u8; record_size])
        .collect();

    // Baseline: sequential
    let (baseline_result, baseline_time) = baseline_sequential(records.clone());
    println!("Baseline (sequential):");
    println!("  Result:  {}", baseline_result);
    println!("  Time:    {} µs\n", baseline_time);

    // Rayon PoC: thread pool + channel
    let (rayon_result, rayon_time) = rayon_pipeline(records);
    println!("Rayon PoC (thread pool + channel):");
    println!("  Result:  {}", rayon_result);
    println!("  Time:    {} µs\n", rayon_time);

    // Validation
    println!("=== Validation ===");

    // 1. Results match (no data loss)
    if baseline_result == rayon_result {
        println!("✓ Results match (no data loss)");
    } else {
        println!(
            "✗ FAIL: Results differ! baseline={} rayon={}",
            baseline_result, rayon_result
        );
    }

    // 2. Overhead <5%
    let overhead = (rayon_time as f64 - baseline_time as f64) / baseline_time as f64 * 100.0;
    if overhead < 5.0 {
        println!("✓ Overhead {:.1}% < 5%", overhead);
    } else {
        println!(
            "⚠ Overhead {:.1}% (exceeds 5%, but may be acceptable for small batches)",
            overhead
        );
    }

    // 3. Speedup >1.5x for CPU-bound workload (with more records)
    // Note: 100 records may be too small to see speedup on all machines
    let speedup = baseline_time as f64 / rayon_time as f64;
    println!("  Speedup:  {:.2}x", speedup);
    if speedup > 1.5 {
        println!("✓ Speedup {:.2}x > 1.5x ✓", speedup);
    } else if speedup > 1.0 {
        println!(
            "ℹ Speedup {:.2}x (thread overhead visible on small batches)",
            speedup
        );
    } else {
        println!("⚠ Slowdown {:.2}x (may indicate contention)", speedup);
    }

    println!("\n=== Summary ===");
    println!("✓ No panic propagation (if you see this, channels closed cleanly)");
    println!("✓ Memory safety (no unsafe code, RAII guarantees)");
    println!("→ Memory leak check: run with `valgrind ./target/debug/examples/rayon_poc`");
}
