// ParseError — error type used by parsing primitives that run with the GIL
// released (boundary scanner, buffered reader). Variants are kept simple so
// they can be constructed and propagated without holding a Py<T>; the
// conversion to a Python exception goes through the main MarcError → Python
// mapping so the same typed-exception classes reach all callers.

use mrrc::MarcError;
use pyo3::PyErr;
use std::fmt;

/// Error type for record parsing operations.
///
/// Safe to use in GIL-released code (within `Python::detach`). All variants
/// hold owned data and no PyO3 references. Conversion to a Python exception
/// (via [`ParseError::to_py_err`]) requires the GIL and routes through the
/// main `MarcError` → typed-exception mapping.
#[derive(Debug, Clone)]
pub enum ParseError {
    /// The record structure is invalid or malformed.
    InvalidRecord(String),
    /// ISO 2709 record boundary detection failed.
    RecordBoundaryError(String),
    /// I/O error reading from file or Python file object.
    IoError(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidRecord(msg) => write!(f, "Invalid record: {msg}"),
            ParseError::RecordBoundaryError(msg) => write!(f, "Record boundary error: {msg}"),
            ParseError::IoError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err.to_string())
    }
}

impl ParseError {
    /// Convert to a Python exception. Must be called with the GIL held.
    ///
    /// Each variant maps to the equivalent `MarcError` and is then routed
    /// through the main typed-exception construction so callers see the
    /// same Python class hierarchy as the synchronous reader path.
    pub fn to_py_err(&self) -> PyErr {
        let marc_err = match self {
            ParseError::InvalidRecord(msg) => MarcError::InvalidField {
                record_index: None,
                byte_offset: None,
                record_byte_offset: None,
                source_name: None,
                record_control_number: None,
                field_tag: None,
                message: msg.clone(),
            },
            ParseError::RecordBoundaryError(msg) => MarcError::InvalidField {
                record_index: None,
                byte_offset: None,
                record_byte_offset: None,
                source_name: None,
                record_control_number: None,
                field_tag: None,
                message: format!("record boundary error: {msg}"),
            },
            ParseError::IoError(msg) => MarcError::IoError {
                cause: std::io::Error::other(msg.clone()),
                record_index: None,
                byte_offset: None,
                source_name: None,
            },
        };
        crate::error::marc_error_to_py_err(marc_err)
    }
}
