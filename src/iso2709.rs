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

use crate::error::{MarcError, Result};
use crate::record::{Field, Subfield};
use crate::recovery::RecoveryMode;
use smallvec::SmallVec;
use std::io::Read;

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
    /// Field tag currently being parsed, when known.
    pub current_field_tag: Option<String>,
    /// Subfield code currently being parsed, when known.
    pub current_subfield_code: Option<u8>,
    /// Indicator position currently being parsed (0 or 1), when known.
    pub current_indicator_position: Option<u8>,
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

    /// Construct an [`MarcError::InvalidLeader`] inheriting the current
    /// stream/record positional state.
    #[must_use]
    pub fn err_invalid_leader(
        &self,
        found: Option<&[u8]>,
        expected: impl Into<String>,
        cause: Option<String>,
    ) -> MarcError {
        let found_bytes = found.map(|b| crate::error::truncate_bytes(b).0);
        MarcError::InvalidLeader {
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            record_byte_offset: Some(self.record_byte_offset()),
            source_name: self.source_name.clone(),
            found: found_bytes,
            expected: Some(expected.into()),
            cause,
        }
    }

    /// Construct an [`MarcError::RecordLengthInvalid`] inheriting the current
    /// stream/record positional state.
    #[must_use]
    pub fn err_record_length_invalid(
        &self,
        found: Option<&[u8]>,
        expected: impl Into<String>,
    ) -> MarcError {
        let found_bytes = found.map(|b| crate::error::truncate_bytes(b).0);
        MarcError::RecordLengthInvalid {
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            source_name: self.source_name.clone(),
            found: found_bytes,
            expected: Some(expected.into()),
        }
    }

    /// Construct an [`MarcError::BaseAddressInvalid`] inheriting the current
    /// stream/record positional state.
    #[must_use]
    pub fn err_base_address_invalid(
        &self,
        found: Option<&[u8]>,
        expected: impl Into<String>,
    ) -> MarcError {
        let found_bytes = found.map(|b| crate::error::truncate_bytes(b).0);
        MarcError::BaseAddressInvalid {
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            source_name: self.source_name.clone(),
            record_control_number: self.record_control_number.clone(),
            found: found_bytes,
            expected: Some(expected.into()),
        }
    }

    /// Construct an [`MarcError::BaseAddressNotFound`] inheriting the
    /// current stream/record positional state.
    #[must_use]
    pub fn err_base_address_not_found(&self) -> MarcError {
        MarcError::BaseAddressNotFound {
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            source_name: self.source_name.clone(),
            record_control_number: self.record_control_number.clone(),
        }
    }

    /// Construct an [`MarcError::DirectoryInvalid`] inheriting the current
    /// stream/record positional state.
    #[must_use]
    pub fn err_directory_invalid(
        &self,
        found: Option<&[u8]>,
        expected: impl Into<String>,
    ) -> MarcError {
        let found_bytes = found.map(|b| crate::error::truncate_bytes(b).0);
        MarcError::DirectoryInvalid {
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            record_byte_offset: Some(self.record_byte_offset()),
            source_name: self.source_name.clone(),
            record_control_number: self.record_control_number.clone(),
            field_tag: self.current_field_tag.clone(),
            found: found_bytes,
            expected: Some(expected.into()),
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
        let (found_bytes, _) = crate::error::truncate_bytes(found);
        MarcError::InvalidIndicator {
            record_index: self.record_index_opt(),
            byte_offset: Some(self.stream_byte_offset),
            record_byte_offset: Some(self.record_byte_offset()),
            source_name: self.source_name.clone(),
            record_control_number: self.record_control_number.clone(),
            field_tag: self.current_field_tag.clone(),
            indicator_position: Some(indicator_position),
            found: Some(found_bytes),
            expected: Some(expected.into()),
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
            field_tag: self.current_field_tag.clone(),
            subfield_code,
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
            field_tag: self.current_field_tag.clone(),
            message: message.into(),
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
            field_tag: self.current_field_tag.clone(),
            message: message.into(),
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
pub fn read_leader_bytes<R: Read>(reader: &mut R) -> Result<Option<[u8; LEADER_LEN]>> {
    let mut buf = [0u8; LEADER_LEN];
    match reader.read_exact(&mut buf) {
        Ok(()) => Ok(Some(buf)),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Read the remaining `record_length - 24` bytes of a MARC record after the leader
/// has already been consumed.
///
/// Returns `(data, was_truncated)` where `was_truncated` is `true` when the read
/// hit EOF before filling the buffer. In [`RecoveryMode::Strict`] a short read
/// returns [`MarcError::TruncatedRecord`] instead of `(data, true)`.
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
) -> Result<(Vec<u8>, bool)> {
    let expected_len = record_length.saturating_sub(LEADER_LEN);
    let mut data = vec![0u8; expected_len];
    match reader.read_exact(&mut data) {
        Ok(()) => Ok((data, false)),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            if recovery_mode == RecoveryMode::Strict {
                Err(MarcError::truncated_msg(
                    "Unexpected end of file while reading record data".to_string(),
                ))
            } else {
                Ok((data, true))
            }
        },
        Err(e) => Err(e.into()),
    }
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
    /// character. Matches the bibliographic and authority readers'
    /// historical behavior.
    Lossy,
    /// Raise an error if subfield value bytes are not valid UTF-8.
    /// Matches the holdings reader's historical behavior.
    Strict,
}

/// Configuration for [`parse_data_field`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataFieldParseConfig {
    /// How to handle unrecognized bytes between subfields.
    pub structure: SubfieldStructureMode,
    /// How to decode subfield value bytes.
    pub utf8: Utf8DecodeMode,
}

impl DataFieldParseConfig {
    /// Configuration matching the bibliographic reader (strict structure,
    /// lossy UTF-8).
    pub const BIBLIOGRAPHIC: Self = Self {
        structure: SubfieldStructureMode::Strict,
        utf8: Utf8DecodeMode::Lossy,
    };

    /// Configuration matching the authority reader (permissive structure,
    /// lossy UTF-8).
    pub const AUTHORITY: Self = Self {
        structure: SubfieldStructureMode::Permissive,
        utf8: Utf8DecodeMode::Lossy,
    };

    /// Configuration matching the holdings reader (permissive structure,
    /// strict UTF-8).
    pub const HOLDINGS: Self = Self {
        structure: SubfieldStructureMode::Permissive,
        utf8: Utf8DecodeMode::Strict,
    };
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
pub fn parse_data_field(
    field_data: &[u8],
    tag: &str,
    config: DataFieldParseConfig,
    ctx: &ParseContext,
) -> Result<Field> {
    if field_data.len() < 2 {
        return Err(ctx.err_invalid_field("Data field too short (needs indicators)"));
    }
    let indicator1 = field_data[0] as char;
    let indicator2 = field_data[1] as char;
    let mut field = Field::new(tag.to_string(), indicator1, indicator2);
    let subfields = parse_subfields(&field_data[2..], config, ctx)?;
    field.subfields = subfields;
    Ok(field)
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
        let code = bytes[pos] as char;
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
        let (data, truncated) = read_record_data(&mut reader, 100, RecoveryMode::Strict).unwrap();
        assert_eq!(data.len(), 76);
        assert!(!truncated);
    }

    #[test]
    fn read_record_data_strict_mode_errors_on_truncation() {
        let mut reader = Cursor::new(vec![b'x'; 50]);
        assert!(matches!(
            read_record_data(&mut reader, 100, RecoveryMode::Strict),
            Err(MarcError::TruncatedRecord { .. })
        ));
    }

    #[test]
    fn read_record_data_lenient_mode_returns_truncated_flag() {
        let mut reader = Cursor::new(vec![b'x'; 50]);
        let (data, truncated) = read_record_data(&mut reader, 100, RecoveryMode::Lenient).unwrap();
        assert_eq!(data.len(), 76);
        assert!(truncated);
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
