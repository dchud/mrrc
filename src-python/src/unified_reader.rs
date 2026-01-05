//! Unified reader supporting both Rust file I/O and Python file-like objects
//!
//! This module implements Phase H.2: RustFile Backend Integration
//! Enables reading MARC records from:
//! - File paths (str or pathlib.Path) via pure Rust I/O (no GIL)
//! - In-memory bytes (bytes, bytearray) via std::io::Cursor
//! - Python file objects (.read() method) with GIL management
//!
//! See Phase H.2 specification: `docs/design/GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_REVISIONS.md`

#![allow(dead_code)] // H.2 implementation, integration with readers happens in this phase

use crate::backend::ReaderBackend;
use crate::parse_error::ParseError;
use pyo3::prelude::*;

/// Unified interface for reading MARC record bytes from any supported source
///
/// Abstracts over Python file objects (Phase B/C) and Rust file I/O (Phase H.2).
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
    /// Type detection (H.1 algorithm):
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_unified_reader_from_rust_path() {
        // Create a minimal MARC record file
        let mut temp_file = NamedTempFile::new().unwrap();
        // Minimal leader + EOF marker
        let minimal_record = b"00027nam a2200013 a 4500\x1E\x1D";
        temp_file.write_all(minimal_record).unwrap();
        temp_file.flush().unwrap();

        // Open via Python with path string
        Python::with_gil(|py| {
            let path = temp_file.path().to_str().unwrap().to_string();
            let path_obj = path.into_py(py);
            let path_bound = path_obj.bind(py);

            let mut reader = UnifiedReader::from_python(path_bound).unwrap();

            // Read should work without GIL for RustFile
            match &reader {
                UnifiedReader::Backend(backend) => {
                    // Should be RustFile variant
                    assert!(matches!(backend, ReaderBackend::RustFile(_)));
                },
                _ => panic!("Expected Backend variant"),
            }
        });
    }

    #[test]
    fn test_unified_reader_from_bytes() {
        // Create in-memory MARC record
        let minimal_record = b"00027nam a2200013 a 4500\x1E\x1D";

        Python::with_gil(|py| {
            let bytes_obj = minimal_record.to_vec().into_py(py);
            let bytes_bound = bytes_obj.bind(py);

            let reader = UnifiedReader::from_python(bytes_bound).unwrap();

            // Should be CursorBackend variant
            match reader {
                UnifiedReader::Backend(backend) => {
                    assert!(matches!(backend, ReaderBackend::CursorBackend(_)));
                },
                _ => panic!("Expected Backend variant"),
            }
        });
    }

    #[test]
    fn test_unified_reader_unsupported_type() {
        Python::with_gil(|py| {
            let num_obj = 42i32.into_py(py);
            let num_bound = num_obj.bind(py);

            let result = UnifiedReader::from_python(num_bound);
            assert!(result.is_err());
            let err = result.unwrap_err().to_string();
            assert!(err.contains("Unsupported input type"));
        });
    }
}
