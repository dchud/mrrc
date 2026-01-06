//! Record boundary detection using 0x1D delimiters for parallel processing.
//!
//! This module provides optimized scanning of MARC record boundaries using the
//! SIMD-accelerated `memchr` crate to locate 0x1D (record terminator) bytes.
//! Boundaries are returned as (offset, length) tuples for use in parallel parsing pipelines.
//!
//! # Example
//!
//! ```no_run
//! use mrrc::boundary_scanner::RecordBoundaryScanner;
//!
//! let buffer = b"...binary MARC data...";
//! let mut scanner = RecordBoundaryScanner::new();
//! let boundaries = scanner.scan(buffer)?;
//!
//! for (offset, len) in boundaries {
//!     println!("Record at offset {} with length {}", offset, len);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::error::{MarcError, Result};

/// The byte value that terminates MARC records (ISO 2709).
/// In ISO 2709 format, records end with 0x1D (not 0x1E, which is the field terminator).
const RECORD_TERMINATOR: u8 = 0x1D;

/// Record boundary scanner using SIMD-accelerated delimiter detection.
///
/// This scanner locates MARC record boundaries by finding 0x1D (record terminator) bytes
/// in a buffer. It's designed for use in parallel processing pipelines
/// where record boundaries must be known before parsing.
#[derive(Debug, Default)]
pub struct RecordBoundaryScanner {
    /// Pre-allocated buffer for reuse across scans
    boundaries: Vec<(usize, usize)>,
}

