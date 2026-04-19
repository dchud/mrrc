//! Error types for MARC operations.
//!
//! Provides [`MarcError`] for all MARC library operations and the [`Result`]
//! convenience type. Each variant carries structured positional metadata
//! describing where in a stream/record/field the problem occurred — see the
//! per-variant requirements in the documentation for which fields are always
//! populated, may be populated, or are not applicable.
//!
//! All variants are `Send + Sync` so errors can cross thread boundaries used
//! by the parallel parsing paths.

use std::fmt;
use thiserror::Error;

/// Maximum length in bytes retained in a [`MarcError`]'s `found` field.
///
/// Bounds the memory cost of an error in lenient/permissive recovery modes
/// where many errors may be accumulated.
pub const FOUND_BYTES_CAP: usize = 32;

/// Truncate a byte slice to at most [`FOUND_BYTES_CAP`] bytes.
///
/// Returns `(bytes, was_truncated)`. The caller is responsible for surfacing
/// the truncation in any rendering it produces (the conventional marker is a
/// trailing `…`).
#[must_use]
pub fn truncate_bytes(input: &[u8]) -> (Vec<u8>, bool) {
    if input.len() > FOUND_BYTES_CAP {
        (input[..FOUND_BYTES_CAP].to_vec(), true)
    } else {
        (input.to_vec(), false)
    }
}

/// Error type for all MARC library operations.
///
/// Each variant carries structured positional metadata: the record index in
/// the stream, byte offsets, the 001 control number, the field/subfield being
/// parsed, and the source filename when known. Optional fields are populated
/// opportunistically — a field that is `None` simply means the information was
/// not available at the point the error was raised, never that it was
/// suppressed.
///
/// The default [`fmt::Display`] impl produces a one-line, actionable summary
/// with byte offset visually subordinate. Use [`MarcError::detailed`] for the
/// multi-line diagnostic format.
#[derive(Error, Debug)]
pub enum MarcError {
    /// The 24-byte leader is malformed or contains values that fail validation.
    InvalidLeader {
        /// 1-based record index in the stream, when known.
        record_index: Option<usize>,
        /// Absolute byte offset within the stream where the error occurred.
        byte_offset: Option<usize>,
        /// Byte offset within the current record (typically 0 for leader errors).
        record_byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
        /// The bytes that triggered the error, capped at [`FOUND_BYTES_CAP`].
        found: Option<Vec<u8>>,
        /// Human-readable description of what was expected.
        expected: Option<String>,
        /// Underlying cause as a string (e.g., from a leader validation routine).
        cause: Option<String>,
    },

    /// The leader's record-length field is invalid (non-numeric, too small, etc.).
    RecordLengthInvalid {
        /// 1-based record index in the stream.
        record_index: Option<usize>,
        /// Absolute byte offset within the stream.
        byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
        /// The bytes that triggered the error, capped at [`FOUND_BYTES_CAP`].
        found: Option<Vec<u8>>,
        /// Human-readable description of what was expected.
        expected: Option<String>,
    },

    /// The leader's base-address-of-data field is invalid.
    BaseAddressInvalid {
        /// 1-based record index in the stream.
        record_index: Option<usize>,
        /// Absolute byte offset within the stream.
        byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
        /// 001 control number, when already extracted.
        record_control_number: Option<String>,
        /// The bytes that triggered the error, capped at [`FOUND_BYTES_CAP`].
        found: Option<Vec<u8>>,
        /// Human-readable description of what was expected.
        expected: Option<String>,
    },

    /// The leader claims a base address of data that does not exist in the stream.
    BaseAddressNotFound {
        /// 1-based record index in the stream.
        record_index: Option<usize>,
        /// Absolute byte offset within the stream.
        byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
        /// 001 control number, when already extracted.
        record_control_number: Option<String>,
    },

    /// A directory entry is structurally invalid (bad tag, length, or start position).
    DirectoryInvalid {
        /// 1-based record index in the stream.
        record_index: Option<usize>,
        /// Absolute byte offset within the stream.
        byte_offset: Option<usize>,
        /// Byte offset within the current record.
        record_byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
        /// 001 control number, when already extracted.
        record_control_number: Option<String>,
        /// Field tag of the entry being parsed, when decodable.
        field_tag: Option<String>,
        /// The bytes that triggered the error, capped at [`FOUND_BYTES_CAP`].
        found: Option<Vec<u8>>,
        /// Human-readable description of what was expected.
        expected: Option<String>,
    },

    /// The record was truncated mid-stream.
    TruncatedRecord {
        /// 1-based record index in the stream.
        record_index: Option<usize>,
        /// Absolute byte offset within the stream.
        byte_offset: Option<usize>,
        /// Byte offset within the current record where truncation was detected.
        record_byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
        /// 001 control number, when already extracted.
        record_control_number: Option<String>,
        /// Expected total record length per the leader.
        expected_length: Option<usize>,
        /// Actual bytes available before truncation.
        actual_length: Option<usize>,
    },

    /// The end-of-record marker was not found where expected.
    EndOfRecordNotFound {
        /// 1-based record index in the stream.
        record_index: Option<usize>,
        /// Absolute byte offset within the stream.
        byte_offset: Option<usize>,
        /// Byte offset within the current record.
        record_byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
        /// 001 control number, when already extracted.
        record_control_number: Option<String>,
    },

