// Python wrapper for AuthorityMarcReader with multi-backend support
//
// Uses shared helpers from reader_helpers.rs for source detection and
// Python file I/O. Only type-specific logic (AuthorityMarcReader, PyAuthorityRecord)
// lives here.

use crate::backend::FILE_READ_BUF_CAPACITY;
use crate::reader_helpers;
use crate::wrappers::PyAuthorityRecord;
use mrrc::authority_reader::AuthorityMarcReader;
use mrrc::recovery::{RecoveryMode, ValidationLevel};
use mrrc::{CharacterCoding, Utf8Handling};
use pyo3::prelude::*;
use std::fs::File;
use std::io::{BufReader, Cursor};

/// Internal enum for Authority reader backends
enum AuthorityReaderBackend {
    RustFile(AuthorityMarcReader<BufReader<File>>),
    CursorBackend(AuthorityMarcReader<Cursor<Vec<u8>>>),
    PythonFile(Py<PyAny>),
}

/// Python wrapper for `AuthorityMarcReader`
///
/// Reads MARC Authority records (Type Z) from different sources with automatic
/// backend selection:
/// - File paths → `RustFile` (parallel-safe)
/// - Bytes/BytesIO → `CursorBackend` (parallel-safe)
/// - Python file objects → `PythonFile` (requires GIL)
#[pyclass(name = "AuthorityMARCReader")]
// wraps AuthorityReaderBackend which does not implement Debug
#[allow(missing_debug_implementations)]
pub struct PyAuthorityMARCReader {
    backend: Option<AuthorityReaderBackend>,
    recovery_mode: RecoveryMode,
    validation_level: ValidationLevel,
    character_coding: CharacterCoding,
    utf8_handling: Utf8Handling,
}

