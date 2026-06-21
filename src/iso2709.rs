//! Shared ISO 2709 parsing primitives used by all three MARC readers
//! (bibliographic, authority, holdings).
//!
//! Consolidates the byte-level parsing logic that would otherwise be
//! duplicated across [`crate::reader`], [`crate::authority_reader`], and
//! [`crate::holdings_reader`]. Each reader still owns its own record-type
//! dispatch (which `Record`/`AuthorityRecord`/`HoldingsRecord` to instantiate
//! and how fields are routed by tag); only the format-level primitives live
//! here.
//!
//! # Scope
//!
//! Provides the I/O loop for reading a leader, the truncation-aware
//! record-data read, single-entry directory parsing, ASCII numeric helpers,
//! and the control-field-tag predicate.
//!
//! Subfield-level parsing lives in each reader: the bib, authority, and
//! holdings readers use subtly different semantics (lossy vs strict UTF-8
//! decoding, error vs skip on unrecognized bytes) that have not been
//! unified.

use crate::error::{BytesNear, MarcError, Result};
use crate::record::{Field, Subfield};
use crate::recovery::RecoveryMode;
use crate::validation::IndicatorValidator;
use smallvec::SmallVec;
use std::io::Read;
use std::sync::OnceLock;

/// MARC 21 per-tag indicator rules, lazily built once and reused by every
/// `parse_data_field` call when `IndicatorMode::Strict` is selected.
fn marc21_indicator_validator() -> &'static IndicatorValidator {
    static V: OnceLock<IndicatorValidator> = OnceLock::new();
    V.get_or_init(IndicatorValidator::new)
}

/// ASCII record terminator (`0x1D`).
pub const RECORD_TERMINATOR: u8 = 0x1D;

/// ASCII field terminator (`0x1E`).
pub const FIELD_TERMINATOR: u8 = 0x1E;

/// ASCII subfield delimiter (`0x1F`).
pub const SUBFIELD_DELIMITER: u8 = 0x1F;

/// Length in bytes of the MARC leader.
pub const LEADER_LEN: usize = 24;

/// Length in bytes of a single directory entry (3 tag + 4 length + 5 start).
pub const DIRECTORY_ENTRY_LEN: usize = 12;

/// Per-stream parsing context carrying positional metadata used to enrich
/// errors raised during ISO 2709 parsing.
///
/// Construction errors via the `ParseContext::err_*` methods automatically
/// inherit the current state (record index, byte offset, source filename,
/// 001 control number, and the field/subfield being parsed) so call sites
/// don't have to thread these fields manually.
///
/// A typical reader loop looks like:
///
/// ```ignore
/// let mut ctx = ParseContext::new().with_source_name("harvest.mrc");
/// loop {
///     ctx.begin_record();
///     // ... read bytes, advance ctx, raise errors via ctx.err_* helpers ...
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct ParseContext {
    /// Filename or stream identifier, propagated to every error raised via
    /// the `err_*` helpers as `source_name`.
    pub source_name: Option<String>,
    /// 1-based record index in the current stream. Incremented by
    /// [`ParseContext::begin_record`].
    pub record_index: usize,
    /// Absolute byte offset in the underlying stream. Should be advanced by
    /// the reader as bytes are consumed.
    pub stream_byte_offset: usize,
    /// Stream byte offset where the current record began. Set by
    /// [`ParseContext::begin_record`] and used to compute the
    /// record-relative byte offset for errors.
    pub record_start_offset: usize,
    /// 001 control field value, populated opportunistically once the field
    /// has been parsed. `None` for errors raised before 001 is available.
    pub record_control_number: Option<String>,
    /// Field tag currently being parsed (3 ASCII bytes per MARC spec).
    /// The fixed-size representation keeps `ParseContext` compact enough
    /// that `parse_data_field` can be `#[inline(always)]` without busting
    /// L1-i cache on parallel workloads — see `parse_data_field` for the
    /// pairing. Converted to `String` lazily via `field_tag_as_string`.
    pub current_field_tag: Option<[u8; 3]>,
    /// Subfield code currently being parsed, when known.
    pub current_subfield_code: Option<u8>,
    /// Indicator position currently being parsed (0 or 1), when known.
    pub current_indicator_position: Option<u8>,
    /// Most recently captured parse buffer, used to populate `bytes_near`
    /// on errors. Shared via `Arc` so the context has no lifetime
    /// parameters and taking the buffer is a refcount bump, not a copy;
    /// callers hand it over via [`ParseContext::set_parse_buffer`].
    current_buffer: Option<std::sync::Arc<Vec<u8>>>,
    /// Absolute stream offset of `current_buffer[0]`.
    current_buffer_base_offset: Option<usize>,
}

impl ParseContext {
    /// Construct a fresh, empty context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder-style setter for the source name (filename or stream id).
    #[must_use]
    pub fn with_source_name(mut self, name: impl Into<String>) -> Self {
        self.source_name = Some(name.into());
        self
    }

    /// Begin parsing a new record: increments the record index, resets the
    /// per-record state, and snapshots the current stream byte offset as the
    /// record's start offset.
    pub fn begin_record(&mut self) {
        self.record_index = self.record_index.saturating_add(1);
        self.record_start_offset = self.stream_byte_offset;
        self.record_control_number = None;
        self.current_field_tag = None;
        self.current_subfield_code = None;
        self.current_indicator_position = None;
        self.current_buffer = None;
        self.current_buffer_base_offset = None;
    }

