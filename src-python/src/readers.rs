// Python wrapper for MARCReader with GIL release support
//
// This module implements three-phase GIL management for concurrent file I/O:
// - Phase 1 (GIL held): Read record bytes from Python file object
// - Phase 2 (GIL released): Parse bytes into MARC record structure
// - Phase 3 (GIL held): Convert result to Python object and handle errors

#![allow(deprecated)]

use crate::buffered_reader::BufferedMarcReader;
use crate::parse_error::ParseError;
use crate::wrappers::PyRecord;
use mrrc::MarcReader;
use pyo3::prelude::*;
use smallvec::SmallVec;

/// Python wrapper for MarcReader with three-phase GIL management
///
/// The three-phase pattern enables GIL release during CPU-intensive parsing:
/// - Phase 1: Read bytes from Python file (GIL held)
/// - Phase 2: Parse bytes to MARC record (GIL released)
/// - Phase 3: Convert to PyRecord (GIL re-acquired)
///
/// This allows multiple threads to read different files concurrently.
#[pyclass(name = "MARCReader")]
pub struct PyMARCReader {
    /// Buffered reader for ISO 2709 record boundary detection
    buffered_reader: Option<BufferedMarcReader>,
}

#[pymethods]
impl PyMARCReader {
    /// Create a new MARCReader from a Python file-like object
    ///
    /// # Arguments
    /// * `file` - A Python file-like object (must support .read(n) method)
    #[new]
    pub fn new(file: Py<PyAny>) -> PyResult<Self> {
        // Initialize BufferedMarcReader (will read and parse incrementally)
        let buffered_reader = BufferedMarcReader::new(file);

        Ok(PyMARCReader {
            buffered_reader: Some(buffered_reader),
        })
    }

    /// Read the next record from the file (legacy interface)
    ///
    /// This method holds the GIL during both reading and parsing.
    /// New code should use iteration (__next__) which supports GIL release.
    pub fn read_record(&mut self) -> PyResult<Option<PyRecord>> {
        Python::with_gil(|py| {
            // Phase 1: Read record bytes (GIL held)
            let record_bytes = self
                .buffered_reader
                .as_mut()
                .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Reader consumed"))?
                .read_next_record_bytes(py)
                .map_err(|e| e.to_py_err())?;

            match record_bytes {
                None => Ok(None),
                Some(bytes) => {
                    // Phase 2: Parse bytes (GIL released)
                    let record = py
                        .allow_threads(|| {
                            // Create a cursor from owned bytes
                            let cursor = std::io::Cursor::new(bytes);
                            let mut parser = MarcReader::new(cursor);

                            // Parse the single record
                            parser.read_record().map_err(|e| {
                                ParseError::InvalidRecord(format!(
                                    "Failed to parse MARC record: {}",
                                    e
                                ))
                            })
                        })
                        .map_err(|e| e.to_py_err())?;

                    // Phase 3: Convert to PyRecord (GIL re-acquired)
                    match record {
                        Some(r) => Ok(Some(PyRecord { inner: r })),
                        None => Ok(None),
                    }
                },
            }
        })
    }

    /// Iterate over all records in the file
    ///
    /// Returns self for iteration (consuming the reader)
    fn __iter__(slf: PyRefMut<'_, Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    /// Get the next record during iteration
    ///
    /// This implements the three-phase GIL release pattern:
    /// - Phase 1: Read record bytes from Python file (GIL held)
    /// - Phase 2: Parse bytes to MARC record (GIL released)
    /// - Phase 3: Convert to PyRecord (GIL re-acquired)
    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRecord> {
        Python::with_gil(|py| {
            // Get mutable reference to buffered reader
            let reader = slf
                .buffered_reader
                .as_mut()
                .ok_or_else(|| pyo3::exceptions::PyStopIteration::new_err(()))?;

            // ===== PHASE 1: Read record bytes (GIL held) =====
            // Call read_next_record_bytes() while holding GIL
            let record_bytes = match reader.read_next_record_bytes(py) {
                Ok(Some(bytes)) => bytes,
                Ok(None) => {
                    // EOF reached - mark reader as consumed
                    slf.buffered_reader = None;
                    return Err(pyo3::exceptions::PyStopIteration::new_err(()));
                },
                Err(e) => return Err(e.to_py_err()),
            };

            // CRITICAL: Copy to owned SmallVec for Phase 2 closure
            // The slice returned by read_next_record_bytes() is valid only during Phase 1.
            // We create an owned copy that can be moved into the allow_threads() closure.
            let record_bytes_owned: SmallVec<[u8; 4096]> = SmallVec::from_slice(&record_bytes);

            // ===== PHASE 2: Parse bytes (GIL released) =====
            // Parse the record while GIL is released to allow other threads to execute
            let parse_result = py.allow_threads(|| {
                // This closure runs WITHOUT the GIL held
                // All data here is owned (no Python references)
                let cursor = std::io::Cursor::new(record_bytes_owned.to_vec());
                let mut parser = MarcReader::new(cursor);

                // Parse the single record from bytes
                parser.read_record().map_err(|e| {
                    ParseError::InvalidRecord(format!(
                        "Failed to parse MARC record from {} bytes: {}",
                        record_bytes_owned.len(),
                        e
                    ))
                })
            });

            // ===== PHASE 3: Convert to PyRecord (GIL re-acquired) =====
            // GIL is automatically re-acquired when exiting allow_threads() block
            match parse_result {
                Ok(Some(record)) => Ok(PyRecord { inner: record }),
                Ok(None) => {
                    // Parser returned None (shouldn't happen in middle of record)
                    Err(pyo3::exceptions::PyRuntimeError::new_err(
                        "Parser returned None for complete record",
                    ))
                },
                Err(parse_error) => Err(parse_error.to_py_err()),
            }
        })
    }

    fn __repr__(&self) -> String {
        if self.buffered_reader.is_some() {
            "<MARCReader active>".to_string()
        } else {
            "<MARCReader consumed>".to_string()
        }
    }
}
