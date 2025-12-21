//! Extract bibliographic data from MARC records and export to CSV.
//!
//! This example demonstrates reading MARC records (from gzip-compressed files or raw binary)
//! and extracting publication dates, authors, and titles into a CSV format.
//!
//! # Usage
//!
//! ```sh
//! cargo run --example marc_to_csv -- <input_file.mrc[.gz]> [output_file.csv]
//! ```
//!
//! If no output file is specified, writes to stdout.
//!
//! # Examples
//!
//! ```sh
//! cargo run --example marc_to_csv -- records.mrc
//! cargo run --example marc_to_csv -- records.mrc.gz output.csv
//! cargo run --example marc_to_csv -- BooksAll.2016.part01.utf8.gz books.csv
//! ```

use std::env;
use std::fs::File;
use std::io::{BufReader, Write};

use flate2::read::GzDecoder;
use mrrc::MarcReader;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input_file.mrc[.gz]> [output_file.csv]", args[0]);
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  input_file   Path to MARC file (supports .gz compression)");
        eprintln!("  output_file  Optional CSV output file (default: stdout)");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = args.get(2).map(std::string::String::as_str);

    // Open input file
    let file = File::open(input_path)
        .map_err(|e| anyhow::anyhow!("Failed to open input file '{input_path}': {e}"))?;

    // Determine if file is gzipped
    let is_gzip = std::path::Path::new(input_path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"));

    // Create appropriate reader based on compression
    let reader: Box<dyn std::io::Read> = if is_gzip {
        Box::new(GzDecoder::new(BufReader::new(file)))
    } else {
        Box::new(BufReader::new(file))
    };

    let mut marc_reader = MarcReader::new(reader);

    // Prepare output
    let mut output: Box<dyn Write> = if let Some(path) = output_path {
        Box::new(
            File::create(path)
                .map_err(|e| anyhow::anyhow!("Failed to create output file '{path}': {e}"))?,
        )
    } else {
        Box::new(std::io::stdout())
    };

    // Write CSV header
    writeln!(output, "title,author,publication_date")?;

    let mut record_count = 0;
    let mut error_count = 0;

    // Process records
    loop {
        match marc_reader.read_record() {
            Ok(Some(record)) => {
                record_count += 1;

                // Extract title (field 245, subfield 'a')
                let title = record
                    .get_field("245")
                    .and_then(|f| f.get_subfield('a'))
                    .unwrap_or("N/A");

                // Extract author (field 100, subfield 'a' - primary author)
                let author = record
                    .get_field("100")
                    .and_then(|f| f.get_subfield('a'))
                    .or_else(|| {
                        // Fallback to field 110 (corporate author)
                        record.get_field("110").and_then(|f| f.get_subfield('a'))
                    })
                    .unwrap_or("N/A");

                // Extract publication date
                // Try field 260 (Publication, Distribution, Etc.) subfield 'c' first
                let pub_date = record
                    .get_field("260")
                    .and_then(|f| f.get_subfield('c'))
                    .or_else(|| {
                        // Fallback to field 008 (Fixed-length data elements)
                        // Position 7-10 contains the publication year for most records
                        record.get_control_field("008").and_then(|field_008| {
                            if field_008.len() >= 11 {
                                let year = &field_008[7..11];
                                // Only use if it looks like a year (4 digits, not all spaces/zeros)
                                if year != "    "
                                    && year != "0000"
                                    && year.chars().all(|c| c.is_ascii_digit())
                                {
                                    Some(year)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                    })
                    .unwrap_or("N/A");

                // Write CSV row with proper escaping
                write_csv_row(&mut output, &[title, author, pub_date])?;
            },
            Ok(None) => {
                // End of file
                break;
            },
            Err(e) => {
                error_count += 1;
                eprintln!("Error reading record {}: {}", record_count + 1, e);
                // Continue processing remaining records
            },
        }
    }

    eprintln!("Processed {record_count} records with {error_count} errors");
    if let Some(path) = output_path {
        eprintln!("CSV written to: {path}");
    }

    Ok(())
}

/// Write a CSV row with proper field escaping
fn write_csv_row<W: Write>(writer: &mut W, fields: &[&str]) -> anyhow::Result<()> {
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            write!(writer, ",")?;
        }

        // Escape quotes and wrap in quotes if needed
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            write!(writer, "\"")?;
            for c in field.chars() {
                if c == '"' {
                    write!(writer, "\"\"")?;
                } else {
                    write!(writer, "{c}")?;
                }
            }
            write!(writer, "\"")?;
        } else {
            write!(writer, "{field}")?;
        }
    }
    writeln!(writer)?;

    Ok(())
}
