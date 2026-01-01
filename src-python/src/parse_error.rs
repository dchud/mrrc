// ParseError enum for GIL-free error handling in buffered reader
//
// This error type is designed to be used within GIL-released sections.
// It contains NO Py<T> references, making it safe to construct and use
// while allow_threads() is active.

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::PyErr;
use std::fmt;

/// Error type for record parsing operations.
///
/// This enum is safe to use in GIL-released code sections (within allow_threads).
/// All variants contain only owned data, no PyO3 references.
#[derive(Debug, Clone)]
pub enum ParseError {
    /// The record structure is invalid or malformed
    InvalidRecord(String),
    /// ISO 2709 record boundary detection failed
    RecordBoundaryError(String),
    /// I/O error reading from file
    IoError(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidRecord(msg) => write!(f, "Invalid record: {}", msg),
            ParseError::RecordBoundaryError(msg) => write!(f, "Record boundary error: {}", msg),
            ParseError::IoError(msg) => write!(f, "IO error: {}", msg),
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
    /// Convert ParseError to PyErr (only call when GIL is held)
    pub fn to_py_err(&self) -> PyErr {
        match self {
            ParseError::InvalidRecord(msg) => {
                PyValueError::new_err(format!("Invalid record: {}", msg))
            },
            ParseError::RecordBoundaryError(msg) => {
                PyValueError::new_err(format!("Record boundary error: {}", msg))
            },
            ParseError::IoError(msg) => PyIOError::new_err(msg.clone()),
        }
    }
}
