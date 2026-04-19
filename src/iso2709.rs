//! Shared ISO 2709 parsing primitives used by all three MARC readers
//! (bibliographic, authority, holdings).
//!
//! This module exists to consolidate the byte-level parsing logic that was
//! previously duplicated across [`crate::reader`], [`crate::authority_reader`],
//! and [`crate::holdings_reader`]. Each reader still owns its own record-type
//! dispatch (which `Record`/`AuthorityRecord`/`HoldingsRecord` to instantiate
//! and how fields are routed by tag); only the format-level primitives live
//! here.
//!
//! # Scope
//!
//! Currently extracted: the I/O loop for reading a leader, the truncation-aware
//! record-data read, single-entry directory parsing, ASCII numeric helpers, and
//! the control-field-tag predicate.
//!
//! Not yet extracted: subfield-level parsing. The three readers have subtly
//! different subfield-parsing semantics (lossy vs strict UTF-8 decoding, error
//! vs skip on unrecognized bytes) that cannot be unified without a semantic
//! change. Convergence is deferred to the error-enrichment work where
//! `ParseContext` will formalize the per-format behavior.

use crate::error::{MarcError, Result};
use crate::recovery::RecoveryMode;
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
        Err(e) => Err(MarcError::IoError(e)),
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
                Err(MarcError::TruncatedRecord(
                    "Unexpected end of file while reading record data".to_string(),
                ))
            } else {
                Ok((data, true))
            }
        },
        Err(e) => Err(MarcError::IoError(e)),
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
/// Returns [`MarcError::InvalidRecord`] if the slice is shorter than 12 bytes,
/// the tag is not valid UTF-8, or either numeric field contains a non-digit byte.
pub fn parse_directory_entry(entry: &[u8]) -> Result<DirectoryEntry> {
    if entry.len() < DIRECTORY_ENTRY_LEN {
        return Err(MarcError::InvalidRecord(format!(
            "Directory entry too short: expected {DIRECTORY_ENTRY_LEN} bytes, got {}",
            entry.len()
        )));
    }
    let tag = std::str::from_utf8(&entry[0..3])
        .map_err(|_| MarcError::InvalidRecord("Invalid tag encoding".to_string()))?
        .to_string();
    let length = parse_4digits(&entry[3..7])?;
    let start = parse_5digits(&entry[7..12])?;
    Ok(DirectoryEntry { tag, length, start })
}

/// Parse a 4-digit ASCII number from bytes.
///
/// # Errors
///
/// Returns [`MarcError::InvalidRecord`] if `bytes` is not exactly 4 bytes long
/// or contains any non-digit byte.
pub fn parse_4digits(bytes: &[u8]) -> Result<usize> {
    if bytes.len() != 4 {
        return Err(MarcError::InvalidRecord(format!(
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
/// Returns [`MarcError::InvalidRecord`] if `bytes` is not exactly 5 bytes long
/// or contains any non-digit byte.
pub fn parse_5digits(bytes: &[u8]) -> Result<usize> {
    if bytes.len() != 5 {
        return Err(MarcError::InvalidRecord(format!(
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
            return Err(MarcError::InvalidRecord(format!(
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
            Err(MarcError::TruncatedRecord(_))
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