    /// Advance the stream byte offset by `n`.
    pub fn advance(&mut self, n: usize) {
        self.stream_byte_offset = self.stream_byte_offset.saturating_add(n);
    }

    /// Compute the byte offset relative to the start of the current record.
    #[must_use]
    pub fn record_byte_offset(&self) -> usize {
        self.stream_byte_offset
            .saturating_sub(self.record_start_offset)
    }

    fn record_index_opt(&self) -> Option<usize> {
        if self.record_index == 0 {
            None
        } else {
            Some(self.record_index)
        }
    }

    /// Decode [`ParseContext::current_field_tag`] (stored as 3 ASCII bytes)
    /// back into an owned `String` for inclusion in a `MarcError` variant.
    /// Allocates only on the error path, once per emitted error.
    fn field_tag_as_string(&self) -> Option<String> {
        self.current_field_tag
            .and_then(|bytes| std::str::from_utf8(&bytes).ok().map(String::from))
    }

    /// Hand the parser's current byte buffer to the context so subsequent
    /// `err_*` helpers can capture a hex-dump-ready window around the error
    /// offset. `buffer_start_offset` is the absolute stream offset of
    /// `buffer[0]`. Takes a shared handle, so this is a refcount bump —
    /// every record passes through here, and copying each record's bytes
    /// just for the under-1% that error was the read path's largest
    /// avoidable cost.
    pub fn set_parse_buffer(
        &mut self,
        buffer: std::sync::Arc<Vec<u8>>,
        buffer_start_offset: usize,
    ) {
        self.current_buffer = Some(buffer);
        self.current_buffer_base_offset = Some(buffer_start_offset);
    }

    /// Clear any previously-set parse buffer; call after a record is
    /// successfully parsed so a later error on a fresh buffer doesn't
    /// capture stale bytes.
    pub fn clear_parse_buffer(&mut self) {
        self.current_buffer = None;
        self.current_buffer_base_offset = None;
    }

    /// Capture a byte-window around the current stream offset for attaching
    /// to an error. Returns `None` when no buffer has been provided or the
    /// current offset is outside the buffer.
    fn capture_bytes_near(&self) -> Option<BytesNear> {
        let buffer = self.current_buffer.as_deref()?;
        let base = self.current_buffer_base_offset?;
        BytesNear::capture(buffer, base, self.stream_byte_offset)
    }

    /// Construct an [`MarcError::DirectoryInvalid`] inheriting the current
    /// stream/record positional state.
    #[must_use]
    pub fn err_directory_invalid(
        &self,
        found: Option<&[u8]>,
        expected: impl Into<String>,
    ) -> MarcError {
        let found_bytes = found.map(crate::error::truncate_bytes);
        MarcError::DirectoryInvalid {
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            record_byte_offset: Some(self.record_byte_offset()),
            source_name: self.source_name.clone(),
            record_control_number: self.record_control_number.clone(),
            field_tag: self.field_tag_as_string(),
            found: found_bytes,
            expected: Some(expected.into()),
            bytes_near: self.capture_bytes_near(),
        }
    }

    /// Construct an [`MarcError::TruncatedRecord`] inheriting the current
    /// stream/record positional state.
    #[must_use]
    pub fn err_truncated_record(
        &self,
        expected_length: Option<usize>,
        actual_length: Option<usize>,
    ) -> MarcError {
        MarcError::TruncatedRecord {
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            record_byte_offset: Some(self.record_byte_offset()),
            source_name: self.source_name.clone(),
            record_control_number: self.record_control_number.clone(),
            expected_length,
            actual_length,
            bytes_near: self.capture_bytes_near(),
        }
    }

    /// Construct an [`MarcError::EndOfRecordNotFound`] inheriting the
    /// current stream/record positional state.
    #[must_use]
    pub fn err_end_of_record_not_found(&self) -> MarcError {
        MarcError::EndOfRecordNotFound {
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            record_byte_offset: Some(self.record_byte_offset()),
            source_name: self.source_name.clone(),
            record_control_number: self.record_control_number.clone(),
            bytes_near: self.capture_bytes_near(),
        }
    }

    /// Construct an [`MarcError::InvalidIndicator`] inheriting the current
    /// stream/record positional state. The indicator position is taken from
    /// the explicit argument (the context's
    /// `current_indicator_position` may not yet be set when this is called).
    #[must_use]
    pub fn err_invalid_indicator(
        &self,
        indicator_position: u8,
        found: &[u8],
        expected: impl Into<String>,
    ) -> MarcError {
        let found_bytes = crate::error::truncate_bytes(found);
        MarcError::InvalidIndicator {
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            record_byte_offset: Some(self.record_byte_offset()),
            source_name: self.source_name.clone(),
            record_control_number: self.record_control_number.clone(),
            field_tag: self.field_tag_as_string(),
            indicator_position: Some(indicator_position),
            found: Some(found_bytes),
            expected: Some(expected.into()),
            bytes_near: self.capture_bytes_near(),
        }
    }

