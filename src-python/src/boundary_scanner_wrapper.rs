//! Python wrapper for the Rust RecordBoundaryScanner.
//!
//! This module provides Python bindings to the Rust `RecordBoundaryScanner` for
//! efficient MARC record boundary detection using SIMD-accelerated memchr.

use mrrc::boundary_scanner::RecordBoundaryScanner as RustBoundaryScanner;
use pyo3::prelude::*;

/// Python wrapper for RecordBoundaryScanner.
///
/// Scans MARC record boundaries by detecting 0x1E delimiters in binary data.
/// Returns (offset, length) tuples for each complete record found.
///
/// # Example
///
/// ```python
/// from mrrc import RecordBoundaryScanner
///
/// scanner = RecordBoundaryScanner()
/// data = b'...binary MARC data...'
/// boundaries = scanner.scan(data)
///
/// for offset, length in boundaries:
///     record_bytes = data[offset:offset+length]
///     # Process record_bytes...
/// ```
#[pyclass(name = "RecordBoundaryScanner")]
pub struct PyRecordBoundaryScanner {
    inner: RustBoundaryScanner,
}

#[pymethods]
impl PyRecordBoundaryScanner {
    /// Create a new RecordBoundaryScanner.
    ///
    /// The scanner uses SIMD-accelerated memchr for efficient boundary detection.
    #[new]
    fn new() -> Self {
        PyRecordBoundaryScanner {
            inner: RustBoundaryScanner::new(),
        }
    }

    /// Scan a buffer for MARC record boundaries.
    ///
    /// Returns a list of (offset, length) tuples for each complete MARC record.
    /// The offset is the byte position where the record starts, and length includes
    /// the terminating 0x1E byte.
    ///
    /// # Arguments
    ///
    /// * `data` - The bytes to scan for record boundaries
    ///
    /// # Returns
    ///
    /// A list of (offset, length) tuples for each complete record.
    ///
    /// # Raises
    ///
    /// Raises `MarcError` if the buffer is empty or no complete records are found.
    ///
    /// # Example
    ///
    /// ```python
    /// scanner = RecordBoundaryScanner()
    /// data = b'\x01\x02\x1E\x03\x04\x1E'
    /// boundaries = scanner.scan(data)
    /// # boundaries = [(0, 3), (3, 3)]
    /// ```
    fn scan(&mut self, data: &[u8]) -> PyResult<Vec<(usize, usize)>> {
        self.inner
            .scan(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Scan a buffer and return boundaries up to a maximum limit.
    ///
    /// Useful for limiting the number of records returned in a single batch.
    /// This is efficient for processing large files in controlled batches.
    ///
    /// # Arguments
    ///
    /// * `data` - The bytes to scan
    /// * `limit` - Maximum number of boundaries to return
    ///
    /// # Returns
    ///
    /// A list of up to `limit` (offset, length) tuples.
    ///
    /// # Raises
    ///
    /// Raises `MarcError` if the buffer is empty or no complete records are found.
    ///
    /// # Example
    ///
    /// ```python
    /// scanner = RecordBoundaryScanner()
    /// data = b'\x01\x1E\x02\x1E\x03\x1E'
    /// boundaries = scanner.scan_limited(data, 2)
    /// # boundaries = [(0, 2), (2, 2)]
    /// ```
    fn scan_limited(&mut self, data: &[u8], limit: usize) -> PyResult<Vec<(usize, usize)>> {
        self.inner
            .scan_limited(data, limit)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Get the number of complete records in a buffer without parsing.
    ///
    /// This is useful for diagnostics and deciding batch sizes. It only counts
    /// 0x1E terminators; it doesn't validate that they form complete MARC records.
    ///
    /// # Arguments
    ///
    /// * `data` - The bytes to scan
    ///
    /// # Returns
    ///
    /// The number of 0x1E terminators (potential complete records) found.
    ///
    /// # Example
    ///
    /// ```python
    /// scanner = RecordBoundaryScanner()
    /// data = b'\x01\x1E\x02\x1E'
    /// count = scanner.count_records(data)
    /// # count = 2
    /// ```
    fn count_records(&self, data: &[u8]) -> usize {
        self.inner.count_records(data)
    }

    /// Clear the scanner's internal state.
    ///
    /// This resets any caches, though the scanner is typically reused across
    /// multiple scan operations without explicit clearing.
    fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get the current capacity of the scanner's internal buffer.
    ///
    /// This is useful for capacity planning in high-throughput scenarios.
    ///
    /// # Returns
    ///
    /// The capacity in number of records the internal buffer can hold.
    fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    fn __repr__(&self) -> String {
        format!("RecordBoundaryScanner(capacity={})", self.capacity())
    }
}