#[pymethods]
impl PyAuthorityMARCReader {
    /// Create a new `AuthorityMARCReader`
    ///
    /// # Arguments
    /// * `source` - File path (str), pathlib.Path, bytes, or file-like object
    /// * `recovery_mode` - Error handling mode: 'permissive' (default),
    ///   'lenient', 'strict'. The Python user surface defaults to
    ///   permissive for pymarc-shape parity; the Rust core defaults
    ///   to strict for explicit-error-handling parity with Rust idiom.
    /// * `validation_level` - What counts as an error during parsing:
    ///   'structural' (default) or '`strict_marc`'.
    /// * `character_coding` - How each record's encoding is chosen:
    ///   'leader' (the default: leader position 09, as pymarc does),
    ///   'utf-8', or 'detect'.
    /// * `force_utf8` - pymarc's spelling of `character_coding='utf-8'`.
    /// * `utf8_handling` - pymarc's option for invalid UTF-8: 'strict'
    ///   (the default: an E301 error, handled by `recovery_mode`),
    ///   'replace' (U+FFFD), or 'ignore' (drop the bytes).
    #[new]
    #[pyo3(signature = (
        source,
        *,
        recovery_mode = "permissive",
        validation_level = "structural",
        character_coding = None,
        force_utf8 = false,
        utf8_handling = "strict",
    ))]
    pub fn new(
        source: &Bound<'_, PyAny>,
        recovery_mode: &str,
        validation_level: &str,
        character_coding: Option<&str>,
        force_utf8: bool,
        utf8_handling: &str,
    ) -> PyResult<Self> {
        let rec_mode = reader_helpers::parse_recovery_mode(recovery_mode)?;
        let val_level = reader_helpers::parse_validation_level(validation_level)?;
        let coding = reader_helpers::parse_character_coding(character_coding, force_utf8)?;
        let utf8 = reader_helpers::parse_utf8_handling(utf8_handling)?;

        // Try file path (str or pathlib.Path)
        if let Some(file) = reader_helpers::try_open_as_path(source)? {
            let file = BufReader::with_capacity(FILE_READ_BUF_CAPACITY, file);
            let reader = AuthorityMarcReader::new(file)
                .with_recovery_mode(rec_mode)
                .with_validation_level(val_level)
                .with_character_coding(coding)
                .with_utf8_handling(utf8);
            return Ok(PyAuthorityMARCReader {
                backend: Some(AuthorityReaderBackend::RustFile(reader)),
                recovery_mode: rec_mode,
                validation_level: val_level,
                character_coding: coding,
                utf8_handling: utf8,
            });
        }

        // Try bytes/bytearray
        if let Some(bytes) = reader_helpers::try_extract_bytes(source)? {
            let reader = AuthorityMarcReader::new(Cursor::new(bytes))
                .with_recovery_mode(rec_mode)
                .with_validation_level(val_level)
                .with_character_coding(coding)
                .with_utf8_handling(utf8);
            return Ok(PyAuthorityMARCReader {
                backend: Some(AuthorityReaderBackend::CursorBackend(reader)),
                recovery_mode: rec_mode,
                validation_level: val_level,
                character_coding: coding,
                utf8_handling: utf8,
            });
        }

        // Try Python file object
        if let Some(py_obj) = reader_helpers::try_as_python_file(source)? {
            return Ok(PyAuthorityMARCReader {
                backend: Some(AuthorityReaderBackend::PythonFile(py_obj)),
                recovery_mode: rec_mode,
                validation_level: val_level,
                character_coding: coding,
                utf8_handling: utf8,
            });
        }

        reader_helpers::unsupported_source_error(source)?;
        unreachable!()
    }

    /// Read the next Authority record
    pub fn read_record(&mut self) -> PyResult<Option<PyAuthorityRecord>> {
        if self.backend.is_none() {
            return Err(pyo3::exceptions::PyStopIteration::new_err(
                "Reader is exhausted",
            ));
        }

        let backend = self.backend.take().unwrap();

        match backend {
            AuthorityReaderBackend::RustFile(mut reader) => match reader.read_record() {
                Ok(Some(record)) => {
                    self.backend = Some(AuthorityReaderBackend::RustFile(reader));
                    Ok(Some(PyAuthorityRecord::from(record)))
                },
                Ok(None) => Ok(None),
                Err(e) => {
                    self.backend = Some(AuthorityReaderBackend::RustFile(reader));
                    Err(crate::error::marc_error_to_py_err(e))
                },
            },
            AuthorityReaderBackend::CursorBackend(mut reader) => match reader.read_record() {
                Ok(Some(record)) => {
                    self.backend = Some(AuthorityReaderBackend::CursorBackend(reader));
                    Ok(Some(PyAuthorityRecord::from(record)))
                },
                Ok(None) => Ok(None),
                Err(e) => {
                    self.backend = Some(AuthorityReaderBackend::CursorBackend(reader));
                    Err(crate::error::marc_error_to_py_err(e))
                },
            },
            AuthorityReaderBackend::PythonFile(py_obj) => {
                let py = unsafe { Python::assume_attached() };
                let bound = py_obj.bind(py);
                match reader_helpers::read_record_bytes_from_python_file(bound)? {
                    None => Ok(None),
                    Some(bytes) => {
                        let cursor = Cursor::new(bytes);
                        let mut parser = AuthorityMarcReader::new(cursor)
                            .with_recovery_mode(self.recovery_mode)
                            .with_validation_level(self.validation_level)
                            .with_character_coding(self.character_coding)
                            .with_utf8_handling(self.utf8_handling);
                        match parser.read_record() {
                            Ok(Some(record)) => {
                                self.backend = Some(AuthorityReaderBackend::PythonFile(py_obj));
                                Ok(Some(PyAuthorityRecord::from(record)))
                            },
                            Ok(None) => Ok(None),
                            Err(e) => Err(crate::error::marc_error_to_py_err(e)),
                        }
                    },
                }
            },
        }
    }

    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyAuthorityRecord> {
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
            None => "AuthorityMARCReader(closed)".to_string(),
            Some(AuthorityReaderBackend::RustFile(_)) => {
                "AuthorityMARCReader(RustFile)".to_string()
            },
            Some(AuthorityReaderBackend::CursorBackend(_)) => {
                "AuthorityMARCReader(CursorBackend)".to_string()
            },
            Some(AuthorityReaderBackend::PythonFile(_)) => {
                "AuthorityMARCReader(PythonFile)".to_string()
            },
        }
    }
}
