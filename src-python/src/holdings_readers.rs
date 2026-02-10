// Python wrapper for HoldingsMarcReader with multi-backend support
//
// This module implements the Python interface for Holdings record reading,
// with automatic backend detection (RustFile, CursorBackend, PythonFile).
//
// Holdings records are specialized MARC records (Type x/y/v/u) used in library
// systems for maintaining inventory and location information.

use crate::wrappers::PyHoldingsRecord;
use mrrc::holdings_reader::HoldingsMarcReader;
use mrrc::recovery::RecoveryMode;
use pyo3::prelude::*;
use std::fs::File;
use std::io::Cursor;

/// Internal enum for Holdings reader backends
enum HoldingsReaderBackend {
    /// Direct file I/O via std::fs::File
    /// Input: str path or pathlib.Path
    RustFile(HoldingsMarcReader<File>),

    /// In-memory reads from bytes via std::io::Cursor
    /// Input: bytes or bytearray
    /// Enables thread-safe parallel parsing without Python interaction
    CursorBackend(HoldingsMarcReader<Cursor<Vec<u8>>>),

    /// Python file-like object (fallback for custom types)
    /// Input: Any object with .read() method
    /// Requires GIL for each read operation
    PythonFile(Py<PyAny>),
}

/// Python wrapper for HoldingsMarcReader
///
/// Reads MARC Holdings records (Type x/y/v/u) from different sources with automatic
/// backend selection:
/// - File paths → RustFile (parallel-safe)
/// - Bytes/BytesIO → CursorBackend (parallel-safe)
/// - Python file objects → PythonFile (requires GIL)
///
/// # Examples
///
/// Reading from a file path (parallel-safe):
/// ```python
/// from mrrc import HoldingsMARCReader
///
/// # RustFile backend automatically selected
/// with HoldingsMARCReader('holdings.mrc') as reader:
///     for record in reader:
///         print(f"Locations: {len(record.locations())}")
/// ```
///
/// Reading from bytes (parallel-safe):
/// ```python
/// reader = HoldingsMARCReader(holdings_bytes)
/// for record in reader:
///     print(f"Type: {record.record_type}")
/// ```
///
/// Reading from Python file object:
/// ```python
/// with open('holdings.mrc', 'rb') as f:
///     reader = HoldingsMARCReader(f)
///     for record in reader:
///         process(record)
/// ```
#[pyclass(name = "HoldingsMARCReader")]
pub struct PyHoldingsMARCReader {
    backend: Option<HoldingsReaderBackend>,
    recovery_mode: RecoveryMode,
}

