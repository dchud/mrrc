//! BIBFRAME batch conversion example with custom configuration and error handling.
//!
//! This example demonstrates:
//! - Batch converting multiple MARC records to BIBFRAME
//! - Using custom `BibframeConfig` options
//! - Error handling in conversions
//! - Performance considerations

use mrrc::bibframe::{marc_to_bibframe, BibframeConfig, RdfFormat};
use mrrc::leader::Leader;
use mrrc::record::{Field, Record};
use std::time::Instant;

fn main() {
    // Configure BIBFRAME conversion with custom settings
    let config = BibframeConfig::new()
        .with_base_uri("http://library.example.org/")
        .with_output_format(RdfFormat::JsonLd)
        .with_authority_linking(true);

    println!("=== Batch BIBFRAME Conversion ===\n");
    println!("Configuration:");
    println!("  Base URI: http://library.example.org/");
    println!("  Output Format: JSON-LD");
    println!("  Authority Linking: Enabled\n");

    // Create a batch of sample records
    let sample_records = vec![
        ("book-001", "Introduction to Rust"),
        ("book-002", "Advanced MARC Processing"),
        ("book-003", "Library Systems Design"),
    ];

    let start = Instant::now();
    let mut total_triples: usize = 0;
    let mut successful: i32 = 0;
    let mut errors: i32 = 0;

    for (control_num, title) in &sample_records {
        // Create MARC record
        let leader = Leader {
            record_length: 1000,
            record_status: 'n',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: ' ',
            character_coding: 'a',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 100,
            encoding_level: ' ',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        };

        let mut record = Record::new(leader);

        // Add control fields
        record.add_control_field("001".to_string(), (*control_num).to_string());
        record.add_control_field(
            "008".to_string(),
            "040520s2023    xxu           000 0 eng  ".to_string(),
        );

        // Add title
        let mut f245 = Field::new("245".to_string(), '1', '0');
        f245.add_subfield('a', format!("{title} /"));
        f245.add_subfield('c', "by Various Authors.".to_string());
        record.add_field(f245);

        // Convert to BIBFRAME
        if let Ok(graph) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            marc_to_bibframe(&record, &config)
        })) {
            let triples = graph.len();
            total_triples += triples;
            successful += 1;

            // Serialize to chosen format
            match graph.serialize(config.output_format) {
                Ok(output) => {
                    println!(
                        "✓ {control_num} ({title}): {triples} triples, {} bytes",
                        output.len()
                    );
                },
                Err(e) => {
                    println!("✗ {control_num} ({title}): Serialization error: {e}");
                    errors += 1;
                },
            }
        } else {
            println!("✗ {control_num} ({title}): Conversion panic");
            errors += 1;
        }
    }

    let elapsed = start.elapsed();

    // Print summary
    println!("\n=== Batch Processing Summary ===");
    println!("Total records: {}", sample_records.len());
    println!("Successful: {successful}");
    println!("Errors: {errors}");
    println!("Total triples: {total_triples}");
    #[allow(clippy::cast_precision_loss)]
    let avg = total_triples as f64 / f64::from(successful);
    println!("Average triples per record: {avg:.1}");
    println!("Processing time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
    if successful > 0 {
        println!(
            "Throughput: {:.0} records/sec",
            f64::from(successful) / elapsed.as_secs_f64()
        );
    }

    // Error handling patterns
    println!("\n=== Error Handling Patterns ===");
    println!("Best practices for production batch processing:");
    println!("  1. Wrap each record conversion in error handler");
    println!("  2. Log detailed error context (record ID, field, position)");
    println!("  3. Continue processing remaining records on error");
    println!("  4. Report summary statistics at end");
    println!("  5. Use Result<> for granular error propagation");
    println!("  6. Consider retry logic for transient errors");
}
