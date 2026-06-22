//! Backend abstraction for `ReaderBackend` enum
//!
//! This module provides a unified interface for different input sources:
//! - `RustFile`: Direct file I/O via `std::fs::File`
//! - `CursorBackend`: In-memory reads from bytes via `std::io::Cursor`
//! - `PythonFile`: Python file-like objects (calls .`read()` method)

use crate::chunked_py_reader::ChunkedPyFileReader;
use crate::parse_error::ParseError;
use mrrc::RecoveryMode;
use pyo3::prelude::*;
use std::fs::File;
use std::io::{BufReader, Cursor, Read};

/// Buffer capacity for file-path backends. Matches the core readers'
/// `from_path` buffering: the per-record read loop issues at least two
/// small reads per record, and 64 KiB amortizes those to roughly one
/// syscall per buffer fill.
pub(crate) const FILE_READ_BUF_CAPACITY: usize = 64 * 1024;

/// A source of complete ISO 2709 record byte-slices, read one record at a
/// time. Abstracts over the concrete input backends so the batching and
/// parsing layer ([`crate::batched_reader::BatchedReader`]) is generic and
/// unit-testable against a mock source.
pub trait RecordByteSource: std::fmt::Debug {
    /// Read the next record's raw bytes. `Ok(None)` signals a clean end of
    /// stream; `Err` is an I/O or boundary failure.
    fn next_record_bytes(&mut self, py: Python<'_>) -> Result<Option<Vec<u8>>, ParseError>;

    /// Backend kind for diagnostics: `"rust_file"`, `"cursor"`, or
    /// `"python_file"`.
    fn backend_kind(&self) -> &'static str;
}

impl RecordByteSource for ReaderBackend {
    fn next_record_bytes(&mut self, py: Python<'_>) -> Result<Option<Vec<u8>>, ParseError> {
        self.read_next_bytes(py)
    }

    fn backend_kind(&self) -> &'static str {
        self.backend_type()
    }
}

/// Unified backend interface for reading MARC records from different sources
///
/// Supports 8 input types:
/// - str, pathlib.Path → `RustFile`
/// - bytes, bytearray → `CursorBackend`
/// - file object, `BytesIO`, socket.socket → `PythonFile`
///
/// The backend carries the active [`RecoveryMode`] so that short body
/// reads can route to either a fatal `TruncatedRecord` (strict) or a
/// pass-through to the parser's `record.errors` dispatch
/// (lenient/permissive). The mode is set at construction by the
/// owning reader and never changes for the lifetime of the backend.
#[derive(Debug)]
pub struct ReaderBackend {
    kind: BackendKind,
    recovery_mode: RecoveryMode,
}

#[derive(Debug)]
enum BackendKind {
    /// Buffered file I/O via `std::fs::File`
    /// Input: str path or pathlib.Path
    RustFile(BufReader<File>),

    /// In-memory reads from bytes via `std::io::Cursor`
    /// Input: bytes or bytearray
    /// Enables thread-safe parallel parsing without Python interaction
    CursorBackend(Cursor<Vec<u8>>),

    /// Python file-like object (fallback for custom types)
    /// Input: Any object with .`read()` method
    /// Reads in large chunks and slices records out in Rust; the GIL is
    /// held only while a chunk is being read, not per record.
    PythonFile(ChunkedPyFileReader),
}

impl ReaderBackend {
    /// Create a `ReaderBackend` from a Python object
    ///
    /// Type detection order:
    /// 1. str → `RustFile`
    /// 2. pathlib.Path → `RustFile`
    /// 3. bytes/bytearray → `CursorBackend`
    /// 4. Object with .`read()` method → `PythonFile`
    /// 5. Unknown type → `TypeError`
    ///
    /// # Arguments
    /// * `source` - Python object (str, Path, bytes, bytearray, or file-like)
    /// * `_py` - Python interpreter handle (not used but required for consistency)
    ///
    /// # Errors
    /// - `TypeError` if input type is not supported
    /// - `FileNotFoundError` if file path doesn't exist (`RustFile`)
    /// - `IOError` if file cannot be opened (`RustFile`)
    pub fn from_python(
        source: &Bound<'_, PyAny>,
        _py: Python,
        recovery_mode: RecoveryMode,
    ) -> PyResult<Self> {
        let kind = Self::kind_from_python(source, recovery_mode)?;
        Ok(ReaderBackend {
            kind,
            recovery_mode,
        })
    }

