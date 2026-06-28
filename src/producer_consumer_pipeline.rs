//! Producer-Consumer Pipeline with Backpressure
//!
//! Implements a bounded channel pipeline for high-performance batch reading
//! from Rust file I/O backend (no GIL overhead).
//!
//! Design:
//! - **Producer:** Background task reading file chunks, scanning boundaries, parsing batches
//! - **Consumer:** Python-facing iterator that drains the bounded channel
//! - **Backpressure:** Channel holds a small number of parsed batches; blocks the producer when full
//! - **GIL:** Producer runs without GIL; consumer manages GIL on retrieval

use crate::boundary_scanner::RecordBoundaryScanner;
use crate::rayon_parser_pool::parse_batch_parallel;
use crate::record::Record;
use crossbeam_channel::{Receiver, Sender, bounded};
use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread;

/// Configuration for the producer-consumer pipeline
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Buffer size for file I/O (bytes)
    pub buffer_size: usize,
    /// Channel capacity in parsed batches (one batch is one parsed file chunk)
    pub channel_capacity: usize,
    /// Batch size for parser pool
    pub batch_size: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            buffer_size: 512 * 1024, // 512 KB
            channel_capacity: 4,     // up to 4 parsed batches buffered
            batch_size: 100,         // 100 records per batch
        }
    }
}

/// Result type for pipeline operations
pub type PipelineResult<T> = Result<T, PipelineError>;

/// Errors that can occur during pipeline operations
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum PipelineError {
    /// I/O error during file reading
    IoError(String),
    /// Parsing error during record boundary scanning
    ScanError(String),
    /// Parsing error during record parsing
    ParseError(String),
    /// Channel send error (producer panicked)
    ChannelSendError,
    /// Channel receive error
    ChannelRecvError,
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::IoError(msg) => write!(f, "I/O error: {msg}"),
            PipelineError::ScanError(msg) => write!(f, "Boundary scan error: {msg}"),
            PipelineError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            PipelineError::ChannelSendError => {
                write!(f, "Channel send error (producer panicked)")
            },
            PipelineError::ChannelRecvError => write!(f, "Channel receive error"),
        }
    }
}

impl std::error::Error for PipelineError {}

/// Producer task: reads file, scans boundaries, parses in parallel, sends to channel
fn producer_task(
    file: File,
    sender: &Sender<Vec<Record>>,
    config: &PipelineConfig,
) -> PipelineResult<()> {
    let mut file = file;
    let mut buffer = vec![0u8; config.buffer_size];
    let mut scanner = RecordBoundaryScanner::new();
    let mut leftover = Vec::new(); // Buffer for partial records from previous chunk

    loop {
        // Read next chunk
        let n = file
            .read(&mut buffer)
            .map_err(|e| PipelineError::IoError(e.to_string()))?;

        if n == 0 {
            // EOF reached - if there's leftover data, it's an incomplete record
            break;
        }

        // Concatenate leftover from previous chunk with current chunk
        let mut current_buffer = leftover.clone();
        current_buffer.extend_from_slice(&buffer[..n]);

        // Scan record boundaries - this may fail if the entire buffer is a partial record
        match scanner.scan(&current_buffer) {
            Ok(boundaries) => {
                // Check if the last boundary is complete (ends at buffer end with 0x1D)
                let all_complete = if let Some(&(offset, len)) = boundaries.last() {
                    offset + len == current_buffer.len()
                } else {
                    false
                };

                // Parse records in parallel
                let records = parse_batch_parallel(&boundaries, &current_buffer)
                    .map_err(|e| PipelineError::ParseError(e.to_string()))?;

                // Send the whole parsed batch as one channel message (blocks if
                // full = backpressure). One send per chunk instead of one per
                // record; the consumer drains a local buffer. Skip empty batches.
                if !records.is_empty() {
                    sender
                        .send(records)
                        .map_err(|_| PipelineError::ChannelSendError)?;
                }

                // If the last boundary doesn't reach the end, save the tail as leftover
                if all_complete {
                    leftover.clear();
                } else if let Some(&(offset, len)) = boundaries.last() {
                    leftover = current_buffer[offset + len..].to_vec();
                }
            },
            Err(_) => {
                // No complete records found in this buffer - entire thing is leftover
                leftover = current_buffer;
            },
        }
    }

    Ok(())
}

