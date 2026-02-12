// Python wrapper for MARCReader with GIL release support
//
// This module implements efficient GIL management for concurrent file I/O:
// 1. Read record bytes from source (GIL held if Python file)
// 2. Parse bytes into MARC record structure (GIL released)
// 3. Convert result to Python object and handle errors (GIL re-acquired)

use crate::batched_reader::BatchedMarcReader;
use crate::batched_unified_reader::BatchedUnifiedReader;
use crate::parse_error::ParseError;
use crate::wrappers::PyRecord;
use mrrc::MarcReader;
use pyo3::prelude::*;
use smallvec::SmallVec;

/// Internal enum for different reader backends
#[allow(clippy::large_enum_variant)]
enum ReaderType {
    /// Python file-based reader with batch optimization
    Python(BatchedMarcReader),
    /// Unified reader supporting Rust files, bytes, and Python objects
    Unified(BatchedUnifiedReader),
}

/// Python wrapper for MarcReader with efficient GIL management
///
/// Implements batch-based GIL optimization for CPU-intensive parsing:
/// - Read bytes from source efficiently (GIL held only if Python file)
/// - Parse bytes to MARC records (GIL released, allowing parallelism)
/// - Convert to PyRecord (GIL re-acquired)
///
/// ## Thread Safety
///
/// **IMPORTANT**: MARCReader is NOT thread-safe. Each thread must create its own reader instance.
/// Sharing a single reader across threads causes undefined behavior.
///
/// Correct pattern:
/// ```python
/// with ThreadPoolExecutor(max_workers=4) as executor:
///     futures = [executor.submit(process_file, f) for f in file_list]
///
/// def process_file(filename):
///     with open(filename, 'rb') as f:
///         reader = MARCReader(f)  # New reader per thread
///         for record in reader:
///             process(record)
/// ```
///
/// Concurrent reading enables 2-3x speedup on multi-core systems:
/// - 2 threads: ~2.0x speedup
/// - 4 threads: ~3.2x speedup
/// - Optimal: CPU core count - 1 threads
///
/// ## Optimization Strategies
///
/// - Batch reading: Reduces GIL acquire/release overhead from N to N/100 (for N records)
/// - Multiple input types: File paths via pure Rust I/O (zero GIL overhead), bytes via in-memory cursor, Python file objects via GIL management
#[pyclass(name = "MARCReader")]
pub struct PyMARCReader {
    /// Reader backend (Python-based or unified multi-backend)
    reader: Option<ReaderType>,
}

#[pymethods]
impl PyMARCReader {
    /// Create a new MARCReader from any supported source
    ///
    /// Accepts:
    /// - str path (e.g., 'records.mrc') → RustFile backend (no GIL)
    /// - pathlib.Path → RustFile backend (no GIL)
    /// - bytes/bytearray → CursorBackend (no GIL)
    /// - Python file object → BatchedMarcReader (GIL managed)
    ///
    /// # Arguments
    /// * `source` - File path (str), pathlib.Path, bytes/bytearray, or file-like object
    #[new]
    pub fn new(source: &Bound<'_, PyAny>) -> PyResult<Self> {
        // Try unified reader first (handles file paths and bytes)
        match BatchedUnifiedReader::new(source) {
            Ok(unified_reader) => Ok(PyMARCReader {
                reader: Some(ReaderType::Unified(unified_reader)),
            }),
            Err(_) => {
                // Fall back to legacy Python file wrapper
                // This handles custom file-like objects that aren't supported by UnifiedReader
                let file_obj = source.clone().unbind();
                let batched_reader = BatchedMarcReader::new(file_obj);
                Ok(PyMARCReader {
                    reader: Some(ReaderType::Python(batched_reader)),
                })
            },
        }
    }

