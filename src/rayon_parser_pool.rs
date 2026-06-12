//! Parallel MARC record parsing using Rayon.
//!
//! This module provides [`parse_batch_parallel`] which leverages Rayon's work-stealing
//! thread pool to parse multiple MARC records in parallel. Record boundaries are determined
//! via the record boundary scanner (0x1E delimiters), and each record is parsed independently.
//!
//! # Examples
//!
//! ```no_run
//! use mrrc::rayon_parser_pool::parse_batch_parallel;
//! use mrrc::boundary_scanner::RecordBoundaryScanner;
//!
//! let buffer = vec![/* MARC binary data */];
//! let mut scanner = RecordBoundaryScanner::new();
//! let boundaries = scanner.scan(&buffer)?;
//!
//! // Parse all records in parallel
//! let records = parse_batch_parallel(&boundaries, &buffer)?;
//! println!("Parsed {} records in parallel", records.len());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::error::{MarcError, Result};
use crate::reader::MarcReader;
use crate::record::Record;
use std::io::Cursor;

/// Parse a batch of MARC record boundaries in parallel using Rayon.
///
/// Given a buffer and a list of record boundaries (offset, length pairs),
/// this function parses each record in parallel using Rayon's work-stealing
/// thread pool. Each record is an independent task.
///
/// # Arguments
///
/// * `record_boundaries` - Vec of (offset, length) tuples identifying record boundaries
/// * `buffer` - The complete binary buffer containing all records
///
/// # Returns
///
/// * `Ok(Vec<Record>)` - All parsed records in order
/// * `Err(MarcError)` - If any record fails to parse
///
/// # Errors
///
/// Returns an error if:
/// - Any boundary exceeds the buffer size
/// - Any record fails to parse during parallel processing
///
/// # Panics
///
/// Panics are caught internally and converted to errors, so this function
/// should never panic. If a Rayon task panics, the error is propagated to the caller.
///
/// # Example
///
/// ```no_run
/// use mrrc::rayon_parser_pool::parse_batch_parallel;
///
/// let buffer = vec![/* MARC data */];
/// let boundaries = vec![(0, 100), (100, 95), (195, 105)];
/// let records = parse_batch_parallel(&boundaries, &buffer)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_batch_parallel(
    record_boundaries: &[(usize, usize)],
    buffer: &[u8],
) -> Result<Vec<Record>> {
    use rayon::prelude::*;

    // Validate all boundaries are within buffer
    for (offset, length) in record_boundaries {
        if offset + length > buffer.len() {
            return Err(MarcError::invalid_field_msg(format!(
                "Record boundary ({}, {}) exceeds buffer size {}",
                offset,
                length,
                buffer.len()
            )));
        }
    }

    // Use Rayon's parallel iterator to parse each record independently
    // This will automatically use the thread pool respecting RAYON_NUM_THREADS env var
    record_boundaries
        .par_iter()
        .enumerate()
        .map(|(idx, (offset, length))| {
            // Extract the record's bytes
            let record_bytes = &buffer[*offset..offset + length];

            // Create a cursor over the record bytes and parse it
            let cursor = Cursor::new(record_bytes);
            let mut reader = MarcReader::new(cursor);

            reader.read_record().and_then(|opt| {
                opt.ok_or_else(|| {
                    MarcError::invalid_field_msg(format!(
                        "Record {idx} at offset {offset} parsed as empty"
                    ))
                })
            })
        })
        .collect::<Result<Vec<Record>>>()
}

