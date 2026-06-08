// Python wrapper for HoldingsMarcReader with multi-backend support
//
// Uses shared helpers from reader_helpers.rs for source detection and
// Python file I/O. Only type-specific logic (HoldingsMarcReader, PyHoldingsRecord)
// lives here.

use crate::reader_helpers;
use crate::wrappers::PyHoldingsRecord;
use mrrc::holdings_reader::HoldingsMarcReader;
use mrrc::recovery::{RecoveryMode, ValidationLevel};
use pyo3::prelude::*;
use std::fs::File;
use std::io::Cursor;

/// Internal enum for Holdings reader backends
enum HoldingsReaderBackend {
    RustFile(HoldingsMarcReader<File>),
    CursorBackend(HoldingsMarcReader<Cursor<Vec<u8>>>),
    PythonFile(Py<PyAny>),
}

/// Python wrapper for HoldingsMarcReader
///
/// Reads MARC Holdings records from different sources with automatic
/// backend selection:
/// - File paths → RustFile (parallel-safe)
/// - Bytes/BytesIO → CursorBackend (parallel-safe)
/// - Python file objects → PythonFile (requires GIL)
#[pyclass(name = "HoldingsMARCReader")]
pub struct PyHoldingsMARCReader {
    backend: Option<HoldingsReaderBackend>,
    recovery_mode: RecoveryMode,
    validation_level: ValidationLevel,
}

#[pymethods]
impl PyHoldingsMARCReader {
    /// Create a new HoldingsMARCReader
    ///
    /// # Arguments
    /// * `source` - File path (str), pathlib.Path, bytes, or file-like object
    /// * `recovery_mode` - Error handling mode: 'permissive' (default),
    ///   'lenient', 'strict'. The Python user surface defaults to
    ///   permissive for pymarc-shape parity; the Rust core defaults
    ///   to strict for explicit-error-handling parity with Rust idiom.
    /// * `validation_level` - What counts as an error during parsing:
    ///   'structural' (default) or 'strict_marc'.
    #[new]
    #[pyo3(signature = (source, *, recovery_mode = "permissive", validation_level = "structural"))]
    pub fn new(
        source: &Bound<'_, PyAny>,
        recovery_mode: &str,
        validation_level: &str,
    ) -> PyResult<Self> {
        let rec_mode = reader_helpers::parse_recovery_mode(recovery_mode)?;
        let val_level = reader_helpers::parse_validation_level(validation_level)?;

        // Try file path (str or pathlib.Path)
        if let Some(file) = reader_helpers::try_open_as_path(source)? {
            let reader = HoldingsMarcReader::new(file)
                .with_recovery_mode(rec_mode)
                .with_validation_level(val_level);
            return Ok(PyHoldingsMARCReader {
                backend: Some(HoldingsReaderBackend::RustFile(reader)),
                recovery_mode: rec_mode,
                validation_level: val_level,
            });
        }

        // Try bytes/bytearray
        if let Some(bytes) = reader_helpers::try_extract_bytes(source)? {
            let reader = HoldingsMarcReader::new(Cursor::new(bytes))
                .with_recovery_mode(rec_mode)
                .with_validation_level(val_level);
            return Ok(PyHoldingsMARCReader {
                backend: Some(HoldingsReaderBackend::CursorBackend(reader)),
                recovery_mode: rec_mode,
                validation_level: val_level,
            });
        }

        // Try Python file object
        if let Some(py_obj) = reader_helpers::try_as_python_file(source)? {
            return Ok(PyHoldingsMARCReader {
                backend: Some(HoldingsReaderBackend::PythonFile(py_obj)),
                recovery_mode: rec_mode,
                validation_level: val_level,
            });
        }

        reader_helpers::unsupported_source_error(source)?;
        unreachable!()
    }

    /// Read the next Holdings record
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
                    Err(crate::error::marc_error_to_py_err(e))
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
                    Err(crate::error::marc_error_to_py_err(e))
                },
            },
            HoldingsReaderBackend::PythonFile(py_obj) => {
                let py = unsafe { Python::assume_attached() };
                let bound = py_obj.bind(py);
                match reader_helpers::read_record_bytes_from_python_file(bound)? {
                    None => Ok(None),
                    Some(bytes) => {
                        let cursor = Cursor::new(bytes);
                        let mut parser = HoldingsMarcReader::new(cursor)
                            .with_recovery_mode(self.recovery_mode)
                            .with_validation_level(self.validation_level);
                        match parser.read_record() {
                            Ok(Some(record)) => {
                                self.backend = Some(HoldingsReaderBackend::PythonFile(py_obj));
                                Ok(Some(PyHoldingsRecord { inner: record }))
                            },
                            Ok(None) => Ok(None),
                            Err(e) => Err(crate::error::marc_error_to_py_err(e)),
                        }
                    },
                }
            },
        };

        result
    }

    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyHoldingsRecord> {
        match slf.read_record()? {
            Some(record) => Ok(record),
            None => Err(pyo3::exceptions::PyStopIteration::new_err(())),
        }
    }

    pub fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __exit__(
        mut slf: PyRefMut<'_, Self>,
        _exc_type: &Bound<'_, PyAny>,
        _exc_val: &Bound<'_, PyAny>,
        _exc_tb: &Bound<'_, PyAny>,
    ) -> PyResult<bool> {
        slf.backend = None;
        Ok(false)
    }

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