    /// An indicator byte was invalid for its position.
    InvalidIndicator {
        /// 1-based record index in the stream.
        record_index: Option<usize>,
        /// Absolute byte offset within the stream.
        byte_offset: Option<usize>,
        /// Byte offset within the current record.
        record_byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
        /// 001 control number, when already extracted.
        record_control_number: Option<String>,
        /// Field tag containing the bad indicator.
        field_tag: Option<String>,
        /// Indicator position (0 or 1).
        indicator_position: Option<u8>,
        /// The bytes that triggered the error, capped at [`FOUND_BYTES_CAP`].
        found: Option<Vec<u8>>,
        /// Human-readable description of what was expected.
        expected: Option<String>,
    },

    /// A subfield code byte was invalid.
    BadSubfieldCode {
        /// 1-based record index in the stream.
        record_index: Option<usize>,
        /// Absolute byte offset within the stream.
        byte_offset: Option<usize>,
        /// Byte offset within the current record.
        record_byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
        /// 001 control number, when already extracted.
        record_control_number: Option<String>,
        /// Field tag containing the bad subfield.
        field_tag: Option<String>,
        /// The offending subfield code byte.
        subfield_code: u8,
    },

    /// A data field is structurally invalid in some way not covered by the
    /// more specific variants above.
    InvalidField {
        /// 1-based record index in the stream.
        record_index: Option<usize>,
        /// Absolute byte offset within the stream.
        byte_offset: Option<usize>,
        /// Byte offset within the current record.
        record_byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
        /// 001 control number, when already extracted.
        record_control_number: Option<String>,
        /// Field tag involved.
        field_tag: Option<String>,
        /// Human-readable description of the problem.
        message: String,
    },

    /// A character encoding conversion failed.
    EncodingError {
        /// 1-based record index in the stream, when known.
        record_index: Option<usize>,
        /// Absolute byte offset within the stream, when known.
        byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
        /// 001 control number, when already extracted.
        record_control_number: Option<String>,
        /// Field tag involved, when applicable.
        field_tag: Option<String>,
        /// Human-readable description of the problem.
        message: String,
    },

    /// An accessor lookup failed: a requested field was not present in the record.
    ///
    /// Unlike the parse-error variants this is not a structural failure, so it
    /// does not carry byte-offset metadata.
    FieldNotFound {
        /// 1-based record index in the stream, when known.
        record_index: Option<usize>,
        /// 001 control number of the record being queried.
        record_control_number: Option<String>,
        /// Field tag that was requested.
        field_tag: String,
    },

    /// An I/O error occurred reading or writing the underlying source/sink.
    IoError {
        /// Underlying I/O error.
        #[source]
        cause: std::io::Error,
        /// 1-based record index in the stream, when known.
        record_index: Option<usize>,
        /// Absolute byte offset within the stream, when known.
        byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
    },

    /// An error occurred during MARCXML parsing.
    XmlError {
        /// Underlying XML parser error. Boxed so any of `quick_xml`'s error
        /// types (`Error`, `DeError`, etc.) can be wrapped.
        #[source]
        cause: Box<dyn std::error::Error + Send + Sync + 'static>,
        /// 1-based record index in the document, when known.
        record_index: Option<usize>,
        /// Byte offset within the source document, when known. For XML this
        /// is typically derived from the parser's line/column position rather
        /// than a raw byte offset; it may be `None` when the parser does not
        /// expose any position information.
        byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
    },

    /// An error occurred during MARCJSON parsing.
    JsonError {
        /// Underlying JSON parser error.
        #[source]
        cause: serde_json::Error,
        /// 1-based record index in the document, when known.
        record_index: Option<usize>,
        /// Byte offset within the source document, when known. Computed from
        /// `serde_json::Error::line()` and `column()` when both are
        /// available; left `None` when the parser does not expose position
        /// information.
        byte_offset: Option<usize>,
        /// Source filename or stream identifier, when known.
        source_name: Option<String>,
    },

    /// An error occurred while writing a MARC record.
    WriterError {
        /// 1-based record index being written, when known.
        record_index: Option<usize>,
        /// 001 control number of the record being written, when known.
        record_control_number: Option<String>,
        /// Human-readable description of the problem.
        message: String,
    },
}

impl MarcError {
    /// Render the error as a multi-line diagnostic with all populated
    /// positional metadata visible. Callers who want the actionable one-liner
    /// should use the default [`fmt::Display`] format instead.
    #[must_use]
    pub fn detailed(&self) -> String {
        let mut out = String::new();
        let kind = self.kind_name();
        let context = self.context_summary();
        if context.is_empty() {
            out.push_str(kind);
        } else {
            out.push_str(kind);
            out.push_str(" at ");
            out.push_str(&context);
        }
        let lines = self.detail_lines();
        let label_width = lines.iter().map(|(l, _)| l.len()).max().unwrap_or(0);
        for (label, value) in &lines {
            out.push_str("\n  ");
            out.push_str(label);
            // Pad each label up to the widest label in this output so columns
            // align even when label lengths vary widely (e.g.,
            // "001:" vs "record-relative:").
            for _ in label.len()..=label_width {
                out.push(' ');
            }
            out.push_str(value);
        }
        out
    }