#[pymethods]
impl PyHoldingsMARCReader {
    /// Create a new HoldingsMARCReader
    ///
    /// # Arguments
    /// * `source` - File path (str), pathlib.Path, bytes, or file-like object
    /// * `recovery_mode` - Error handling mode: 'strict' (default), 'lenient', 'permissive'
    #[new]
    #[pyo3(signature = (source, recovery_mode = "strict"))]
    pub fn new(source: &Bound<'_, PyAny>, recovery_mode: &str) -> PyResult<Self> {
        let rec_mode = match recovery_mode {
            "lenient" => RecoveryMode::Lenient,
            "permissive" => RecoveryMode::Permissive,
            _ => RecoveryMode::Strict,
        };

        // Try str path first
        if let Ok(path_str) = source.extract::<String>() {
            return match File::open(&path_str) {
                Ok(file) => {
                    let reader = HoldingsMarcReader::new(file).with_recovery_mode(rec_mode);
                    Ok(PyHoldingsMARCReader {
                        backend: Some(HoldingsReaderBackend::RustFile(reader)),
                        recovery_mode: rec_mode,
                    })
                },
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

        // Try pathlib.Path
        let fspath_method = source.getattr("__fspath__");
        if let Ok(method) = fspath_method {
            if method.is_callable() {
                if let Ok(path_obj) = method.call0() {
                    if let Ok(path_str) = path_obj.extract::<String>() {
                        return match File::open(&path_str) {
                            Ok(file) => {
                                let reader =
                                    HoldingsMarcReader::new(file).with_recovery_mode(rec_mode);
                                Ok(PyHoldingsMARCReader {
                                    backend: Some(HoldingsReaderBackend::RustFile(reader)),
                                    recovery_mode: rec_mode,
                                })
                            },
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

        // Try bytes/bytearray
        if let Ok(bytes_data) = source.extract::<Vec<u8>>() {
            let cursor = Cursor::new(bytes_data);
            let reader = HoldingsMarcReader::new(cursor).with_recovery_mode(rec_mode);
            return Ok(PyHoldingsMARCReader {
                backend: Some(HoldingsReaderBackend::CursorBackend(reader)),
                recovery_mode: rec_mode,
            });
        }

        // Try file-like object with .read() method
        let read_method = source.getattr("read");
        if let Ok(method) = read_method {
            if method.is_callable() {
                return Ok(PyHoldingsMARCReader {
                    backend: Some(HoldingsReaderBackend::PythonFile(source.clone().unbind())),
                    recovery_mode: rec_mode,
                });
            }
        }

        // Unknown type
        let type_name = source.get_type().name()?;
        Err(pyo3::exceptions::PyTypeError::new_err(format!(
            "Unsupported input type: {}. Supported types: str (file path), pathlib.Path, \
             bytes, bytearray, or file-like object (with .read() method)",
            type_name
        )))
    }

    /// Read the next Holdings record
    ///
    /// # Returns
    /// * `HoldingsRecord` on success
    /// * `None` if end of file reached
    ///
    /// # Raises
    /// * `ValueError` - If record is invalid or malformed
    /// * `IOError` - If read operation fails
    pub fn read_record(&mut self) -> PyResult<Option<PyHoldingsRecord>> {
        if self.backend.is_none() {
            return Err(pyo3::exceptions::PyStopIteration::new_err(
                "Reader is exhausted",
            ));
        }

        let backend = self.backend.take().unwrap();

        let result = match backend {
            HoldingsReaderBackend::RustFile(mut reader) => match reader.read_record() {
                Ok(Some(record)) => {
                    self.backend = Some(HoldingsReaderBackend::RustFile(reader));
                    Ok(Some(PyHoldingsRecord { inner: record }))
                },
                Ok(None) => Ok(None),
                Err(e) => {
                    self.backend = Some(HoldingsReaderBackend::RustFile(reader));
                    Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "Failed to read record: {}",
                        e
                    )))
                },
            },
            HoldingsReaderBackend::CursorBackend(mut reader) => match reader.read_record() {
                Ok(Some(record)) => {
                    self.backend = Some(HoldingsReaderBackend::CursorBackend(reader));
                    Ok(Some(PyHoldingsRecord { inner: record }))
                },
                Ok(None) => Ok(None),
                Err(e) => {
                    self.backend = Some(HoldingsReaderBackend::CursorBackend(reader));
                    Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "Failed to read record: {}",
                        e
                    )))
                },
            },
            HoldingsReaderBackend::PythonFile(py_obj) => {
                let py = unsafe { Python::assume_attached() };
                {
                    let obj = py_obj.bind(py);
                    let read_method = obj.getattr("read").map_err(|e| {
                        pyo3::exceptions::PyIOError::new_err(format!(
                            "Object missing .read() method: {}",
                            e
                        ))
                    })?;

                    // Read leader (24 bytes)
                    let leader_result = read_method.call1((24usize,)).map_err(|e| {
                        pyo3::exceptions::PyIOError::new_err(format!(
                            "Failed to read leader: {}",
                            e
                        ))
                    })?;

                    let leader: Vec<u8> = leader_result.extract().map_err(|_| {
                        pyo3::exceptions::PyValueError::new_err("Leader must be bytes")
                    })?;

                    if leader.is_empty() {
                        return Ok(None);
                    }

                    if leader.len() != 24 {
                        return Err(pyo3::exceptions::PyValueError::new_err(format!(
                            "Incomplete leader: expected 24 bytes, got {}",
                            leader.len()
                        )));
                    }

                    // Parse record length
                    let record_length_str = String::from_utf8_lossy(&leader[0..5]);
                    let record_length: usize = record_length_str.trim().parse().map_err(|_| {
                        pyo3::exceptions::PyValueError::new_err(format!(
                            "Invalid record length in leader: '{}'",
                            record_length_str
                        ))
                    })?;

                    if record_length < 24 {
                        return Err(pyo3::exceptions::PyValueError::new_err(format!(
                            "Record length too small: {} (minimum 24)",
                            record_length
                        )));
                    }

                    // Read remainder
                    let record_data_bytes =
                        read_method.call1((record_length - 24,)).map_err(|e| {
                            pyo3::exceptions::PyIOError::new_err(format!(
                                "Failed to read record data: {}",
                                e
                            ))
                        })?;

                    let record_data: Vec<u8> = record_data_bytes.extract().map_err(|_| {
                        pyo3::exceptions::PyValueError::new_err("Record data must be bytes")
                    })?;

                    if record_data.len() != record_length - 24 {
                        return Err(pyo3::exceptions::PyValueError::new_err(format!(
                            "Truncated record: expected {} bytes, got {}",
                            record_length - 24,
                            record_data.len()
                        )));
                    }

                    // Assemble and parse
                    let mut complete_record = Vec::with_capacity(record_length);
                    complete_record.extend_from_slice(&leader);
                    complete_record.extend_from_slice(&record_data);

                    let cursor = std::io::Cursor::new(complete_record);
                    let mut parser =
                        HoldingsMarcReader::new(cursor).with_recovery_mode(self.recovery_mode);

                    match parser.read_record() {
                        Ok(Some(record)) => Ok(Some(PyHoldingsRecord { inner: record })),
                        Ok(None) => Ok(None),
                        Err(e) => Err(pyo3::exceptions::PyValueError::new_err(format!(
                            "Failed to parse record: {}",
                            e
                        ))),
                    }
                }
            },
        };

        // If we hit EOF (Ok(None)), don't restore the backend
        match &result {
            Ok(None) => {},    // EOF, backend stays None
            Err(_) => {},      // Error, backend already re-set above
            Ok(Some(_)) => {}, // Success, backend already re-set above
        }

        result
    }

    /// Iterator protocol support
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Iterator next
    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyHoldingsRecord> {
        match slf.read_record()? {
            Some(record) => Ok(record),
            None => Err(pyo3::exceptions::PyStopIteration::new_err(())),
        }
    }

    /// Context manager support
    pub fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Context manager cleanup
    pub fn __exit__(
        mut slf: PyRefMut<'_, Self>,
        _exc_type: &Bound<'_, PyAny>,
        _exc_val: &Bound<'_, PyAny>,
        _exc_tb: &Bound<'_, PyAny>,
    ) -> PyResult<bool> {
        slf.backend = None;
        Ok(false)
    }

    /// Get string representation
    pub fn __repr__(&self) -> String {
        match &self.backend {
            None => "HoldingsMARCReader(closed)".to_string(),
            Some(HoldingsReaderBackend::RustFile(_)) => "HoldingsMARCReader(RustFile)".to_string(),
            Some(HoldingsReaderBackend::CursorBackend(_)) => {
                "HoldingsMARCReader(CursorBackend)".to_string()
            },
            Some(HoldingsReaderBackend::PythonFile(_)) => {
                "HoldingsMARCReader(PythonFile)".to_string()
            },
        }
    }
}
