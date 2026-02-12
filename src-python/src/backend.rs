//! Backend abstraction for ReaderBackend enum
//!
//! This module provides a unified interface for different input sources:
//! - `RustFile`: Direct file I/O via std::fs::File
//! - `CursorBackend`: In-memory reads from bytes via std::io::Cursor
//! - `PythonFile`: Python file-like objects (calls .read() method)

use crate::parse_error::ParseError;
use pyo3::prelude::*;
use std::fs::File;
use std::io::{Cursor, Read};

/// Unified backend interface for reading MARC records from different sources
///
/// Supports 8 input types:
/// - str, pathlib.Path → RustFile
/// - bytes, bytearray → CursorBackend
/// - file object, BytesIO, socket.socket → PythonFile
#[derive(Debug)]
pub enum ReaderBackend {
    /// Direct file I/O via std::fs::File
    /// Input: str path or pathlib.Path
    RustFile(File),

    /// In-memory reads from bytes via std::io::Cursor
    /// Input: bytes or bytearray
    /// Enables thread-safe parallel parsing without Python interaction
    CursorBackend(Cursor<Vec<u8>>),

    /// Python file-like object (fallback for custom types)
    /// Input: Any object with .read() method
    /// Requires GIL for each read operation
    PythonFile(Py<PyAny>),
}

impl ReaderBackend {
    /// Create a ReaderBackend from a Python object
    ///
    /// Type detection order:
    /// 1. str → RustFile
    /// 2. pathlib.Path → RustFile
    /// 3. bytes/bytearray → CursorBackend
    /// 4. Object with .read() method → PythonFile
    /// 5. Unknown type → TypeError
    ///
    /// # Arguments
    /// * `source` - Python object (str, Path, bytes, bytearray, or file-like)
    /// * `_py` - Python interpreter handle (not used but required for consistency)
    ///
    /// # Errors
    /// - `TypeError` if input type is not supported
    /// - `FileNotFoundError` if file path doesn't exist (RustFile)
    /// - `IOError` if file cannot be opened (RustFile)
    pub fn from_python(source: &Bound<'_, PyAny>, _py: Python) -> PyResult<Self> {
        // 1. Try str path
        if let Ok(path_str) = source.extract::<String>() {
            return match File::open(&path_str) {
                Ok(file) => Ok(ReaderBackend::RustFile(file)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    Err(pyo3::exceptions::PyFileNotFoundError::new_err(format!(
                        "No such file or directory: '{}'",
                        path_str
                    )))
                },
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    Err(pyo3::exceptions::PyPermissionError::new_err(format!(
                        "Permission denied: '{}'",
                        path_str
                    )))
                },
                Err(e) => Err(pyo3::exceptions::PyIOError::new_err(format!(
                    "Failed to open file '{}': {}",
                    path_str, e
                ))),
            };
        }