    /// Construct an [`MarcError::BadSubfieldCode`] inheriting the current
    /// stream/record positional state.
    #[must_use]
    pub fn err_bad_subfield_code(&self, subfield_code: u8) -> MarcError {
        MarcError::BadSubfieldCode {
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            record_byte_offset: Some(self.record_byte_offset()),
            source_name: self.source_name.clone(),
            record_control_number: self.record_control_number.clone(),
            field_tag: self.field_tag_as_string(),
            subfield_code,
            bytes_near: self.capture_bytes_near(),
        }
    }

    /// Construct an [`MarcError::InvalidField`] inheriting the current
    /// stream/record positional state.
    #[must_use]
    pub fn err_invalid_field(&self, message: impl Into<String>) -> MarcError {
        MarcError::InvalidField {
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            record_byte_offset: Some(self.record_byte_offset()),
            source_name: self.source_name.clone(),
            record_control_number: self.record_control_number.clone(),
            field_tag: self.field_tag_as_string(),
            message: message.into(),
            bytes_near: self.capture_bytes_near(),
        }
    }

    /// Construct an [`MarcError::EncodingError`] inheriting the current
    /// stream/record positional state.
    #[must_use]
    pub fn err_encoding(&self, message: impl Into<String>) -> MarcError {
        MarcError::EncodingError {
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            source_name: self.source_name.clone(),
            record_control_number: self.record_control_number.clone(),
            field_tag: self.field_tag_as_string(),
            message: message.into(),
            bytes_near: self.capture_bytes_near(),
        }
    }

    /// Construct an [`MarcError::IoError`] inheriting the current
    /// stream/record positional state.
    #[must_use]
    pub fn err_io(&self, cause: std::io::Error) -> MarcError {
        MarcError::IoError {
            cause,
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            source_name: self.source_name.clone(),
        }
    }

    /// Construct an [`MarcError::XmlError`] wrapping a `quick_xml` error type.
    /// Any error implementing `std::error::Error + Send + Sync + 'static` is
    /// accepted (handles both `quick_xml::Error` and `quick_xml::DeError`).
    #[must_use]
    pub fn err_xml(&self, cause: impl std::error::Error + Send + Sync + 'static) -> MarcError {
        MarcError::XmlError {
            cause: Box::new(cause),
            record_index: self.record_index_opt(),
            byte_offset: None,
            source_name: self.source_name.clone(),
        }
    }

    /// Construct an [`MarcError::JsonError`] wrapping a `serde_json` error.
    /// Position information (`line`/`column`) is preserved on the wrapped
    /// cause; `byte_offset` is left `None` because translating a
    /// (line, column) pair to a byte offset requires the original input,
    /// which is not in scope here.
    #[must_use]
    pub fn err_json(&self, cause: serde_json::Error) -> MarcError {
        MarcError::JsonError {
            cause,
            record_index: self.record_index_opt(),
            byte_offset: None,
            source_name: self.source_name.clone(),
        }
    }
}