    /// Stable error code for this variant (`E001`–`E4xx`). Forms the
    /// canonical programmatic identifier — callers can match on this rather
    /// than on the variant name to keep handlers stable across enum
    /// restructures. See [`MarcError::slug`] for the human-friendly
    /// counterpart and [`MarcError::help_url`] for the docs URL.
    ///
    /// Codes never get re-purposed: a retired check leaves its docs entry
    /// in place pointing to a replacement. See `CONTRIBUTING.md` for the
    /// full stability policy.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            MarcError::RecordLengthInvalid { .. } => "E001",
            MarcError::InvalidLeader { .. } => "E002",
            MarcError::BaseAddressInvalid { .. } => "E003",
            MarcError::BaseAddressNotFound { .. } => "E004",
            MarcError::TruncatedRecord { .. } => "E005",
            MarcError::EndOfRecordNotFound { .. } => "E006",
            MarcError::IoError { .. } => "E007",
            MarcError::DirectoryInvalid { .. } => "E101",
            MarcError::FieldNotFound { .. } => "E105",
            MarcError::InvalidField { .. } => "E106",
            MarcError::InvalidIndicator { .. } => "E201",
            MarcError::BadSubfieldCode { .. } => "E202",
            MarcError::EncodingError { .. } => "E301",
            MarcError::XmlError { .. } => "E401",
            MarcError::JsonError { .. } => "E402",
            MarcError::WriterError { .. } => "E404",
        }
    }

    /// Human-friendly slug for this variant (e.g., `"invalid_indicator"`).
    /// Stable counterpart to [`MarcError::code`]; both are guaranteed not
    /// to change for an existing variant.
    #[must_use]
    pub fn slug(&self) -> &'static str {
        match self {
            MarcError::RecordLengthInvalid { .. } => "record_length_invalid",
            MarcError::InvalidLeader { .. } => "leader_invalid",
            MarcError::BaseAddressInvalid { .. } => "base_address_invalid",
            MarcError::BaseAddressNotFound { .. } => "base_address_not_found",
            MarcError::TruncatedRecord { .. } => "truncated_record",
            MarcError::EndOfRecordNotFound { .. } => "end_of_record_not_found",
            MarcError::IoError { .. } => "io_error",
            MarcError::DirectoryInvalid { .. } => "directory_invalid",
            MarcError::FieldNotFound { .. } => "field_not_found",
            MarcError::InvalidField { .. } => "invalid_field",
            MarcError::InvalidIndicator { .. } => "invalid_indicator",
            MarcError::BadSubfieldCode { .. } => "bad_subfield_code",
            MarcError::EncodingError { .. } => "utf8_invalid",
            MarcError::XmlError { .. } => "marcxml_invalid",
            MarcError::JsonError { .. } => "marcjson_invalid",
            MarcError::WriterError { .. } => "record_too_large_for_iso2709",
        }
    }

    /// Canonical docs URL for this error code, pointing at the `#Exxx`
    /// anchor on the error-codes reference page.
    ///
    /// The base URL defaults to the GitHub Pages-hosted docs site. Set the
    /// `MRRC_DOCS_BASE_URL` environment variable to override (e.g., to
    /// redirect to an internal mirror). The variable holds the docs site
    /// root; this method appends `/reference/error-codes/#Exxx`.
    #[must_use]
    pub fn help_url(&self) -> String {
        format!("{}/reference/error-codes/#{}", docs_base_url(), self.code())
    }

    /// Short `PascalCase` name for the variant, used in `detailed()` headers
    /// and as the leading token of the underlying-cause-less default `Display`.
    fn kind_name(&self) -> &'static str {
        match self {
            MarcError::InvalidLeader { .. } => "InvalidLeader",
            MarcError::RecordLengthInvalid { .. } => "RecordLengthInvalid",
            MarcError::BaseAddressInvalid { .. } => "BaseAddressInvalid",
            MarcError::BaseAddressNotFound { .. } => "BaseAddressNotFound",
            MarcError::DirectoryInvalid { .. } => "DirectoryInvalid",
            MarcError::TruncatedRecord { .. } => "TruncatedRecord",
            MarcError::EndOfRecordNotFound { .. } => "EndOfRecordNotFound",
            MarcError::InvalidIndicator { .. } => "InvalidIndicator",
            MarcError::BadSubfieldCode { .. } => "BadSubfieldCode",
            MarcError::InvalidField { .. } => "InvalidField",
            MarcError::EncodingError { .. } => "EncodingError",
            MarcError::FieldNotFound { .. } => "FieldNotFound",
            MarcError::IoError { .. } => "IoError",
            MarcError::XmlError { .. } => "XmlError",
            MarcError::JsonError { .. } => "JsonError",
            MarcError::WriterError { .. } => "WriterError",
        }
    }

    /// Build a "record N, field T" style context summary if those fields are
    /// populated; returns the empty string if neither is available.
    fn context_summary(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(idx) = self.record_index() {
            parts.push(format!("record {idx}"));
        }
        if let Some(tag) = self.field_tag() {
            parts.push(format!("field {tag}"));
        }
        parts.join(", ")
    }

    /// Produce the (label, value) detail lines for `detailed()` output, in
    /// display order. Skips lines whose value is unavailable.
    fn detail_lines(&self) -> Vec<(&'static str, String)> {
        let mut lines: Vec<(&'static str, String)> = Vec::new();
        if let Some(s) = self.source_name() {
            lines.push(("source:", s.to_string()));
        }
        if let Some(cn) = self.record_control_number() {
            lines.push(("001:", cn.to_string()));
        }
        match self {
            MarcError::InvalidIndicator {
                indicator_position,
                found,
                expected,
                ..
            } => {
                if let (Some(pos), Some(exp)) = (indicator_position, expected) {
                    let found_repr = found
                        .as_deref()
                        .map_or_else(|| "?".to_string(), format_found_bytes_python_repr);
                    // Label carries the indicator number + colon; value is
                    // just the found/expected so column alignment in
                    // detailed() matches the Python side byte-for-byte.
                    let label = if *pos == 0 {
                        "indicator 0:"
                    } else {
                        "indicator 1:"
                    };
                    lines.push((label, format!("found {found_repr}, expected {exp}")));
                }
            },
            MarcError::BadSubfieldCode { subfield_code, .. } => {
                lines.push((
                    "subfield:",
                    format!(
                        "invalid code byte 0x{subfield_code:02X} ({:?})",
                        *subfield_code as char
                    ),
                ));
            },
            MarcError::TruncatedRecord {
                expected_length,
                actual_length,
                ..
            } => {
                if let (Some(exp), Some(act)) = (expected_length, actual_length) {
                    lines.push(("length:", format!("expected {exp} bytes, found {act}")));
                }
            },
            _ => {},
        }
        if let Some(off) = self.byte_offset() {
            lines.push(("byte offset:", format!("0x{off:X} ({off}) in stream")));
        }
        if let Some(off) = self.record_byte_offset() {
            lines.push(("record-relative:", format!("byte {off}")));
        }
        if let Some(msg) = self.message_text() {
            lines.push(("message:", msg.to_string()));
        }
        lines
    }

    /// Best-effort one-line `Display` rendering: leads with positional context
    /// (when available) and the problem description; appends the byte offset
    /// in hex/decimal as a visually subordinate trailer.
    fn render_oneline(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut header_parts: Vec<String> = Vec::new();
        if let Some(idx) = self.record_index() {
            header_parts.push(format!("record {idx}"));
        }
        if let Some(cn) = self.record_control_number() {
            header_parts.push(format!("001 '{cn}'"));
        }
        if let Some(tag) = self.field_tag() {
            header_parts.push(format!("field {tag}"));
        }
        if let MarcError::InvalidIndicator {
            indicator_position: Some(pos),
            ..
        } = self
        {
            header_parts.push(format!("ind{pos}"));
        }
        if header_parts.is_empty() {
            // No positional context available — lead with the variant name so
            // the message at least identifies what kind of error it is.
            write!(f, "{}: ", self.kind_name())?;
        } else {
            write!(f, "[{}] ", header_parts.join(" · "))?;
        }
        write!(f, "{}", self.body_text())?;
        if let Some(off) = self.byte_offset() {
            write!(f, "  (byte 0x{off:X} / {off})")?;
        }
        Ok(())
    }

    /// The "what went wrong" body, distinct from the positional header and the
    /// trailing byte offset.
    fn body_text(&self) -> String {
        match self {
            MarcError::InvalidLeader {
                found,
                expected,
                cause,
                ..
            } => match (found, expected, cause) {
                (Some(f), Some(e), _) => format!(
                    "invalid leader: found {} — expected {e}",
                    format_found_bytes_python_repr(f)
                ),
                (_, _, Some(c)) => format!("invalid leader: {c}"),
                _ => "invalid leader".to_string(),
            },
            MarcError::RecordLengthInvalid {
                found, expected, ..
            } => match (found, expected) {
                (Some(f), Some(e)) => format!(
                    "invalid record length {} — expected {e}",
                    format_found_bytes_python_repr(f)
                ),
                _ => "invalid record length".to_string(),
            },
            MarcError::BaseAddressInvalid {
                found, expected, ..
            } => match (found, expected) {
                (Some(f), Some(e)) => format!(
                    "invalid base address {} — expected {e}",
                    format_found_bytes_python_repr(f)
                ),
                _ => "invalid base address".to_string(),
            },
            MarcError::BaseAddressNotFound { .. } => "base address not found".to_string(),
            MarcError::DirectoryInvalid {
                found, expected, ..
            } => match (found, expected) {
                (Some(f), Some(e)) => format!(
                    "invalid directory entry {} — expected {e}",
                    format_found_bytes_python_repr(f)
                ),
                _ => "invalid directory entry".to_string(),
            },
            MarcError::TruncatedRecord {
                expected_length,
                actual_length,
                ..
            } => match (expected_length, actual_length) {
                (Some(e), Some(a)) => format!("truncated record: expected {e} bytes, found {a}"),
                _ => "truncated record".to_string(),
            },
            MarcError::EndOfRecordNotFound { .. } => "end-of-record marker not found".to_string(),
            MarcError::InvalidIndicator {
                found, expected, ..
            } => match (found, expected) {
                (Some(f), Some(e)) => format!(
                    "invalid {} — expected {e}",
                    format_found_bytes_python_repr(f)
                ),
                _ => "invalid indicator".to_string(),
            },
            MarcError::BadSubfieldCode { subfield_code, .. } => {
                format!("invalid subfield code 0x{subfield_code:02X}")
            },
            MarcError::InvalidField { message, .. } => format!("invalid field: {message}"),
            MarcError::EncodingError { message, .. } => format!("encoding error: {message}"),
            MarcError::FieldNotFound { field_tag, .. } => {
                format!("field {field_tag} not found")
            },
            MarcError::IoError { cause, .. } => format!("I/O error: {cause}"),
            MarcError::XmlError { cause, .. } => format!("XML parse error: {cause}"),
            MarcError::JsonError { cause, .. } => format!("JSON parse error: {cause}"),
            MarcError::WriterError { message, .. } => format!("writer error: {message}"),
        }
    }

    fn record_index(&self) -> Option<usize> {
        match self {
            MarcError::InvalidLeader { record_index, .. }
            | MarcError::RecordLengthInvalid { record_index, .. }
            | MarcError::BaseAddressInvalid { record_index, .. }
            | MarcError::BaseAddressNotFound { record_index, .. }
            | MarcError::DirectoryInvalid { record_index, .. }
            | MarcError::TruncatedRecord { record_index, .. }
            | MarcError::EndOfRecordNotFound { record_index, .. }
            | MarcError::InvalidIndicator { record_index, .. }
            | MarcError::BadSubfieldCode { record_index, .. }
            | MarcError::InvalidField { record_index, .. }
            | MarcError::EncodingError { record_index, .. }
            | MarcError::FieldNotFound { record_index, .. }
            | MarcError::IoError { record_index, .. }
            | MarcError::XmlError { record_index, .. }
            | MarcError::JsonError { record_index, .. }
            | MarcError::WriterError { record_index, .. } => *record_index,
        }
    }

    fn record_control_number(&self) -> Option<&str> {
        match self {
            MarcError::BaseAddressInvalid {
                record_control_number,
                ..
            }
            | MarcError::BaseAddressNotFound {
                record_control_number,
                ..
            }
            | MarcError::DirectoryInvalid {
                record_control_number,
                ..
            }
            | MarcError::TruncatedRecord {
                record_control_number,
                ..
            }
            | MarcError::EndOfRecordNotFound {
                record_control_number,
                ..
            }
            | MarcError::InvalidIndicator {
                record_control_number,
                ..
            }
            | MarcError::BadSubfieldCode {
                record_control_number,
                ..
            }
            | MarcError::InvalidField {
                record_control_number,
                ..
            }
            | MarcError::EncodingError {
                record_control_number,
                ..
            }
            | MarcError::FieldNotFound {
                record_control_number,
                ..
            }
            | MarcError::WriterError {
                record_control_number,
                ..
            } => record_control_number.as_deref(),
            _ => None,
        }
    }

    fn field_tag(&self) -> Option<&str> {
        match self {
            MarcError::DirectoryInvalid { field_tag, .. }
            | MarcError::InvalidIndicator { field_tag, .. }
            | MarcError::BadSubfieldCode { field_tag, .. }
            | MarcError::InvalidField { field_tag, .. }
            | MarcError::EncodingError { field_tag, .. } => field_tag.as_deref(),
            MarcError::FieldNotFound { field_tag, .. } => Some(field_tag.as_str()),
            _ => None,
        }
    }

    fn byte_offset(&self) -> Option<usize> {
        match self {
            MarcError::InvalidLeader { byte_offset, .. }
            | MarcError::RecordLengthInvalid { byte_offset, .. }
            | MarcError::BaseAddressInvalid { byte_offset, .. }
            | MarcError::BaseAddressNotFound { byte_offset, .. }
            | MarcError::DirectoryInvalid { byte_offset, .. }
            | MarcError::TruncatedRecord { byte_offset, .. }
            | MarcError::EndOfRecordNotFound { byte_offset, .. }
            | MarcError::InvalidIndicator { byte_offset, .. }
            | MarcError::BadSubfieldCode { byte_offset, .. }
            | MarcError::InvalidField { byte_offset, .. }
            | MarcError::EncodingError { byte_offset, .. }
            | MarcError::IoError { byte_offset, .. }
            | MarcError::XmlError { byte_offset, .. }
            | MarcError::JsonError { byte_offset, .. } => *byte_offset,
            _ => None,
        }
    }

    fn record_byte_offset(&self) -> Option<usize> {
        match self {
            MarcError::InvalidLeader {
                record_byte_offset, ..
            }
            | MarcError::DirectoryInvalid {
                record_byte_offset, ..
            }
            | MarcError::TruncatedRecord {
                record_byte_offset, ..
            }
            | MarcError::EndOfRecordNotFound {
                record_byte_offset, ..
            }
            | MarcError::InvalidIndicator {
                record_byte_offset, ..
            }
            | MarcError::BadSubfieldCode {
                record_byte_offset, ..
            }
            | MarcError::InvalidField {
                record_byte_offset, ..
            } => *record_byte_offset,
            _ => None,
        }
    }

    fn source_name(&self) -> Option<&str> {
        match self {
            MarcError::InvalidLeader { source_name, .. }
            | MarcError::RecordLengthInvalid { source_name, .. }
            | MarcError::BaseAddressInvalid { source_name, .. }
            | MarcError::BaseAddressNotFound { source_name, .. }
            | MarcError::DirectoryInvalid { source_name, .. }
            | MarcError::TruncatedRecord { source_name, .. }
            | MarcError::EndOfRecordNotFound { source_name, .. }
            | MarcError::InvalidIndicator { source_name, .. }
            | MarcError::BadSubfieldCode { source_name, .. }
            | MarcError::InvalidField { source_name, .. }
            | MarcError::EncodingError { source_name, .. }
            | MarcError::IoError { source_name, .. }
            | MarcError::XmlError { source_name, .. }
            | MarcError::JsonError { source_name, .. } => source_name.as_deref(),
            _ => None,
        }
    }

    fn message_text(&self) -> Option<&str> {
        match self {
            MarcError::InvalidField { message, .. }
            | MarcError::EncodingError { message, .. }
            | MarcError::WriterError { message, .. } => Some(message.as_str()),
            _ => None,
        }
    }
}