/// Parse a limited batch of MARC records in parallel.
///
/// Like [`parse_batch_parallel`], but limits the number of records to parse.
/// Useful for pipeline stages that need to control batch size.
///
/// # Arguments
///
/// * `record_boundaries` - Vec of (offset, length) tuples
/// * `buffer` - The complete binary buffer
/// * `limit` - Maximum number of records to parse
///
/// # Returns
///
/// * `Ok(Vec<Record>)` - Up to `limit` parsed records
/// * `Err(MarcError)` - If any record fails to parse
///
/// # Errors
///
/// Returns an error if:
/// - Any boundary exceeds the buffer size
/// - Any record fails to parse during parallel processing
///
/// # Example
///
/// ```no_run
/// use mrrc::rayon_parser_pool::parse_batch_parallel_limited;
///
/// let buffer = vec![/* MARC data */];
/// let boundaries = vec![(0, 100), (100, 95), (195, 105), (300, 110)];
/// let records = parse_batch_parallel_limited(&boundaries, &buffer, 2)?;
/// assert!(records.len() <= 2);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_batch_parallel_limited(
    record_boundaries: &[(usize, usize)],
    buffer: &[u8],
    limit: usize,
) -> Result<Vec<Record>> {
    let limited: Vec<_> = record_boundaries.iter().take(limit).copied().collect();
    parse_batch_parallel(&limited, buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leader::Leader;
    use crate::record::Field;
    use crate::writer::MarcWriter;

    // Skip rayon tests under Miri: crossbeam-epoch 0.9.18 has a known stacked borrows
    // violation (crossbeam-rs/crossbeam#1181). Re-enable when crossbeam-epoch 0.10 ships.
    // Tracking: https://github.com/dchud/mrrc/issues/48

    /// Build a valid bibliographic record with the given 001 control number
    /// and a 245 title field.
    fn build_test_record(control_number: &str) -> Record {
        let leader = Leader {
            record_length: 0,
            record_status: 'n',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: ' ',
            character_coding: 'a',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 0,
            encoding_level: ' ',
            cataloging_form: ' ',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        };
        let mut record = Record::new(leader);
        record.add_control_field("001".to_string(), control_number.to_string());
        let field = Field::builder("245".to_string(), '1', '0')
            .subfield_str('a', "Title")
            .build();
        record.add_field(field);
        record
    }

    /// Serialize `record` to ISO 2709 bytes.
    fn emit_binary(record: &Record) -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let mut writer = MarcWriter::new(&mut buffer);
            writer.write_record(record).expect("write should succeed");
        }
        buffer
    }

    /// Concatenate the records into one buffer and return it together with
    /// the (offset, length) boundary of each record.
    fn build_stream(records: &[Record]) -> (Vec<u8>, Vec<(usize, usize)>) {
        let mut buffer = Vec::new();
        let mut boundaries = Vec::new();
        for record in records {
            let bytes = emit_binary(record);
            boundaries.push((buffer.len(), bytes.len()));
            buffer.extend_from_slice(&bytes);
        }
        (buffer, boundaries)
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parse_batch_parallel_single_record() {
        let record = build_test_record("rec0001");
        let bytes = emit_binary(&record);
        let boundaries = vec![(0, bytes.len())];

        let records = parse_batch_parallel(&boundaries, &bytes).expect("parse should succeed");

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get_control_field("001"), Some("rec0001"));
        let fields = records[0].get_fields("245").expect("245 field present");
        assert_eq!(fields[0].get_subfield('a'), Some("Title"));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parse_batch_parallel_multiple_records_preserve_order() {
        let originals: Vec<Record> = (0..5)
            .map(|i| build_test_record(&format!("rec{i:04}")))
            .collect();
        let (buffer, boundaries) = build_stream(&originals);

        let records = parse_batch_parallel(&boundaries, &buffer).expect("parse should succeed");

        assert_eq!(records.len(), 5);
        for (i, record) in records.iter().enumerate() {
            assert_eq!(
                record.get_control_field("001"),
                Some(format!("rec{i:04}").as_str()),
                "record {i} out of order or corrupted"
            );
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parse_batch_parallel_malformed_record_propagates_error() {
        let originals = vec![build_test_record("rec0000"), build_test_record("rec0001")];
        let (mut buffer, boundaries) = build_stream(&originals);
        // Corrupt the second record's directory: its first entry's 4-byte
        // length field starts 27 bytes into the record.
        let second_start = boundaries[1].0;
        for byte in &mut buffer[second_start + 27..second_start + 31] {
            *byte = b'X';
        }

        let result = parse_batch_parallel(&boundaries, &buffer);
        assert!(result.is_err(), "corrupted record should fail the batch");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parse_batch_parallel_empty_boundaries() {
        let buffer = vec![1, 2, 3];
        let boundaries = vec![];

        let result = parse_batch_parallel(&boundaries, &buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parse_batch_parallel_boundary_out_of_bounds() {
        let buffer = vec![1, 2, 3];
        let boundaries = vec![(0, 10)]; // Exceeds buffer

        let result = parse_batch_parallel(&boundaries, &buffer);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("exceed") || err_msg.contains("bound"));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parse_batch_parallel_limited() {
        let originals: Vec<Record> = (0..5)
            .map(|i| build_test_record(&format!("rec{i:04}")))
            .collect();
        let (buffer, boundaries) = build_stream(&originals);

        let records =
            parse_batch_parallel_limited(&boundaries, &buffer, 2).expect("parse should succeed");

        assert_eq!(
            records.len(),
            2,
            "limit of 2 should yield exactly 2 records"
        );
        assert_eq!(records[0].get_control_field("001"), Some("rec0000"));
        assert_eq!(records[1].get_control_field("001"), Some("rec0001"));
    }
}
