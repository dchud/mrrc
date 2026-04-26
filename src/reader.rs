//! Reading MARC records from binary streams.
//!
//! This module provides [`MarcReader`] for reading ISO 2709 formatted MARC records
//! from any source that implements [`std::io::Read`].
//!
//! # Examples
//!
//! Reading records from a file:
//!
//! ```no_run
//! use mrrc::MarcReader;
//! use std::fs::File;
//!
//! let file = File::open("records.mrc")?;
//! let mut reader = MarcReader::new(file);
//!
//! while let Some(record) = reader.read_record()? {
//!     println!("Record type: {}", record.leader.record_type);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Reading from a buffer:
//!
//! ```
//! use mrrc::MarcReader;
//! use std::io::Cursor;
//!
//! let data = b"...binary MARC data...";
//! let cursor = Cursor::new(data.to_vec());
//! let mut reader = MarcReader::new(cursor);
//!
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::error::Result;
use crate::formats::FormatReader;
use crate::iso2709::{DataFieldParseConfig, ParseContext};
use crate::iso2709_skeleton::{parse_iso2709_record, Iso2709Builder};
use crate::leader::Leader;
use crate::record::{Field, Record};
use crate::recovery::{self, RecoveryCap, RecoveryMode};
use std::io::Read;

/// Reader for ISO 2709 binary MARC format.
///
/// `MarcReader` reads one MARC record at a time from any source implementing [`std::io::Read`].
/// Records are fully parsed and returned as [`Record`] instances.
///
/// # Examples
///
/// ```
/// use mrrc::MarcReader;
/// use std::io::Cursor;
///
/// let binary_data = vec![]; // MARC binary data
/// let cursor = Cursor::new(binary_data);
/// let mut reader = MarcReader::new(cursor);
///
/// match reader.read_record() {
///     Ok(Some(record)) => println!("Record type: {}", record.leader.record_type),
///     Ok(None) => println!("End of file"),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
#[derive(Debug)]
pub struct MarcReader<R: Read> {
    reader: R,
    recovery_mode: RecoveryMode,
    records_read: usize,
    ctx: ParseContext,
    cap: RecoveryCap,
}

impl<R: Read> MarcReader<R> {
    /// Create a new MARC reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - Any source implementing [`std::io::Read`]
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::MarcReader;
    /// use std::io::Cursor;
    ///
    /// let data = vec![];
    /// let cursor = Cursor::new(data);
    /// let reader = MarcReader::new(cursor);
    /// ```
    pub fn new(reader: R) -> Self {
        MarcReader {
            reader,
            recovery_mode: RecoveryMode::Strict,
            records_read: 0,
            ctx: ParseContext::new(),
            cap: RecoveryCap::new(),
        }
    }

    /// Set the recovery mode for handling malformed records.
    ///
    /// The recovery mode determines how the reader handles truncated or
    /// malformed MARC records:
    /// - `Strict`: Return errors immediately (default)
    /// - `Lenient`: Attempt to recover and salvage valid data
    /// - `Permissive`: Be very lenient, accepting partial data
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::{MarcReader, RecoveryMode};
    /// use std::io::Cursor;
    ///
    /// let data = vec![];
    /// let cursor = Cursor::new(data);
    /// let mut reader = MarcReader::new(cursor)
    ///     .with_recovery_mode(RecoveryMode::Lenient);
    /// ```
    #[must_use]
    pub fn with_recovery_mode(mut self, mode: RecoveryMode) -> Self {
        self.recovery_mode = mode;
        self
    }

    /// Attach a source identifier (filename or stream id) to errors raised by
    /// this reader. Populates `source_name` on every emitted error where
    /// applicable. Use [`MarcReader::from_path`] when constructing from a
    /// filesystem path to set this automatically.
    #[must_use]
    pub fn with_source(mut self, name: impl Into<String>) -> Self {
        self.ctx.source_name = Some(name.into());
        self
    }