impl MarcError {
    /// Construct an [`MarcError::InvalidField`] with only a message — used at
    /// call sites that have a textual error description but no positional
    /// metadata available. Subsequent enrichment work attaches positional
    /// fields where they can be derived from a `ParseContext`.
    #[must_use]
    pub(crate) fn invalid_field_msg(msg: impl Into<String>) -> Self {
        MarcError::InvalidField {
            record_index: None,
            byte_offset: None,
            record_byte_offset: None,
            source_name: None,
            record_control_number: None,
            field_tag: None,
            message: msg.into(),
        }
    }

    /// Construct an [`MarcError::EncodingError`] with only a message.
    #[must_use]
    pub(crate) fn encoding_msg(msg: impl Into<String>) -> Self {
        MarcError::EncodingError {
            record_index: None,
            byte_offset: None,
            source_name: None,
            record_control_number: None,
            field_tag: None,
            message: msg.into(),
        }
    }

    /// Construct an [`MarcError::InvalidLeader`] from a textual cause.
    #[must_use]
    pub(crate) fn leader_msg(cause: impl Into<String>) -> Self {
        MarcError::InvalidLeader {
            record_index: None,
            byte_offset: None,
            record_byte_offset: None,
            source_name: None,
            found: None,
            expected: None,
            cause: Some(cause.into()),
        }
    }
}

