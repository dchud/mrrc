//! Reading MARC Holdings records from binary streams.
//!
//! This module provides specialized reading functionality for MARC Holdings records (Type x/y/v/u).
//! Holdings records use the same ISO 2709 binary format as bibliographic and authority records
//! but are parsed into [`HoldingsRecord`] structures.
//!
//! # Holdings MARC Reader
//!
//! `HoldingsMarcReader` supports the
//! unified `ReaderBackend` enum interface enabling:
//! - **`RustFile`**: Direct file I/O via `std::fs::File` (enables Rayon parallelism)
//! - **`CursorBackend`**: In-memory reads from bytes via `std::io::Cursor`
//! - **`PythonFile`**: Python file-like objects (requires GIL, used sequentially)
//!
//! Example with `RustFile` for parallel processing:
//! ```no_run
//! use mrrc::holdings_reader::HoldingsMarcReader;
//! use std::fs::File;
//!
//! let file = File::open("holdings_records.mrc")?;
//! let mut reader = HoldingsMarcReader::new(file);
//!
//! while let Some(record) = reader.read_record()? {
//!     println!("Holdings locations: {}", record.locations().len());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::error::{MarcError, Result};
use crate::holdings_record::HoldingsRecord;
use crate::leader::Leader;
use crate::record::Field;
use crate::recovery::RecoveryMode;
use std::io::Read;

const FIELD_TERMINATOR: u8 = 0x1E;
const SUBFIELD_DELIMITER: u8 = 0x1F;

/// Reader for ISO 2709 binary MARC Holdings records.
///
/// `HoldingsMarcReader` reads Holdings record data and converts it to
/// [`HoldingsRecord`] instances. It reuses the same binary format as bibliographic
/// records but organizes fields by their functional role (locations, captions, enumeration, etc.).
///
/// # Backends
///
/// The reader automatically detects the input source:
/// - File paths → `RustFile` backend (parallel-safe)
/// - Bytes/BytesIO → `CursorBackend` (parallel-safe)
/// - Python file objects → `PythonFile` (sequential, requires GIL)
#[derive(Debug)]
pub struct HoldingsMarcReader<R: Read> {
    reader: R,
    recovery_mode: RecoveryMode,
}

impl<R: Read> HoldingsMarcReader<R> {
    /// Create a new Holdings MARC reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - Any source implementing [`std::io::Read`]
    #[must_use]
    pub fn new(reader: R) -> Self {
        HoldingsMarcReader {
            reader,
            recovery_mode: RecoveryMode::Strict,
        }
    }

    /// Set the recovery mode for handling malformed records.
    ///
    /// The recovery mode determines how the reader handles truncated or
    /// malformed Holdings records:
    /// - `Strict`: Return errors immediately (default)
    /// - `Lenient`: Attempt to recover and salvage valid data
    /// - `Permissive`: Be very lenient, accepting partial data
    #[must_use]
    pub fn with_recovery_mode(mut self, mode: RecoveryMode) -> Self {
        self.recovery_mode = mode;
        self
    }

