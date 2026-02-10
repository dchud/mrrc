//! Producer-Consumer Pipeline with Backpressure
//!
//! Implements a bounded channel pipeline for high-performance batch reading
//! from Rust file I/O backend (no GIL overhead).
//!
//! Design:
//! - **Producer:** Background task reading file chunks, scanning boundaries, parsing batches
//! - **Consumer:** Python-facing iterator that drains the bounded channel
//! - **Backpressure:** Channel capacity = 1000 records; blocks producer when full
//! - **GIL:** Producer runs without GIL; consumer manages GIL on retrieval

use crate::boundary_scanner::RecordBoundaryScanner;
use crate::rayon_parser_pool::parse_batch_parallel;
use crate::record::Record;
use crossbeam_channel::{bounded, Receiver, Sender};
use std::fs::File;
use std::io::Read;
use std::thread;

/// Configuration for the producer-consumer pipeline
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Buffer size for file I/O (bytes)
    pub buffer_size: usize,
    /// Channel capacity (records)
    pub channel_capacity: usize,
    /// Batch size for parser pool
    pub batch_size: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            buffer_size: 512 * 1024, // 512 KB
            channel_capacity: 1000,  // 1000 records
            batch_size: 100,         // 100 records per batch
        }
    }
}

/// Result type for pipeline operations
pub type PipelineResult<T> = Result<T, PipelineError>;

/// Errors that can occur during pipeline operations
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
    sender: &Sender<Record>,
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

                // Send records to channel (blocks if full = backpressure)
                for record in records {
                    sender
                        .send(record)
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
    receiver: Receiver<Record>,
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
            _producer_handle: Some(producer_handle),
        })
    }

    /// Try to get next record without blocking
    ///
    /// Returns:
    /// - Ok(Some(record)) if record available
    /// - Ok(None) if channel empty or closed
    /// - Err if producer panicked
    ///
    /// # Errors
    ///
    /// Currently returns Ok(None) for both empty and disconnected states.
    pub fn try_next(&self) -> PipelineResult<Option<Record>> {
        use crossbeam_channel::TryRecvError;

        match self.receiver.try_recv() {
            Ok(record) => Ok(Some(record)),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => Ok(None),
        }
    }

    /// Get next record, blocking if necessary
    ///
    /// Returns:
    /// - Ok(Some(record)) if record available
    /// - Ok(None) if EOF (channel closed and empty)
    /// - Err if producer panicked
    ///
    /// # Errors
    ///
    /// Currently returns Ok(None) on channel disconnection.
    pub fn next(&self) -> PipelineResult<Option<Record>> {
        match self.receiver.recv() {
            Ok(record) => Ok(Some(record)),
            Err(_) => Ok(None), // Channel closed = EOF
        }
    }

    /// Consume pipeline and return an iterator over records
    ///
    /// Yields records until EOF. Blocks if producer is slow.
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> impl Iterator<Item = PipelineResult<Record>> {
        self.receiver.into_iter().map(Ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_default() {
        let config = PipelineConfig::default();
        assert_eq!(config.buffer_size, 512 * 1024);
        assert_eq!(config.channel_capacity, 1000);
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
}