/// Read the 24-byte MARC leader from `reader`.
///
/// Returns `Ok(None)` on a clean end-of-file (no bytes available), `Ok(Some(bytes))`
/// when a full leader is read, and `Err(MarcError::IoError)` for partial reads or
/// other I/O errors.
///
/// # Errors
///
/// Returns [`MarcError::IoError`] if reading from the underlying source fails for
/// any reason other than a clean EOF before the first byte.
///
/// The error path uses the context-free `From<std::io::Error>` conversion
/// rather than [`ParseContext::err_io`]. This is deliberate: the leader is
/// read at a record boundary, *before* the caller runs
/// [`ParseContext::begin_record`] for the record about to be parsed (the
/// read may yet return a clean EOF, meaning there is no such record). At
/// that point the context's `record_index` still names the *previous*
/// record, so enriching here would misattribute a boundary-read failure.
/// Once data-area reads begin, [`read_record_data`] does carry context.
pub fn read_leader_bytes<R: Read>(reader: &mut R) -> Result<Option<[u8; LEADER_LEN]>> {
    let mut buf = [0u8; LEADER_LEN];
    match reader.read_exact(&mut buf) {
        Ok(()) => Ok(Some(buf)),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Growth step for [`read_record_data`]'s record buffer. The buffer is
/// extended one chunk at a time as bytes actually arrive, so its capacity
/// never exceeds the smaller of the leader's claimed length and the bytes
/// the stream delivered plus one chunk — a stub record claiming the ISO
/// 2709 maximum cannot force a maximum-length allocation.
pub const READ_CHUNK_LEN: usize = 8192;

/// Read the remaining `record_length - 24` bytes of a MARC record after the
/// leader has already been consumed.
///
/// Returns `(data, bytes_read)` where `data` holds exactly the bytes the
/// stream delivered (`data.len() == bytes_read`); a count short of
/// `record_length - 24` means the read hit EOF mid-record. The buffer grows
/// in [`READ_CHUNK_LEN`] steps, so its capacity is bounded by
/// `min(record_length - 24, bytes_read + READ_CHUNK_LEN)` rather than by
/// the leader's claim. In [`RecoveryMode::Strict`] a short read returns a
/// [`MarcError::TruncatedRecord`] enriched with the current positional
/// context (record index, byte offset, source filename, 001 if already
/// extracted) plus the expected/actual byte counts.
///
/// # Errors
///
/// - [`MarcError::TruncatedRecord`] when EOF is reached mid-record and
///   `recovery_mode` is [`RecoveryMode::Strict`].
/// - [`MarcError::IoError`] for other underlying I/O failures.
pub fn read_record_data<R: Read>(
    reader: &mut R,
    record_length: usize,
    recovery_mode: RecoveryMode,
    ctx: &ParseContext,
) -> Result<(Vec<u8>, usize)> {
    let expected_len = record_length.saturating_sub(LEADER_LEN);
    let mut data: Vec<u8> = Vec::new();
    let mut bytes_read = 0;
    while bytes_read < expected_len {
        if bytes_read == data.len() {
            let target = expected_len.min(bytes_read + READ_CHUNK_LEN);
            // reserve_exact (not resize's amortized reserve) keeps the
            // capacity bound exact; at most ceil(99975 / 8192) = 13
            // reallocations for a maximum-length record.
            data.reserve_exact(target - data.len());
            data.resize(target, 0);
        }
        match reader.read(&mut data[bytes_read..]) {
            Ok(0) => break,
            Ok(n) => bytes_read += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {},
            Err(e) => return Err(ctx.err_io(e)),
        }
    }
    data.truncate(bytes_read);
    if bytes_read < expected_len && recovery_mode == RecoveryMode::Strict {
        return Err(ctx.err_truncated_record(Some(expected_len), Some(bytes_read)));
    }
    Ok((data, bytes_read))
}

/// A parsed 12-byte directory entry: tag, field length in bytes, and start
/// position relative to the record's data area.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryEntry {
    /// Three-character field tag (e.g., `"245"`).
    pub tag: String,
    /// Field length in bytes, including indicators, subfields, and field terminator.
    pub length: usize,
    /// Field start position, relative to the start of the record's data area.
    pub start: usize,
}

/// Parse a single 12-byte directory entry: 3 bytes tag + 4 bytes length + 5 bytes start.
///
/// # Errors
///
/// Returns an error if the slice is shorter than 12 bytes,
/// the tag is not valid UTF-8, or either numeric field contains a non-digit byte.
pub fn parse_directory_entry(entry: &[u8]) -> Result<DirectoryEntry> {
    if entry.len() < DIRECTORY_ENTRY_LEN {
        return Err(MarcError::invalid_field_msg(format!(
            "Directory entry too short: expected {DIRECTORY_ENTRY_LEN} bytes, got {}",
            entry.len()
        )));
    }
    let tag = std::str::from_utf8(&entry[0..3])
        .map_err(|_| MarcError::invalid_field_msg("Invalid tag encoding".to_string()))?
        .to_string();
    let length = parse_4digits(&entry[3..7])?;
    let start = parse_5digits(&entry[7..12])?;
    Ok(DirectoryEntry { tag, length, start })
}

/// Parse a 4-digit ASCII number from bytes.
///
/// # Errors
///
/// Returns an error if `bytes` is not exactly 4 bytes long
/// or contains any non-digit byte.
pub fn parse_4digits(bytes: &[u8]) -> Result<usize> {
    if bytes.len() != 4 {
        return Err(MarcError::invalid_field_msg(format!(
            "Expected 4-digit field, got {} bytes",
            bytes.len()
        )));
    }
    parse_ascii_digits(bytes)
}

/// Parse a 5-digit ASCII number from bytes.
///
/// # Errors
///
/// Returns an error if `bytes` is not exactly 5 bytes long
/// or contains any non-digit byte.
pub fn parse_5digits(bytes: &[u8]) -> Result<usize> {
    if bytes.len() != 5 {
        return Err(MarcError::invalid_field_msg(format!(
            "Expected 5-digit field, got {} bytes",
            bytes.len()
        )));
    }
    parse_ascii_digits(bytes)
}

/// Parse an arbitrary slice of ASCII digit bytes into a `usize`. Internal helper
/// behind the fixed-width [`parse_4digits`] / [`parse_5digits`] entry points;
/// performs no length check of its own.
fn parse_ascii_digits(bytes: &[u8]) -> Result<usize> {
    let mut result = 0usize;
    for &byte in bytes {
        if byte.is_ascii_digit() {
            result = result * 10 + (byte - b'0') as usize;
        } else {
            return Err(MarcError::invalid_field_msg(format!(
                "Invalid numeric field: expected digits, got byte {}",
                byte as char
            )));
        }
    }
    Ok(result)
}

/// Whether a tag identifies a control field (`001`–`009`, plus `00X`-style
/// numeric tags below `010`).
#[must_use]
pub fn is_control_field_tag(tag: &str) -> bool {
    tag.len() == 3 && tag.starts_with('0') && tag.chars().all(|c| c.is_ascii_digit()) && tag < "010"
}

/// Append `value` to `buf` as a zero-padded ASCII decimal of at least
/// `width` digits, written directly without a heap `format!` allocation.
///
/// Used for the ISO 2709 directory's field-length (width 4) and
/// starting-position (width 5) entries — written once per field by all three
/// writers. Matches `format!("{value:0width$}")`: a value needing more than
/// `width` digits is written in full (the directory entry is then malformed,
/// exactly as the `format!` path was).
pub(crate) fn push_zero_padded(buf: &mut Vec<u8>, value: usize, width: usize) {
    const DIGITS: &[u8; 10] = b"0123456789";
    let mut digits = 1;
    let mut n = value;
    while n >= 10 {
        n /= 10;
        digits += 1;
    }
    let total = digits.max(width);
    let start = buf.len();
    buf.resize(start + total, b'0');
    let mut n = value;
    let mut i = start + total;
    while n > 0 {
        i -= 1;
        buf[i] = DIGITS[n % 10];
        n /= 10;
    }
}

/// Validate that a field tag fits the ISO 2709 directory's fixed-width
/// 3-byte tag field. Tags must be exactly 3 ASCII bytes; non-ASCII
/// characters re-encode to multiple UTF-8 bytes when written and
/// overflow the directory entry. The bibliographic, authority, and
/// holdings writers all share this rule; the reader's directory walker
/// enforces the parse-side counterpart by firing
/// [`MarcError::DirectoryInvalid`] (E101) on non-ASCII tag bytes.
///
/// # Errors
///
/// Returns [`MarcError::WriterError`] (E404) when `tag.len() != 3` or
/// any byte of `tag` is not ASCII. The caller may pass `None` for
/// `record_index` and `record_control_number` if no per-record context
/// is available.
pub fn validate_directory_tag(
    tag: &str,
    record_index: Option<usize>,
    record_control_number: Option<&str>,
) -> Result<()> {
    if tag.len() == 3 && tag.as_bytes().iter().all(u8::is_ascii) {
        return Ok(());
    }
    Err(MarcError::WriterError {
        record_index,
        record_control_number: record_control_number.map(String::from),
        message: format!(
            "Field tag {tag:?} is not 3 ASCII bytes (got {} bytes); cannot fit into the ISO 2709 directory entry's tag field",
            tag.len()
        ),
    })
}

/// ISO 2709 stores both record length and base-address-of-data as
/// 5-ASCII-digit fields in the leader (bytes 0-4 and 12-16). Values
/// above this cannot be represented; the writer must refuse the record
/// before serialization rather than emit a 6-digit field that produces
/// an unparseable leader.
pub const ISO2709_MAX_FIELD: usize = 99_999;

/// Reject a record whose serialized total length or base address would
/// overflow the leader's 5-digit fields. Shared by the bibliographic,
/// authority, and holdings writers; the bound is fixed by the ISO 2709
/// leader layout, not by writer convention.
///
/// # Errors
///
/// Returns [`MarcError::WriterError`] (E404) with the documented
/// positional context (`record_index`, `record_control_number`) and a
/// `message` naming which limit was exceeded.
pub fn check_iso2709_size(
    record_length: usize,
    base_address: usize,
    record_index: Option<usize>,
    record_control_number: Option<&str>,
) -> Result<()> {
    if record_length > ISO2709_MAX_FIELD {
        return Err(MarcError::WriterError {
            record_index,
            record_control_number: record_control_number.map(String::from),
            message: format!(
                "Record length exceeds ISO 2709 limit ({record_length} bytes; max {ISO2709_MAX_FIELD})"
            ),
        });
    }
    if base_address > ISO2709_MAX_FIELD {
        return Err(MarcError::WriterError {
            record_index,
            record_control_number: record_control_number.map(String::from),
            message: format!(
                "Base address exceeds ISO 2709 limit ({base_address} bytes; max {ISO2709_MAX_FIELD})"
            ),
        });
    }
    Ok(())
}

/// How to handle unrecognized bytes encountered while walking subfield
/// boundaries inside a data field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubfieldStructureMode {
    /// Raise an error if a byte that should be a subfield delimiter isn't
    /// one. Matches the bibliographic reader's historical behavior.
    Strict,
    /// Silently skip any byte between subfields that isn't a recognized
    /// delimiter or terminator. Matches the authority and holdings readers'
    /// historical behavior.
    Permissive,
}