        // 2. Try pathlib.Path via __fspath__()
        let fspath_method = source.getattr("__fspath__");
        if let Ok(method) = fspath_method {
            if method.is_callable() {
                if let Ok(path_obj) = method.call0() {
                    if let Ok(path_str) = path_obj.extract::<String>() {
                        return match File::open(&path_str) {
                            Ok(file) => Ok(ReaderBackend::RustFile(file)),
                            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                                Err(pyo3::exceptions::PyFileNotFoundError::new_err(format!(
                                    "No such file or directory: '{}'",
                                    path_str
                                )))
                            },
                            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                                Err(pyo3::exceptions::PyPermissionError::new_err(format!(
                                    "Permission denied: '{}'",
                                    path_str
                                )))
                            },
                            Err(e) => Err(pyo3::exceptions::PyIOError::new_err(format!(
                                "Failed to open file '{}': {}",
                                path_str, e
                            ))),
                        };
                    }
                }
            }
        }

        // 3. Try bytes/bytearray
        if let Ok(bytes_data) = source.extract::<Vec<u8>>() {
            return Ok(ReaderBackend::CursorBackend(Cursor::new(bytes_data)));
        }

        // 4. Try file-like object with .read() method
        let read_method = source.getattr("read");
        if let Ok(method) = read_method {
            if method.is_callable() {
                // Store as PythonFile backend
                return Ok(ReaderBackend::PythonFile(source.clone().unbind()));
            }
        }

        // 5. Unknown type - fail fast with descriptive error
        let type_name = source.get_type().name()?;
        Err(pyo3::exceptions::PyTypeError::new_err(format!(
            "Unsupported input type: {}. Supported types: str (file path), pathlib.Path, \
             bytes, bytearray, or file-like object (with .read() method). \
             Examples: 'records.mrc', Path('records.mrc'), b'binary data', \
             open('records.mrc', 'rb'), io.BytesIO(data), socket.socket(...)",
            type_name
        )))
    }

    /// Return the backend type as a string for diagnostics
    pub fn backend_type(&self) -> &str {
        match self {
            ReaderBackend::RustFile(_) => "rust_file",
            ReaderBackend::CursorBackend(_) => "cursor",
            ReaderBackend::PythonFile(_) => "python_file",
        }
    }

    /// Read the next MARC record from this backend
    ///
    /// For RustFile and CursorBackend: reads directly without GIL
    /// For PythonFile: requires GIL to call .read()
    ///
    /// # Arguments
    /// * `py` - Python interpreter handle (required for PythonFile)
    ///
    /// # Returns
    /// - `Ok(Some(bytes))` - Successfully read record bytes
    /// - `Ok(None)` - EOF reached
    /// - `Err(ParseError)` - Read or parsing error
    pub fn read_next_bytes(&mut self, py: Python) -> Result<Option<Vec<u8>>, ParseError> {
        match self {
            ReaderBackend::RustFile(file) => Self::read_record_bytes_from_reader(file),
            ReaderBackend::CursorBackend(cursor) => Self::read_record_bytes_from_reader(cursor),
            ReaderBackend::PythonFile(py_obj) => {
                // Need GIL to call Python .read() method
                let obj = py_obj.bind(py);
                Self::read_record_bytes_from_python(obj)
            },
        }
    }

    /// Internal helper: Read record bytes from any std::io::Read implementation
    fn read_record_bytes_from_reader<R: Read>(
        reader: &mut R,
    ) -> Result<Option<Vec<u8>>, ParseError> {
        // Read leader (24 bytes)
        let mut leader = [0u8; 24];
        match reader.read_exact(&mut leader) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(None); // EOF
            },
            Err(e) => {
                return Err(ParseError::IoError(format!(
                    "Failed to read record leader: {}",
                    e
                )))
            },
        }

        // Parse record length from leader (bytes 0-4, ASCII digits)
        let record_length_str = String::from_utf8_lossy(&leader[0..5]);
        let record_length: usize = record_length_str.trim().parse().map_err(|_| {
            ParseError::InvalidRecord(format!(
                "Invalid record length in leader: '{}'",
                record_length_str
            ))
        })?;

        if record_length < 24 {
            return Err(ParseError::InvalidRecord(format!(
                "Record length too small: {} (minimum 24)",
                record_length
            )));
        }

        // Read remainder of record (record_length - 24 bytes)
        let mut record_data = vec![0u8; record_length - 24];
        match reader.read_exact(&mut record_data) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Err(ParseError::InvalidRecord(format!(
                    "Truncated record: expected {} bytes, got {}",
                    record_length - 24,
                    record_data.len()
                )))
            },
            Err(e) => {
                return Err(ParseError::IoError(format!(
                    "Failed to read record data: {}",
                    e
                )))
            },
        }

        // Assemble complete record bytes
        let mut complete_record = Vec::with_capacity(record_length);
        complete_record.extend_from_slice(&leader);
        complete_record.extend_from_slice(&record_data);

        Ok(Some(complete_record))
    }

    /// Internal helper: Read record bytes from Python file-like object
    fn read_record_bytes_from_python(
        py_obj: &Bound<'_, PyAny>,
    ) -> Result<Option<Vec<u8>>, ParseError> {
        // Read leader (24 bytes)
        let read_method = py_obj
            .getattr("read")
            .map_err(|e| ParseError::IoError(format!("Object missing .read() method: {}", e)))?;

        let leader_result = read_method.call1((24usize,)).map_err(|e| {
            ParseError::IoError(format!("Failed to read leader from Python file: {}", e))
        })?;

        let leader: Vec<u8> = leader_result
            .extract()
            .map_err(|_| ParseError::InvalidRecord("Leader must be bytes".to_string()))?;

        if leader.len() != 24 {
            // EOF or partial read
            if leader.is_empty() {
                return Ok(None); // EOF
            }
            return Err(ParseError::InvalidRecord(format!(
                "Incomplete leader: expected 24 bytes, got {}",
                leader.len()
            )));
        }

        // Parse record length from leader
        let record_length_str = String::from_utf8_lossy(&leader[0..5]);
        let record_length: usize = record_length_str.trim().parse().map_err(|_| {
            ParseError::InvalidRecord(format!(
                "Invalid record length in leader: '{}'",
                record_length_str
            ))
        })?;

        if record_length < 24 {
            return Err(ParseError::InvalidRecord(format!(
                "Record length too small: {} (minimum 24)",
                record_length
            )));
        }

        // Read remainder of record
        let record_data_bytes = read_method.call1((record_length - 24,)).map_err(|e| {
            ParseError::IoError(format!(
                "Failed to read record data from Python file: {}",
                e
            ))
        })?;

        let record_data: Vec<u8> = record_data_bytes
            .extract()
            .map_err(|_| ParseError::InvalidRecord("Record data must be bytes".to_string()))?;

        if record_data.len() != record_length - 24 {
            return Err(ParseError::InvalidRecord(format!(
                "Truncated record: expected {} bytes, got {}",
                record_length - 24,
                record_data.len()
            )));
        }

        // Assemble complete record
        let mut complete_record = Vec::with_capacity(record_length);
        complete_record.extend_from_slice(&leader);
        complete_record.extend_from_slice(&record_data);

        Ok(Some(complete_record))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_reader_backend_creation() {
        // Test that enum can be instantiated
        let file = File::open("/dev/null").unwrap();
        let _backend = ReaderBackend::RustFile(file);

        let cursor = Cursor::new(vec![]);
        let _backend = ReaderBackend::CursorBackend(cursor);
    }
}