/// Consumer-facing pipeline handle
#[derive(Debug)]
pub struct ProducerConsumerPipeline {
    receiver: Receiver<Vec<Record>>,
    /// Records drained from the most recent batch but not yet handed out. The
    /// channel delivers a `Vec<Record>` per chunk; the consumer hands records
    /// out one at a time from here. A `Mutex` provides the interior mutability
    /// the `&self` accessors need; the consumer is single-threaded, so the lock
    /// is uncontended and is never held across the blocking channel `recv`.
    buffer: Mutex<VecDeque<Record>>,
    /// Optional handle to producer thread for join semantics
    _producer_handle: Option<thread::JoinHandle<PipelineResult<()>>>,
}

impl ProducerConsumerPipeline {
    /// Create a new pipeline from a file path
    ///
    /// Spawns producer thread that reads and parses in background.
    /// Consumer drains results via `next()`.
    ///
    /// # Errors
    ///
    /// Returns `PipelineError::IoError` if file cannot be opened.
    pub fn from_file(path: &str, config: &PipelineConfig) -> PipelineResult<Self> {
        let file = File::open(path).map_err(|e| PipelineError::IoError(e.to_string()))?;

        let (sender, receiver) = bounded(config.channel_capacity);

        let producer_config = config.clone();
        let producer_handle = thread::spawn(move || producer_task(file, &sender, &producer_config));

        Ok(ProducerConsumerPipeline {
            receiver,
            buffer: Mutex::new(VecDeque::new()),
            _producer_handle: Some(producer_handle),
        })
    }

    /// Lock the local record buffer, recovering from a poisoned lock (a
    /// poisoned buffer still holds valid records — no need to abort).
    fn lock_buffer(&self) -> MutexGuard<'_, VecDeque<Record>> {
        self.buffer.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Try to get next record without blocking
    ///
    /// Returns:
    /// - Ok(Some(record)) if a record is buffered or a batch is waiting
    /// - Ok(None) if nothing is currently available (channel empty or closed)
    /// - Err if producer panicked
    ///
    /// # Errors
    ///
    /// Currently returns Ok(None) for both empty and disconnected states.
    pub fn try_next(&self) -> PipelineResult<Option<Record>> {
        use crossbeam_channel::TryRecvError;

        loop {
            if let Some(record) = self.lock_buffer().pop_front() {
                return Ok(Some(record));
            }
            // Buffer empty: pull the next batch without blocking and refill.
            match self.receiver.try_recv() {
                Ok(batch) => self.lock_buffer().extend(batch),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return Ok(None),
            }
        }
    }

    /// Get next record, blocking if necessary
    ///
    /// Returns:
    /// - Ok(Some(record)) if a record is available (buffered or from a batch)
    /// - Ok(None) if EOF (channel closed and the buffer is drained)
    /// - Err if producer panicked
    ///
    /// # Errors
    ///
    /// Currently returns Ok(None) on channel disconnection.
    pub fn next(&self) -> PipelineResult<Option<Record>> {
        loop {
            if let Some(record) = self.lock_buffer().pop_front() {
                return Ok(Some(record));
            }
            // Buffer empty: block for the next batch, then refill. The lock is
            // not held across recv, so a blocked consumer never holds it.
            match self.receiver.recv() {
                Ok(batch) => self.lock_buffer().extend(batch),
                Err(_) => return Ok(None), // Channel closed and drained = EOF
            }
        }
    }

