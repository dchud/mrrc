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

use crate::error::Result;
use crate::holdings_record::HoldingsRecord;
use crate::iso2709::{self, DataFieldParseConfig, ParseContext, FIELD_TERMINATOR, LEADER_LEN};
use crate::leader::Leader;
use crate::recovery::RecoveryMode;
use std::io::Read;

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
    ctx: ParseContext,
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
            ctx: ParseContext::new(),
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

    /// Attach a source identifier (filename or stream id) to errors raised by
    /// this reader. Populates `source_name` on every emitted error where
    /// applicable. Use [`HoldingsMarcReader::from_path`] when constructing
    /// from a filesystem path to set this automatically.
    #[must_use]
    pub fn with_source(mut self, name: impl Into<String>) -> Self {
        self.ctx.source_name = Some(name.into());
        self
    }
}

impl HoldingsMarcReader<std::fs::File> {
    /// Open `path` for reading and create a [`HoldingsMarcReader`] whose
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

impl<R: Read> HoldingsMarcReader<R> {
    /// Read the next Holdings record from the stream.
    ///
    /// Returns `Ok(None)` when the end of file is reached.
    ///
    /// # Errors
    ///
    /// Returns an error if the binary data is malformed or an I/O error occurs.
    #[allow(clippy::too_many_lines)]
    pub fn read_record(&mut self) -> Result<Option<HoldingsRecord>> {
        // Read the leader (24 bytes). EOF here is a clean stream end.
        let Some(leader_bytes) = iso2709::read_leader_bytes(&mut self.reader)? else {
            return Ok(None);
        };

        self.ctx.begin_record();

        let leader = Leader::from_bytes(&leader_bytes)?;
        leader.validate_for_reading()?;

        // Verify this is a holdings record (Type x/y/v/u)
        if !matches!(leader.record_type, 'x' | 'y' | 'v' | 'u') {
            return Err(self.ctx.err_invalid_field(format!(
                "Expected holdings record type (x/y/v/u), got '{}'",
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

        if record_data.len() < (record_length - 24) && self.recovery_mode != RecoveryMode::Strict {
            return Err(self.ctx.err_truncated_record(
                Some(record_length.saturating_sub(LEADER_LEN)),
                Some(record_data.len()),
            ));
        }

        let directory_bytes = &record_data[..directory_size];
        let data_section = &record_data[directory_size..];

        let mut record = HoldingsRecord::new(leader);

        let mut i = 0;
        while i + 12 <= directory_bytes.len() {
            // Stop on directory terminator (consistent with bib + authority).
            if directory_bytes[i] == FIELD_TERMINATOR {
                break;
            }

            let tag = std::str::from_utf8(&directory_bytes[i..i + 3])
                .map_err(|_| self.ctx.err_invalid_field("Invalid tag encoding"))?
                .to_string();

            let length = iso2709::parse_4digits(&directory_bytes[i + 3..i + 7])?;
            let start = iso2709::parse_5digits(&directory_bytes[i + 7..i + 12])?;

            let end = start + length;
            if end > data_section.len() {
                self.ctx.current_field_tag = Some(tag.clone());
                return Err(self
                    .ctx
                    .err_invalid_field(format!("Field {tag} extends beyond data section")));
            }

            let field_data = &data_section[start..end];

            if iso2709::is_control_field_tag(&tag) {
                // Control field: strip the trailing field terminator. Holdings
                // historically used strict UTF-8 here, so propagate that
                // behavior via the EncodingError variant.
                let raw = field_data
                    .get(..field_data.len().saturating_sub(1))
                    .unwrap_or(&[]);
                let value = std::str::from_utf8(raw)
                    .map_err(|e| {
                        self.ctx
                            .err_encoding(format!("Invalid UTF-8 in control field {tag}: {e}"))
                    })?
                    .to_string();
                if tag == "001" {
                    self.ctx.record_control_number = Some(value.clone());
                }
                record.add_control_field(tag, value);
                i += 12;
                continue;
            }

            // Data field
            if field_data.len() < 3 {
                self.ctx.current_field_tag = Some(tag.clone());
                return Err(self
                    .ctx
                    .err_invalid_field("Data field too short for indicators"));
            }

            self.ctx.current_field_tag = Some(tag.clone());
            let parsed = iso2709::parse_data_field(
                field_data,
                &tag,
                DataFieldParseConfig::HOLDINGS,
                &self.ctx,
            );
            self.ctx.current_field_tag = None;
            let field = parsed?;

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

            i += 12;
        }

        self.ctx.advance(record_length.saturating_sub(LEADER_LEN));
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
