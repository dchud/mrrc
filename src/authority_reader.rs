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
use crate::error::Result;
use crate::iso2709::{DataFieldParseConfig, ParseContext};
use crate::iso2709_skeleton::{parse_iso2709_record, Iso2709Builder};
use crate::leader::Leader;
use crate::record::Field;
use crate::recovery::{RecoveryCap, RecoveryMode};
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
    cap: RecoveryCap,
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
            cap: RecoveryCap::new(),
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
    pub fn read_record(&mut self) -> Result<Option<AuthorityRecord>> {
        parse_iso2709_record::<R, AuthorityBuilder>(
            &mut self.reader,
            &mut self.ctx,
            &mut self.cap,
            self.recovery_mode,
        )
    }
}

/// Adapter for the authority reader's per-record state. Wraps an
/// [`AuthorityRecord`] and dispatches data fields by tag into the
/// record's semantic slots (heading, tracings, notes, linking entries).
struct AuthorityBuilder {
    record: AuthorityRecord,
}

impl Iso2709Builder for AuthorityBuilder {
    type Output = AuthorityRecord;

    fn parse_config() -> DataFieldParseConfig {
        DataFieldParseConfig::AUTHORITY
    }

    /// Authority records carry leader byte 6 = `'z'`; reject any other
    /// record type.
    fn validate_record_type(leader: &Leader, ctx: &ParseContext) -> Result<()> {
        if leader.record_type == 'z' {
            Ok(())
        } else {
            Err(ctx.err_invalid_field(format!(
                "Expected authority record type 'z', got '{}'",
                leader.record_type
            )))
        }
    }

    fn new_for(leader: Leader) -> Self {
        AuthorityBuilder {
            record: AuthorityRecord::new(leader),
        }
    }

    /// Authority control fields use the same lossy decode as the default
    /// but additionally trim a trailing `SUBFIELD_DELIMITER` (0x1F) — a
    /// historical quirk in real-world authority data preserved here for
    /// bytewise compatibility.
    fn decode_control_field_value(
        field_bytes: &[u8],
        _tag: &str,
        _ctx: &ParseContext,
    ) -> Result<String> {
        Ok(String::from_utf8_lossy(field_bytes)
            .trim_end_matches(['\x1E', '\x1F'])
            .to_string())
    }

    /// Authority skips data fields shorter than 2 bytes (can't read
    /// indicators). Treated as a strict-Err / lenient-skip event with cap
    /// accounting, matching the rest of the recovery shape across readers
    /// — silently dropping fields would diverge from the per-field
    /// recovery contract the skeleton enforces everywhere else.
    fn validate_data_field_bytes(field_bytes: &[u8], _tag: &str, ctx: &ParseContext) -> Result<()> {
        if field_bytes.len() < 2 {
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
            // 1XX — the main heading
            "100" | "110" | "111" | "130" | "148" | "150" | "151" | "155" => {
                self.record.set_heading(field);
            },
            // 4XX — see from tracings
            "400" | "410" | "411" | "430" | "448" | "450" | "451" | "455" => {
                self.record.add_see_from_tracing(field);
            },
            // 5XX — see also from tracings
            "500" | "510" | "511" | "530" | "548" | "550" | "551" | "555" => {
                self.record.add_see_also_tracing(field);
            },
            // 66X–68X — notes
            "660" | "661" | "662" | "663" | "664" | "665" | "666" | "667" | "668" | "669"
            | "670" | "671" | "672" | "673" | "674" | "675" | "676" | "677" | "678" | "679"
            | "680" | "681" | "682" | "683" | "684" | "685" | "686" | "687" | "688" | "689" => {
                self.record.add_note(field);
            },
            // 7XX — heading linking entries
            "700" | "710" | "711" | "730" | "748" | "750" | "751" | "755" => {
                self.record.add_linking_entry(field);
            },
            _ => {
                self.record.add_field(field);
            },
        }
    }

    fn finalize(self) -> AuthorityRecord {
        self.record
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::MarcError;
    use crate::iso2709::FIELD_TERMINATOR;
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
