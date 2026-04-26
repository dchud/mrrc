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
use crate::iso2709::{DataFieldParseConfig, ParseContext};
use crate::iso2709_skeleton::{parse_iso2709_record, Iso2709Builder};
use crate::leader::Leader;
use crate::record::Field;
use crate::recovery::{RecoveryCap, RecoveryMode};
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
    cap: RecoveryCap,
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
            cap: RecoveryCap::new(),
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
    /// [`crate::recovery::DEFAULT_MAX_ERRORS`].
    #[must_use]
    pub fn with_max_errors(mut self, n: usize) -> Self {
        self.cap.set_max(n);
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
    pub fn read_record(&mut self) -> Result<Option<HoldingsRecord>> {
        parse_iso2709_record::<R, HoldingsBuilder>(
            &mut self.reader,
            &mut self.ctx,
            &mut self.cap,
            self.recovery_mode,
        )
    }
}

/// Adapter for the holdings reader's per-record state. Wraps a
/// [`HoldingsRecord`] and dispatches data fields by tag into the
/// record's semantic slots (locations, captions, enumeration, textual
/// holdings, item information).
struct HoldingsBuilder {
    record: HoldingsRecord,
}

impl Iso2709Builder for HoldingsBuilder {
    type Output = HoldingsRecord;

    fn parse_config() -> DataFieldParseConfig {
        DataFieldParseConfig::HOLDINGS
    }

    /// Holdings records carry leader byte 6 in `{x, y, v, u}`; reject any
    /// other record type.
    fn validate_record_type(leader: &Leader, ctx: &ParseContext) -> Result<()> {
        if matches!(leader.record_type, 'x' | 'y' | 'v' | 'u') {
            Ok(())
        } else {
            Err(ctx.err_invalid_field(format!(
                "Expected holdings record type (x/y/v/u), got '{}'",
                leader.record_type
            )))
        }
    }

    fn new_for(leader: Leader) -> Self {
        HoldingsBuilder {
            record: HoldingsRecord::new(leader),
        }
    }

    /// Holdings is uniquely strict on UTF-8 here — bad bytes raise
    /// `EncodingError` rather than being decoded lossily, preserving the
    /// historical reader behavior via this hook.
    fn decode_control_field_value(
        field_bytes: &[u8],
        tag: &str,
        ctx: &ParseContext,
    ) -> Result<String> {
        let raw = field_bytes
            .get(..field_bytes.len().saturating_sub(1))
            .unwrap_or(&[]);
        std::str::from_utf8(raw)
            .map(str::to_string)
            .map_err(|e| ctx.err_encoding(format!("Invalid UTF-8 in control field {tag}: {e}")))
    }

    /// Holdings rejects data fields shorter than 3 bytes (must hold both
    /// indicators and at least one byte of subfield data); strict-Err /
    /// lenient-skip dispatched by the skeleton.
    fn validate_data_field_bytes(field_bytes: &[u8], _tag: &str, ctx: &ParseContext) -> Result<()> {
        if field_bytes.len() < 3 {
            Err(ctx.err_invalid_field("Data field too short for indicators"))
        } else {
            Ok(())
        }
    }

    fn add_control_field(&mut self, tag: String, value: String) {
        self.record.add_control_field(tag, value);
    }

    fn add_data_field(&mut self, tag: String, field: Field) {
        match tag.as_str() {
            "852" => self.record.add_location(field),
            "853" => self.record.add_captions_basic(field),
            "854" => self.record.add_captions_supplements(field),
            "855" => self.record.add_captions_indexes(field),
            "863" => self.record.add_enumeration_basic(field),
            "864" => self.record.add_enumeration_supplements(field),
            "865" => self.record.add_enumeration_indexes(field),
            "866" => self.record.add_textual_holdings_basic(field),
            "867" => self.record.add_textual_holdings_supplements(field),
            "868" => self.record.add_textual_holdings_indexes(field),
            "876" | "877" | "878" => self.record.add_item_information(field),
            _ => self.record.add_field(field),
        }
    }

    fn finalize(self) -> HoldingsRecord {
        self.record
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::MarcError;
    use crate::iso2709::FIELD_TERMINATOR;

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
            matches!(err, MarcError::InvalidField { ref message, .. } if message.contains("exceeds data area")),
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