    fn kind_from_python(
        source: &Bound<'_, PyAny>,
        recovery_mode: RecoveryMode,
    ) -> PyResult<BackendKind> {
        // 1. Try str path
        if let Ok(path_str) = source.extract::<String>() {
            return match File::open(&path_str) {
                Ok(file) => Ok(BackendKind::RustFile(BufReader::with_capacity(
                    FILE_READ_BUF_CAPACITY,
                    file,
                ))),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    Err(pyo3::exceptions::PyFileNotFoundError::new_err(format!(
                        "No such file or directory: '{path_str}'"
                    )))
                },
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    Err(pyo3::exceptions::PyPermissionError::new_err(format!(
                        "Permission denied: '{path_str}'"
                    )))
                },
                Err(e) => Err(pyo3::exceptions::PyIOError::new_err(format!(
                    "Failed to open file '{path_str}': {e}"
                ))),
            };
        }

        // 2. Try pathlib.Path via __fspath__()
        let fspath_method = source.getattr("__fspath__");
        if let Ok(method) = fspath_method
            && method.is_callable()
            && let Ok(path_obj) = method.call0()
            && let Ok(path_str) = path_obj.extract::<String>()
        {
            return match File::open(&path_str) {
                Ok(file) => Ok(BackendKind::RustFile(BufReader::with_capacity(
                    FILE_READ_BUF_CAPACITY,
                    file,
                ))),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    Err(pyo3::exceptions::PyFileNotFoundError::new_err(format!(
                        "No such file or directory: '{path_str}'"
                    )))
                },
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    Err(pyo3::exceptions::PyPermissionError::new_err(format!(
                        "Permission denied: '{path_str}'"
                    )))
                },
                Err(e) => Err(pyo3::exceptions::PyIOError::new_err(format!(
                    "Failed to open file '{path_str}': {e}"
                ))),
            };
        }

        // 3. Try bytes/bytearray
        if let Ok(bytes_data) = source.extract::<Vec<u8>>() {
            return Ok(BackendKind::CursorBackend(Cursor::new(bytes_data)));
        }

        // 4. Try file-like object with .read() method
        let read_method = source.getattr("read");
        if let Ok(method) = read_method
            && method.is_callable()
        {
            // Store as PythonFile backend, reading in large chunks.
            return Ok(BackendKind::PythonFile(ChunkedPyFileReader::new(
                source,
                recovery_mode,
            )?));
        }

        // 5. Unknown type - fail fast with descriptive error
        let type_name = source.get_type().name()?;
        Err(pyo3::exceptions::PyTypeError::new_err(format!(
            "Unsupported input type: {type_name}. Supported types: str (file path), pathlib.Path, \
             bytes, bytearray, or file-like object (with .read() method). \
             Examples: 'records.mrc', Path('records.mrc'), b'binary data', \
             open('records.mrc', 'rb'), io.BytesIO(data), socket.socket(...)"
        )))
    }

    /// Return the backend type as a string for diagnostics
    pub fn backend_type(&self) -> &'static str {
        match &self.kind {
            BackendKind::RustFile(_) => "rust_file",
            BackendKind::CursorBackend(_) => "cursor",
            BackendKind::PythonFile(_) => "python_file",
        }
    }

    /// Read the next MARC record from this backend
    ///
    /// For `RustFile` and `CursorBackend`: reads directly without GIL
    /// For `PythonFile`: requires GIL to call .`read()`
    ///
    /// # Arguments
    /// * `py` - Python interpreter handle (required for `PythonFile`)
    ///
    /// # Returns
    /// - `Ok(Some(bytes))` - Successfully read record bytes
    /// - `Ok(None)` - EOF reached
    /// - `Err(ParseError)` - Read or parsing error
    pub fn read_next_bytes(&mut self, py: Python) -> Result<Option<Vec<u8>>, ParseError> {
        let recovery_mode = self.recovery_mode;
        match &mut self.kind {
            BackendKind::RustFile(file) => Self::read_record_bytes_from_reader(file, recovery_mode),
            BackendKind::CursorBackend(cursor) => {
                Self::read_record_bytes_from_reader(cursor, recovery_mode)
            },
            BackendKind::PythonFile(chunked) => {
                // GIL is held only while a chunk is read; the chunked reader
                // serves most records straight from its buffer.
                chunked.read_next_record_bytes(py)
            },
        }
    }

    /// Internal helper: Read record bytes from any `std::io::Read` implementation
    fn read_record_bytes_from_reader<R: Read>(
        reader: &mut R,
        recovery_mode: RecoveryMode,
    ) -> Result<Option<Vec<u8>>, ParseError> {
        // Read leader (24 bytes)
        let mut leader = [0u8; 24];
        match reader.read_exact(&mut leader) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(None); // EOF
            },
            Err(e) => {
                return Err(ParseError::io_error(format!(
                    "Failed to read record leader: {e}"
                )));
            },
        }

        // Parse record length from leader (bytes 0-4, ASCII digits)
        let record_length: usize = std::str::from_utf8(&leader[0..5])
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .ok_or_else(|| {
                ParseError::record_length_invalid(&leader[0..5], "5 ASCII digits")
                    .with_bytes_near(&leader, 0)
            })?;

        if record_length < 24 {
            return Err(
                ParseError::record_length_invalid(&leader[0..5], "at least 24")
                    .with_bytes_near(&leader, 0),
            );
        }

        // Read remainder of record (record_length - 24 bytes). `take` +
        // `read_to_end` grows the buffer as bytes arrive — no zero-init
        // memset per record — retries on `Interrupted` internally, and
        // reports the actual count when the stream ends short (so the typed
        // truncation error below carries the right `actual_length`).
        let expected_body_len = record_length - 24;
        let mut record_data = Vec::with_capacity(expected_body_len);
        let bytes_read = reader
            .take(expected_body_len as u64)
            .read_to_end(&mut record_data)
            .map_err(|e| ParseError::io_error(format!("Failed to read record data: {e}")))?;
        if bytes_read != expected_body_len {
            // Truncation: the leader (24 bytes) was read, then the
            // body fell short. In Strict mode this surfaces as a
            // fatal E005 with record_byte_offset = 24 marking where
            // within the record the truncation was detected. In
            // Lenient/Permissive we hand the leader + partial body
            // through to the parser, which dispatches its own E005
            // onto `record.errors` via the recovery cap.
            if recovery_mode == RecoveryMode::Strict {
                return Err(ParseError::truncated_record(expected_body_len, bytes_read)
                    .with_record_byte_offset(24));
            }
            record_data.truncate(bytes_read);
        }

        // Assemble record bytes (full record in Strict; leader +
        // whatever body was read in Lenient/Permissive on short reads).
        let mut complete_record = Vec::with_capacity(24 + record_data.len());
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
        // Test that backend can be instantiated
        let file = File::open("/dev/null").unwrap();
        let _backend = ReaderBackend {
            kind: BackendKind::RustFile(BufReader::with_capacity(FILE_READ_BUF_CAPACITY, file)),
            recovery_mode: RecoveryMode::Strict,
        };

        let cursor = Cursor::new(vec![]);
        let _backend = ReaderBackend {
            kind: BackendKind::CursorBackend(cursor),
            recovery_mode: RecoveryMode::Strict,
        };
    }
}