impl RecordBoundaryScanner {
    /// Create a new boundary scanner with default capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::boundary_scanner::RecordBoundaryScanner;
    ///
    /// let scanner = RecordBoundaryScanner::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            boundaries: Vec::with_capacity(100),
        }
    }

    /// Scan a buffer for record boundaries.
    ///
    /// Returns a vector of (offset, length) tuples for each record found.
    /// The offset is the byte position where the record starts, and length
    /// includes the terminating 0x1D byte (record terminator).
    ///
    /// # Arguments
    ///
    /// * `buffer` - The bytes to scan for record boundaries
    ///
    /// # Returns
    ///
    /// A vector of (offset, length) tuples for each complete record.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is empty or no complete records (no 0x1D terminators) are found.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::boundary_scanner::RecordBoundaryScanner;
    ///
    /// let data = vec![1, 2, 3, 0x1D, 4, 5, 0x1D];  // 0x1D = record terminator
    /// let mut scanner = RecordBoundaryScanner::new();
    /// let boundaries = scanner.scan(&data)?;
    ///
    /// assert_eq!(boundaries.len(), 2);
    /// assert_eq!(boundaries[0], (0, 4)); // offset 0, length 4 (includes 0x1D)
    /// assert_eq!(boundaries[1], (4, 3)); // offset 4, length 3 (includes 0x1D)
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn scan(&mut self, buffer: &[u8]) -> Result<Vec<(usize, usize)>> {
        if buffer.is_empty() {
            return Err(MarcError::InvalidRecord("buffer is empty".to_string()));
        }

        self.boundaries.clear();
        let mut offset = 0;

        // Use memchr for SIMD-accelerated scanning of 0x1D terminators
        for terminator_pos in memchr::memchr_iter(RECORD_TERMINATOR, buffer) {
            let record_len = terminator_pos - offset + 1; // +1 to include terminator
            self.boundaries.push((offset, record_len));
            offset = terminator_pos + 1;
        }

        if self.boundaries.is_empty() {
            return Err(MarcError::InvalidRecord(
                "no complete MARC records found (no 0x1D record terminators)".to_string(),
            ));
        }

        Ok(self.boundaries.clone())
    }

    /// Scan a buffer and return boundaries up to a maximum limit.
    ///
    /// Useful for limiting the number of records returned in a single batch.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The bytes to scan
    /// * `limit` - Maximum number of boundaries to return
    ///
    /// # Returns
    ///
    /// A vector of up to `limit` boundaries.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is empty or no complete records are found.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::boundary_scanner::RecordBoundaryScanner;
    ///
    /// let data = vec![1, 0x1D, 2, 0x1D, 3, 0x1D];  // 0x1D = record terminator
    /// let mut scanner = RecordBoundaryScanner::new();
    /// let boundaries = scanner.scan_limited(&data, 2)?;
    ///
    /// assert_eq!(boundaries.len(), 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn scan_limited(&mut self, buffer: &[u8], limit: usize) -> Result<Vec<(usize, usize)>> {
        let all_boundaries = self.scan(buffer)?;
        Ok(all_boundaries.into_iter().take(limit).collect())
    }

    /// Get the number of complete records in a buffer without parsing.
    ///
    /// This is useful for diagnostics and deciding batch sizes.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The bytes to scan
    ///
    /// # Returns
    ///
    /// The number of 0x1D terminators (complete records) found.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::boundary_scanner::RecordBoundaryScanner;
    ///
    /// let data = vec![1, 0x1D, 2, 0x1D];
    /// let mut scanner = RecordBoundaryScanner::new();
    /// let count = scanner.count_records(&data);
    ///
    /// assert_eq!(count, 2);
    /// ```
    #[must_use]
    pub fn count_records(&self, buffer: &[u8]) -> usize {
        memchr::memchr_iter(RECORD_TERMINATOR, buffer).count()
    }

    /// Clear internal state and return capacity information.
    ///
    /// Returns the current capacity of the internal boundaries buffer,
    /// useful for capacity planning in high-throughput scenarios.
    pub fn clear(&mut self) {
        self.boundaries.clear();
    }

    /// Get the current capacity of the scanner.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.boundaries.capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_single_record() {
        let data = vec![1, 2, 3, 0x1D];
        let mut scanner = RecordBoundaryScanner::new();
        let boundaries = scanner.scan(&data).unwrap();

        assert_eq!(boundaries.len(), 1);
        assert_eq!(boundaries[0], (0, 4));
    }

    #[test]
    fn test_scan_multiple_records() {
        let data = vec![1, 2, 0x1D, 3, 4, 0x1D, 5, 0x1D];
        let mut scanner = RecordBoundaryScanner::new();
        let boundaries = scanner.scan(&data).unwrap();

        assert_eq!(boundaries.len(), 3);
        assert_eq!(boundaries[0], (0, 3));
        assert_eq!(boundaries[1], (3, 3));
        assert_eq!(boundaries[2], (6, 2));
    }

    #[test]
    fn test_scan_empty_buffer() {
        let data = vec![];
        let mut scanner = RecordBoundaryScanner::new();
        let result = scanner.scan(&data);

        assert!(result.is_err());
    }

    #[test]
    fn test_scan_no_terminators() {
        let data = vec![1, 2, 3, 4];
        let mut scanner = RecordBoundaryScanner::new();
        let result = scanner.scan(&data);

        assert!(result.is_err());
    }

    #[test]
    fn test_scan_limited() {
        let data = vec![1, 0x1D, 2, 0x1D, 3, 0x1D];
        let mut scanner = RecordBoundaryScanner::new();
        let boundaries = scanner.scan_limited(&data, 2).unwrap();

        assert_eq!(boundaries.len(), 2);
        assert_eq!(boundaries[0], (0, 2));
        assert_eq!(boundaries[1], (2, 2));
    }

    #[test]
    fn test_count_records() {
        let data = vec![1, 0x1D, 2, 0x1D, 3, 4];
        let scanner = RecordBoundaryScanner::new();

        assert_eq!(scanner.count_records(&data), 2);
    }

    #[test]
    fn test_count_records_empty() {
        let data = vec![];
        let scanner = RecordBoundaryScanner::new();

        assert_eq!(scanner.count_records(&data), 0);
    }

    #[test]
    fn test_reuse_scanner() {
        let mut scanner = RecordBoundaryScanner::new();

        // First scan
        let data1 = vec![1, 0x1D, 2, 0x1D];
        let boundaries1 = scanner.scan(&data1).unwrap();
        assert_eq!(boundaries1.len(), 2);

        // Second scan (should reuse internal buffer)
        let data2 = vec![1, 0x1D];
        let boundaries2 = scanner.scan(&data2).unwrap();
        assert_eq!(boundaries2.len(), 1);

        // Verify no cross-contamination
        assert_eq!(boundaries2[0], (0, 2));
    }

    #[test]
    fn test_large_buffer_performance() {
        // Create a buffer with 1000 records (avoid 0x1D bytes in data)
        let mut data = Vec::new();
        for i in 0..1000 {
            // Each record: pattern of safe bytes + terminator
            // Use bytes < 0x1D and > 0x1D to avoid collisions
            data.push(if i % 2 == 0 { 0x01 } else { 0x02 });
            data.push(0x1D);
        }

        let mut scanner = RecordBoundaryScanner::new();
        let boundaries = scanner.scan(&data).unwrap();

        assert_eq!(boundaries.len(), 1000);
        // Spot check first and last: offsets should be at even positions
        assert_eq!(boundaries[0], (0, 2));
        assert_eq!(boundaries[999], (1998, 2));
    }
}
