//! Reading MARC Authority records from binary streams.
//!
//! This module provides specialized reading functionality for MARC Authority records (Type Z).
//! Authority records use the same ISO 2709 binary format as bibliographic records but are
//! parsed into [`AuthorityRecord`] structures instead of [`crate::record::Record`] structures.
//!
//! # Authority MARC Reader
//!
//! `AuthorityMarcReader` supports the
//! unified `ReaderBackend` enum interface enabling:
//! - **`RustFile`**: Direct file I/O via `std::fs::File` (enables Rayon parallelism)
//! - **`CursorBackend`**: In-memory reads from bytes via `std::io::Cursor`
//! - **`PythonFile`**: Python file-like objects (requires GIL, used sequentially)
//!
//! Example with `RustFile` for parallel processing:
//! ```no_run
//! use mrrc::authority_reader::AuthorityMarcReader;
//! use std::fs::File;
//!
//! let file = File::open("authority_records.mrc")?;
//! let mut reader = AuthorityMarcReader::new(file);
//!
//! while let Some(record) = reader.read_record()? {
//!     if let Some(heading) = record.heading() {
//!         println!("Authority: {}", heading.value());
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::authority_record::AuthorityRecord;
use crate::error::{MarcError, Result};
use crate::leader::Leader;
use crate::record::Field;
use crate::recovery::RecoveryMode;
use std::io::Read;

const FIELD_TERMINATOR: u8 = 0x1E;
const SUBFIELD_DELIMITER: u8 = 0x1F;

/// Reader for ISO 2709 binary MARC Authority records.
///
/// `AuthorityMarcReader` reads Authority record data and converts it to
/// [`AuthorityRecord`] instances. It reuses the same binary format as bibliographic
/// records but organizes fields by their functional role (heading, tracings, notes, etc.).
///
/// # Backends
///
/// The reader automatically detects the input source:
/// - File paths → `RustFile` backend (parallel-safe)
/// - Bytes/BytesIO → `CursorBackend` (parallel-safe)
/// - Python file objects → `PythonFile` (sequential, requires GIL)
#[derive(Debug)]
pub struct AuthorityMarcReader<R: Read> {
    reader: R,
    recovery_mode: RecoveryMode,
}

impl<R: Read> AuthorityMarcReader<R> {
    /// Create a new Authority MARC reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - Any source implementing [`std::io::Read`]
    #[must_use]
    pub fn new(reader: R) -> Self {
        AuthorityMarcReader {
            reader,
            recovery_mode: RecoveryMode::Strict,
        }
    }

    /// Set the recovery mode for handling malformed records.
    ///
    /// The recovery mode determines how the reader handles truncated or
    /// malformed Authority records:
    /// - `Strict`: Return errors immediately (default)
    /// - `Lenient`: Attempt to recover and salvage valid data
    /// - `Permissive`: Be very lenient, accepting partial data
    #[must_use]
    pub fn with_recovery_mode(mut self, mode: RecoveryMode) -> Self {
        self.recovery_mode = mode;
        self
    }