impl fmt::Display for MarcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.detailed())
        } else {
            self.render_oneline(f)
        }
    }
}

/// Format a byte slice as Python-style `b'...'` repr, escaping non-printable
/// bytes. Mirrors what users will see on the Python side via `repr(err.found)`.
fn format_found_bytes_python_repr(bytes: &[u8]) -> String {
    let mut out = String::from("b'");
    for &b in bytes {
        match b {
            b'\\' => out.push_str(r"\\"),
            b'\'' => out.push_str(r"\'"),
            b'\n' => out.push_str(r"\n"),
            b'\r' => out.push_str(r"\r"),
            b'\t' => out.push_str(r"\t"),
            0x20..=0x7E => out.push(b as char),
            _ => {
                use std::fmt::Write;
                let _ = write!(out, "\\x{b:02x}");
            },
        }
    }
    out.push('\'');
    out
}

/// Default base URL for the docs site. Used by [`MarcError::help_url`] when
/// the `MRRC_DOCS_BASE_URL` environment variable is not set.
pub const DEFAULT_DOCS_BASE_URL: &str = "https://dchud.github.io/mrrc";

fn docs_base_url() -> String {
    std::env::var("MRRC_DOCS_BASE_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .map_or_else(
            || DEFAULT_DOCS_BASE_URL.to_string(),
            |s| s.trim_end_matches('/').to_string(),
        )
}