    /// Cap the number of recovered errors tolerated in one stream before the
    /// reader raises [`crate::MarcError::FatalReaderError`] and halts.
    ///
    /// Only meaningful in [`RecoveryMode::Lenient`] and
    /// [`RecoveryMode::Permissive`]: in [`RecoveryMode::Strict`] the first
    /// error already aborts the stream, so no cap applies.
    ///
    /// Passing `0` disables the cap (unbounded accumulation — callers accept
    /// the memory risk explicitly). The default when the builder is not
    /// called is [`crate::recovery::DEFAULT_MAX_ERRORS`].
    ///
    /// After the cap is hit the reader is exhausted — subsequent
    /// [`MarcReader::read_record`] calls return `Ok(None)`.
    #[must_use]
    pub fn with_max_errors(mut self, n: usize) -> Self {
        self.cap.set_max(n);
        self
    }
}

impl MarcReader<std::fs::File> {
    /// Open `path` for reading and create a [`MarcReader`] whose errors
    /// include the path as their `source_name`.
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

impl<R: Read> MarcReader<R> {
    /// Read a single MARC record.
    ///
    /// Returns `Ok(Some(record))` if a record was successfully read, `Ok(None)` if EOF
    /// was reached, or `Err` if a parsing error occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::MarcReader;
    /// use std::io::Cursor;
    ///
    /// # let data = vec![];
    /// # let cursor = Cursor::new(data);
    /// let mut reader = MarcReader::new(cursor);
    ///
    /// match reader.read_record() {
    ///     Ok(Some(record)) => { /* process record */ },
    ///     Ok(None) => println!("End of file"),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The binary data is malformed
    /// - The record structure is invalid
    /// - An I/O error occurs
    pub fn read_record(&mut self) -> Result<Option<Record>> {
        let result = parse_iso2709_record::<R, BibBuilder>(
            &mut self.reader,
            &mut self.ctx,
            &mut self.cap,
            self.recovery_mode,
        )?;
        if result.is_some() {
            self.records_read += 1;
        }
        Ok(result)
    }
}

/// Adapter for the bibliographic reader's per-record state. Wraps a
/// [`Record`] and threads it through the shared [`parse_iso2709_record`]
/// skeleton.
struct BibBuilder {
    record: Record,
}

// `#[inline]` on the per-field trait methods below is a measured
// requirement, not stylistic noise. Without these, monomorphization of
// `parse_iso2709_record::<_, BibBuilder>` does not consistently inline
// the per-field calls, and parallel reader benchmarks regress measurably.
// Pairs with the `#[inline(always)]` on `iso2709::parse_data_field`.
// Re-verify with `cargo bench --bench parallel_benchmarks parallel_4x`
// before changing.
impl Iso2709Builder for BibBuilder {
    type Output = Record;

    #[inline]
    fn parse_config() -> DataFieldParseConfig {
        DataFieldParseConfig::BIBLIOGRAPHIC
    }

    #[inline]
    fn new_for(leader: Leader) -> Self {
        BibBuilder {
            record: Record::new(leader),
        }
    }

    #[inline]
    fn add_control_field(&mut self, tag: String, value: String) {
        self.record.add_control_field(tag, value);
    }

    #[inline]
    fn add_data_field(&mut self, _tag: String, field: Field) {
        self.record.add_field(field);
    }

    /// Bibliographic reader honors the truncated-record salvage path via
    /// [`recovery::try_recover_record`]; authority + holdings rely on the
    /// trait's default `None` and fall through to best-effort parsing.
    fn try_recover_truncated(
        leader: Leader,
        partial_data: &[u8],
        base_address: usize,
        mode: RecoveryMode,
        ctx: &ParseContext,
    ) -> Option<Result<Record>> {
        Some(recovery::try_recover_record(
            leader,
            partial_data,
            base_address,
            mode,
            ctx,
        ))
    }

    #[inline]
    fn finalize(self) -> Record {
        self.record
    }
}

// Implement the FormatReader trait for MarcReader
impl<R: Read + std::fmt::Debug> FormatReader for MarcReader<R> {
    fn read_record(&mut self) -> Result<Option<Record>> {
        // Delegate to the existing implementation
        MarcReader::read_record(self)
    }

