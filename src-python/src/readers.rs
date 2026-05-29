// Python wrapper for MARCReader with GIL release support
//
// This module implements efficient GIL management for concurrent file I/O:
// 1. Read record bytes from source (GIL held if Python file)
// 2. Parse bytes into MARC record structure (GIL released)
// 3. Convert result to Python object and handle errors (GIL re-acquired)

use crate::batched_reader::BatchedMarcReader;
use crate::batched_unified_reader::BatchedUnifiedReader;
use crate::wrappers::PyRecord;
use mrrc::{MarcReader, RecoveryMode, ValidationLevel};
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
    /// Recovery mode for handling malformed records
    recovery_mode: RecoveryMode,
    /// What counts as an error during parsing (orthogonal to
    /// `recovery_mode`).
    validation_level: ValidationLevel,
    /// Optional cap on accumulated recovered errors per stream
    /// (lenient/permissive only). `None` (default) disables the
    /// Python-side cap; `Some(0)` matches the Rust API's
    /// no-cap sentinel; `Some(N)` trips `FatalReaderError` (E099)
    /// once the (N+1)-th recovered error lands.
    ///
    /// The cap is implemented here rather than threaded into the
    /// per-record `MarcReader` because each parse uses an ephemeral
    /// reader instance; the Rust-side cap would reset every record
    /// and never trip. We accumulate `record.errors.len()` across
    /// successive records and raise `FatalReaderError` once the cap
    /// is exceeded — observationally equivalent to the Rust cap for
    /// users of the Python `MARCReader`.
    max_errors: Option<usize>,
    /// Running count of recovered errors across all records read so
    /// far. Used together with `max_errors` to trip the wrapper-level
    /// recovery cap. Reset to zero at construction.
    accumulated_errors: usize,
    /// 1-based count of records yielded to the caller. Used as the
    /// `record_index` in the synthesized `FatalReaderError` when the
    /// cap trips.
    records_yielded: usize,
    /// Bytes of the most recent record chunk read from the source,
    /// before any parse attempt. Populated on every `__next__` /
    /// `read_record` call that successfully reads a chunk (regardless
    /// of whether the parse subsequently succeeds or fails). The
    /// `mrrc.MARCReader` Python wrapper exposes this as the
    /// pymarc-compatible `current_chunk` accessor used to diagnose
    /// records swallowed by `permissive=True`.
    last_chunk: Option<Vec<u8>>,
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
    /// * `recovery_mode` - Error handling mode: 'permissive' (default),
    ///   'lenient', 'strict'. The Python user surface defaults to
    ///   permissive for pymarc-shape parity; the Rust core defaults
    ///   to strict for explicit-error-handling parity with Rust idiom.
    /// * `validation_level` - What counts as an error during parsing:
    ///   'structural' (default) or 'strict_marc'. Orthogonal to
    ///   `recovery_mode`.
    /// * `max_errors` - Optional cap on accumulated recovered errors
    ///   per stream in lenient/permissive mode. `None` (default)
    ///   disables the wrapper-level cap; `0` also disables it (no-cap
    ///   sentinel). `Some(N)` trips `FatalReaderError` (E099) once more
    ///   than N recovered errors accumulate across the stream. (Each
    ///   record is parsed by an ephemeral reader, so the Rust core's
    ///   per-record `DEFAULT_MAX_ERRORS` never accumulates across the
    ///   stream — this wrapper cap is the only cross-stream limit.)
    #[new]
    #[pyo3(signature = (
        source,
        recovery_mode = "permissive",
        validation_level = "structural",
        max_errors = None,
    ))]
    pub fn new(
        source: &Bound<'_, PyAny>,
        recovery_mode: &str,
        validation_level: &str,
        max_errors: Option<usize>,
    ) -> PyResult<Self> {
        let rec_mode = crate::reader_helpers::parse_recovery_mode(recovery_mode)?;
        let val_level = crate::reader_helpers::parse_validation_level(validation_level)?;

        // Try unified reader first (handles file paths and bytes)
        match BatchedUnifiedReader::new(source, rec_mode) {
            Ok(unified_reader) => Ok(PyMARCReader {
                reader: Some(ReaderType::Unified(unified_reader)),
                recovery_mode: rec_mode,
                validation_level: val_level,
                max_errors,
                accumulated_errors: 0,
                records_yielded: 0,
                last_chunk: None,
            }),
            Err(_) => {
                // Fall back to legacy Python file wrapper
                // This handles custom file-like objects that aren't supported by UnifiedReader
                let file_obj = source.clone().unbind();
                let batched_reader = BatchedMarcReader::new(file_obj, rec_mode);
                Ok(PyMARCReader {
                    reader: Some(ReaderType::Python(batched_reader)),
                    recovery_mode: rec_mode,
                    validation_level: val_level,
                    max_errors,
                    accumulated_errors: 0,
                    records_yielded: 0,
                    last_chunk: None,
                })
            },
        }
    }

    /// Bytes of the most recent record chunk read from the source.
    ///
    /// Populated on every successful chunk read, regardless of whether
    /// the parse step then succeeds or fails. Cleared to `None` only
    /// at construction; on EOF the prior value is retained so callers
    /// can still inspect the last attempted record. Used by the
    /// `mrrc.MARCReader` Python wrapper to expose pymarc-compatible
    /// `current_chunk` semantics.
    #[getter]
    fn last_chunk(&self) -> Option<&[u8]> {
        self.last_chunk.as_deref()
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
        Python::attach(|py| {
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
                    // Stash the chunk for pymarc-compat `current_chunk`. Set
                    // unconditionally — success and failure callers both want
                    // it (success readers can compare; failure readers can
                    // diagnose).
                    self.last_chunk = Some(bytes_owned.to_vec());

                    // Step 2: Parse bytes (GIL released). Return the raw
                    // MarcError across the GIL boundary so the typed variant
                    // and all positional context (byte_offset, bytes_near,
                    // etc.) survive to the Python exception. Boxed to keep
                    // the Result small (clippy::result_large_err).
                    let rec_mode = self.recovery_mode;
                    let val_level = self.validation_level;
                    let parse_result: Result<Option<mrrc::Record>, Box<mrrc::MarcError>> = py
                        .detach(|| {
                            let cursor = std::io::Cursor::new(bytes_owned.to_vec());
                            let mut parser = MarcReader::new(cursor)
                                .with_recovery_mode(rec_mode)
                                .with_validation_level(val_level);
                            parser.read_record().map_err(Box::new)
                        });
                    let record =
                        parse_result.map_err(|e| crate::error::marc_error_to_py_err(*e))?;

                    // Step 3: Convert to PyRecord (GIL re-acquired)
                    match record {
                        Some(r) => {
                            // Apply the wrapper-level recovery cap before
                            // handing the record back: if max_errors was
                            // configured and this record's accumulated
                            // count exceeds it, surface FatalReaderError
                            // instead of yielding.
                            if let Err(e) = self.note_record_errors(&r) {
                                self.reader = None;
                                return Err(crate::error::marc_error_to_py_err(*e));
                            }
                            self.records_yielded = self.records_yielded.saturating_add(1);
                            Ok(Some(PyRecord { inner: r }))
                        },
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
        // CRITICAL: Use assume_attached() to get an unbound Python handle that
        // properly releases the GIL in Phase 2. A bound handle (slf.py()) does not.
        // SAFETY: PyRefMut guarantees the GIL is held; this is the idiomatic way to get
        // an unbound Python handle without re-acquiring (which would panic if already held).
        // See GIL_RELEASE_IMPLEMENTATION_PLAN.md Part 2 Fix 2 (lines 149-235).
        let py = unsafe { Python::assume_attached() };

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
        // Stash the chunk for pymarc-compat `current_chunk`. Set
        // unconditionally — both the parse-succeeds and parse-fails
        // exit paths in step 3 want this populated.
        slf.last_chunk = Some(record_bytes_owned.to_vec());

        // ===== STEP 2: Parse bytes (GIL released) =====
        // Parse the record while GIL is released to allow other threads to execute.
        // CRITICAL: The closure returns Result<Option<mrrc::Record>, MarcError>
        // (NOT PyResult). We defer conversion to PyErr until AFTER detach()
        // returns (GIL re-acquired) because PyErr construction needs the GIL.
        // Returning the raw MarcError preserves the typed variant and all
        // positional context (byte_offset, bytes_near, etc.) through to the
        // Python exception — collapsing to a generic ParseError::invalid_record
        // here would lose the typed shape and drop `bytes_near`.
        let rec_mode = slf.recovery_mode;
        let val_level = slf.validation_level;
        let parse_result: Result<Option<mrrc::Record>, Box<mrrc::MarcError>> = py.detach(|| {
            let cursor = std::io::Cursor::new(record_bytes_owned.to_vec());
            let mut parser = MarcReader::new(cursor)
                .with_recovery_mode(rec_mode)
                .with_validation_level(val_level);
            parser.read_record().map_err(Box::new)
        });

        // ===== STEP 3: Convert to PyRecord (GIL re-acquired) =====
        // GIL is automatically re-acquired when exiting detach() block.
        // NOW we can safely construct PyErr from MarcError.
        match parse_result {
            Ok(Some(record)) => {
                // Apply the wrapper-level recovery cap before yielding;
                // if exceeded, swap in FatalReaderError and mark the
                // reader consumed so the iterator terminates on the
                // next call.
                if let Err(e) = slf.note_record_errors(&record) {
                    slf.reader = None;
                    return Err(crate::error::marc_error_to_py_err(*e));
                }
                slf.records_yielded = slf.records_yielded.saturating_add(1);
                Ok(PyRecord { inner: record })
            },
            Ok(None) => {
                // Parser returned None (shouldn't happen in middle of record)
                Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Parser returned None for complete record",
                ))
            },
            Err(marc_error) => Err(crate::error::marc_error_to_py_err(*marc_error)),
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

impl PyMARCReader {
    /// Account for a newly-parsed record against the wrapper-level
    /// recovery cap. Adds the record's recovered errors to the
    /// running count and, if the cap has been exceeded, builds the
    /// matching `FatalReaderError` to surface as the iterator's
    /// next exception. The caller is responsible for marking the
    /// reader consumed after this returns `Err` so subsequent reads
    /// terminate the iterator.
    ///
    /// Returns `Ok(())` when the cap is disabled (`max_errors=None`
    /// or `Some(0)`) or still under the configured limit.
    ///
    /// Lives in a non-`#[pymethods]` impl block so PyO3 does not try
    /// to expose it as a Python method (the `&mrrc::Record` argument
    /// is not a Python-extractable type).
    fn note_record_errors(&mut self, record: &mrrc::Record) -> Result<(), Box<mrrc::MarcError>> {
        let Some(cap) = self.max_errors else {
            return Ok(());
        };
        if cap == 0 {
            return Ok(());
        }
        self.accumulated_errors = self.accumulated_errors.saturating_add(record.errors.len());
        if self.accumulated_errors > cap {
            return Err(Box::new(mrrc::MarcError::FatalReaderError {
                cap,
                errors_seen: self.accumulated_errors,
                record_index: Some(self.records_yielded.saturating_add(1)),
                source_name: None,
            }));
        }
        Ok(())
    }
}