    /// Consume pipeline and return an iterator over records
    ///
    /// Yields records until EOF. Blocks if producer is slow.
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> impl Iterator<Item = PipelineResult<Record>> {
        // Hand out any records already buffered by next()/try_next(), then
        // flatten the remaining batches off the channel.
        let buffered = self
            .buffer
            .into_inner()
            .unwrap_or_else(PoisonError::into_inner);
        buffered
            .into_iter()
            .chain(self.receiver.into_iter().flatten())
            .map(Ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_default() {
        let config = PipelineConfig::default();
        assert_eq!(config.buffer_size, 512 * 1024);
        assert_eq!(config.channel_capacity, 4);
        assert_eq!(config.batch_size, 100);
    }

    #[test]
    fn test_pipeline_error_display() {
        let err = PipelineError::IoError("test error".to_string());
        assert_eq!(format!("{err}"), "I/O error: test error");

        let err = PipelineError::ScanError("scan failed".to_string());
        assert_eq!(format!("{err}"), "Boundary scan error: scan failed");

        let err = PipelineError::ParseError("parse failed".to_string());
        assert_eq!(format!("{err}"), "Parse error: parse failed");

        let err = PipelineError::ChannelSendError;
        assert_eq!(format!("{err}"), "Channel send error (producer panicked)");
    }

    #[test]
    fn test_pipeline_file_not_found() {
        let config = PipelineConfig::default();
        let result = ProducerConsumerPipeline::from_file("/nonexistent/path", &config);
        assert!(result.is_err());
    }

    /// Build a valid bibliographic record with the given 001 control number.
    fn build_record(control_number: &str) -> Record {
        use crate::leader::Leader;
        use crate::record::Field;
        let leader = Leader {
            record_length: 0,
            record_status: 'n',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: ' ',
            character_coding: 'a',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 0,
            encoding_level: ' ',
            cataloging_form: ' ',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        };
        let mut record = Record::new(leader);
        record.add_control_field("001".to_string(), control_number.to_string());
        let field = Field::builder("245".to_string(), '1', '0')
            .subfield_str('a', "Title")
            .build();
        record.add_field(field);
        record
    }

    /// The pipeline must deliver every record exactly once, in input order,
    /// across many file chunks. A small buffer and channel capacity force the
    /// records to span many chunks (and exercise backpressure), so the test
    /// guards the producer's batching and the consumer's drain regardless of
    /// how records are grouped onto the channel.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_pipeline_preserves_order_and_count_across_chunks() {
        use crate::writer::MarcWriter;
        use std::io::Write;

        let n = 50;
        let mut bytes = Vec::new();
        for i in 0..n {
            let record = build_record(&format!("rec{i:04}"));
            let mut buf = Vec::new();
            MarcWriter::new(&mut buf)
                .write_record(&record)
                .expect("write should succeed");
            bytes.extend_from_slice(&buf);
        }

        let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
        tmp.write_all(&bytes).expect("write temp");

        // Small buffer + small capacity force many chunks and backpressure.
        let config = PipelineConfig {
            buffer_size: 256,
            channel_capacity: 4,
            batch_size: 100,
        };
        let pipeline =
            ProducerConsumerPipeline::from_file(tmp.path().to_str().expect("utf8 path"), &config)
                .expect("pipeline opens");

        let got: Vec<Record> = pipeline.into_iter().map(|r| r.expect("record")).collect();

        assert_eq!(got.len(), n, "all records delivered");
        for (i, rec) in got.iter().enumerate() {
            assert_eq!(
                rec.get_control_field("001"),
                Some(format!("rec{i:04}").as_str()),
                "record {i} out of order or corrupted"
            );
        }
    }

    /// `next()` and `try_next()` must hand out records one at a time in order
    /// even though the channel now delivers whole batches — the local buffer
    /// has to be drained before the next batch is pulled.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_pipeline_next_drains_batches_in_order() {
        use crate::writer::MarcWriter;
        use std::io::Write;

        let n = 30;
        let mut bytes = Vec::new();
        for i in 0..n {
            let record = build_record(&format!("rec{i:04}"));
            let mut buf = Vec::new();
            MarcWriter::new(&mut buf)
                .write_record(&record)
                .expect("write should succeed");
            bytes.extend_from_slice(&buf);
        }

        let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
        tmp.write_all(&bytes).expect("write temp");

        let config = PipelineConfig {
            buffer_size: 512,
            channel_capacity: 2,
            batch_size: 100,
        };
        let pipeline =
            ProducerConsumerPipeline::from_file(tmp.path().to_str().expect("utf8 path"), &config)
                .expect("pipeline opens");

        let mut seen = 0;
        while let Some(record) = pipeline.next().expect("next should succeed") {
            assert_eq!(
                record.get_control_field("001"),
                Some(format!("rec{seen:04}").as_str()),
                "record {seen} out of order"
            );
            seen += 1;
        }
        assert_eq!(seen, n, "next() delivered every record");
    }
}