    fn records_read(&self) -> Option<usize> {
        Some(self.records_read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    use crate::iso2709::{FIELD_TERMINATOR, RECORD_TERMINATOR, SUBFIELD_DELIMITER};

    #[test]
    fn test_read_simple_record() {
        // Manually build a valid MARC record
        let mut record_bytes = Vec::new();

        // Data area: field 245
        let mut field_245 = Vec::new();
        field_245.extend_from_slice(b"10"); // Indicators
        field_245.push(SUBFIELD_DELIMITER);
        field_245.push(b'a');
        field_245.extend_from_slice(b"Test title");
        field_245.push(FIELD_TERMINATOR);

        // Directory (without terminator yet)
        let mut directory = Vec::new();
        directory.extend_from_slice(b"245");
        directory.extend_from_slice(format!("{:04}", field_245.len()).as_bytes());
        directory.extend_from_slice(b"00000");

        // Base address is after leader + directory + directory terminator
        let base_address = 24 + directory.len() + 1; // +1 for directory terminator
        directory.push(FIELD_TERMINATOR);
        let record_length = base_address + field_245.len() + 1;

        // Leader (must be exactly 24 bytes)
        let mut leader = Vec::new();
        leader.extend_from_slice(format!("{record_length:05}").as_bytes()); // 0-4
        leader.push(b'n'); // 5: status
        leader.push(b'a'); // 6: type
        leader.push(b'm'); // 7: bib level
        leader.push(b' '); // 8: control type
        leader.push(b'a'); // 9: character coding
        leader.push(b'2'); // 10: indicator count
        leader.push(b'2'); // 11: subfield code count
        leader.extend_from_slice(format!("{base_address:05}").as_bytes()); // 12-16
        leader.push(b' '); // 17: encoding level
        leader.push(b' '); // 18: cataloging form
        leader.push(b' '); // 19: multipart level
        leader.extend_from_slice(b"4500"); // 20-23: reserved

        // Assemble
        record_bytes.extend_from_slice(&leader);
        record_bytes.extend_from_slice(&directory);
        record_bytes.extend_from_slice(&field_245);
        record_bytes.push(RECORD_TERMINATOR);

        let cursor = Cursor::new(record_bytes);
        let mut reader = MarcReader::new(cursor);

        let record = reader.read_record().unwrap().unwrap();

        assert_eq!(record.leader.record_type, 'a');
        let fields = record.get_fields("245");
        assert!(fields.is_some());
        let field = &fields.unwrap()[0];
        assert_eq!(field.indicator1, '1');
        assert_eq!(field.indicator2, '0');

        let title = field.get_subfield('a');
        assert_eq!(title, Some("Test title"));
    }

    #[test]
    fn test_eof_returns_none() {
        let data = vec![];
        let cursor = Cursor::new(data);
        let mut reader = MarcReader::new(cursor);

        let result = reader.read_record().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_read_multiple_records() {
        // Build two records
        let mut all_bytes = Vec::new();

        for _ in 0..2 {
            let mut field_245 = Vec::new();
            field_245.extend_from_slice(b"10");
            field_245.push(SUBFIELD_DELIMITER);
            field_245.push(b'a');
            field_245.extend_from_slice(b"Test title");
            field_245.push(FIELD_TERMINATOR);

            let mut directory = Vec::new();
            directory.extend_from_slice(b"245");
            directory.extend_from_slice(format!("{:04}", field_245.len()).as_bytes());
            directory.extend_from_slice(b"00000");

            let base_address = 24 + directory.len() + 1;
            directory.push(FIELD_TERMINATOR);
            let record_length = base_address + field_245.len() + 1;

            let mut leader = Vec::new();
            leader.extend_from_slice(format!("{record_length:05}").as_bytes()); // 0-4
            leader.push(b'n'); // 5
            leader.push(b'a'); // 6
            leader.push(b'm'); // 7
            leader.push(b' '); // 8
            leader.push(b'a'); // 9
            leader.push(b'2'); // 10
            leader.push(b'2'); // 11
            leader.extend_from_slice(format!("{base_address:05}").as_bytes()); // 12-16
            leader.push(b' '); // 17
            leader.push(b' '); // 18
            leader.push(b' '); // 19
            leader.extend_from_slice(b"4500"); // 20-23

            all_bytes.extend_from_slice(&leader);
            all_bytes.extend_from_slice(&directory);
            all_bytes.extend_from_slice(&field_245);
            all_bytes.push(RECORD_TERMINATOR);
        }

        let cursor = Cursor::new(all_bytes);
        let mut reader = MarcReader::new(cursor);

        let record1 = reader.read_record().unwrap();
        assert!(record1.is_some());

        let record2 = reader.read_record().unwrap();
        assert!(record2.is_some());

        let record3 = reader.read_record().unwrap();
        assert!(record3.is_none());
    }

    #[test]
    fn test_format_reader_trait() {
        // Build two records
        let mut all_bytes = Vec::new();

        for _ in 0..2 {
            let mut field_245 = Vec::new();
            field_245.extend_from_slice(b"10");
            field_245.push(SUBFIELD_DELIMITER);
            field_245.push(b'a');
            field_245.extend_from_slice(b"Test title");
            field_245.push(FIELD_TERMINATOR);

            let mut directory = Vec::new();
            directory.extend_from_slice(b"245");
            directory.extend_from_slice(format!("{:04}", field_245.len()).as_bytes());
            directory.extend_from_slice(b"00000");

            let base_address = 24 + directory.len() + 1;
            directory.push(FIELD_TERMINATOR);
            let record_length = base_address + field_245.len() + 1;

            let mut leader = Vec::new();
            leader.extend_from_slice(format!("{record_length:05}").as_bytes());
            leader.push(b'n');
            leader.push(b'a');
            leader.push(b'm');
            leader.push(b' ');
            leader.push(b'a');
            leader.push(b'2');
            leader.push(b'2');
            leader.extend_from_slice(format!("{base_address:05}").as_bytes());
            leader.push(b' ');
            leader.push(b' ');
            leader.push(b' ');
            leader.extend_from_slice(b"4500");

            all_bytes.extend_from_slice(&leader);
            all_bytes.extend_from_slice(&directory);
            all_bytes.extend_from_slice(&field_245);
            all_bytes.push(RECORD_TERMINATOR);
        }

        let cursor = Cursor::new(all_bytes);
        let mut reader = MarcReader::new(cursor);

        // Verify records_read starts at 0
        assert_eq!(reader.records_read(), Some(0));

        // Use the FormatReader trait method read_all
        let records = FormatReader::read_all(&mut reader).unwrap();
        assert_eq!(records.len(), 2);

        // Verify records_read counter
        assert_eq!(reader.records_read(), Some(2));
    }

    #[test]
    fn test_format_reader_iterator() {
        use crate::formats::FormatReaderExt;

        // Build two records
        let mut all_bytes = Vec::new();

        for _ in 0..3 {
            let mut field_245 = Vec::new();
            field_245.extend_from_slice(b"10");
            field_245.push(SUBFIELD_DELIMITER);
            field_245.push(b'a');
            field_245.extend_from_slice(b"Test title");
            field_245.push(FIELD_TERMINATOR);

            let mut directory = Vec::new();
            directory.extend_from_slice(b"245");
            directory.extend_from_slice(format!("{:04}", field_245.len()).as_bytes());
            directory.extend_from_slice(b"00000");

            let base_address = 24 + directory.len() + 1;
            directory.push(FIELD_TERMINATOR);
            let record_length = base_address + field_245.len() + 1;

            let mut leader = Vec::new();
            leader.extend_from_slice(format!("{record_length:05}").as_bytes());
            leader.push(b'n');
            leader.push(b'a');
            leader.push(b'm');
            leader.push(b' ');
            leader.push(b'a');
            leader.push(b'2');
            leader.push(b'2');
            leader.extend_from_slice(format!("{base_address:05}").as_bytes());
            leader.push(b' ');
            leader.push(b' ');
            leader.push(b' ');
            leader.extend_from_slice(b"4500");

            all_bytes.extend_from_slice(&leader);
            all_bytes.extend_from_slice(&directory);
            all_bytes.extend_from_slice(&field_245);
            all_bytes.push(RECORD_TERMINATOR);
        }

        let cursor = Cursor::new(all_bytes);
        let mut reader = MarcReader::new(cursor);

        // Use the FormatReaderExt iterator
        let mut count = 0;
        for result in reader.records() {
            result.unwrap();
            count += 1;
        }
        assert_eq!(count, 3);
        assert_eq!(reader.records_read(), Some(3));
    }

    #[test]
    fn test_malformed_leader_record_length_too_small() {
        // Build a 24-byte leader where record_length (bytes 0-4) = 00010 (< 24)
        let leader = b"00010nam a2200025 i 4500";
        let cursor = Cursor::new(leader.to_vec());
        let mut reader = MarcReader::new(cursor);
        let result = reader.read_record();
        assert!(result.is_err(), "expected Err for record_length < 24");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Record length must be at least 24"),
            "got: {err}"
        );
    }

    /// Build a record with a single malformed directory entry (non-digit
    /// bytes in the field-length positions). In lenient/permissive mode this
    /// triggers `note_recovery_error` once per record; in strict mode it
    /// surfaces a parse error.
    fn build_bad_record() -> Vec<u8> {
        // Directory: 12-byte entry with bad field-length, then terminator.
        let mut directory = Vec::new();
        directory.extend_from_slice(b"245ABCD00000");
        directory.push(FIELD_TERMINATOR);

        let base_address = 24 + directory.len();
        let record_length = base_address + 1; // +1 for RECORD_TERMINATOR

        let mut leader = Vec::new();
        leader.extend_from_slice(format!("{record_length:05}").as_bytes());
        leader.extend_from_slice(b"nam a22");
        leader.extend_from_slice(format!("{base_address:05}").as_bytes());
        leader.extend_from_slice(b" i 4500");
        assert_eq!(leader.len(), 24);

        let mut out = Vec::new();
        out.extend_from_slice(&leader);
        out.extend_from_slice(&directory);
        out.push(RECORD_TERMINATOR);
        out
    }

    #[test]
    fn test_max_errors_cap_trips_on_stream_of_malformed_records() {
        // 5 malformed records, cap at 3 → the 4th read should trip the cap.
        let mut stream = Vec::new();
        for _ in 0..5 {
            stream.extend_from_slice(&build_bad_record());
        }
        let mut reader = MarcReader::new(Cursor::new(stream))
            .with_recovery_mode(RecoveryMode::Lenient)
            .with_max_errors(3);

        // Reads 1..=3: each records one recovery error; error_count reaches 3
        // which does not exceed the cap. Records come back with no fields.
        for _ in 0..3 {
            let rec = reader.read_record().unwrap();
            assert!(rec.is_some());
        }

        // Read 4: would increment error_count to 4, which exceeds cap.
        let err = reader.read_record().expect_err("cap should trip");
        match err {
            crate::error::MarcError::FatalReaderError {
                cap,
                errors_seen,
                record_index,
                ..
            } => {
                assert_eq!(cap, 3);
                assert_eq!(errors_seen, 4);
                // 4th record in the stream (1-indexed).
                assert_eq!(record_index, Some(4));
            },
            other => panic!("expected FatalReaderError, got {other:?}"),
        }

        // Subsequent reads: reader is exhausted.
        assert!(reader.read_record().unwrap().is_none());
        assert!(reader.read_record().unwrap().is_none());
    }

    #[test]
    fn test_max_errors_zero_disables_cap() {
        // 50 malformed records with cap=0 must all be read without tripping.
        let mut stream = Vec::new();
        for _ in 0..50 {
            stream.extend_from_slice(&build_bad_record());
        }
        let mut reader = MarcReader::new(Cursor::new(stream))
            .with_recovery_mode(RecoveryMode::Lenient)
            .with_max_errors(0);

        let mut count = 0;
        while reader.read_record().unwrap().is_some() {
            count += 1;
        }
        assert_eq!(count, 50);
    }

    #[test]
    fn test_max_errors_inert_in_strict_mode() {
        // In strict mode, the first malformed record returns an error
        // immediately — the cap never has a chance to trip, even with cap=1.
        let stream = build_bad_record();
        let mut reader = MarcReader::new(Cursor::new(stream))
            .with_recovery_mode(RecoveryMode::Strict)
            .with_max_errors(1);
        let err = reader.read_record().expect_err("strict mode should error");
        // Any variant other than FatalReaderError — the cap did not trip.
        assert!(
            !matches!(err, crate::error::MarcError::FatalReaderError { .. }),
            "strict mode should never produce FatalReaderError, got {err:?}"
        );
    }

    #[test]
    fn test_malformed_leader_base_address_too_small() {
        // Build a 24-byte leader where base_address (bytes 12-16) = 00010 (< 24)
        let leader = b"00050nam a2200010 i 4500";
        let cursor = Cursor::new(leader.to_vec());
        let mut reader = MarcReader::new(cursor);
        let result = reader.read_record();
        assert!(result.is_err(), "expected Err for base_address < 24");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Base address of data must be at least 24"),
            "got: {err}"
        );
    }
}
