// Error handling and mapping from Rust to Python exceptions.
//
// This is a temporary fall-through mapping that produces bare PyValueError /
// PyIOError exceptions with formatted strings. Python-side typed exception
// construction with positional kwargs lands in a subsequent change once the
// Python exception class hierarchy is updated to accept the new attributes.

use mrrc::MarcError;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::PyErr;

/// Map a Rust [`MarcError`] to a Python exception.
pub fn marc_error_to_py_err(err: MarcError) -> PyErr {
    match err {
        MarcError::IoError { cause, .. } => PyIOError::new_err(cause.to_string()),
        MarcError::XmlError { cause, .. } => {
            PyValueError::new_err(format!("XML parse error: {}", cause))
        },
        MarcError::JsonError { cause, .. } => {
            PyValueError::new_err(format!("JSON parse error: {}", cause))
        },
        other => PyValueError::new_err(other.to_string()),
    }
}
