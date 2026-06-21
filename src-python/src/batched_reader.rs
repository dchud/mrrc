//! Batched MARC reader: one queue/parse state machine over any record-byte
//! source.
//!
//! [`BatchedReader`] reads a bounded batch of record byte-slices from a
//! [`RecordByteSource`] while the GIL is held, parses the **whole batch in a
//! single `py.detach`** (one GIL release per batch, not per record), and
//! serves the parsed outcomes from an internal queue. Each queue element is
//! a parsed [`mrrc::Record`] (or its typed error) paired with the source
//! bytes for the pymarc-compatible `current_chunk` accessor — never a copied
//! byte buffer.
//!
//! This replaces two near-identical queue state machines (the former
//! `batched_reader.rs` over a Python-file wrapper and `batched_unified_reader.rs`
//! over a unified backend) with one generic implementation. Record-reading is
//! delegated to the [`RecordByteSource`]; everything here — batching, the
//! 200-record / 300 KB bounds, offset tracking, and parsing — is shared.

use crate::backend::RecordByteSource;
use crate::parse_error::ParseError;
use mrrc::{MarcError, Record, RecoveryMode, ValidationLevel};
use std::collections::VecDeque;
use std::sync::Arc;

use pyo3::Python;

/// Maximum records buffered in one batch (bounds queue allocation).
const MAX_RECORDS_PER_BATCH: usize = 200;
/// Maximum bytes read into one batch (bounds memory on large records).
const MAX_BYTES_PER_BATCH: usize = 300_000;
/// Target records per batch before the hard limits apply.
const TARGET_BATCH_SIZE: usize = 100;

/// One record's outcome, queued in source order.
///
/// `Parsed` and `ParseFailed` both carry the source bytes behind an `Arc`
/// so the caller can expose them as `current_chunk` (the parser shares the
/// same allocation — no copy). `SourceError` carries no bytes: the source
/// could not produce a complete record.
#[derive(Debug)]
pub enum RecordOutcome {
    /// A successfully parsed record and the bytes it parsed from.
    Parsed { bytes: Arc<Vec<u8>>, record: Record },
    /// The parser rejected otherwise-readable bytes (strict-mode structural
    /// defect, or an unrecoverable error in lenient/permissive).
    ParseFailed {
        bytes: Arc<Vec<u8>>,
        error: Box<MarcError>,
    },
    /// The parser returned no record for a complete byte-slice. Defensive:
    /// the byte reader never yields empty record bytes, so this is
    /// unreachable in practice; preserved so each caller keeps its prior
    /// handling (EOF for `read_record`, a runtime error for `__next__`).
    ParseReturnedNone { bytes: Arc<Vec<u8>> },
    /// The source failed to read the next record's bytes (I/O, boundary, or
    /// a strict-mode truncation), already annotated with stream position.
    SourceError(ParseError),
}

/// Batched, parse-on-read state machine over a [`RecordByteSource`].
#[derive(Debug)]
pub struct BatchedReader<S: RecordByteSource> {
    /// Underlying source of one record's bytes at a time.
    source: S,
    /// Parsed outcomes ready to serve, in source order.
    queue: VecDeque<RecordOutcome>,
    /// Once set, the source is exhausted and no further reads are issued.
    eof: bool,
    recovery_mode: RecoveryMode,
    validation_level: ValidationLevel,
    /// Count of records successfully read from the source so far. Used to
    /// stamp `record_index` (1-based) onto a source error.
    records_read: usize,
    /// Absolute byte offset of the next record to read — the total bytes of
    /// all records read so far. Used to stamp `byte_offset` onto a source
    /// error whose leaf reader knows only its record-relative offset.
    bytes_consumed: usize,
}

impl<S: RecordByteSource> BatchedReader<S> {
    /// Wrap a record-byte source with batching and parsing.
    pub fn new(source: S, recovery_mode: RecoveryMode, validation_level: ValidationLevel) -> Self {
        BatchedReader {
            source,
            queue: VecDeque::new(),
            eof: false,
            recovery_mode,
            validation_level,
            records_read: 0,
            bytes_consumed: 0,
        }
    }

    /// Backend kind for diagnostics ("rust_file" | "cursor" | "python_file").
    pub fn backend_kind(&self) -> &'static str {
        self.source.backend_kind()
    }

    /// Serve the next parsed record outcome.
    ///
    /// Pops from the queue; when the queue is empty and the source is not
    /// yet exhausted, reads and parses one batch first. Returns `None` only
    /// at a clean end of stream.
    pub fn next_record(&mut self, py: Python<'_>) -> Option<RecordOutcome> {
        if let Some(outcome) = self.queue.pop_front() {
            return Some(outcome);
        }
        if self.eof {
            return None;
        }
        self.fill_batch(py);
        self.queue.pop_front()
    }

    /// Read up to one batch of record bytes (GIL held), parse them all in a
    /// single `py.detach`, and enqueue the outcomes in source order.
    fn fill_batch(&mut self, py: Python<'_>) {
        // === Phase 1: read record bytes (GIL held) ===
        let mut batch_bytes: Vec<Arc<Vec<u8>>> = Vec::with_capacity(TARGET_BATCH_SIZE);
        let mut batch_byte_total = 0usize;
        let mut source_error: Option<ParseError> = None;

        while batch_bytes.len() < MAX_RECORDS_PER_BATCH && batch_byte_total <= MAX_BYTES_PER_BATCH {
            match self.source.next_record_bytes(py) {
                Ok(Some(bytes)) => {
                    self.records_read = self.records_read.saturating_add(1);
                    self.bytes_consumed = self.bytes_consumed.saturating_add(bytes.len());
                    batch_byte_total = batch_byte_total.saturating_add(bytes.len());
                    batch_bytes.push(Arc::new(bytes));
                },
                Ok(None) => {
                    self.eof = true;
                    break;
                },
                Err(e) => {
                    // Annotate with the inter-record position the leaf reader
                    // can't see: which record (1-based) failed and the
                    // absolute stream offset of that record's start, plus any
                    // record-relative offset the leaf set.
                    let next_record_index = self.records_read.saturating_add(1);
                    let absolute_offset = match e.context.record_byte_offset {
                        Some(intra) => self.bytes_consumed.saturating_add(intra),
                        None => self.bytes_consumed,
                    };
                    source_error = Some(
                        e.with_record_index(next_record_index)
                            .with_byte_offset(absolute_offset),
                    );
                    break;
                },
            }
        }

        // === Phase 2: parse the whole batch in one GIL release ===
        let recovery_mode = self.recovery_mode;
        let validation_level = self.validation_level;
        let parsed: Vec<Result<Option<Record>, Box<MarcError>>> = if batch_bytes.is_empty() {
            Vec::new()
        } else {
            py.detach(|| {
                batch_bytes
                    .iter()
                    .map(|bytes| {
                        mrrc::parse_record_from_shared_bytes(bytes, recovery_mode, validation_level)
                            .map_err(Box::new)
                    })
                    .collect()
            })
        };

        // === Phase 3: enqueue outcomes in source order (GIL re-acquired) ===
        for (bytes, result) in batch_bytes.into_iter().zip(parsed) {
            let outcome = match result {
                Ok(Some(record)) => RecordOutcome::Parsed { bytes, record },
                Ok(None) => RecordOutcome::ParseReturnedNone { bytes },
                Err(error) => RecordOutcome::ParseFailed { bytes, error },
            };
            self.queue.push_back(outcome);
        }
        if let Some(error) = source_error {
            self.queue.push_back(RecordOutcome::SourceError(error));
        }
    }
}