/// Convenience type alias for [`std::result::Result`] with [`MarcError`].
pub type Result<T> = std::result::Result<T, MarcError>;

// Backwards-compatible conversion so existing `?` propagation of `io::Error`
// continues to work without surrounding context.
impl From<std::io::Error> for MarcError {
    fn from(cause: std::io::Error) -> Self {
        MarcError::IoError {
            cause,
            record_index: None,
            byte_offset: None,
            source_name: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_bytes_short_input_passes_through() {
        let (out, truncated) = truncate_bytes(b"hello");
        assert_eq!(out, b"hello");
        assert!(!truncated);
    }

    #[test]
    fn truncate_bytes_long_input_capped() {
        let input = vec![b'x'; 100];
        let (out, truncated) = truncate_bytes(&input);
        assert_eq!(out.len(), FOUND_BYTES_CAP);
        assert!(truncated);
    }

    #[test]
    fn display_invalid_indicator_produces_actionable_oneliner() {
        let err = MarcError::InvalidIndicator {
            record_index: Some(847),
            byte_offset: Some(7217),
            record_byte_offset: Some(42),
            source_name: Some("harvest.mrc".into()),
            record_control_number: Some("ocm01234567".into()),
            field_tag: Some("245".into()),
            indicator_position: Some(1),
            found: Some(b":".to_vec()),
            expected: Some("digit or space".into()),
        };
        let s = err.to_string();
        assert!(s.starts_with("[record 847"), "got: {s}");
        assert!(s.contains("001 'ocm01234567'"), "got: {s}");
        assert!(s.contains("field 245"), "got: {s}");
        assert!(s.contains("ind1"), "got: {s}");
        assert!(s.contains("(byte 0x1C31 / 7217)"), "got: {s}");
    }

    #[test]
    fn detailed_invalid_indicator_multiline() {
        let err = MarcError::InvalidIndicator {
            record_index: Some(847),
            byte_offset: Some(7217),
            record_byte_offset: Some(42),
            source_name: Some("harvest.mrc".into()),
            record_control_number: Some("ocm01234567".into()),
            field_tag: Some("245".into()),
            indicator_position: Some(1),
            found: Some(b":".to_vec()),
            expected: Some("digit or space".into()),
        };
        let d = err.detailed();
        assert!(
            d.starts_with("InvalidIndicator at record 847, field 245"),
            "got: {d}"
        );
        assert!(d.contains("source:"), "got: {d}");
        assert!(d.contains("harvest.mrc"), "got: {d}");
        assert!(d.contains("001:"), "got: {d}");
        assert!(d.contains("indicator"), "got: {d}");
        assert!(d.contains("byte offset:"), "got: {d}");
        assert!(d.contains("0x1C31 (7217)"), "got: {d}");
        assert!(d.contains("record-relative:"), "got: {d}");
    }

    #[test]
    fn io_error_source_chain_walks() {
        let io = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "boom");
        let err = MarcError::IoError {
            cause: io,
            record_index: Some(1),
            byte_offset: Some(0),
            source_name: None,
        };
        let chain = std::error::Error::source(&err);
        assert!(chain.is_some());
        assert!(chain.unwrap().to_string().contains("boom"));
    }

    #[test]
    fn from_io_error_blanket_conversion() {
        fn returns_io() -> std::io::Result<()> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "nope"))
        }
        fn wraps() -> Result<()> {
            returns_io()?;
            Ok(())
        }
        let err = wraps().unwrap_err();
        assert!(matches!(err, MarcError::IoError { .. }));
    }

    // -- Snapshot tests for the externally-visible error format ----------
    //
    // These pin the on-the-wire wording of Display (one-liner) and
    // detailed() (multi-line) outputs across representative variants.
    // Run `cargo insta review` to inspect/accept changes when these
    // snapshots drift.

    fn invalid_indicator_full() -> MarcError {
        MarcError::InvalidIndicator {
            record_index: Some(847),
            byte_offset: Some(7217),
            record_byte_offset: Some(42),
            source_name: Some("harvest.mrc".into()),
            record_control_number: Some("ocm01234567".into()),
            field_tag: Some("245".into()),
            indicator_position: Some(1),
            found: Some(b":".to_vec()),
            expected: Some("digit or space".into()),
        }
    }

    #[test]
    fn snapshot_display_invalid_indicator_full_context() {
        insta::assert_snapshot!(invalid_indicator_full().to_string());
    }

    #[test]
    fn snapshot_detailed_invalid_indicator_full_context() {
        insta::assert_snapshot!(invalid_indicator_full().detailed());
    }

    #[test]
    fn snapshot_display_no_context_falls_back_to_kind_name() {
        let err = MarcError::BaseAddressNotFound {
            record_index: None,
            byte_offset: None,
            source_name: None,
            record_control_number: None,
        };
        insta::assert_snapshot!(err.to_string());
    }

    #[test]
    fn snapshot_display_directory_invalid_with_truncated_found() {
        let big_input: Vec<u8> = (b'a'..=b'z').cycle().take(60).collect();
        let (truncated, _was_truncated) = truncate_bytes(&big_input);
        let err = MarcError::DirectoryInvalid {
            record_index: Some(3),
            byte_offset: Some(0x100),
            record_byte_offset: Some(24),
            source_name: Some("collection.mrc".into()),
            record_control_number: Some("oc00000003".into()),
            field_tag: Some("245".into()),
            found: Some(truncated),
            expected: Some("12-byte numeric directory entry".into()),
        };
        insta::assert_snapshot!(err.to_string());
    }

    #[test]
    fn snapshot_detailed_truncated_record() {
        let err = MarcError::TruncatedRecord {
            record_index: Some(12),
            byte_offset: Some(0x4000),
            record_byte_offset: Some(0x80),
            source_name: Some("partial.mrc".into()),
            record_control_number: Some("oc00000012".into()),
            expected_length: Some(1024),
            actual_length: Some(640),
        };
        insta::assert_snapshot!(err.detailed());
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn code_slug_help_url_for_every_variant() {
        // One representative instance per variant. If a variant is added
        // and the test panics on missing arms, add the variant + its
        // expected (code, slug) pair to the table — and don't forget to
        // mirror it in the Python _CODE_TABLE in tests/python/test_errors.py.
        let cases: Vec<(MarcError, &'static str, &'static str)> = vec![
            (
                MarcError::RecordLengthInvalid {
                    record_index: None,
                    byte_offset: None,
                    source_name: None,
                    found: None,
                    expected: None,
                },
                "E001",
                "record_length_invalid",
            ),
            (
                MarcError::InvalidLeader {
                    record_index: None,
                    byte_offset: None,
                    record_byte_offset: None,
                    source_name: None,
                    found: None,
                    expected: None,
                    cause: None,
                },
                "E002",
                "leader_invalid",
            ),
            (
                MarcError::BaseAddressInvalid {
                    record_index: None,
                    byte_offset: None,
                    source_name: None,
                    record_control_number: None,
                    found: None,
                    expected: None,
                },
                "E003",
                "base_address_invalid",
            ),
            (
                MarcError::BaseAddressNotFound {
                    record_index: None,
                    byte_offset: None,
                    source_name: None,
                    record_control_number: None,
                },
                "E004",
                "base_address_not_found",
            ),
            (
                MarcError::TruncatedRecord {
                    record_index: None,
                    byte_offset: None,
                    record_byte_offset: None,
                    source_name: None,
                    record_control_number: None,
                    expected_length: None,
                    actual_length: None,
                },
                "E005",
                "truncated_record",
            ),
            (
                MarcError::EndOfRecordNotFound {
                    record_index: None,
                    byte_offset: None,
                    record_byte_offset: None,
                    source_name: None,
                    record_control_number: None,
                },
                "E006",
                "end_of_record_not_found",
            ),
            (
                MarcError::IoError {
                    cause: std::io::Error::other("test"),
                    record_index: None,
                    byte_offset: None,
                    source_name: None,
                },
                "E007",
                "io_error",
            ),
            (
                MarcError::DirectoryInvalid {
                    record_index: None,
                    byte_offset: None,
                    record_byte_offset: None,
                    source_name: None,
                    record_control_number: None,
                    field_tag: None,
                    found: None,
                    expected: None,
                },
                "E101",
                "directory_invalid",
            ),
            (
                MarcError::FieldNotFound {
                    record_index: None,
                    record_control_number: None,
                    field_tag: "245".into(),
                },
                "E105",
                "field_not_found",
            ),
            (
                MarcError::InvalidField {
                    record_index: None,
                    byte_offset: None,
                    record_byte_offset: None,
                    source_name: None,
                    record_control_number: None,
                    field_tag: None,
                    message: "test".into(),
                },
                "E106",
                "invalid_field",
            ),
            (
                MarcError::InvalidIndicator {
                    record_index: None,
                    byte_offset: None,
                    record_byte_offset: None,
                    source_name: None,
                    record_control_number: None,
                    field_tag: None,
                    indicator_position: None,
                    found: None,
                    expected: None,
                },
                "E201",
                "invalid_indicator",
            ),
            (
                MarcError::BadSubfieldCode {
                    record_index: None,
                    byte_offset: None,
                    record_byte_offset: None,
                    source_name: None,
                    record_control_number: None,
                    field_tag: None,
                    subfield_code: 0,
                },
                "E202",
                "bad_subfield_code",
            ),
            (
                MarcError::EncodingError {
                    record_index: None,
                    byte_offset: None,
                    source_name: None,
                    record_control_number: None,
                    field_tag: None,
                    message: "test".into(),
                },
                "E301",
                "utf8_invalid",
            ),
            (
                MarcError::XmlError {
                    cause: Box::new(std::io::Error::other("test")),
                    record_index: None,
                    byte_offset: None,
                    source_name: None,
                },
                "E401",
                "marcxml_invalid",
            ),
            (
                MarcError::JsonError {
                    cause: serde_json::from_str::<serde_json::Value>("not json").unwrap_err(),
                    record_index: None,
                    byte_offset: None,
                    source_name: None,
                },
                "E402",
                "marcjson_invalid",
            ),
            (
                MarcError::WriterError {
                    record_index: None,
                    record_control_number: None,
                    message: "test".into(),
                },
                "E404",
                "record_too_large_for_iso2709",
            ),
        ];
        let mut codes = std::collections::HashSet::new();
        let mut slugs = std::collections::HashSet::new();
        for (err, expected_code, expected_slug) in cases {
            assert_eq!(err.code(), expected_code, "wrong code for {err:?}");
            assert_eq!(err.slug(), expected_slug, "wrong slug for {err:?}");
            assert_eq!(
                err.help_url(),
                format!("{DEFAULT_DOCS_BASE_URL}/reference/error-codes/#{expected_code}"),
            );
            assert!(
                codes.insert(expected_code),
                "duplicate code {expected_code}"
            );
            assert!(
                slugs.insert(expected_slug),
                "duplicate slug {expected_slug}"
            );
        }
    }

    #[test]
    fn snapshot_display_writer_error() {
        let err = MarcError::WriterError {
            record_index: Some(99),
            record_control_number: Some("oc00000099".into()),
            message: "Record length exceeds 4GB limit (5000000000 bytes)".into(),
        };
        insta::assert_snapshot!(err.to_string());
    }
}
