// ParseError — error type used by parsing primitives that run with the GIL
// released (boundary scanner, buffered reader). Holds owned data only (no
// Py<T> references), making construction safe inside `Python::detach`.
//
// ParseError carries an optional positional context (record index, byte
// offset, source filename) populated by callers that have that information
// at construction time. Conversion to a Python exception via [`to_py_err`]
// routes through the main `MarcError` → typed-exception mapping so callers
// see the same Python class hierarchy as the synchronous reader path,
// including positional attributes when populated.

use mrrc::MarcError;
use pyo3::PyErr;
use std::fmt;

/// Error type for record parsing operations.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub context: ParseErrorContext,
}

/// Discriminator for [`ParseError`] kinds. Exposed publicly so callers can
/// match on the specific failure mode.
#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    /// The record structure is invalid or malformed.
    InvalidRecord(String),
    /// ISO 2709 record boundary detection failed.
    RecordBoundaryError(String),
    /// The record was truncated mid-stream. Carries the expected and
    /// actual byte counts when known.
    TruncatedRecord {
        expected_length: Option<usize>,
        actual_length: Option<usize>,
    },
    /// I/O error reading from file or Python file object.
    IoError(String),
}

/// Optional positional context attached to a [`ParseError`]. Populated by
/// call sites that have the information at construction time; left
/// at default (all `None`) when not.
#[derive(Debug, Clone, Default)]
pub struct ParseErrorContext {
    /// 1-based record index in the input stream.
    pub record_index: Option<usize>,
    /// Absolute byte offset within the input stream.
    pub byte_offset: Option<usize>,
    /// Source filename or stream identifier.
    pub source_name: Option<String>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ParseErrorKind::InvalidRecord(msg) => write!(f, "Invalid record: {msg}"),
            ParseErrorKind::RecordBoundaryError(msg) => write!(f, "Record boundary error: {msg}"),
            ParseErrorKind::TruncatedRecord {
                expected_length,
                actual_length,
            } => match (expected_length, actual_length) {
                (Some(e), Some(a)) => write!(f, "Truncated record: expected {e} bytes, got {a}"),
                _ => write!(f, "Truncated record"),
            },
            ParseErrorKind::IoError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::io_error(err.to_string())
    }
}

// The builder methods below are infrastructure for callers that have
// positional context to attach. Few existing call sites in the boundary
// scanner / buffered reader paths track this information today; populating
// it requires per-reader records_read counters that don't currently exist.
// The methods are deliberately retained so context can be added incrementally
// at sites that gain tracking without further ParseError surgery.
#[allow(dead_code)]
impl ParseError {
    /// Construct an `InvalidRecord` error with the given message and no
    /// positional context. Use builder methods (`with_record_index`,
    /// `with_byte_offset`, `with_source`) to attach context.
    pub fn invalid_record(msg: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::InvalidRecord(msg.into()),
            context: ParseErrorContext::default(),
        }
    }

    /// Construct a `RecordBoundaryError` with the given message and no
    /// positional context.
    pub fn record_boundary_error(msg: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::RecordBoundaryError(msg.into()),
            context: ParseErrorContext::default(),
        }
    }

    /// Construct a `TruncatedRecord` error with the expected/actual byte
    /// counts. The boundary-scanner / buffered-reader paths use this when
    /// they detect a record shorter than its declared length so the typed
    /// Python exception (`mrrc.TruncatedRecord`) surfaces with byte-count
    /// metadata rather than a generic `InvalidField`.
    pub fn truncated_record(expected_length: usize, actual_length: usize) -> Self {
        Self {
            kind: ParseErrorKind::TruncatedRecord {
                expected_length: Some(expected_length),
                actual_length: Some(actual_length),
            },
            context: ParseErrorContext::default(),
        }
    }

    /// Construct an `IoError` with the given stringified cause and no
    /// positional context.
    pub fn io_error(msg: impl Into<String>) -> Self {
        Self {
            kind: ParseErrorKind::IoError(msg.into()),
            context: ParseErrorContext::default(),
        }
    }

    /// Attach a 1-based record index to this error.
    #[must_use]
    pub fn with_record_index(mut self, n: usize) -> Self {
        self.context.record_index = Some(n);
        self
    }

    /// Attach an absolute byte offset to this error.
    #[must_use]
    pub fn with_byte_offset(mut self, n: usize) -> Self {
        self.context.byte_offset = Some(n);
        self
    }

    /// Attach a source filename or stream identifier to this error.
    #[must_use]
    pub fn with_source(mut self, name: impl Into<String>) -> Self {
        self.context.source_name = Some(name.into());
        self
    }

    /// Convert to a Python exception. Must be called with the GIL held.
    ///
    /// Each kind maps to the equivalent `MarcError`, with any populated
    /// positional context attached, and is then routed through the main
    /// typed-exception construction so callers see the same Python class
    /// hierarchy as the synchronous reader path.
    pub fn to_py_err(&self) -> PyErr {
        let marc_err = match &self.kind {
            ParseErrorKind::InvalidRecord(msg) => MarcError::InvalidField {
                record_index: self.context.record_index,
                byte_offset: self.context.byte_offset,
                record_byte_offset: None,
                source_name: self.context.source_name.clone(),
                record_control_number: None,
                field_tag: None,
                message: msg.clone(),
            },
            ParseErrorKind::RecordBoundaryError(msg) => MarcError::InvalidField {
                record_index: self.context.record_index,
                byte_offset: self.context.byte_offset,
                record_byte_offset: None,
                source_name: self.context.source_name.clone(),
                record_control_number: None,
                field_tag: None,
                message: format!("record boundary error: {msg}"),
            },
            ParseErrorKind::TruncatedRecord {
                expected_length,
                actual_length,
            } => MarcError::TruncatedRecord {
                record_index: self.context.record_index,
                byte_offset: self.context.byte_offset,
                record_byte_offset: None,
                source_name: self.context.source_name.clone(),
                record_control_number: None,
                expected_length: *expected_length,
                actual_length: *actual_length,
            },
            ParseErrorKind::IoError(msg) => MarcError::IoError {
                cause: std::io::Error::other(msg.clone()),
                record_index: self.context.record_index,
                byte_offset: self.context.byte_offset,
                source_name: self.context.source_name.clone(),
            },
        };
        crate::error::marc_error_to_py_err(marc_err)
    }
}
