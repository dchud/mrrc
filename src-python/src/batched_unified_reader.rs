//! Batched reader supporting both Rust file I/O and Python file-like objects
//!
//! Maintains a queue-based state machine with support for multiple input types:
//! - File paths: native Rust I/O (zero GIL overhead)
//! - Bytes/BytesIO: in-memory cursor (zero GIL overhead)
//! - Python file objects: GIL management for compatibility
//!
//! Design:
//! - `VecDeque<SmallVec>` for O(1) front-pop during iteration
//! - Hard limits: 200 records or 300KB per batch
//! - EOF state machine ensures idempotent behavior
//! - Single GIL acquire/release cycle per batch (for Python files only)

use crate::parse_error::ParseError;
use crate::unified_reader::UnifiedReader;
use mrrc::RecoveryMode;
use pyo3::prelude::*;
use smallvec::SmallVec;
use std::collections::VecDeque;

/// State machine for batched MARC reading from unified sources
///
/// Supports both Rust file I/O (no GIL) and Python file objects (with GIL management).
#[derive(Debug)]
pub struct BatchedUnifiedReader {
    /// Unified reader handling both Rust and Python backends
    unified_reader: UnifiedReader,

    /// Queue of record bytes, ready for parsing
    /// Using VecDeque for O(1) pop_front() performance
    record_queue: VecDeque<SmallVec<[u8; 4096]>>,

    /// Cumulative capacity tracking (bytes in queue)
    queue_capacity_bytes: usize,

    /// EOF flag - once set, reading stops and __next__ returns None idempotently
    eof_reached: bool,

    /// Target batch size for read_batch() calls
    /// Fixed at 100 for initial implementation (validated in C.Gate)
    batch_size: usize,

    /// Count of records successfully read from the source so far.
    /// Used to attach `record_index` (1-based) to errors fired during
    /// the next record's read attempt.
    records_read: usize,

    /// Absolute byte offset of the start of the next record to read,
    /// equivalently: the total byte count of all successfully-read
    /// records emitted so far. Used to attach `byte_offset` to errors
    /// when the leaf reader knows only its record-relative offset.
    bytes_consumed: usize,
}

impl BatchedUnifiedReader {
    /// Create a new BatchedUnifiedReader from a Python object
    ///
    /// Type detection routes to appropriate backend:
    /// - str/pathlib.Path → RustFile (no GIL)
    /// - bytes/bytearray → CursorBackend (no GIL)
    /// - file-like object → PythonFile (GIL required)
    ///
    /// # Arguments
    /// * `source` - Python object (str, Path, bytes, bytearray, or file-like)
    ///
    /// # Errors
    /// - `TypeError` if input type is not supported
    /// - `FileNotFoundError` if file path doesn't exist
    /// - `IOError` if file cannot be opened (only for file paths)
    pub fn new(source: &Bound<'_, PyAny>, recovery_mode: RecoveryMode) -> PyResult<Self> {
        let unified_reader = UnifiedReader::from_python(source, recovery_mode)?;

        Ok(BatchedUnifiedReader {
            unified_reader,
            record_queue: VecDeque::new(),
            queue_capacity_bytes: 0,
            eof_reached: false,
            batch_size: 100,
            records_read: 0,
            bytes_consumed: 0,
        })
    }

    /// Read the next record bytes from the queue or file
    ///
    /// Implements state machine:
    /// 1. If queue not empty: pop and return
    /// 2. If EOF reached: return None (idempotent)
    /// 3. Otherwise: read_batch() and process
    ///
    /// Returns:
    /// - Ok(Some(bytes)) - A complete MARC record as owned bytes
    /// - Ok(None) - End of file reached (subsequent calls also return None)
    /// - Err(ParseError) - I/O or parsing boundary error
    pub fn read_next_record_bytes(
        &mut self,
        py: Python<'_>,
    ) -> Result<Option<Vec<u8>>, ParseError> {
        // === STATE: CHECK_QUEUE_NON_EMPTY ===
        if !self.record_queue.is_empty() {
            if let Some(record) = self.record_queue.pop_front() {
                self.queue_capacity_bytes = self.queue_capacity_bytes.saturating_sub(record.len());
                return Ok(Some(record.to_vec()));
            }
        }

        // === STATE: CHECK_EOF_STATE ===
        if self.eof_reached {
            // Idempotent: subsequent calls return None without I/O
            return Ok(None);
        }

        // === STATE: READ_BATCH ===
        let batch = self.read_batch(py)?;

        if batch.is_empty() {
            // EOF reached during batch read
            self.eof_reached = true;
            return Ok(None);
        }

        // Non-empty batch: extend queue and pop first record
        for record in batch {
            self.queue_capacity_bytes = self.queue_capacity_bytes.saturating_add(record.len());
            self.record_queue.push_back(record);
        }

        // Re-check queue (should not be empty now)
        if let Some(record) = self.record_queue.pop_front() {
            self.queue_capacity_bytes = self.queue_capacity_bytes.saturating_sub(record.len());
            Ok(Some(record.to_vec()))
        } else {
            // Defensive: should never happen if batch was non-empty
            Ok(None)
        }
    }

