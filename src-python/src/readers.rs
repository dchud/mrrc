// Python wrapper for MARCReader with GIL release support
//
// This module implements efficient GIL management for file I/O:
// 1. Read a batch of record bytes from the source (GIL held)
// 2. Parse the whole batch in a single GIL release (py.detach)
// 3. Serve parsed records from a queue, one per __next__ (GIL held)

use crate::backend::ReaderBackend;
use crate::batched_reader::{BatchedReader, RecordOutcome};
use crate::wrappers::PyRecord;
use pyo3::prelude::*;

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
/// - Batch parsing: one `py.detach` per batch (up to 200 records / 300 KB)
///   releases the GIL once for the whole batch, not once per record.
/// - Multiple input types: file paths and bytes parse without touching the
///   GIL; Python file objects read in chunks under a single GIL hold per batch.
#[pyclass(name = "MARCReader")]
pub struct PyMARCReader {
    /// Batched reader over the unified record-byte backend. `None` once the
    /// stream is exhausted or a fatal cap error has consumed the reader.
    /// The reader owns the recovery mode and validation level.
    reader: Option<BatchedReader<ReaderBackend>>,
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
    /// records swallowed by `permissive=True`. Held behind a shared
    /// `Arc` so stashing the most recent chunk costs a refcount bump,
    /// not a byte copy: the same allocation is borrowed by the parser
    /// (via `parse_record_from_shared_bytes`) and retained here.
    last_chunk: Option<std::sync::Arc<Vec<u8>>>,
}

#[pymethods]
impl PyMARCReader {
    /// Create a new MARCReader from any supported source
    ///
    /// Accepts:
    /// - str path (e.g., 'records.mrc') → RustFile backend (no GIL)
    /// - pathlib.Path → RustFile backend (no GIL)
    /// - bytes/bytearray → CursorBackend (no GIL)
    /// - Python file object → chunked Python-file backend (GIL managed)
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
        *,
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

        // One backend handles every source: str/path → RustFile, bytes →
        // Cursor, any .read() object → chunked Python-file. A bad path or
        // unsupported type surfaces its error (FileNotFoundError, TypeError,
        // …) here at construction.
        let backend = ReaderBackend::from_python(source, source.py(), rec_mode)?;
        Ok(PyMARCReader {
            reader: Some(BatchedReader::new(backend, rec_mode, val_level)),
            max_errors,
            accumulated_errors: 0,
            records_yielded: 0,
            last_chunk: None,
        })
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
        self.last_chunk.as_ref().map(|chunk| chunk.as_slice())
    }

    /// Read the next record from the file (legacy interface)
    ///
    /// This method holds the GIL during both reading and parsing.
    /// New code should use iteration (__next__) which supports GIL release.
    ///
    /// Note: serves from the batched reader's parsed-record queue; a parse
    /// that yields no record returns `None` here (EOF-equivalent).
    pub fn read_record(&mut self) -> PyResult<Option<PyRecord>> {
        Python::attach(|py| {
            let outcome = {
                let reader = self
                    .reader
                    .as_mut()
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Reader consumed"))?;
                reader.next_record(py)
            };
            match outcome {
                None => Ok(None),
                Some(outcome) => self.apply_outcome(outcome),
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
    /// - Batch parsing: a whole batch (up to 200 records / 300 KB) is parsed
    ///   in one `py.detach`, releasing the GIL once per batch rather than per
    ///   record. Parsed records are served from an internal queue.
    /// - Multi-backend support: file paths and bytes parse with zero GIL
    ///   overhead; Python files read in chunks under a single GIL hold.
    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRecord> {
        // Obtain a Python handle while the GIL is held by PyRefMut. The
        // batched reader releases the GIL internally (py.detach) to parse a
        // batch, so an unbound handle is required.
        // SAFETY: PyRefMut guarantees the GIL is held for the duration of
        // this call, so the attachment assumption holds. This is the
        // idiomatic way to obtain an unbound handle without re-acquiring,
        // which would panic when the GIL is already held.
        let py = unsafe { Python::assume_attached() };

        let outcome = {
            let reader = slf
                .reader
                .as_mut()
                .ok_or_else(|| pyo3::exceptions::PyStopIteration::new_err(()))?;
            reader.next_record(py)
        };

        let outcome = match outcome {
            Some(outcome) => outcome,
            None => {
                // Clean end of stream — mark the reader consumed.
                slf.reader = None;
                return Err(pyo3::exceptions::PyStopIteration::new_err(()));
            },
        };

        match slf.apply_outcome(outcome)? {
            Some(record) => Ok(record),
            None => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Parser returned None for complete record",
            )),
        }
    }

    /// Return the backend type: "rust_file", "cursor", or "python_file"
    #[getter]
    fn backend_type(&self) -> PyResult<String> {
        match &self.reader {
            Some(reader) => Ok(reader.backend_kind().to_string()),
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
    /// Turn a queued [`RecordOutcome`] into the value `__next__` /
    /// `read_record` return. `Ok(Some(record))` yields a record;
    /// `Ok(None)` means the parser produced no record for a complete slice
    /// (callers diverge: EOF for `read_record`, a runtime error for
    /// `__next__`); `Err` raises. `current_chunk` is updated for every
    /// outcome that carries bytes (success or failure), matching the prior
    /// per-record stash; a source-read error leaves it unchanged.
    fn apply_outcome(&mut self, outcome: RecordOutcome) -> PyResult<Option<PyRecord>> {
        match outcome {
            RecordOutcome::Parsed { bytes, record } => {
                self.last_chunk = Some(bytes);
                // Apply the wrapper-level recovery cap before yielding; if
                // exceeded, surface FatalReaderError and consume the reader
                // so iteration terminates on the next call.
                if let Err(e) = self.note_record_errors(&record) {
                    self.reader = None;
                    return Err(crate::error::marc_error_to_py_err(*e));
                }
                self.records_yielded = self.records_yielded.saturating_add(1);
                Ok(Some(PyRecord::from(record)))
            },
            RecordOutcome::ParseFailed { bytes, error } => {
                self.last_chunk = Some(bytes);
                Err(crate::error::marc_error_to_py_err(*error))
            },
            RecordOutcome::ParseReturnedNone { bytes } => {
                self.last_chunk = Some(bytes);
                Ok(None)
            },
            RecordOutcome::SourceError(e) => Err(e.to_py_err()),
        }
    }

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
            return Err(Box::new(
                mrrc::MarcError::fatal_reader_error(cap, self.accumulated_errors)
                    .with_record_index(Some(self.records_yielded.saturating_add(1))),
            ));
        }
        Ok(())
    }
}
