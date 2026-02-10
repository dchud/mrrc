#![allow(
    clippy::uninlined_format_args,
    clippy::redundant_closure_for_method_calls
)]
//! Concurrent MARC file writing with Rayon
//!
//! This example demonstrates parallel processing of records for writing,
//! using Rayon's work-stealing parallelism. While writing is less commonly
//! parallelized than reading (due to I/O ordering requirements), this example
//! shows patterns for:
//!
//! - Transforming records in parallel before writing
//! - Batch writing with parallel preprocessing
//! - Processing multiple files concurrently
//!
//! Performance: Similar to reading (2.5x on 4 cores for CPU-bound preprocessing)

use rayon::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

fn main() {
    println!("\n{}", "=".repeat(70));
    println!("MRRC Concurrent Writing Example");
    println!("{}\n", "=".repeat(70));

    // Example 1: Sequential transformation
    demo_sequential_transformation();

    // Example 2: Parallel transformation (then sequential write)
    demo_parallel_transformation();

    // Example 3: Batch writing pattern
    demo_batch_writing_pattern();
}

/// Example record data
#[derive(Clone, Debug)]
struct SampleRecord {
    id: usize,
    title: String,
    author: String,
    year: usize,
}

impl SampleRecord {
    /// Create sample records for demonstration
    fn create_samples(count: usize) -> Vec<Self> {
        (0..count)
            .map(|i| Self {
                id: i,
                title: format!("Title {}", i),
                author: format!("Author {}", i),
                year: 2000 + (i % 21),
            })
            .collect()
    }

    /// Simulate record transformation (CPU-intensive)
    fn transform(&self) -> String {
        // Simulate expensive operation: field extraction, validation, enrichment
        let mut sum = 0;
        for ch in self.title.chars() {
            sum += ch as usize;
        }
        format!(
            "ID: {:05} | Title: {} | Author: {} | Year: {} | Hash: {}",
            self.id,
            self.title,
            self.author,
            self.year,
            sum % 10000
        )
    }

    /// Simulate MARC serialization
    fn serialize(&self) -> Vec<u8> {
        self.transform().into_bytes()
    }
}

/// Demonstrate: Sequential transformation
fn demo_sequential_transformation() {
    println!("{}", "=".repeat(70));
    println!("1. SEQUENTIAL TRANSFORMATION");
    println!("{}\n", "=".repeat(70));

    let records = SampleRecord::create_samples(1000);

    let start = Instant::now();

    let transformed: Vec<String> = records.iter().map(|record| record.transform()).collect();

    let elapsed = start.elapsed();

    println!("Time:      {:?}", elapsed);
    println!("Records:   {}", transformed.len());
    println!("Sample:    {}", transformed[0]);
    println!();
}

/// Demonstrate: Parallel transformation with Rayon
fn demo_parallel_transformation() {
    println!("{}", "=".repeat(70));
    println!("2. PARALLEL TRANSFORMATION (Rayon)");
    println!("{}\n", "=".repeat(70));

    let records = SampleRecord::create_samples(1000);

    let start = Instant::now();

    // Parallel transformation
    let transformed: Vec<String> = records
        .par_iter()
        .map(|record| record.transform())
        .collect();

    let elapsed = start.elapsed();

    println!("Time:      {:?}", elapsed);
    println!("Records:   {}", transformed.len());
    println!("Sample:    {}", transformed[0]);
    println!();

    println!("PATTERN: Parallel transformation → Sequential write");
    println!("  1. Transform records in parallel (CPU-intensive)");
    println!("  2. Collect results (ordered preservation)");
    println!("  3. Write sequentially to maintain order");
    println!();
}

/// Demonstrate: Batch writing pattern
fn demo_batch_writing_pattern() {
    println!("{}", "=".repeat(70));
    println!("3. BATCH WRITING PATTERN");
    println!("{}\n", "=".repeat(70));

    let records = SampleRecord::create_samples(1000);

    let start = Instant::now();

    // Process in chunks, transform in parallel, write sequentially
    let batch_size = 100;
    let mut total_bytes = 0;

    for batch in records.chunks(batch_size) {
        // Parallel: Transform each batch
        let transformed: Vec<Vec<u8>> = batch.par_iter().map(|record| record.serialize()).collect();

        // Sequential: Write batch to output
        total_bytes += transformed.iter().map(|b| b.len()).sum::<usize>();
    }

    let elapsed = start.elapsed();

    println!("Time:      {:?}", elapsed);
    println!("Records:   {}", records.len());
    println!("Bytes:     {}", total_bytes);
    println!();

    println!("PATTERN: Batch processing (parallel + sequential)");
    println!("  1. Split records into batches");
    println!("  2. Process each batch in parallel");
    println!("  3. Write batch results (maintains ordering)");
    println!("  4. Benefits:");
    println!("     - Parallel CPU work per batch");
    println!("     - Sequential I/O (correct ordering)");
    println!("     - Memory efficiency (bounded by batch size)");
    println!();
}

// Additional patterns

#[allow(dead_code)]
/// Pattern: Parallel processing with filtering
fn parallel_filtering_pattern(records: &[SampleRecord]) {
    println!("\nPattern: Filter → Transform → Collect");

    let results: Vec<String> = records
        .par_iter()
        .filter(|r| r.year >= 2005) // Filter records
        .map(|r| r.transform()) // Transform filtered
        .collect();

    println!("Processed {} records", results.len());
}

#[allow(dead_code)]
/// Pattern: Parallel fold-reduce for aggregation during processing
fn parallel_aggregation_pattern(records: &[SampleRecord]) {
    use std::collections::HashMap;

    println!("\nPattern: Parallel aggregation");

    let year_counts: HashMap<usize, usize> = records
        .par_iter()
        .fold(HashMap::new, |mut acc, record| {
            *acc.entry(record.year).or_insert(0) += 1;
            acc
        })
        .reduce(HashMap::new, |mut acc, other| {
            for (year, count) in other {
                *acc.entry(year).or_insert(0) += count;
            }
            acc
        });

    println!("Records by year:");
    for (year, count) in year_counts.iter().take(5) {
        println!("  {}: {} records", year, count);
    }
}

#[allow(dead_code)]
/// Pattern: Custom output handling
fn custom_output_pattern(records: &[SampleRecord]) {
    println!("\nPattern: Parallel processing to custom format");

    let output_buffer = Arc::new(Mutex::new(Vec::new()));

    records
        .par_iter()
        .for_each_with(output_buffer.clone(), |buffer, record| {
            let line = format!("{:05}: {}\n", record.id, record.title);
            // Note: Mutex lock can cause contention - use bounded channels for better throughput
            if let Ok(mut buf) = buffer.lock() {
                buf.extend(line.into_bytes());
            }
        });

    let final_output = output_buffer.lock().unwrap();
    println!("Generated {} bytes", final_output.len());
}