    /// Read a batch of records with a single GIL acquire/release cycle
    ///
    /// # GIL Contract
    /// - Entry: py parameter guarantees GIL is held
    /// - For RustFile/CursorBackend: reads proceed without GIL overhead
    /// - For PythonFile: uses GIL to call .read() method
    /// - Exit: GIL state depends on backend type
    ///
    /// # Batch Size Semantics
    /// - Target: 100 records per batch
    /// - Hard limits:
    ///   - 200 records maximum (prevents unbounded allocation)
    ///   - 300 KB maximum (prevents memory spikes on large records)
    /// - If limits are hit, returns partial batch (variable size)
    /// - Caller continues with next read_batch() call
    ///
    /// # Returns
    /// Vec<SmallVec<[u8; 4096]>> - Records read in this batch (may be < batch_size)
    /// Empty vec indicates EOF (no more records available)
    fn read_batch(&mut self, py: Python<'_>) -> Result<Vec<SmallVec<[u8; 4096]>>, ParseError> {
        let mut batch = Vec::with_capacity(self.batch_size);
        let mut bytes_read = 0usize;

        // Read up to batch_size records, respecting hard limits
        while batch.len() < self.batch_size {
            // Hard limit: 200 records or 300KB
            if batch.len() >= 200 || bytes_read > 300_000 {
                break;
            }

            match self.unified_reader.read_next_bytes(py) {
                Ok(Some(record_bytes)) => {
                    bytes_read = bytes_read.saturating_add(record_bytes.len());
                    self.records_read = self.records_read.saturating_add(1);
                    self.bytes_consumed = self.bytes_consumed.saturating_add(record_bytes.len());
                    batch.push(SmallVec::from_slice(&record_bytes));
                },
                Ok(None) => {
                    // EOF reached during batch read
                    break;
                },
                Err(e) => {
                    // Attach the inter-record positional context the
                    // leaf reader couldn't see: which record (1-based)
                    // the error was raised against, and the absolute
                    // stream byte offset of the start of that record's
                    // body (the leaf reader sets `record_byte_offset`
                    // for the within-record position).
                    let next_record_index = self.records_read.saturating_add(1);
                    let absolute_offset = match e.context.record_byte_offset {
                        Some(intra) => self.bytes_consumed.saturating_add(intra),
                        None => self.bytes_consumed,
                    };
                    return Err(e
                        .with_record_index(next_record_index)
                        .with_byte_offset(absolute_offset));
                },
            }
        }

        Ok(batch)
    }

    /// Return the backend type as a string for diagnostics
    pub fn backend_type(&self) -> &str {
        self.unified_reader.backend_type()
    }

    /// Check if the reader has reached EOF
    ///
    /// Returns true after EOF is set (idempotent).
    #[allow(dead_code)]
    pub fn is_eof(&self) -> bool {
        self.eof_reached
    }

    /// Get current queue size (for diagnostics/testing)
    #[allow(dead_code)]
    pub fn queue_len(&self) -> usize {
        self.record_queue.len()
    }

    /// Get current queue capacity in bytes (for diagnostics/testing)
    #[allow(dead_code)]
    pub fn queue_capacity_bytes(&self) -> usize {
        self.queue_capacity_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batched_unified_reader_queue_operations() {
        // Create minimal records for testing
        let r1: SmallVec<[u8; 4096]> = SmallVec::from_slice(b"record1");
        let r2: SmallVec<[u8; 4096]> = SmallVec::from_slice(b"record2");

        let mut queue = VecDeque::new();
        queue.push_back(r1);
        queue.push_back(r2);

        assert_eq!(queue.len(), 2);
    }
}
