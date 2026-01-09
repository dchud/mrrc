//! PyO3 bindings for the Producer-Consumer Pipeline
//!
//! Exposes [`ProducerConsumerPipeline`] as a Python class, enabling high-performance
//! batch reading with backpressure management from Python code.

use crate::wrappers::PyRecord;
use mrrc::producer_consumer_pipeline::{PipelineConfig, ProducerConsumerPipeline};
use pyo3::exceptions::PyStopIteration;
use pyo3::prelude::*;

/// A producer-consumer pipeline for high-performance MARC reading with backpressure.
///
/// Design:
/// - **Producer:** Background thread reading file chunks, scanning boundaries, parsing in parallel
/// - **Consumer:** Main thread iterating over parsed records
/// - **Backpressure:** Channel capacity limits buffer to 1000 records
///
/// # Example
///
/// ```python
/// from mrrc import ProducerConsumerPipeline
///
/// # Create pipeline from file
/// pipeline = ProducerConsumerPipeline.from_file("records.mrc")
///
/// # Consume records (blocks if producer is slow)
/// record_count = 0
/// while True:
///     record = pipeline.next()
///     if record is None:
///         break
///     record_count += 1
///     print(f"Record: {record.title()}")
///
/// print(f"Processed {record_count} records")
/// ```
#[pyclass(name = "ProducerConsumerPipeline")]
pub struct PyProducerConsumerPipeline {
    inner: Option<ProducerConsumerPipeline>,
}

#[pymethods]
impl PyProducerConsumerPipeline {
    /// Create a new pipeline from a file path.
    ///
    /// Spawns a background producer thread that reads, scans boundaries,
    /// and parses records in parallel. Consumer drains results via `next()`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to MARC file
    /// * `buffer_size` - Optional: File I/O buffer size (default: 512 KB)
    /// * `channel_capacity` - Optional: Channel capacity in records (default: 1000)
    ///
    /// # Raises
    ///
    /// `FileNotFoundError` if the file does not exist.
    /// `IOError` if the file cannot be opened.
    ///
    /// # Example
    ///
    /// ```python
    /// # Use defaults
    /// pipeline = ProducerConsumerPipeline.from_file("records.mrc")
    ///
    /// # Custom configuration
    /// pipeline = ProducerConsumerPipeline.from_file(
    ///     "records.mrc",
    ///     buffer_size=1024*1024,  # 1 MB
    ///     channel_capacity=500    # 500 records
    /// )
    /// ```
    #[staticmethod]
    #[pyo3(signature = (path, buffer_size=None, channel_capacity=None))]
    pub fn from_file(
        path: &str,
        buffer_size: Option<usize>,
        channel_capacity: Option<usize>,
    ) -> PyResult<Self> {
        let config = PipelineConfig {
            buffer_size: buffer_size.unwrap_or(512 * 1024),
            channel_capacity: channel_capacity.unwrap_or(1000),
            batch_size: 100, // Fixed at 100 per spec
        };

        let pipeline = ProducerConsumerPipeline::from_file(path, &config)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PyProducerConsumerPipeline {
            inner: Some(pipeline),
        })
    }

    /// Get the next record without blocking.
    ///
    /// Returns immediately if a record is available in the channel,
    /// or None if the channel is empty. Non-blocking operation.
    ///
    /// # Returns
    ///
    /// A parsed MARC record if available, or None if the channel is empty.
    ///
    /// # Example
    ///
    /// ```python
    /// pipeline = ProducerConsumerPipeline.from_file("records.mrc")
    ///
    /// # Non-blocking poll
    /// record = pipeline.try_next()
    /// while record is not None:
    ///     print(f"Got record: {record.title()}")
    ///     record = pipeline.try_next()
    ///
    /// # Might be None here even if producer still has records
    /// ```
    pub fn try_next(&mut self) -> PyResult<Option<PyRecord>> {
        let pipeline = self
            .inner
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Pipeline closed"))?;

        let record = pipeline
            .try_next()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(record.map(|r| PyRecord { inner: r }))
    }

    /// Get the next record, blocking if necessary.
    ///
    /// Blocks until a record is available from the producer, or until EOF.
    /// Use this for sequential iteration.
    ///
    /// # Returns
    ///
    /// A parsed MARC record, or None if EOF reached (channel closed and empty).
    ///
    /// # Example
    ///
    /// ```python
    /// pipeline = ProducerConsumerPipeline.from_file("records.mrc")
    ///
    /// # Blocking iteration
    /// record_count = 0
    /// while True:
    ///     record = pipeline.next()
    ///     if record is None:
    ///         break
    ///     record_count += 1
    ///     process(record)
    ///
    /// print(f"Processed {record_count} records")
    /// ```
    pub fn next(&mut self) -> PyResult<Option<PyRecord>> {
        let pipeline = self
            .inner
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Pipeline closed"))?;

        let record = pipeline
            .next()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(record.map(|r| PyRecord { inner: r }))
    }

    /// Iterate over all records in the pipeline.
    ///
    /// Consumes the pipeline, yielding records sequentially.
    /// Blocks if the producer is slow.
    ///
    /// # Raises
    ///
    /// `RuntimeError` if the producer thread panics.
    ///
    /// # Example
    ///
    /// ```python
    /// pipeline = ProducerConsumerPipeline.from_file("records.mrc")
    ///
    /// # Pythonic iteration
    /// for record in pipeline:
    ///     print(f"Record: {record.title()}")
    /// ```
    pub fn __iter__(slf: PyRefMut<'_, Self>) -> PyResult<PyRefMut<'_, Self>> {
        Ok(slf)
    }

    /// Get the next record in iteration.
    ///
    /// Implements the iterator protocol for use with `for` loops.
    pub fn __next__(slf: PyRefMut<'_, Self>) -> PyResult<PyRecord> {
        let pipeline = slf
            .inner
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Pipeline closed"))?;

        match pipeline.next() {
            Ok(Some(record)) => Ok(PyRecord { inner: record }),
            Ok(None) => Err(PyErr::new::<PyStopIteration, _>("EOF")),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string(),
            )),
        }
    }

    /// Representation for debugging.
    pub fn __repr__(&self) -> String {
        format!(
            "ProducerConsumerPipeline({})",
            if self.inner.is_some() {
                "active"
            } else {
                "closed"
            }
        )
    }
}
