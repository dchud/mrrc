//! PyO3 bindings for the Rayon parser pool.
//!
//! Exposes [`parse_batch_parallel`] as a Python function, allowing
//! parallel MARC record parsing from Python code.

use crate::wrappers::PyRecord;
use mrrc::rayon_parser_pool;
use pyo3::prelude::*;

/// Parse a batch of MARC record boundaries in parallel using Rayon.
///
/// # Arguments
///
/// * `boundaries` - List of (offset, length) tuples identifying record boundaries
/// * `buffer` - The complete binary buffer containing all records (bytes or bytearray)
///
/// # Returns
///
/// A list of PyRecord instances, one for each boundary.
///
/// # Raises
///
/// `MarcError` if:
/// - Any boundary exceeds the buffer size
/// - Any record fails to parse
///
/// # Examples
///
/// ```python
/// from mrrc import RecordBoundaryScanner
/// from mrrc.rayon_parser_pool import parse_batch_parallel
/// import io
///
/// # Read a MARC file
/// with open('records.mrc', 'rb') as f:
///     buffer = f.read()
///
/// # Scan for record boundaries
/// scanner = RecordBoundaryScanner()
/// boundaries = scanner.scan(buffer)
///
/// # Parse records in parallel
/// records = parse_batch_parallel(boundaries, buffer)
/// print(f"Parsed {len(records)} records in parallel")
/// ```
#[pyfunction]
pub fn parse_batch_parallel(
    _py: Python<'_>,
    boundaries: Vec<(usize, usize)>,
    buffer: &[u8],
) -> PyResult<Vec<PyRecord>> {
    // Call the Rust implementation
    let records = rayon_parser_pool::parse_batch_parallel(&boundaries, buffer).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Parse error: {}", e))
    })?;

    // Convert Rust records to PyRecord wrappers (GIL already held)
    Ok(records
        .into_iter()
        .map(|record| PyRecord { inner: record })
        .collect())
}

/// Parse a limited batch of MARC records in parallel.
///
/// Like [`parse_batch_parallel`], but limits the number of records to parse.
///
/// # Arguments
///
/// * `boundaries` - List of (offset, length) tuples
/// * `buffer` - The complete binary buffer
/// * `limit` - Maximum number of records to parse
///
/// # Returns
///
/// A list of up to `limit` PyRecord instances.
///
/// # Example
///
/// ```python
/// from mrrc import RecordBoundaryScanner
/// from mrrc.rayon_parser_pool import parse_batch_parallel_limited
///
/// with open('records.mrc', 'rb') as f:
///     buffer = f.read()
///
/// scanner = RecordBoundaryScanner()
/// boundaries = scanner.scan(buffer)
///
/// # Parse only first 10 records in parallel
/// records = parse_batch_parallel_limited(boundaries, buffer, 10)
/// ```
#[pyfunction]
pub fn parse_batch_parallel_limited(
    _py: Python<'_>,
    boundaries: Vec<(usize, usize)>,
    buffer: &[u8],
    limit: usize,
) -> PyResult<Vec<PyRecord>> {
    // Call the Rust implementation
    let records = rayon_parser_pool::parse_batch_parallel_limited(&boundaries, buffer, limit)
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Parse error: {}", e))
        })?;

    // Convert to PyRecord (GIL already held)
    Ok(records
        .into_iter()
        .map(|record| PyRecord { inner: record })
        .collect())
}
