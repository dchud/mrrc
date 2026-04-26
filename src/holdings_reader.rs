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
use crate::iso2709::{self, DataFieldParseConfig, ParseContext, FIELD_TERMINATOR, LEADER_LEN};
use crate::leader::Leader;
use crate::reader::DEFAULT_MAX_ERRORS;
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
    max_errors: usize,
    error_count: usize,
    cap_exceeded: bool,
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
            max_errors: DEFAULT_MAX_ERRORS,
            error_count: 0,
            cap_exceeded: false,
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

    /// Cap the number of recovered errors tolerated in one stream before the
    /// reader raises [`crate::MarcError::FatalReaderError`] and halts.
    ///
    /// See [`crate::MarcReader::with_max_errors`] for semantics; passing `0`
    /// disables the cap (unbounded accumulation). Default when not set is
    /// [`DEFAULT_MAX_ERRORS`].
    #[must_use]
    pub fn with_max_errors(mut self, n: usize) -> Self {
        self.max_errors = n;
        self
    }

    /// Record a recovered parse failure against the stream cap.
    fn note_recovery_error(&mut self) -> Result<()> {
        if self.max_errors == 0 {
            return Ok(());
        }
        self.error_count += 1;
        if self.error_count > self.max_errors {
            self.cap_exceeded = true;
            let idx = self.ctx.record_index;
            return Err(MarcError::FatalReaderError {
                cap: self.max_errors,
                errors_seen: self.error_count,
                record_index: if idx == 0 { None } else { Some(idx) },
                source_name: self.ctx.source_name.clone(),
            });
        }
        Ok(())
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
        if self.cap_exceeded {
            return Ok(None);
        }

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

        // Hand the record buffer to the context for bytes_near capture on
        // any error raised during directory/field parsing.
        let record_data_offset = self.ctx.stream_byte_offset;
        self.ctx.set_parse_buffer(&record_data, record_data_offset);

        if record_data.len() < (record_length - 24) {
            if self.recovery_mode == RecoveryMode::Strict {
                return Err(self.ctx.err_truncated_record(
                    Some(record_length.saturating_sub(LEADER_LEN)),
                    Some(record_data.len()),
                ));
            }
            // Lenient/permissive: count the recovery and fall through to
            // best-effort directory parsing. (A holdings-specific
            // try_recover_record salvage path is bd-lkcy territory.)
            self.note_recovery_error()?;
        }

        // Clamp directory + data slices to actual buffer length so a short
        // read in lenient mode does not panic.
        let directory_end = std::cmp::min(directory_size, record_data.len());
        let directory_bytes = &record_data[..directory_end];
        let data_section = if directory_end < record_data.len() {
            &record_data[directory_end..]
        } else {
            &[][..]
        };

        let mut record = HoldingsRecord::new(leader);

        let mut i = 0;
        while i + 12 <= directory_bytes.len() {
            // Move stream_byte_offset to the current directory byte so
            // errors below carry a precise byte_offset and the bytes_near
            // hex-dump caret lands at the actual offending byte.
            self.ctx.stream_byte_offset = record_data_offset + i;
            // Stop on directory terminator (consistent with bib + authority).
            if directory_bytes[i] == FIELD_TERMINATOR {
                break;
            }

            let tag = std::str::from_utf8(&directory_bytes[i..i + 3])
                .map_err(|_| self.ctx.err_invalid_field("Invalid tag encoding"))?
                .to_string();

            let length = match iso2709::parse_4digits(&directory_bytes[i + 3..i + 7]) {
                Ok(len) => len,
                Err(e) => {
                    if self.recovery_mode == RecoveryMode::Strict {
                        return Err(e);
                    }
                    self.note_recovery_error()?;
                    i += 12;
                    continue;
                },
            };
            let start = match iso2709::parse_5digits(&directory_bytes[i + 7..i + 12]) {
                Ok(s) => s,
                Err(e) => {
                    if self.recovery_mode == RecoveryMode::Strict {
                        return Err(e);
                    }
                    self.note_recovery_error()?;
                    i += 12;
                    continue;
                },
            };

            let end = start + length;
            if end > data_section.len() {
                if self.recovery_mode == RecoveryMode::Strict {
                    self.ctx.current_field_tag = tag.as_bytes().try_into().ok();
                    return Err(self
                        .ctx
                        .err_invalid_field(format!("Field {tag} extends beyond data section")));
                }
                self.note_recovery_error()?;
                i += 12;
                continue;
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
                if self.recovery_mode == RecoveryMode::Strict {
                    self.ctx.current_field_tag = tag.as_bytes().try_into().ok();
                    return Err(self
                        .ctx
                        .err_invalid_field("Data field too short for indicators"));
                }
                self.note_recovery_error()?;
                i += 12;
                continue;
            }

            self.ctx.current_field_tag = tag.as_bytes().try_into().ok();
            self.ctx.stream_byte_offset = record_data_offset + directory_size + start;
            let parsed = iso2709::parse_data_field(
                field_data,
                &tag,
                DataFieldParseConfig::HOLDINGS,
                &self.ctx,
            );
            self.ctx.current_field_tag = None;
            let field = match parsed {
                Ok(f) => f,
                Err(e) => {
                    if self.recovery_mode == RecoveryMode::Strict {
                        return Err(e);
                    }
                    self.note_recovery_error()?;
                    i += 12;
                    continue;
                },
            };

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

    /// Wrap an arbitrary `directory` (must end with `\x1e` if it represents a
    /// terminated directory) and `data` in a minimal valid holdings leader
    /// (type 'x') and append a record terminator.
    fn build_holdings_record_with(directory: &[u8], data: &[u8]) -> Vec<u8> {
        let base_address = 24 + directory.len();
        let record_length = base_address + data.len() + 1;

        let mut leader = Vec::new();
        leader.extend_from_slice(format!("{record_length:05}").as_bytes());
        leader.push(b'n'); // status
        leader.push(b'x'); // type: holdings (single-part item)
        leader.push(b'|'); // bibliographic level
        leader.push(b' ');
        leader.push(b' ');
        leader.push(b'2');
        leader.push(b'2');
        leader.extend_from_slice(format!("{base_address:05}").as_bytes());
        leader.push(b'1');
        leader.push(b'a');
        leader.push(b' ');
        leader.extend_from_slice(b"4500");
        assert_eq!(leader.len(), 24);

        let mut out = Vec::new();
        out.extend_from_slice(&leader);
        out.extend_from_slice(directory);
        out.extend_from_slice(data);
        out.push(0x1D); // RECORD_TERMINATOR
        out
    }

    #[test]
    fn test_holdings_bad_field_length_strict_errors() {
        let mut directory = Vec::new();
        directory.extend_from_slice(b"852ABCD00000");
        directory.push(FIELD_TERMINATOR);
        let bytes = build_holdings_record_with(&directory, &[]);

        let mut reader = HoldingsMarcReader::new(std::io::Cursor::new(bytes))
            .with_recovery_mode(RecoveryMode::Strict);
        assert!(reader.read_record().is_err(), "strict should propagate");
    }

    #[test]
    fn test_holdings_bad_field_length_lenient_recovers() {
        let mut directory = Vec::new();
        directory.extend_from_slice(b"852ABCD00000");
        directory.push(FIELD_TERMINATOR);
        let bytes = build_holdings_record_with(&directory, &[]);

        let mut reader = HoldingsMarcReader::new(std::io::Cursor::new(bytes))
            .with_recovery_mode(RecoveryMode::Lenient);
        let rec = reader.read_record().expect("lenient should not error");
        assert!(rec.is_some());
    }

    #[test]
    fn test_holdings_bad_start_position_strict_errors() {
        let mut directory = Vec::new();
        directory.extend_from_slice(b"8520005XYZAB");
        directory.push(FIELD_TERMINATOR);
        let bytes = build_holdings_record_with(&directory, &[]);

        let mut reader = HoldingsMarcReader::new(std::io::Cursor::new(bytes))
            .with_recovery_mode(RecoveryMode::Strict);
        assert!(reader.read_record().is_err(), "strict should propagate");
    }

    #[test]
    fn test_holdings_bad_start_position_lenient_recovers() {
        let mut directory = Vec::new();
        directory.extend_from_slice(b"8520005XYZAB");
        directory.push(FIELD_TERMINATOR);
        let bytes = build_holdings_record_with(&directory, &[]);

        let mut reader = HoldingsMarcReader::new(std::io::Cursor::new(bytes))
            .with_recovery_mode(RecoveryMode::Lenient);
        let rec = reader.read_record().expect("lenient should not error");
        assert!(rec.is_some());
    }

    #[test]
    fn test_holdings_field_exceeds_data_strict_errors() {
        // Directory entry claims length=999 starting at position 0; data area empty.
        let mut directory = Vec::new();
        directory.extend_from_slice(b"852099900000");
        directory.push(FIELD_TERMINATOR);
        let bytes = build_holdings_record_with(&directory, &[]);

        let mut reader = HoldingsMarcReader::new(std::io::Cursor::new(bytes))
            .with_recovery_mode(RecoveryMode::Strict);
        let err = reader.read_record().expect_err("strict should error");
        assert!(
            matches!(err, MarcError::InvalidField { ref message, .. } if message.contains("extends beyond data section")),
            "expected InvalidField about extends-beyond, got: {err:?}"
        );
    }

    #[test]
    fn test_holdings_field_exceeds_data_lenient_recovers() {
        let mut directory = Vec::new();
        directory.extend_from_slice(b"852099900000");
        directory.push(FIELD_TERMINATOR);
        let bytes = build_holdings_record_with(&directory, &[]);

        let mut reader = HoldingsMarcReader::new(std::io::Cursor::new(bytes))
            .with_recovery_mode(RecoveryMode::Lenient);
        let rec = reader.read_record().expect("lenient should not error");
        assert!(rec.is_some());
    }

    #[test]
    fn test_holdings_data_field_too_short_strict_errors() {
        // Directory entry: data field tag with length=1 (less than 3 → too short
        // for indicators+terminator). One byte of data: 'A'.
        let mut directory = Vec::new();
        directory.extend_from_slice(b"852000100000");
        directory.push(FIELD_TERMINATOR);
        let data = b"A";
        let bytes = build_holdings_record_with(&directory, data);

        let mut reader = HoldingsMarcReader::new(std::io::Cursor::new(bytes))
            .with_recovery_mode(RecoveryMode::Strict);
        let err = reader.read_record().expect_err("strict should error");
        assert!(
            matches!(err, MarcError::InvalidField { ref message, .. } if message.contains("too short")),
            "expected InvalidField about too-short, got: {err:?}"
        );
    }

    #[test]
    fn test_holdings_data_field_too_short_lenient_recovers() {
        let mut directory = Vec::new();
        directory.extend_from_slice(b"852000100000");
        directory.push(FIELD_TERMINATOR);
        let data = b"A";
        let bytes = build_holdings_record_with(&directory, data);

        let mut reader = HoldingsMarcReader::new(std::io::Cursor::new(bytes))
            .with_recovery_mode(RecoveryMode::Lenient);
        let rec = reader.read_record().expect("lenient should not error");
        assert!(rec.is_some());
    }

    #[test]
    fn test_holdings_max_errors_cap_trips() {
        // Each record has one bad-field-length entry → one note_recovery_error.
        let mut directory = Vec::new();
        directory.extend_from_slice(b"852ABCD00000");
        directory.push(FIELD_TERMINATOR);
        let one_record = build_holdings_record_with(&directory, &[]);

        let mut stream = Vec::new();
        for _ in 0..5 {
            stream.extend_from_slice(&one_record);
        }
        let mut reader = HoldingsMarcReader::new(std::io::Cursor::new(stream))
            .with_recovery_mode(RecoveryMode::Lenient)
            .with_max_errors(3);

        for _ in 0..3 {
            assert!(reader.read_record().unwrap().is_some());
        }
        let err = reader.read_record().expect_err("cap should trip");
        assert!(
            matches!(
                err,
                MarcError::FatalReaderError {
                    cap: 3,
                    errors_seen: 4,
                    record_index: Some(4),
                    ..
                }
            ),
            "unexpected error: {err:?}"
        );
        assert!(reader.read_record().unwrap().is_none());
    }

    #[test]
    fn test_holdings_max_errors_zero_disables() {
        let mut directory = Vec::new();
        directory.extend_from_slice(b"852ABCD00000");
        directory.push(FIELD_TERMINATOR);
        let one_record = build_holdings_record_with(&directory, &[]);

        let mut stream = Vec::new();
        for _ in 0..20 {
            stream.extend_from_slice(&one_record);
        }
        let mut reader = HoldingsMarcReader::new(std::io::Cursor::new(stream))
            .with_recovery_mode(RecoveryMode::Lenient)
            .with_max_errors(0);

        let mut count = 0;
        while reader.read_record().unwrap().is_some() {
            count += 1;
        }
        assert_eq!(count, 20);
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
