// Python wrapper for MARCReader with GIL release support
//
// This module implements three-phase GIL management for concurrent file I/O:
// - Phase 1 (GIL held): Read record bytes from Python file object
// - Phase 2 (GIL released): Parse bytes into MARC record structure
// - Phase 3 (GIL held): Convert result to Python object and handle errors

#![allow(deprecated)]

use crate::batched_reader::BatchedMarcReader;
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
///
/// Phase C Enhancement (C.2):
/// - Uses BatchedMarcReader for queue-based state machine
/// - Reduces GIL acquire/release overhead from N to N/100 (for N records)
/// - Reads 100 records per GIL acquisition, serves from queue
#[pyclass(name = "MARCReader")]
pub struct PyMARCReader {
    /// Batched reader for efficient queue-based record delivery
    /// Implements state machine: CHECK_QUEUE → CHECK_EOF → READ_BATCH
    batched_reader: Option<BatchedMarcReader>,
}

#[pymethods]
impl PyMARCReader {
    /// Create a new MARCReader from a Python file-like object
    ///
    /// # Arguments
    /// * `file` - A Python file-like object (must support .read(n) method)
    #[new]
    pub fn new(file: Py<PyAny>) -> PyResult<Self> {
        // Initialize BatchedMarcReader (Phase C: queue-based state machine)
        let batched_reader = BatchedMarcReader::new(file);

        Ok(PyMARCReader {
            batched_reader: Some(batched_reader),
        })
    }

    /// Read the next record from the file (legacy interface)
    ///
    /// This method holds the GIL during both reading and parsing.
    /// New code should use iteration (__next__) which supports GIL release.
    ///
    /// Note: With Phase C (C.2), this now uses BatchedMarcReader's queue state machine.
    pub fn read_record(&mut self) -> PyResult<Option<PyRecord>> {
        Python::with_gil(|py| {
            // Phase 1: Read record bytes (GIL held)
            // Uses queue-based state machine: CHECK_QUEUE → CHECK_EOF → READ_BATCH
            let record_bytes = self
                .batched_reader
                .as_mut()
                .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Reader consumed"))?
                .read_next_record_bytes(py)
                .map_err(|e| e.to_py_err())?;

            match record_bytes {
                None => Ok(None),
                Some(bytes) => {
                    // CRITICAL: Copy to owned SmallVec for Phase 2 closure
                    let bytes_owned: SmallVec<[u8; 4096]> = SmallVec::from_slice(&bytes);

                    // Phase 2: Parse bytes (GIL released)
                    // CRITICAL FIX: Use Python::detach() which properly releases the GIL.
                    let record = py
                        .detach(|| {
                            // Create a cursor from owned bytes
                            let cursor = std::io::Cursor::new(bytes_owned.to_vec());
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
    ///   - Uses queue-based state machine (CHECK_QUEUE → CHECK_EOF → READ_BATCH)
    /// - Phase 2: Parse bytes to MARC record (GIL released)
    /// - Phase 3: Convert to PyRecord (GIL re-acquired)
    ///
    /// Phase C Enhancement (C.2):
    /// - Reads up to 100 records per GIL acquire/release cycle
    /// - Records are buffered in an internal queue (VecDeque)
    /// - Subsequent calls return from queue without GIL overhead
    /// - Reduces GIL contention from N acquire/release to N/100
    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRecord> {
        // ===== PHASE 1: Read record bytes (GIL held) =====
        // Must get Python handle while GIL is held by PyRefMut
        // CRITICAL: Use assume_gil_acquired() to get an unbound Python handle that
        // properly releases the GIL in Phase 2. A bound handle (slf.py()) does not.
        // SAFETY: PyRefMut guarantees the GIL is held; this is the idiomatic way to get
        // an unbound Python handle without re-acquiring (which would panic if already held).
        // See GIL_RELEASE_IMPLEMENTATION_PLAN.md Part 2 Fix 2 (lines 149-235).
        let py = unsafe { Python::assume_gil_acquired() };

        // Get mutable reference to batched reader
        let reader = slf
            .batched_reader
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyStopIteration::new_err(()))?;

        // Call read_next_record_bytes() while holding GIL
        // State machine: if queue non-empty, pop; else if EOF, return None; else read_batch()
        let record_bytes = match reader.read_next_record_bytes(py) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                // EOF reached - mark reader as consumed
                slf.batched_reader = None;
                return Err(pyo3::exceptions::PyStopIteration::new_err(()));
            },
            Err(e) => return Err(e.to_py_err()),
        };

        // CRITICAL: Copy to owned SmallVec for Phase 2 closure
        // The slice returned by read_next_record_bytes() is valid only during Phase 1.
        // We create an owned copy that can be moved into the detach() closure.
        let record_bytes_owned: SmallVec<[u8; 4096]> = SmallVec::from_slice(&record_bytes);

        // ===== PHASE 2: Parse bytes (GIL released) =====
        // Parse the record while GIL is released to allow other threads to execute.
        // CRITICAL: The closure returns Result<Option<mrrc::Record>, ParseError> (NOT PyResult).
        // We defer conversion to PyErr until AFTER detach() returns (GIL re-acquired).
        // This is required because PyErr construction needs the GIL.
        // NOTE: Use detach() instead of allow_threads() - detach() properly releases GIL in PyO3 0.27
        let parse_result: Result<Option<mrrc::Record>, crate::parse_error::ParseError> =
            py.detach(|| {
                // This closure runs WITHOUT the GIL held
                // All data here is owned (no Python references)
                // Return Rust errors only; defer PyErr conversion to Phase 3
                let cursor = std::io::Cursor::new(record_bytes_owned.to_vec());
                let mut parser = MarcReader::new(cursor);

                // Parse the single record from bytes
                // Return ParseError (Rust type), not PyErr
                parser.read_record().map_err(|e| {
                    ParseError::InvalidRecord(format!(
                        "Failed to parse MARC record from {} bytes: {}",
                        record_bytes_owned.len(),
                        e
                    ))
                })
            });

        // ===== PHASE 3: Convert to PyRecord (GIL re-acquired) =====
        // GIL is automatically re-acquired when exiting detach() block.
        // NOW we can safely construct PyErr from ParseError.
        match parse_result {
            Ok(Some(record)) => Ok(PyRecord { inner: record }),
            Ok(None) => {
                // Parser returned None (shouldn't happen in middle of record)
                Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Parser returned None for complete record",
                ))
            },
            Err(parse_error) => {
                // Convert ParseError to PyErr AFTER GIL is re-acquired
                Err(parse_error.to_py_err())
            },
        }
    }

    fn __repr__(&self) -> String {
        if self.batched_reader.is_some() {
            "<MARCReader active>".to_string()
        } else {
            "<MARCReader consumed>".to_string()
        }
    }
}