/// How to convert subfield value bytes into a `String`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Utf8DecodeMode {
    /// Replace invalid UTF-8 sequences with the Unicode replacement
    /// character. Selected by [`crate::ValidationLevel::Structural`].
    Lossy,
    /// Raise an error if subfield value bytes are not valid UTF-8.
    /// Selected by [`crate::ValidationLevel::StrictMarc`].
    Strict,
}

/// How to handle indicator bytes that aren't an ASCII digit (`0`-`9`)
/// or space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorMode {
    /// Accept any byte as an indicator without validation. Selected by
    /// [`crate::ValidationLevel::Structural`].
    Lossy,
    /// Raise [`crate::MarcError::InvalidIndicator`] (E201) on any
    /// byte that isn't an ASCII digit or space. Selected by
    /// [`crate::ValidationLevel::StrictMarc`].
    Strict,
}

/// How to handle subfield-code bytes that aren't printable ASCII
/// (`is_ascii_graphic`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubfieldCodeMode {
    /// Accept any byte as a subfield code without validation. Selected
    /// by [`crate::ValidationLevel::Structural`].
    Lossy,
    /// Raise [`crate::MarcError::BadSubfieldCode`] (E202) on any byte
    /// that isn't printable ASCII. Selected by
    /// [`crate::ValidationLevel::StrictMarc`].
    Strict,
}