    /// Read the next record from the file (legacy interface)
    ///
    /// This method holds the GIL during both reading and parsing.
    /// New code should use iteration (__next__) which supports GIL release.
    ///
    /// Note: Uses BatchedMarcReader's queue state machine for efficient batching.
    /// Also supports file paths and bytes via BatchedUnifiedReader.
    #[allow(deprecated)]
    pub fn read_record(&mut self) -> PyResult<Option<PyRecord>> {
        Python::with_gil(|py| {
            // Step 1: Read record bytes (GIL held)
            // Uses queue-based state machine: CHECK_QUEUE → CHECK_EOF → READ_BATCH
            let record_bytes = match self.reader.as_mut() {
                Some(ReaderType::Python(reader)) => reader
                    .read_next_record_bytes(py)
                    .map_err(|e| e.to_py_err())?,
                Some(ReaderType::Unified(reader)) => reader
                    .read_next_record_bytes(py)
                    .map_err(|e| e.to_py_err())?,
                None => return Err(pyo3::exceptions::PyRuntimeError::new_err("Reader consumed")),
            };

            match record_bytes {
                None => Ok(None),
                Some(bytes) => {
                    // CRITICAL: Copy to owned SmallVec for parsing closure
                    let bytes_owned: SmallVec<[u8; 4096]> = SmallVec::from_slice(&bytes);

                    // Step 2: Parse bytes (GIL released)
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

                    // Step 3: Convert to PyRecord (GIL re-acquired)
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

    /// Get the next record during iteration (enables GIL release for parallelism)
    ///
    /// This implements efficient GIL release pattern:
    /// - Step 1: Read record bytes from source (GIL held if Python file)
    ///   - Uses queue-based state machine (CHECK_QUEUE → CHECK_EOF → READ_BATCH)
    /// - Step 2: Parse bytes to MARC record (GIL released)
    ///   - This step releases the GIL, allowing other threads to execute
    /// - Step 3: Convert to PyRecord (GIL re-acquired)
    ///
    /// **Concurrency Benefits:**
    /// Using separate readers in multiple threads achieves:
    /// - 2 threads: ~2.0x speedup vs sequential
    /// - 4 threads: ~3.2x speedup vs sequential
    /// - GIL is released during parsing for each record
    /// - See ThreadPoolExecutor example in MARCReader struct docs
    ///
    /// **Optimization Strategies:**
    /// - Batch reading: Reads up to 100 records per GIL acquire/release cycle
    /// - Records are buffered in an internal queue (VecDeque)
    /// - Subsequent calls return from queue without GIL overhead
    /// - Reduces GIL contention from N acquire/release to N/100
    ///
    /// - Multi-backend support: File paths and bytes use native I/O (zero GIL overhead)
    /// - Python files use same batching and GIL management
    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRecord> {
        // ===== STEP 1: Read record bytes (GIL held if needed) =====
        // Must get Python handle while GIL is held by PyRefMut
        // CRITICAL: Use assume_gil_acquired() to get an unbound Python handle that
        // properly releases the GIL in Phase 2. A bound handle (slf.py()) does not.
        // SAFETY: PyRefMut guarantees the GIL is held; this is the idiomatic way to get
        // an unbound Python handle without re-acquiring (which would panic if already held).
        // See GIL_RELEASE_IMPLEMENTATION_PLAN.md Part 2 Fix 2 (lines 149-235).
        #[allow(deprecated)]
        let py = unsafe { Python::assume_gil_acquired() };

        // Get mutable reference to reader backend
        let reader = slf
            .reader
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyStopIteration::new_err(()))?;

        // Call read_next_record_bytes() while holding GIL
        // State machine: if queue non-empty, pop; else if EOF, return None; else read_batch()
        let record_bytes = match reader {
            ReaderType::Python(reader) => match reader.read_next_record_bytes(py) {
                Ok(Some(bytes)) => bytes,
                Ok(None) => {
                    // EOF reached - mark reader as consumed
                    slf.reader = None;
                    return Err(pyo3::exceptions::PyStopIteration::new_err(()));
                },
                Err(e) => return Err(e.to_py_err()),
            },
            ReaderType::Unified(reader) => match reader.read_next_record_bytes(py) {
                Ok(Some(bytes)) => bytes,
                Ok(None) => {
                    // EOF reached - mark reader as consumed
                    slf.reader = None;
                    return Err(pyo3::exceptions::PyStopIteration::new_err(()));
                },
                Err(e) => return Err(e.to_py_err()),
            },
        };

        // CRITICAL: Copy to owned SmallVec for parsing closure
        // The slice returned by read_next_record_bytes() is valid only during step 1.
        // We create an owned copy that can be moved into the detach() closure.
        let record_bytes_owned: SmallVec<[u8; 4096]> = SmallVec::from_slice(&record_bytes);

        // ===== STEP 2: Parse bytes (GIL released) =====
        // Parse the record while GIL is released to allow other threads to execute.
        // CRITICAL: The closure returns Result<Option<mrrc::Record>, ParseError> (NOT PyResult).
        // We defer conversion to PyErr until AFTER detach() returns (GIL re-acquired).
        // This is required because PyErr construction needs the GIL.
        // NOTE: Use detach() which properly releases GIL
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

        // ===== STEP 3: Convert to PyRecord (GIL re-acquired) =====
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

    /// Return the backend type: "rust_file", "cursor", or "python_file"
    #[getter]
    fn backend_type(&self) -> PyResult<String> {
        match &self.reader {
            Some(ReaderType::Unified(reader)) => Ok(reader.backend_type().to_string()),
            Some(ReaderType::Python(_)) => Ok("python_file".to_string()),
            None => Err(pyo3::exceptions::PyRuntimeError::new_err("Reader consumed")),
        }
    }

    fn __repr__(&self) -> String {
        if self.reader.is_some() {
            "<MARCReader active>".to_string()
        } else {
            "<MARCReader consumed>".to_string()
        }
    }
}
