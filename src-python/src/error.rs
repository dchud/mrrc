// Error handling and mapping from Rust to Python exceptions

use mrrc::MarcError;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::PyErr;

/// Map Rust MarcError to Python exceptions
pub fn marc_error_to_py_err(err: MarcError) -> PyErr {
    match err {
        MarcError::InvalidRecord(msg) => PyValueError::new_err(format!("Invalid record: {}", msg)),
        MarcError::InvalidLeader(msg) => PyValueError::new_err(format!("Invalid leader: {}", msg)),
        MarcError::InvalidField(msg) => PyValueError::new_err(format!("Invalid field: {}", msg)),
        MarcError::EncodingError(msg) => PyValueError::new_err(format!("Encoding error: {}", msg)),
        MarcError::ParseError(msg) => PyValueError::new_err(format!("Parse error: {}", msg)),
        MarcError::TruncatedRecord(msg) => {
            PyValueError::new_err(format!("Truncated record: {}", msg))
        },
        MarcError::IoError(io_err) => PyIOError::new_err(io_err.to_string()),
    }
}
