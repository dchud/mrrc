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
#[cfg(test)]
use crate::error::MarcError;
use crate::error::Result;
use crate::iso2709::{self, DataFieldParseConfig, ParseContext, FIELD_TERMINATOR, LEADER_LEN};
use crate::leader::Leader;
use crate::recovery::RecoveryMode;
use std::io::Read;

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
    ctx: ParseContext,
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
            ctx: ParseContext::new(),
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

    /// Attach a source identifier (filename or stream id) to errors raised by
    /// this reader. Populates `source_name` on every emitted error where
    /// applicable. Use [`AuthorityMarcReader::from_path`] when constructing
    /// from a filesystem path to set this automatically.
    #[must_use]
    pub fn with_source(mut self, name: impl Into<String>) -> Self {
        self.ctx.source_name = Some(name.into());
        self
    }
}

impl AuthorityMarcReader<std::fs::File> {
    /// Open `path` for reading and create an [`AuthorityMarcReader`] whose
    /// errors include the path as their `source_name`.
    ///
    /// # Errors
    ///
    /// Returns the underlying [`std::io::Error`] if the file cannot be opened.
    pub fn from_path(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        let file = std::fs::File::open(path)?;
        Ok(Self::new(file).with_source(path.display().to_string()))
    }
}

impl<R: Read> AuthorityMarcReader<R> {
    /// Read the next Authority record from the stream.
    ///
    /// Returns `Ok(None)` when the end of file is reached.
    ///
    /// # Errors
    ///
    /// Returns an error if the binary data is malformed or an I/O error occurs.
    #[allow(clippy::too_many_lines)]
    pub fn read_record(&mut self) -> Result<Option<AuthorityRecord>> {
        // Read the leader (24 bytes). EOF here is a clean stream end.
        let Some(leader_bytes) = iso2709::read_leader_bytes(&mut self.reader)? else {
            return Ok(None);
        };

        self.ctx.begin_record();

        // Leader errors bypass ParseContext; enrich with a byte window so
        // `detailed()` can render a hex dump.
        let leader_offset = self.ctx.stream_byte_offset;
        let leader = Leader::from_bytes(&leader_bytes)
            .map_err(|e| e.with_bytes_near(&leader_bytes, leader_offset))?;
        leader
            .validate_for_reading()
            .map_err(|e| e.with_bytes_near(&leader_bytes, leader_offset))?;

        // Verify this is an authority record (Type Z)
        if leader.record_type != 'z' {
            return Err(self.ctx.err_invalid_field(format!(
                "Expected authority record type 'z', got '{}'",
                leader.record_type
            )));
        }

        let record_length = leader.record_length as usize;
        let base_address = leader.data_base_address as usize;
        let directory_size = base_address - 24;

        self.ctx.advance(LEADER_LEN);

        // Read record data. In non-Strict modes a short read returns
        // `(buffer, true)`; salvage from a partial buffer is not implemented,
        // so the recovery dispatch below is unreachable in practice (the read
        // primitive returns a buffer of full length even on a short read).
        let (record_data, _was_truncated) = iso2709::read_record_data(
            &mut self.reader,
            record_length,
            self.recovery_mode,
            &self.ctx,
        )?;

        // Hand the record buffer to the context for bytes_near capture on
        // any error raised during directory/field parsing.
        let record_data_offset = self.ctx.stream_byte_offset;
        self.ctx.set_parse_buffer(&record_data, record_data_offset);

        if record_data.len() < (record_length - 24) && self.recovery_mode != RecoveryMode::Strict {
            return Err(self.ctx.err_truncated_record(
                Some(record_length.saturating_sub(LEADER_LEN)),
                Some(record_data.len()),
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

        let mut record = AuthorityRecord::new(leader);

        // Parse directory and extract fields
        let mut pos = 0;
        while pos < directory.len() {
            // Move stream_byte_offset to the current directory byte so
            // errors below carry a precise byte_offset (and the bytes_near
            // hex-dump caret lands at the actual offending byte).
            self.ctx.stream_byte_offset = record_data_offset + pos;
            if directory[pos] == FIELD_TERMINATOR {
                break;
            }

            if pos + 12 > directory.len() {
                if self.recovery_mode == RecoveryMode::Strict {
                    return Err(self.ctx.err_directory_invalid(
                        Some(&directory[pos..]),
                        "complete 12-byte directory entry",
                    ));
                }
                break;
            }

            // Parse directory entry (12 bytes: tag + length + start position)
            let tag = std::str::from_utf8(&directory[pos..pos + 3])
                .map_err(|_| self.ctx.err_invalid_field("Invalid field tag"))?
                .to_string();

            let length = iso2709::parse_4digits(&directory[pos + 3..pos + 7])?;
            let start = iso2709::parse_5digits(&directory[pos + 7..pos + 12])?;

            pos += 12;

            // Extract field data
            let field_data_start = data_start + start;
            let field_data_end = std::cmp::min(field_data_start + length, record_data.len());

            if field_data_start >= record_data.len() {
                continue;
            }

            let field_bytes = &record_data[field_data_start..field_data_end];

            // Check if this is a control field (00X-009)
            if iso2709::is_control_field_tag(&tag) {
                // Control field - no indicators or subfields
                let value = String::from_utf8_lossy(field_bytes)
                    .trim_end_matches(['\x1E', '\x1F'])
                    .to_string();
                if tag == "001" {
                    self.ctx.record_control_number = Some(value.clone());
                }
                record.add_control_field(tag, value);
            } else {
                // Data field - has indicators and subfields. Authority preserves
                // its historical permissive/lossy behavior via the AUTHORITY
                // config; bytes in field_bytes shorter than 2 are ignored to
                // match the prior `if field_bytes.len() < 2 { continue }` guard.
                if field_bytes.len() < 2 {
                    continue;
                }
                self.ctx.current_field_tag = Some(tag.clone());
                self.ctx.stream_byte_offset = record_data_offset + field_data_start;
                let parsed = iso2709::parse_data_field(
                    field_bytes,
                    &tag,
                    DataFieldParseConfig::AUTHORITY,
                    &self.ctx,
                );
                self.ctx.current_field_tag = None;
                let field = match parsed {
                    Ok(f) => f,
                    Err(e) => {
                        if self.recovery_mode == RecoveryMode::Strict {
                            return Err(e);
                        }
                        continue;
                    },
                };

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

        // Restore stream_byte_offset to the end of the current record.
        // The directory/field loop above moved it mid-record for precise
        // error alignment; this restores the bytes-consumed invariant.
        self.ctx.stream_byte_offset = record_data_offset + record_length.saturating_sub(LEADER_LEN);
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
            Err(MarcError::InvalidField { ref message, .. })
                if message.contains("Expected authority record type") =>
            {
                // Expected error
            },
            other => panic!("Should have returned type mismatch error, got: {other:?}"),
        }
    }
}
