//! Chunked reader for Python file-like objects.
//!
//! The per-record Python read path used to issue two `file.read()` calls
//! and a fresh `getattr("read")` for every record, then copy the result
//! twice (`extract::<Vec<u8>>` into a `Vec`, then into the record buffer).
//! At ~150 records per 256 KiB that is two GIL-held Python calls per record
//! where one per chunk suffices.
//!
//! `ChunkedPyFileReader` reads large fixed-size chunks into an internal
//! buffer, binds the `read` method once at construction, borrows each
//! chunk through `PyBytes::as_bytes` instead of extracting an owned
//! `Vec`, and slices complete ISO 2709 records out of the buffer in Rust.
//! Record boundaries are found from the leader's 5-digit length field, so
//! records that span a chunk boundary are reassembled transparently.
//!
//! Error behavior matches the previous `backend.rs` Python path exactly:
//! an empty first read is EOF; a partial leader is `Incomplete leader`; a
//! short body is a fatal `TruncatedRecord` (strict) or a pass-through of
//! the partial record to the parser's `record.errors` dispatch
//! (lenient/permissive).

use crate::parse_error::ParseError;
use mrrc::RecoveryMode;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// Bytes requested per `read()` call. One chunk holds roughly 150 average
/// MARC records, so a stream is drained in one Python call per chunk rather
/// than two per record.
const CHUNK_SIZE: usize = 256 * 1024;

/// Buffered reader over a Python file-like object that serves complete
/// ISO 2709 records while reading from the source in large chunks.
#[derive(Debug)]
pub struct ChunkedPyFileReader {
    /// The file object's `read` method, captured once so each refill is a
    /// direct call rather than a per-record `getattr`.
    read_method: Py<PyAny>,
    /// Bytes read from the source but not yet served as records.
    buffer: Vec<u8>,
    /// Offset of the first unconsumed byte in `buffer`.
    pos: usize,
    /// Set once `read()` returns empty; no further Python calls are made.
    eof: bool,
    recovery_mode: RecoveryMode,
}

impl ChunkedPyFileReader {
    /// Bind the object's `read` method once and wrap it for chunked reading.
    pub fn new(file_obj: &Bound<'_, PyAny>, recovery_mode: RecoveryMode) -> PyResult<Self> {
        let read_method = file_obj.getattr("read")?.unbind();
        Ok(ChunkedPyFileReader {
            read_method,
            buffer: Vec::new(),
            pos: 0,
            eof: false,
            recovery_mode,
        })
    }

    /// Unconsumed bytes currently buffered.
    fn available(&self) -> usize {
        self.buffer.len() - self.pos
    }

    /// Read chunks until at least `needed` unconsumed bytes are buffered or
    /// the source is exhausted. Compacts the consumed prefix first so the
    /// buffer does not grow without bound; this runs at most once per chunk
    /// (callers only fill when a record is not already fully buffered).
    fn fill_to(&mut self, py: Python<'_>, needed: usize) -> Result<(), ParseError> {
        if self.pos > 0 {
            self.buffer.drain(..self.pos);
            self.pos = 0;
        }
        while !self.eof && self.buffer.len() < needed {
            let result = self
                .read_method
                .bind(py)
                .call1((CHUNK_SIZE,))
                .map_err(|e| ParseError::io_error(format!("Python read() failed: {}", e)))?;

            // Fast path: a `bytes` return is borrowed and copied once into
            // the buffer. `bytearray`/other buffer types fall back to an
            // owned extract so custom file-likes keep working.
            match result.cast::<PyBytes>() {
                Ok(py_bytes) => {
                    let slice = py_bytes.as_bytes();
                    if slice.is_empty() {
                        self.eof = true;
                        break;
                    }
                    self.buffer.extend_from_slice(slice);
                },
                Err(_) => {
                    let owned: Vec<u8> = result.extract().map_err(|_| {
                        ParseError::invalid_record("read() must return bytes".to_string())
                    })?;
                    if owned.is_empty() {
                        self.eof = true;
                        break;
                    }
                    self.buffer.extend_from_slice(&owned);
                },
            }
        }
        Ok(())
    }

    /// Read the next complete MARC record from the buffered source.
    ///
    /// Returns `Ok(None)` at a clean end of stream, `Ok(Some(bytes))` for a
    /// complete record (or, in lenient/permissive mode, a short final
    /// record the parser will diagnose), and `Err` for a malformed leader
    /// or a strict-mode truncation.
    pub fn read_next_record_bytes(
        &mut self,
        py: Python<'_>,
    ) -> Result<Option<Vec<u8>>, ParseError> {
        // A complete record needs at least the 24-byte leader.
        if self.available() < 24 {
            self.fill_to(py, 24)?;
        }
        let avail = self.available();
        if avail == 0 {
            return Ok(None); // clean EOF on a record boundary
        }
        if avail < 24 {
            // Some bytes, but not a full leader — same shape as the former
            // backend.rs Python path's "Incomplete leader" error.
            let near = &self.buffer[self.pos..];
            return Err(ParseError::invalid_record(format!(
                "Incomplete leader: expected 24 bytes, got {}",
                avail
            ))
            .with_bytes_near(near, 0));
        }

        let record_length = {
            let leader = &self.buffer[self.pos..self.pos + 24];
            let record_length: usize = std::str::from_utf8(&leader[0..5])
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or_else(|| {
                    ParseError::record_length_invalid(&leader[0..5], "5 ASCII digits")
                        .with_bytes_near(leader, 0)
                })?;
            if record_length < 24 {
                return Err(
                    ParseError::record_length_invalid(&leader[0..5], "at least 24")
                        .with_bytes_near(leader, 0),
                );
            }
            record_length
        };

        // Ensure the whole record is buffered before slicing it out.
        if self.available() < record_length {
            self.fill_to(py, record_length)?;
        }
        let take = self.available().min(record_length);
        let record = self.buffer[self.pos..self.pos + take].to_vec();
        self.pos += take;

        if take < record_length {
            // Body fell short of the declared length. The leader (24 bytes)
            // is present; the shortfall is in the body.
            if self.recovery_mode == RecoveryMode::Strict {
                return Err(ParseError::truncated_record(record_length - 24, take - 24)
                    .with_record_byte_offset(24));
            }
            // Lenient/Permissive: hand the leader + partial body to the
            // parser, which dispatches its own E005 onto `record.errors`.
        }

        Ok(Some(record))
    }
}
