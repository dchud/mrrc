// Python wrapper for MARCWriter with three-phase GIL release support
//
// This module implements three-phase GIL management for concurrent file I/O:
// - Phase 1 (GIL held): Extract record data from Python PyRecord object
// - Phase 2 (GIL released): Serialize record to MARC bytes (CPU-intensive)
// - Phase 3 (GIL held): Write serialized bytes to Python file object

use crate::wrappers::PyRecord;
use mrrc::MarcWriter;
use pyo3::prelude::*;

/// Python wrapper for MarcWriter with three-phase GIL release pattern
///
/// The three-phase pattern enables GIL release during CPU-intensive serialization:
/// - Phase 1: Extract record data from Python PyRecord (GIL held)
/// - Phase 2: Serialize record to bytes (GIL released)
/// - Phase 3: Write bytes to Python file object (GIL re-acquired)
///
/// This allows multiple threads to write different files concurrently.
#[pyclass(name = "MARCWriter")]
pub struct PyMARCWriter {
    file_obj: Py<PyAny>,
    closed: bool,
}

#[pymethods]
impl PyMARCWriter {
    /// Create a new MARCWriter for a Python file-like object
    ///
    /// # Arguments
    /// * `file` - A Python file-like object (must support .write(bytes) method)
    #[new]
    pub fn new(file: Py<PyAny>) -> PyResult<Self> {
        Ok(PyMARCWriter {
            file_obj: file,
            closed: false,
        })
    }

    /// Write a record to the file with three-phase GIL management
    ///
    /// This implements the three-phase GIL release pattern:
    /// - Phase 1: Extract record data from Python PyRecord (GIL held)
    /// - Phase 2: Serialize record to bytes (GIL released)
    /// - Phase 3: Write bytes to Python file object (GIL re-acquired)
    pub fn write_record(&mut self, record: &PyRecord) -> PyResult<()> {
        if self.closed {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer has been closed",
            ));
        }

        // ✅ CORRECT: Get Python handle assuming GIL is already held
        // PyO3 0.27: use assume_attached() which assumes GIL is attached to current thread
        // Methods in #[pymethods] always have GIL held by the Python interpreter
        let py = unsafe { Python::assume_attached() };

        // ===== PHASE 1: Extract record data (GIL held) =====
        // We receive a PyRecord (the Rust wrapper around a Record)
        // Clone the inner Rust record for Phase 2
        let record_copy = record.inner.clone();

        // ===== PHASE 2: Serialize to bytes (GIL released) =====
        // Serialize the record to MARC bytes without holding the GIL
        // SAFETY: py.allow_threads() is deprecated but still works and actually releases GIL
        #[allow(deprecated)]
        let record_bytes: Vec<u8> = py.allow_threads(|| {
            // This closure runs WITHOUT the GIL held
            let mut buffer = Vec::new();
            let mut writer = MarcWriter::new(&mut buffer);

            // Serialize the record to bytes
            writer.write_record(&record_copy).map_err(|e| {
                std::io::Error::other(format!("Failed to serialize MARC record: {}", e))
            })?;

            Ok::<Vec<u8>, std::io::Error>(buffer)
        })?;

        // ===== PHASE 3: Write bytes to file (GIL re-acquired) =====
        // GIL is automatically re-acquired when exiting allow_threads() block
        let file_ref = self.file_obj.bind(py);
        let write_method = file_ref.getattr("write").map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "File object has no write method: {}",
                e
            ))
        })?;

        write_method.call1((record_bytes,)).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to write record bytes: {}",
                e
            ))
        })?;

        Ok(())
    }

    /// Close the writer and flush the buffer
    pub fn close(&mut self) -> PyResult<()> {
        if !self.closed {
            // ✅ CORRECT: Get Python handle assuming GIL is already held
            let py = unsafe { Python::assume_attached() };
            let file_ref = self.file_obj.bind(py);
            if let Ok(flush_method) = file_ref.getattr("flush") {
                let _ = flush_method.call0();
            }
            self.closed = true;
        }
        Ok(())
    }

    /// Context manager support: enter
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Context manager support: exit
    #[pyo3(signature = (_exc_type=None, _exc_val=None, _exc_tb=None))]
    fn __exit__(
        &mut self,
        _exc_type: Option<Py<PyAny>>,
        _exc_val: Option<Py<PyAny>>,
        _exc_tb: Option<Py<PyAny>>,
    ) -> PyResult<bool> {
        self.close()?;
        Ok(false)
    }

    fn __repr__(&self) -> String {
        if self.closed {
            "<MARCWriter closed>".to_string()
        } else {
            "<MARCWriter active>".to_string()
        }
    }
}
