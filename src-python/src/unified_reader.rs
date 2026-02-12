//! Unified reader supporting both Rust file I/O and Python file-like objects
//!
//! Enables reading MARC records from:
//! - File paths (str or pathlib.Path) via pure Rust I/O (no GIL)
//! - In-memory bytes (bytes, bytearray) via std::io::Cursor
//! - Python file objects (.read() method) with GIL management

use crate::backend::ReaderBackend;
use crate::parse_error::ParseError;
use pyo3::prelude::*;

/// Unified interface for reading MARC record bytes from any supported source
///
/// Abstracts over Python file objects and Rust file I/O.
/// Allows `PyMARCReader` to accept file paths without Python GIL overhead.
#[derive(Debug)]
pub enum UnifiedReader {
    /// Rust-backed reader (File or Cursor) - no GIL needed
    Backend(ReaderBackend),

    /// Python file-like object - requires GIL for each read
    Python(Py<PyAny>),
}

impl UnifiedReader {
    /// Create UnifiedReader from a Python object
    ///
    /// Type detection algorithm:
    /// 1. str → RustFile (no GIL needed)
    /// 2. pathlib.Path → RustFile (no GIL needed)
    /// 3. bytes/bytearray → CursorBackend (no GIL needed)
    /// 4. Object with .read() → PythonFile (GIL required)
    /// 5. Unknown → TypeError
    pub fn from_python(source: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py = source.py();
        match ReaderBackend::from_python(source, py) {
            Ok(backend) => Ok(UnifiedReader::Backend(backend)),
            Err(e) => {
                // ReaderBackend::from_python failed
                // Check if it's a file-system error (FileNotFoundError, PermissionError, IOError)
                // vs a type-not-supported error (TypeError)
                if e.is_instance_of::<pyo3::exceptions::PyFileNotFoundError>(py)
                    || e.is_instance_of::<pyo3::exceptions::PyPermissionError>(py)
                    || e.is_instance_of::<pyo3::exceptions::PyIOError>(py)
                {
                    // File system error - propagate it
                    return Err(e);
                }

                // Type-related error - try Python file wrapper
                match source.getattr("read") {
                    Ok(method) if method.is_callable() => {
                        let py_file = source.clone().unbind();
                        Ok(UnifiedReader::Python(py_file))
                    },
                    _ => {
                        // Still no .read() method found - not a supported type
                        let type_name = source
                            .get_type()
                            .name()
                            .map(|n| n.to_string())
                            .unwrap_or_else(|_| "unknown".to_string());
                        Err(pyo3::exceptions::PyTypeError::new_err(format!(
                            "Unsupported input type: {}. Supported types: str (file path), pathlib.Path, \
                             bytes, bytearray, or file-like object (with .read() method). \
                             Examples: 'records.mrc', Path('records.mrc'), b'binary data', \
                             open('records.mrc', 'rb'), io.BytesIO(data), socket.socket(...)",
                            type_name
                        )))
                    },
                }
            },
        }
    }

    /// Return the backend type as a string for diagnostics
    pub fn backend_type(&self) -> &str {
        match self {
            UnifiedReader::Backend(backend) => backend.backend_type(),
            UnifiedReader::Python(_) => "python_file",
        }
    }

    /// Read the next MARC record bytes from this unified reader
    ///
    /// For Backend variants (RustFile, CursorBackend):
    /// - No GIL required; reads directly from Rust I/O
    ///
    /// For Python variant:
    /// - GIL must be held by caller
    /// - Calls Python .read() method
    ///
    /// # Arguments
    /// * `py` - Python interpreter (needed for PythonFile backend)
    ///
    /// # Returns
    /// - `Ok(Some(bytes))` - Successfully read record bytes (owned)
    /// - `Ok(None)` - EOF reached
    /// - `Err(ParseError)` - Read or parsing error
    pub fn read_next_bytes(&mut self, py: Python) -> Result<Option<Vec<u8>>, ParseError> {
        match self {
            UnifiedReader::Backend(backend) => {
                // Backend handles its own GIL requirements (none for RustFile/Cursor)
                backend.read_next_bytes(py)
            },
            UnifiedReader::Python(py_file) => {
                // Python file-like object - read via .read() method
                Self::read_record_bytes_from_python(py, py_file)
            },
        }
    }

    /// Helper: Read record bytes from Python file-like object
    fn read_record_bytes_from_python(
        py: Python,
        py_file: &Py<PyAny>,
    ) -> Result<Option<Vec<u8>>, ParseError> {
        let file_obj = py_file.bind(py);

        // Read first 5 bytes (record length header)
        let read_method = file_obj
            .getattr("read")
            .map_err(|e| ParseError::IoError(format!("Object missing .read() method: {}", e)))?;

        let length_result = read_method.call1((5usize,)).map_err(|e| {
            ParseError::IoError(format!(
                "Failed to read length header from Python file: {}",
                e
            ))
        })?;

        let length_bytes: Vec<u8> = length_result.extract().map_err(|_| {
            ParseError::InvalidRecord("Record length header must be bytes".to_string())
        })?;

        if length_bytes.is_empty() {
            return Ok(None); // EOF
        }

        if length_bytes.len() < 5 {
            return Err(ParseError::RecordBoundaryError(format!(
                "Incomplete record length header: got {} bytes, expected 5",
                length_bytes.len()
            )));
        }

        // Parse record length
        let record_length_str = String::from_utf8_lossy(&length_bytes[0..5]);
        let record_length: usize = record_length_str.trim().parse().map_err(|_| {
            ParseError::InvalidRecord(format!(
                "Invalid record length in header: '{}'",
                record_length_str
            ))
        })?;

        if record_length < 24 {
            return Err(ParseError::InvalidRecord(format!(
                "Record length {} is too small (minimum 24)",
                record_length
            )));
        }

        // Read remaining bytes (record_length - 5 for already-read header)
        let remaining = record_length - 5;
        let record_data_result = read_method.call1((remaining,)).map_err(|e| {
            ParseError::IoError(format!(
                "Failed to read record data from Python file: {}",
                e
            ))
        })?;

        let record_data: Vec<u8> = record_data_result
            .extract()
            .map_err(|_| ParseError::InvalidRecord("Record data must be bytes".to_string()))?;

        if record_data.len() != remaining {
            return Err(ParseError::InvalidRecord(format!(
                "Truncated record: expected {} bytes, got {}",
                remaining,
                record_data.len()
            )));
        }

        // Assemble complete record
        let mut complete_record = Vec::with_capacity(record_length);
        complete_record.extend_from_slice(&length_bytes);
        complete_record.extend_from_slice(&record_data);

        Ok(Some(complete_record))
    }
}

// NOTE: Unit tests for UnifiedReader require Python object creation which varies by PyO3 version.
// Python integration tests in tests/python/test_*.py verify the full functionality.
// These backend tests are tested implicitly through the full test suite.