    /// Read the next Authority record from the stream.
    ///
    /// Returns `Ok(None)` when the end of file is reached.
    ///
    /// # Errors
    ///
    /// Returns an error if the binary data is malformed or an I/O error occurs.
    #[allow(clippy::too_many_lines)]
    pub fn read_record(&mut self) -> Result<Option<AuthorityRecord>> {
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
        leader.validate_for_reading()?;

        // Verify this is an authority record (Type Z)
        if leader.record_type != 'z' {
            return Err(MarcError::InvalidRecord(format!(
                "Expected authority record type 'z', got '{}'",
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
            // For now, treat truncated authority records as a strict error
            // In the future, we could implement recovery logic
            return Err(MarcError::TruncatedRecord(
                "Authority record is truncated".to_string(),
            ));
        }

        // Extract directory and data sections
        let directory_end = std::cmp::min(directory_size, record_data.len());
        let directory = if directory_end > 0 {
            &record_data[..directory_end]
        } else {
            &[]
        };
        let data_start = std::cmp::min(base_address - 24, record_data.len());
        let data = if data_start < record_data.len() {
            &record_data[data_start..]
        } else {
            &[]
        };
        let _ = data; // Will be used when parsing fields

        let mut record = AuthorityRecord::new(leader);

        // Parse directory and extract fields
        let mut pos = 0;
        while pos < directory.len() {
            if directory[pos] == FIELD_TERMINATOR {
                break;
            }

            if pos + 12 > directory.len() {
                if self.recovery_mode == RecoveryMode::Strict {
                    return Err(MarcError::InvalidRecord(
                        "Incomplete directory entry".to_string(),
                    ));
                }
                break;
            }

            // Parse directory entry (12 bytes: tag + length + start position)
            let tag = std::str::from_utf8(&directory[pos..pos + 3])
                .map_err(|_| MarcError::InvalidRecord("Invalid field tag".to_string()))?
                .to_string();

            let length = std::str::from_utf8(&directory[pos + 3..pos + 7])
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or_else(|| MarcError::InvalidRecord("Invalid field length".to_string()))?;

            let start = std::str::from_utf8(&directory[pos + 7..pos + 12])
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or_else(|| {
                    MarcError::InvalidRecord("Invalid field start position".to_string())
                })?;

            pos += 12;

            // Extract field data
            let field_data_start = data_start + start;
            let field_data_end = std::cmp::min(field_data_start + length, record_data.len());

            if field_data_start >= record_data.len() {
                continue;
            }

            let field_bytes = &record_data[field_data_start..field_data_end];

            // Check if this is a control field (00X-009)
            if tag.len() == 3 && tag.as_str() < "010" {
                // Control field - no indicators or subfields
                let value = String::from_utf8_lossy(field_bytes)
                    .trim_end_matches(['\x1E', '\x1F'])
                    .to_string();
                record.add_control_field(tag, value);
            } else {
                // Data field - has indicators and subfields
                if field_bytes.len() < 2 {
                    continue;
                }

                let indicator1 = field_bytes[0] as char;
                let indicator2 = field_bytes[1] as char;

                let mut field = Field {
                    tag: tag.clone(),
                    indicator1,
                    indicator2,
                    subfields: smallvec::SmallVec::new(),
                };

                // Parse subfields
                let mut subfield_pos = 2;
                while subfield_pos < field_bytes.len() {
                    if field_bytes[subfield_pos] == SUBFIELD_DELIMITER {
                        if subfield_pos + 1 < field_bytes.len() {
                            let code = field_bytes[subfield_pos + 1] as char;
                            subfield_pos += 2;

                            // Find the end of this subfield
                            let mut subfield_end = subfield_pos;
                            while subfield_end < field_bytes.len()
                                && field_bytes[subfield_end] != SUBFIELD_DELIMITER
                                && field_bytes[subfield_end] != FIELD_TERMINATOR
                            {
                                subfield_end += 1;
                            }

                            let value =
                                String::from_utf8_lossy(&field_bytes[subfield_pos..subfield_end])
                                    .to_string();
                            field
                                .subfields
                                .push(crate::record::Subfield { code, value });

                            subfield_pos = subfield_end;
                        } else {
                            break;
                        }
                    } else if field_bytes[subfield_pos] == FIELD_TERMINATOR {
                        break;
                    } else {
                        subfield_pos += 1;
                    }
                }

                // Organize field by its function based on tag
                match tag.as_str() {
                    "100" | "110" | "111" | "130" | "148" | "150" | "151" | "155" => {
                        // 1XX fields - the main heading
                        record.set_heading(field);
                    },
                    "400" | "410" | "411" | "430" | "448" | "450" | "451" | "455" => {
                        // 4XX fields - see from tracings
                        record.add_see_from_tracing(field);
                    },
                    "500" | "510" | "511" | "530" | "548" | "550" | "551" | "555" => {
                        // 5XX fields - see also from tracings
                        record.add_see_also_tracing(field);
                    },
                    "660" | "661" | "662" | "663" | "664" | "665" | "666" | "667" | "668"
                    | "669" | "670" | "671" | "672" | "673" | "674" | "675" | "676" | "677"
                    | "678" | "679" | "680" | "681" | "682" | "683" | "684" | "685" | "686"
                    | "687" | "688" | "689" => {
                        // 66X-68X fields - notes
                        record.add_note(field);
                    },
                    "700" | "710" | "711" | "730" | "748" | "750" | "751" | "755" => {
                        // 7XX fields - heading linking entries
                        record.add_linking_entry(field);
                    },
                    _ => {
                        // Other fields
                        record.add_field(field);
                    },
                }
            }
        }

        Ok(Some(record))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_authority_reader_creation() {
        let data = vec![];
        let cursor = Cursor::new(data);
        let _reader = AuthorityMarcReader::new(cursor);
    }

    #[test]
    fn test_authority_reader_empty() {
        let data = vec![];
        let cursor = Cursor::new(data);
        let mut reader = AuthorityMarcReader::new(cursor);

        match reader.read_record() {
            Ok(None) => {}, // Expected
            Ok(Some(_)) => panic!("Should not have read a record"),
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[test]
    fn test_authority_reader_wrong_type() {
        // Create minimal bibliographic record leader (type 'a')
        let mut data = vec![];
        // Record length
        data.extend_from_slice(b"00029");
        // Record status
        data.push(b'n');
        // Type of record - 'a' for bibliographic (not 'z' for authority)
        data.push(b'a');
        // Bibliographic level
        data.push(b'm');
        // Control record type
        data.push(b' ');
        // Character coding
        data.push(b' ');
        // Indicator count
        data.push(b'2');
        // Subfield code count
        data.push(b'2');
        // Base address of data
        data.extend_from_slice(b"00025");
        // Encoding level
        data.push(b' ');
        // Cataloging form
        data.push(b'a');
        // Multipart level
        data.push(b' ');
        // Reserved
        data.extend_from_slice(b"4500");
        // Minimal directory and data
        data.push(FIELD_TERMINATOR);
        // Record terminator
        data.push(0x1D);

        let cursor = Cursor::new(data);
        let mut reader = AuthorityMarcReader::new(cursor);

        match reader.read_record() {
            Err(MarcError::InvalidRecord(msg))
                if msg.contains("Expected authority record type") =>
            {
                // Expected error
            },
            other => panic!("Should have returned type mismatch error, got: {other:?}"),
        }
    }
}