/// Configuration for [`parse_data_field`].
///
/// `structure` is per-reader-type (bibliographic = Strict, authority +
/// holdings = Permissive). The other three flow from
/// [`crate::ValidationLevel`]: every reader behaves the same way at
/// each level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataFieldParseConfig {
    /// How to handle unrecognized bytes between subfields.
    pub structure: SubfieldStructureMode,
    /// How to decode subfield value bytes.
    pub utf8: Utf8DecodeMode,
    /// How to handle out-of-range indicator bytes.
    pub indicator: IndicatorMode,
    /// How to handle out-of-range subfield-code bytes.
    pub subfield_code: SubfieldCodeMode,
}

impl DataFieldParseConfig {
    /// Translate a [`crate::ValidationLevel`] into the
    /// validation-level-driven mode triple (`utf8`, `indicator`,
    /// `subfield_code`). The `structure` mode is owned by the per-reader
    /// constructor below.
    const fn modes_for(
        level: crate::ValidationLevel,
    ) -> (Utf8DecodeMode, IndicatorMode, SubfieldCodeMode) {
        match level {
            crate::ValidationLevel::Structural => (
                Utf8DecodeMode::Lossy,
                IndicatorMode::Lossy,
                SubfieldCodeMode::Lossy,
            ),
            crate::ValidationLevel::StrictMarc => (
                Utf8DecodeMode::Strict,
                IndicatorMode::Strict,
                SubfieldCodeMode::Strict,
            ),
        }
    }

    /// Bibliographic reader config (strict structure: any byte that
    /// should be a subfield delimiter but isn't raises an error).
    #[must_use]
    pub const fn bibliographic(level: crate::ValidationLevel) -> Self {
        let (utf8, indicator, subfield_code) = Self::modes_for(level);
        Self {
            structure: SubfieldStructureMode::Strict,
            utf8,
            indicator,
            subfield_code,
        }
    }

    /// Authority reader config (permissive structure: unrecognized bytes
    /// between subfields are silently skipped).
    #[must_use]
    pub const fn authority(level: crate::ValidationLevel) -> Self {
        let (utf8, indicator, subfield_code) = Self::modes_for(level);
        Self {
            structure: SubfieldStructureMode::Permissive,
            utf8,
            indicator,
            subfield_code,
        }
    }

    /// Holdings reader config (permissive structure: unrecognized bytes
    /// between subfields are silently skipped).
    #[must_use]
    pub const fn holdings(level: crate::ValidationLevel) -> Self {
        let (utf8, indicator, subfield_code) = Self::modes_for(level);
        Self {
            structure: SubfieldStructureMode::Permissive,
            utf8,
            indicator,
            subfield_code,
        }
    }
}

/// Parse a data field's indicator bytes followed by its subfields, producing
/// a [`Field`].
///
/// The first two bytes of `field_data` are interpreted as indicators 1 and 2.
/// Subsequent bytes are walked subfield-by-subfield until either the slice is
/// exhausted or a [`FIELD_TERMINATOR`] is encountered.
///
/// The `config` argument selects between the historical per-reader behaviors
/// for handling unrecognized bytes and decoding subfield values; see
/// [`DataFieldParseConfig`].
///
/// The `ctx` is used to attach positional metadata to any error raised; the
/// caller is responsible for setting `ctx.current_field_tag` to `tag` before
/// the call so subfield-level errors include the field context.
///
/// # Errors
///
/// Returns [`MarcError::InvalidField`] if `field_data` is shorter than 2
/// bytes (insufficient for indicators), [`MarcError::BadSubfieldCode`] (when
/// applicable in future enrichment work), or [`MarcError::EncodingError`] if
/// `config.utf8` is [`Utf8DecodeMode::Strict`] and a subfield value contains
/// invalid UTF-8.
// Forced inline: removing this regresses read-hot-path throughput
// ~15% (see CHANGELOG v0.8 perf restoration entry). Pairs with the
// compact `current_field_tag: Option<[u8; 3]>` in `ParseContext`.
#[allow(clippy::inline_always)]
#[inline(always)]
pub fn parse_data_field(
    field_data: &[u8],
    tag: &str,
    config: DataFieldParseConfig,
    ctx: &ParseContext,
) -> Result<Field> {
    if field_data.len() < 2 {
        return Err(ctx.err_invalid_field("Data field too short (needs indicators)"));
    }
    let i1 = field_data[0];
    let i2 = field_data[1];
    // Per MARC 21, indicator bytes are ASCII digit (b'0'..=b'9') or
    // space (b' '); any other byte fires `InvalidIndicator` (E201) at
    // `IndicatorMode::Strict`. Under `IndicatorMode::Lossy` the byte
    // is accepted as-is.
    if config.indicator == IndicatorMode::Strict {
        if !is_valid_indicator(i1) {
            return Err(ctx.err_invalid_indicator(0, &[i1], "ASCII digit (0-9) or space"));
        }
        if !is_valid_indicator(i2) {
            return Err(ctx.err_invalid_indicator(1, &[i2], "ASCII digit (0-9) or space"));
        }
        // Per-tag MARC 21 indicator semantics (e.g., 245 ind1 must be 0/1).
        // Tags without rules are accepted as-is.
        if let Some(rules) = marc21_indicator_validator().get_rules(tag) {
            if !rules.indicator1.is_valid(i1 as char) {
                return Err(ctx.err_invalid_indicator(0, &[i1], rules.indicator1.expected_human()));
            }
            if !rules.indicator2.is_valid(i2 as char) {
                return Err(ctx.err_invalid_indicator(1, &[i2], rules.indicator2.expected_human()));
            }
        }
    }
    let mut field = Field::new(tag.to_string(), i1 as char, i2 as char);
    let subfields = parse_subfields(&field_data[2..], config, ctx)?;
    field.subfields = subfields;
    Ok(field)
}

