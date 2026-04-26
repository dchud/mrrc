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
use crate::iso2709::{self, DataFieldParseConfig, ParseContext, FIELD_TERMINATOR, LEADER_LEN};
use crate::leader::Leader;
use crate::reader::DEFAULT_MAX_ERRORS;
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
    max_errors: usize,
    error_count: usize,
    cap_exceeded: bool,
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
            max_errors: DEFAULT_MAX_ERRORS,
            error_count: 0,
            cap_exceeded: false,
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

    /// Cap the number of recovered errors tolerated in one stream before the
    /// reader raises [`MarcError::FatalReaderError`] and halts.
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

        if record_data.len() < (record_length - 24) {
            if self.recovery_mode == RecoveryMode::Strict {
                return Err(self.ctx.err_truncated_record(
                    Some(record_length.saturating_sub(LEADER_LEN)),
                    Some(record_data.len()),
                ));
            }
            // Lenient/permissive: count the recovery and fall through to
            // best-effort directory parsing. (An authority-specific
            // try_recover_record salvage path is bd-lkcy territory; for now
            // the per-field lenient branches below salvage what they can.)
            self.note_recovery_error()?;
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
                self.note_recovery_error()?;
                break;
            }

            // Parse directory entry (12 bytes: tag + length + start position)
            let tag = std::str::from_utf8(&directory[pos..pos + 3])
                .map_err(|_| self.ctx.err_invalid_field("Invalid field tag"))?
                .to_string();

            let length = match iso2709::parse_4digits(&directory[pos + 3..pos + 7]) {
                Ok(len) => len,
                Err(e) => {
                    if self.recovery_mode == RecoveryMode::Strict {
                        return Err(e);
                    }
                    self.note_recovery_error()?;
                    pos += 12;
                    continue;
                },
            };
            let start = match iso2709::parse_5digits(&directory[pos + 7..pos + 12]) {
                Ok(s) => s,
                Err(e) => {
                    if self.recovery_mode == RecoveryMode::Strict {
                        return Err(e);
                    }
                    self.note_recovery_error()?;
                    pos += 12;
                    continue;
                },
            };

            pos += 12;

            // Extract field data. In strict mode, a field whose claimed end
            // exceeds the buffer is a hard error. In lenient/permissive mode
            // we count the recovery and salvage what bytes are available.
            let field_data_start = data_start + start;
            let field_data_end_unclamped = field_data_start + length;
            if field_data_end_unclamped > record_data.len() {
                if self.recovery_mode == RecoveryMode::Strict {
                    self.ctx.current_field_tag = tag.as_bytes().try_into().ok();
                    return Err(self.ctx.err_invalid_field(format!(
                        "Field {tag} exceeds data area (end {field_data_end_unclamped} > {})",
                        record_data.len()
                    )));
                }
                self.note_recovery_error()?;
            }
            let field_data_end = std::cmp::min(field_data_end_unclamped, record_data.len());

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
                self.ctx.current_field_tag = tag.as_bytes().try_into().ok();
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
                        self.note_recovery_error()?;
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

    /// Build a malformed authority record whose directory is 6 bytes long
    /// (not a multiple of 12 and not terminated). In lenient/permissive mode
    /// this triggers `note_recovery_error` exactly once per record via the
    /// "incomplete directory entry" branch.
    fn build_bad_authority_record() -> Vec<u8> {
        let directory: [u8; 6] = *b"245000";
        let base_address: usize = 24 + directory.len();
        let record_length: usize = base_address + 1;

        let mut leader = Vec::new();
        leader.extend_from_slice(format!("{record_length:05}").as_bytes());
        leader.push(b'n'); // status
        leader.push(b'z'); // type: authority
        leader.push(b' ');
        leader.push(b' ');
        leader.push(b' ');
        leader.push(b'2');
        leader.push(b'2');
        leader.extend_from_slice(format!("{base_address:05}").as_bytes());
        leader.push(b' ');
        leader.push(b' ');
        leader.push(b' ');
        leader.extend_from_slice(b"4500");
        assert_eq!(leader.len(), 24);

        let mut out = Vec::new();
        out.extend_from_slice(&leader);
        out.extend_from_slice(&directory);
        out.push(0x1D); // RECORD_TERMINATOR
        out
    }

    /// Wrap an arbitrary `directory` (must end with `\x1e` if it represents a
    /// terminated directory) in a minimal valid authority leader and append a
    /// record terminator. Optional `data` is the post-directory data area; if
    /// empty, the record has no field data.
    fn build_authority_record_with(directory: &[u8], data: &[u8]) -> Vec<u8> {
        let base_address = 24 + directory.len();
        let record_length = base_address + data.len() + 1;

        let mut leader = Vec::new();
        leader.extend_from_slice(format!("{record_length:05}").as_bytes());
        leader.push(b'n'); // status
        leader.push(b'z'); // type: authority
        leader.push(b' ');
        leader.push(b' ');
        leader.push(b' ');
        leader.push(b'2');
        leader.push(b'2');
        leader.extend_from_slice(format!("{base_address:05}").as_bytes());
        leader.push(b' ');
        leader.push(b' ');
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
    fn test_authority_bad_field_length_strict_errors() {
        // Directory entry with non-numeric bytes in the 4-digit field-length
        // field. Strict mode should propagate the parse error.
        let mut directory = Vec::new();
        directory.extend_from_slice(b"245ABCD00000");
        directory.push(FIELD_TERMINATOR);
        let bytes = build_authority_record_with(&directory, &[]);

        let mut reader =
            AuthorityMarcReader::new(Cursor::new(bytes)).with_recovery_mode(RecoveryMode::Strict);
        assert!(
            reader.read_record().is_err(),
            "strict mode should propagate bad field_length"
        );
    }

    #[test]
    fn test_authority_bad_field_length_lenient_recovers() {
        // Same fixture: lenient mode should swallow the bad entry and return
        // an (empty) authority record.
        let mut directory = Vec::new();
        directory.extend_from_slice(b"245ABCD00000");
        directory.push(FIELD_TERMINATOR);
        let bytes = build_authority_record_with(&directory, &[]);

        let mut reader =
            AuthorityMarcReader::new(Cursor::new(bytes)).with_recovery_mode(RecoveryMode::Lenient);
        let rec = reader.read_record().expect("lenient should not error");
        assert!(rec.is_some(), "lenient should yield a (partial) record");
    }

    #[test]
    fn test_authority_bad_start_position_strict_errors() {
        let mut directory = Vec::new();
        directory.extend_from_slice(b"2450005XYZAB");
        directory.push(FIELD_TERMINATOR);
        let bytes = build_authority_record_with(&directory, &[]);

        let mut reader =
            AuthorityMarcReader::new(Cursor::new(bytes)).with_recovery_mode(RecoveryMode::Strict);
        assert!(
            reader.read_record().is_err(),
            "strict mode should propagate bad start_position"
        );
    }

    #[test]
    fn test_authority_bad_start_position_lenient_recovers() {
        let mut directory = Vec::new();
        directory.extend_from_slice(b"2450005XYZAB");
        directory.push(FIELD_TERMINATOR);
        let bytes = build_authority_record_with(&directory, &[]);

        let mut reader =
            AuthorityMarcReader::new(Cursor::new(bytes)).with_recovery_mode(RecoveryMode::Lenient);
        let rec = reader.read_record().expect("lenient should not error");
        assert!(rec.is_some());
    }

    #[test]
    fn test_authority_field_exceeds_data_strict_errors() {
        // Directory entry claims length=999 starting at position 0, but the
        // data area is empty. In strict mode this is a hard error; in
        // lenient mode the (clamped) extraction yields an empty/short field
        // that is silently skipped.
        let mut directory = Vec::new();
        directory.extend_from_slice(b"245099900000");
        directory.push(FIELD_TERMINATOR);
        let bytes = build_authority_record_with(&directory, &[]);

        let mut reader =
            AuthorityMarcReader::new(Cursor::new(bytes)).with_recovery_mode(RecoveryMode::Strict);
        let err = reader.read_record().expect_err("strict should error");
        assert!(
            matches!(err, MarcError::InvalidField { ref message, .. } if message.contains("exceeds data area")),
            "expected InvalidField about exceeded data area, got: {err:?}"
        );
    }

    #[test]
    fn test_authority_field_exceeds_data_lenient_recovers() {
        let mut directory = Vec::new();
        directory.extend_from_slice(b"245099900000");
        directory.push(FIELD_TERMINATOR);
        let bytes = build_authority_record_with(&directory, &[]);

        let mut reader =
            AuthorityMarcReader::new(Cursor::new(bytes)).with_recovery_mode(RecoveryMode::Lenient);
        let rec = reader.read_record().expect("lenient should not error");
        assert!(rec.is_some());
    }

    #[test]
    fn test_authority_max_errors_cap_trips_on_field_length_failures() {
        // Each record has one bad-field-length entry → one note_recovery_error
        // per record. Cap of 3 means the 4th read trips.
        let mut directory = Vec::new();
        directory.extend_from_slice(b"245ABCD00000");
        directory.push(FIELD_TERMINATOR);
        let one_record = build_authority_record_with(&directory, &[]);

        let mut stream = Vec::new();
        for _ in 0..5 {
            stream.extend_from_slice(&one_record);
        }
        let mut reader = AuthorityMarcReader::new(Cursor::new(stream))
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
    fn test_authority_max_errors_cap_trips() {
        let mut stream = Vec::new();
        for _ in 0..5 {
            stream.extend_from_slice(&build_bad_authority_record());
        }
        let mut reader = AuthorityMarcReader::new(Cursor::new(stream))
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
    fn test_authority_max_errors_zero_disables() {
        let mut stream = Vec::new();
        for _ in 0..20 {
            stream.extend_from_slice(&build_bad_authority_record());
        }
        let mut reader = AuthorityMarcReader::new(Cursor::new(stream))
            .with_recovery_mode(RecoveryMode::Lenient)
            .with_max_errors(0);

        let mut count = 0;
        while reader.read_record().unwrap().is_some() {
            count += 1;
        }
        assert_eq!(count, 20);
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