    /// Read the next Holdings record from the stream.
    ///
    /// Returns `Ok(None)` when the end of file is reached.
    ///
    /// # Errors
    ///
    /// Returns an error if the binary data is malformed or an I/O error occurs.
    #[allow(clippy::too_many_lines)]
    pub fn read_record(&mut self) -> Result<Option<HoldingsRecord>> {
        // Read the leader (24 bytes)
        let mut leader_bytes = vec![0u8; 24];
        match self.reader.read_exact(&mut leader_bytes) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(None);
            },
            Err(e) => return Err(MarcError::IoError(e)),
        }

        let leader = Leader::from_bytes(&leader_bytes)?;

        // Verify this is a holdings record (Type x/y/v/u)
        if !matches!(leader.record_type, 'x' | 'y' | 'v' | 'u') {
            return Err(MarcError::InvalidRecord(format!(
                "Expected holdings record type (x/y/v/u), got '{}'",
                leader.record_type
            )));
        }

        // Calculate directory and data sizes
        let record_length = leader.record_length as usize;
        let base_address = leader.data_base_address as usize;
        let directory_size = base_address - 24;

        // Read record data
        let mut record_data = vec![0u8; record_length - 24];
        match self.reader.read_exact(&mut record_data) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                if self.recovery_mode == RecoveryMode::Strict {
                    return Err(MarcError::TruncatedRecord(
                        "Unexpected end of file while reading record data".to_string(),
                    ));
                }
            },
            Err(e) => return Err(MarcError::IoError(e)),
        }

        // Handle truncation in recovery mode
        if record_data.len() < (record_length - 24) && self.recovery_mode != RecoveryMode::Strict {
            return Err(MarcError::TruncatedRecord(
                "Holdings record is truncated".to_string(),
            ));
        }

        // Extract directory and data sections
        let directory_bytes = &record_data[..directory_size];
        let data_section = &record_data[directory_size..];

        // Parse directory (entries are 12 bytes each: 3 bytes tag + 4 bytes length + 5 bytes start position)
        let mut fields: indexmap::IndexMap<String, Vec<Field>> = indexmap::IndexMap::new();
        let mut control_fields: indexmap::IndexMap<String, String> = indexmap::IndexMap::new();

        let mut i = 0;
        while i + 12 <= directory_bytes.len() {
            let tag = std::str::from_utf8(&directory_bytes[i..i + 3])
                .map_err(|_| MarcError::InvalidRecord("Invalid tag encoding".to_string()))?
                .to_string();

            let length_str = std::str::from_utf8(&directory_bytes[i + 3..i + 7])
                .map_err(|_| MarcError::InvalidRecord("Invalid field length".to_string()))?;
            let length: usize = length_str
                .parse()
                .map_err(|_| MarcError::InvalidRecord("Invalid field length value".to_string()))?;

            let start_str = std::str::from_utf8(&directory_bytes[i + 7..i + 12])
                .map_err(|_| MarcError::InvalidRecord("Invalid start position".to_string()))?;
            let start: usize = start_str.parse().map_err(|_| {
                MarcError::InvalidRecord("Invalid start position value".to_string())
            })?;

            let end = start + length;
            if end > data_section.len() {
                return Err(MarcError::InvalidRecord(
                    "Field extends beyond data section".to_string(),
                ));
            }

            let field_data = &data_section[start..end];

            // Parse field content
            if tag.chars().all(|c| c.is_ascii_digit()) && tag.parse::<u16>().unwrap_or(0) < 10 {
                // Control field (000-009)
                let value = std::str::from_utf8(&field_data[..field_data.len() - 1])
                    .map_err(|_| {
                        MarcError::InvalidRecord("Invalid control field encoding".to_string())
                    })?
                    .to_string();
                control_fields.insert(tag, value);
            } else {
                // Data field
                if field_data.len() < 3 {
                    return Err(MarcError::InvalidRecord(
                        "Data field too short for indicators".to_string(),
                    ));
                }

                let indicator1 = field_data[0] as char;
                let indicator2 = field_data[1] as char;
                let mut subfields = smallvec::SmallVec::new();
                let mut j = 2;

                while j < field_data.len() - 1 {
                    if field_data[j] == SUBFIELD_DELIMITER {
                        j += 1;
                        if j >= field_data.len() - 1 {
                            break;
                        }
                        let code = field_data[j] as char;
                        j += 1;
                        let mut value_bytes = Vec::new();
                        while j < field_data.len()
                            && field_data[j] != SUBFIELD_DELIMITER
                            && field_data[j] != FIELD_TERMINATOR
                        {
                            value_bytes.push(field_data[j]);
                            j += 1;
                        }
                        let value = std::str::from_utf8(&value_bytes)
                            .map_err(|_| {
                                MarcError::InvalidRecord("Invalid subfield encoding".to_string())
                            })?
                            .to_string();
                        subfields.push(crate::record::Subfield { code, value });
                    } else {
                        j += 1;
                    }
                }

                let field = Field {
                    tag: tag.clone(),
                    indicator1,
                    indicator2,
                    subfields,
                };

                fields.entry(tag).or_default().push(field);
            }

            i += 12;
        }

        // Create HoldingsRecord and organize fields by type
        let mut record = HoldingsRecord::new(leader);

        // Add control fields
        for (tag, value) in control_fields {
            record.add_control_field(tag, value);
        }

        // Organize data fields by their functional role
        for (tag, field_list) in fields {
            for field in field_list {
                match tag.as_str() {
                    "852" => record.add_location(field),
                    "853" => record.add_captions_basic(field),
                    "854" => record.add_captions_supplements(field),
                    "855" => record.add_captions_indexes(field),
                    "863" => record.add_enumeration_basic(field),
                    "864" => record.add_enumeration_supplements(field),
                    "865" => record.add_enumeration_indexes(field),
                    "866" => record.add_textual_holdings_basic(field),
                    "867" => record.add_textual_holdings_supplements(field),
                    "868" => record.add_textual_holdings_indexes(field),
                    "876" | "877" | "878" => record.add_item_information(field),
                    _ => record.add_field(field),
                }
            }
        }

        Ok(Some(record))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_holdings_record_roundtrip() {
        // Use writer to create valid binary, then reader to parse it
        use crate::holdings_record::HoldingsRecord;
        use crate::leader::Leader;
        use crate::record::{Field, Subfield};

        let leader = Leader {
            record_length: 1024,
            record_status: 'n',
            record_type: 'x',
            bibliographic_level: '|',
            control_record_type: ' ',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 300,
            encoding_level: '1',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        };

        let mut record = HoldingsRecord::new(leader);
        record.add_control_field("001".to_string(), "ocm00098765".to_string());

        let location = Field {
            tag: "852".to_string(),
            indicator1: ' ',
            indicator2: '1',
            subfields: smallvec::smallvec![Subfield {
                code: 'b',
                value: "Main Library".to_string(),
            }],
        };
        record.add_location(location);

        // Write it
        let mut buffer = Vec::new();
        {
            let mut writer = crate::holdings_writer::HoldingsMarcWriter::new(&mut buffer);
            writer
                .write_record(&record)
                .expect("Failed to write record");
        }

        // Read it back
        let reader = std::io::Cursor::new(buffer);
        let mut marc_reader = HoldingsMarcReader::new(reader);
        let parsed = marc_reader.read_record().expect("Failed to read record");
        assert!(parsed.is_some());

        let parsed = parsed.unwrap();
        assert_eq!(parsed.leader.record_type, 'x');
        assert_eq!(parsed.get_control_field("001"), Some("ocm00098765"));
        assert_eq!(parsed.locations().len(), 1);
    }

    #[test]
    fn test_read_holdings_record_eof() {
        let binary_data = Vec::new();
        let reader = std::io::Cursor::new(binary_data);
        let mut marc_reader = HoldingsMarcReader::new(reader);

        let record = marc_reader.read_record().expect("Failed to read");
        assert!(record.is_none());
    }

    #[test]
    fn test_read_invalid_record_type() {
        let mut data = Vec::new();
        // Leader with invalid record type 'a' (bibliographic)
        let leader = "00325a  a2200121u  4500";
        data.extend_from_slice(leader.as_bytes());
        data.extend_from_slice(&[0u8; 100]);

        let reader = std::io::Cursor::new(data);
        let mut marc_reader = HoldingsMarcReader::new(reader);

        let result = marc_reader.read_record();
        assert!(result.is_err());
    }
}