#[inline]
fn is_valid_indicator(b: u8) -> bool {
    b.is_ascii_digit() || b == b' '
}

/// Walk the subfield bytes of a data field (everything after the two
/// indicator bytes) and produce a vector of [`Subfield`]s.
///
/// Behavior on unrecognized bytes between subfields is controlled by
/// `config.structure`; UTF-8 decoding of subfield values is controlled by
/// `config.utf8`. See [`DataFieldParseConfig`] for the mappings to the
/// per-reader historical behaviors.
///
/// # Errors
///
/// Returns [`MarcError::InvalidField`] if `config.structure` is
/// [`SubfieldStructureMode::Strict`] and an unrecognized byte is encountered
/// where a subfield delimiter was expected. Returns
/// [`MarcError::EncodingError`] if `config.utf8` is
/// [`Utf8DecodeMode::Strict`] and a subfield value contains invalid UTF-8.
pub fn parse_subfields(
    bytes: &[u8],
    config: DataFieldParseConfig,
    ctx: &ParseContext,
) -> Result<SmallVec<[Subfield; 4]>> {
    let mut subfields: SmallVec<[Subfield; 4]> = SmallVec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        let byte = bytes[pos];
        if byte == FIELD_TERMINATOR {
            break;
        }
        if byte != SUBFIELD_DELIMITER {
            match config.structure {
                SubfieldStructureMode::Strict => {
                    return Err(ctx.err_invalid_field("Expected subfield delimiter"));
                },
                SubfieldStructureMode::Permissive => {
                    pos += 1;
                    continue;
                },
            }
        }
        // We're on a delimiter; the next byte (if any) is the code.
        pos += 1;
        if pos >= bytes.len() {
            break;
        }
        let code_byte = bytes[pos];
        // Subfield codes must be printable, non-space ASCII per MARC 21
        // (`is_ascii_graphic` covers a-z / 0-9 plus punctuation; rejects
        // NUL, control bytes, space, and high bytes). At
        // `SubfieldCodeMode::Strict` a violation raises E202; under
        // `SubfieldCodeMode::Lossy` the byte is accepted as the code.
        if config.subfield_code == SubfieldCodeMode::Strict && !code_byte.is_ascii_graphic() {
            return Err(ctx.err_bad_subfield_code(code_byte));
        }
        let code = code_byte as char;
        pos += 1;
        let mut end = pos;
        while end < bytes.len()
            && bytes[end] != SUBFIELD_DELIMITER
            && bytes[end] != FIELD_TERMINATOR
        {
            end += 1;
        }
        let value_bytes = &bytes[pos..end];
        let value = match config.utf8 {
            Utf8DecodeMode::Lossy => String::from_utf8_lossy(value_bytes).to_string(),
            Utf8DecodeMode::Strict => std::str::from_utf8(value_bytes)
                .map_err(|e| ctx.err_encoding(format!("Invalid UTF-8 in subfield value: {e}")))?
                .to_string(),
        };
        subfields.push(Subfield { code, value });
        pos = end;
    }
    Ok(subfields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn set_parse_buffer_shares_the_allocation() {
        // The context must take the record bytes by refcount bump, not by
        // copy — every record passes through set_parse_buffer, and the
        // per-record alloc+memcpy this guards against was the read path's
        // largest avoidable cost (PERF-2).
        let data = std::sync::Arc::new(b"00100nam a2200049 i 4500".to_vec());
        let mut ctx = ParseContext::new();
        ctx.set_parse_buffer(std::sync::Arc::clone(&data), 0);
        assert!(std::sync::Arc::ptr_eq(
            ctx.current_buffer.as_ref().expect("buffer was set"),
            &data
        ));
    }

    #[test]
    fn errors_capture_bytes_near_from_shared_buffer() {
        let data = std::sync::Arc::new(b"00100nam a2200049 i 4500".to_vec());
        let mut ctx = ParseContext::new();
        ctx.set_parse_buffer(std::sync::Arc::clone(&data), 0);
        ctx.stream_byte_offset = 5;
        let err = ctx.err_invalid_field("test");
        let window = err.bytes_near().expect("bytes_near captured");
        assert!(
            window.bytes.windows(3).any(|w| w == b"nam"),
            "bytes_near window missing leader bytes: {:?}",
            window.bytes
        );
    }

    #[test]
    fn read_leader_bytes_eof_returns_none() {
        let mut reader = Cursor::new(Vec::<u8>::new());
        assert!(matches!(read_leader_bytes(&mut reader), Ok(None)));
    }

    #[test]
    fn read_leader_bytes_full_returns_bytes() {
        let leader = b"00100nam a2200049 i 4500".to_vec();
        let mut reader = Cursor::new(leader.clone());
        let bytes = read_leader_bytes(&mut reader).unwrap().unwrap();
        assert_eq!(&bytes[..], &leader[..]);
    }

    #[test]
    fn read_leader_bytes_partial_treated_as_eof() {
        // Matches existing reader behavior: any UnexpectedEof while reading the
        // leader (including a short partial read) is treated as a clean EOF.
        let mut reader = Cursor::new(vec![b'0'; 10]);
        assert!(matches!(read_leader_bytes(&mut reader), Ok(None)));
    }

    #[test]
    fn read_record_data_full_read() {
        let mut reader = Cursor::new(vec![b'x'; 76]);
        let ctx = ParseContext::new();
        let (data, bytes_read) =
            read_record_data(&mut reader, 100, RecoveryMode::Strict, &ctx).unwrap();
        assert_eq!(data.len(), 76);
        assert_eq!(bytes_read, 76);
    }

    #[test]
    fn read_record_data_strict_mode_errors_on_truncation() {
        let mut reader = Cursor::new(vec![b'x'; 50]);
        let mut ctx = ParseContext::new().with_source_name("test.mrc");
        ctx.begin_record();
        let err = read_record_data(&mut reader, 100, RecoveryMode::Strict, &ctx)
            .expect_err("strict mode should error on truncation");
        match err {
            MarcError::TruncatedRecord {
                record_index,
                source_name,
                expected_length,
                actual_length,
                ..
            } => {
                assert_eq!(record_index, Some(1));
                assert_eq!(source_name.as_deref(), Some("test.mrc"));
                assert_eq!(expected_length, Some(76));
                assert_eq!(actual_length, Some(50));
            },
            other => panic!("expected TruncatedRecord, got {other:?}"),
        }
    }

    #[test]
    fn read_record_data_lenient_mode_returns_truncated_count() {
        let mut reader = Cursor::new(vec![b'x'; 50]);
        let ctx = ParseContext::new();
        let (data, bytes_read) =
            read_record_data(&mut reader, 100, RecoveryMode::Lenient, &ctx).unwrap();
        assert_eq!(
            data.len(),
            50,
            "buffer holds exactly the bytes the stream delivered"
        );
        assert_eq!(bytes_read, 50, "actual bytes read short of expected_len");
    }

    #[test]
    fn read_record_data_allocation_bounded_by_available_bytes() {
        // A 25-byte stub whose leader claims the ISO 2709 maximum must not
        // allocate the claimed 99975-byte body; the buffer is bounded by
        // the bytes actually available plus one growth chunk.
        let available = 1usize;
        let mut reader = Cursor::new(vec![b'x'; available]);
        let ctx = ParseContext::new();
        let (data, bytes_read) =
            read_record_data(&mut reader, 99_999, RecoveryMode::Lenient, &ctx).unwrap();
        assert_eq!(bytes_read, available);
        assert_eq!(data.len(), available);
        assert!(
            data.capacity() <= available + READ_CHUNK_LEN,
            "capacity {} exceeds available {} + chunk {}",
            data.capacity(),
            available,
            READ_CHUNK_LEN
        );
    }

    #[test]
    fn parse_4digits_valid() {
        assert_eq!(parse_4digits(b"0042").unwrap(), 42);
        assert_eq!(parse_4digits(b"9999").unwrap(), 9999);
    }

    #[test]
    fn parse_4digits_wrong_length() {
        assert!(parse_4digits(b"42").is_err());
        assert!(parse_4digits(b"00042").is_err());
    }

    #[test]
    fn parse_4digits_non_digit() {
        assert!(parse_4digits(b"00X2").is_err());
    }

    #[test]
    fn parse_5digits_valid() {
        assert_eq!(parse_5digits(b"00042").unwrap(), 42);
        assert_eq!(parse_5digits(b"99999").unwrap(), 99999);
    }

    #[test]
    fn parse_5digits_wrong_length() {
        assert!(parse_5digits(b"0042").is_err());
        assert!(parse_5digits(b"000042").is_err());
    }

    #[test]
    fn parse_directory_entry_valid() {
        let entry = b"245001500042";
        let parsed = parse_directory_entry(entry).unwrap();
        assert_eq!(parsed.tag, "245");
        assert_eq!(parsed.length, 15);
        assert_eq!(parsed.start, 42);
    }

    #[test]
    fn parse_directory_entry_too_short() {
        assert!(parse_directory_entry(b"24500").is_err());
    }

    #[test]
    fn parse_directory_entry_invalid_length() {
        let entry = b"245XX1500042";
        assert!(parse_directory_entry(entry).is_err());
    }

    #[test]
    fn is_control_field_tag_recognizes_control_tags() {
        assert!(is_control_field_tag("001"));
        assert!(is_control_field_tag("008"));
        assert!(is_control_field_tag("009"));
    }

    #[test]
    fn is_control_field_tag_rejects_data_tags() {
        assert!(!is_control_field_tag("010"));
        assert!(!is_control_field_tag("245"));
        assert!(!is_control_field_tag("999"));
    }

    #[test]
    fn is_control_field_tag_rejects_non_numeric() {
        assert!(!is_control_field_tag("LDR"));
        assert!(!is_control_field_tag("00A"));
    }

    #[test]
    fn is_control_field_tag_rejects_wrong_length() {
        assert!(!is_control_field_tag("01"));
        assert!(!is_control_field_tag("0010"));
    }
}
