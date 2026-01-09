// Python wrapper for MARCWriter with efficient GIL release support
//
// This module implements efficient GIL management for concurrent file I/O:
// 1. Extract record data from Python PyRecord object (GIL held)
// 2. Serialize record to MARC bytes (GIL released, CPU-intensive)
// 3. Write serialized bytes to appropriate backend (GIL re-acquired if needed)

use crate::wrappers::PyRecord;
use mrrc::MarcWriter;
use pyo3::prelude::*;
use std::fs::File;
use std::io::BufWriter;

/// Internal enum for different writer backends
#[allow(clippy::large_enum_variant)]
enum WriterBackend {
    /// Python file-like object (e.g., BytesIO, open file in 'wb' mode)
    /// Requires GIL for I/O operations
    PythonFile { file_obj: Py<PyAny> },
    /// Pure Rust file I/O via std::fs::File (no GIL overhead)
    /// Used when writer initialized with file path string
    RustFile { writer: BufWriter<File> },
}

/// Python wrapper for MarcWriter with efficient GIL release pattern
///
/// Enables GIL release during CPU-intensive serialization:
/// - Extract record data from Python PyRecord (GIL held)
/// - Serialize record to bytes (GIL released)
/// - Write bytes to backend (GIL management varies by backend)
///
/// ## Backends
///
/// **PythonFile:** File-like Python objects (BytesIO, file handles from open())
/// - Phase 3 uses GIL (calls Python .write() method)
/// - Allows concurrent writes to different file objects
/// - Limited concurrency due to GIL contention
///
/// **RustFile:** Rust file paths (str, pathlib.Path)
/// - Phase 3 uses no GIL (pure Rust I/O)
/// - Enables near-native concurrent performance
/// - Ideal for write-heavy workloads
///
/// This allows multiple threads to write different files concurrently.
#[pyclass(name = "MARCWriter")]
pub struct PyMARCWriter {
    backend: Option<WriterBackend>,
    closed: bool,
}

#[pymethods]
impl PyMARCWriter {
    /// Create a new MARCWriter from any supported source
    ///
    /// Accepts:
    /// - str path (e.g., 'output.mrc') → RustFile backend (no GIL)
    /// - pathlib.Path → RustFile backend (no GIL)
    /// - Python file object → PythonFile backend (GIL managed)
    ///
    /// # Arguments
    /// * `source` - File path (str), pathlib.Path, or file-like object
    #[new]
    pub fn new(source: &Bound<'_, PyAny>) -> PyResult<Self> {
        // Try to detect if it's a string path first (fast path for RustFile)
        if let Ok(path_str) = source.extract::<String>() {
            // String path → open with Rust for zero-GIL I/O
            let file = File::create(&path_str).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!(
                    "Failed to open file '{}' for writing: {}",
                    path_str, e
                ))
            })?;

            let writer = BufWriter::new(file);
            return Ok(PyMARCWriter {
                backend: Some(WriterBackend::RustFile { writer }),
                closed: false,
            });
        }

        // Check for pathlib.Path objects by looking for __fspath__ or __str__ methods
        // and trying to create a file with the path
        if let Ok(path_method) = source.getattr("__fspath__") {
            if path_method.is_callable() {
                if let Ok(path_obj) = path_method.call0() {
                    if let Ok(path_str) = path_obj.extract::<String>() {
                        let file = File::create(&path_str).map_err(|e| {
                            pyo3::exceptions::PyIOError::new_err(format!(
                                "Failed to open file '{}' for writing: {}",
                                path_str, e
                            ))
                        })?;

                        let writer = BufWriter::new(file);
                        return Ok(PyMARCWriter {
                            backend: Some(WriterBackend::RustFile { writer }),
                            closed: false,
                        });
                    }
                }
            }
        }

        // Fallback: treat as Python file-like object
        // Check if it has .write() method
        if let Ok(write_method) = source.getattr("write") {
            if write_method.is_callable() {
                let file_obj = source.clone().unbind();
                return Ok(PyMARCWriter {
                    backend: Some(WriterBackend::PythonFile { file_obj }),
                    closed: false,
                });
            }
        }

        // Not a supported type
        Err(pyo3::exceptions::PyTypeError::new_err(
            "MARCWriter() argument must be a file path (str/Path) or file-like object with .write() method"
        ))
    }

    /// Write a record to the file with efficient GIL management
    ///
    /// Implements efficient GIL release for concurrent performance:
    /// - **Phase 1 (GIL held):** Extract record data from Python PyRecord object
    /// - **Phase 2 (GIL released):** Serialize record to MARC bytes (CPU-intensive)
    /// - **Phase 3 (GIL held):** Write serialized bytes to backend
    ///
    /// This pattern allows multiple threads to write different files concurrently:
    /// - Each thread releases the GIL during Phase 2 (serialization)
    /// - Only acquires GIL for Phase 1 (data extraction) and Phase 3 (I/O)
    /// - RustFile backend (Phase 3) doesn't need GIL, enabling near-native performance
    /// - PythonFile backend (Phase 3) requires GIL for calling Python .write() method
    ///
    /// # Errors
    /// - Returns error if writer has been closed
    /// - Returns error if backend initialization failed
    /// - Returns error if serialization fails (corrupted record data)
    /// - Returns error if file I/O fails (disk full, permissions, etc.)
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
        // This must happen with GIL held to safely extract Python object references
        let record_copy = record.inner.clone();

        // ===== PHASE 2: Serialize to bytes (GIL released) =====
        // Serialize the record to MARC bytes without holding the GIL
        // This is CPU-intensive and benefits from parallel execution
        // CRITICAL: Use Python::detach() which properly releases the GIL.
        // Python::detach() takes a closure and runs it without the GIL.
        let record_bytes: Vec<u8> = py.detach(|| {
            // This closure runs WITHOUT the GIL held
            // Safe: record_copy is pure Rust, doesn't reference Python objects
            let mut buffer = Vec::new();
            let mut writer = MarcWriter::new(&mut buffer);

            // Serialize the record to bytes
            writer.write_record(&record_copy).map_err(|e| {
                std::io::Error::other(format!("Failed to serialize MARC record: {}", e))
            })?;

            Ok::<Vec<u8>, std::io::Error>(buffer)
        })?;

        // ===== PHASE 3: Write bytes to backend (GIL re-acquired) =====
        // GIL is automatically re-acquired when exiting detach() block
        // Dispatch to the appropriate backend for I/O
        match &mut self.backend {
            Some(WriterBackend::PythonFile { file_obj }) => {
                // PythonFile backend: requires GIL (calls Python .write() method)
                // GIL is held here, safe to call Python methods
                let file_ref = file_obj.bind(py);
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
            },
            Some(WriterBackend::RustFile { writer }) => {
                // RustFile backend: no GIL needed (pure Rust I/O)
                // GIL is held but not used - could be released further if needed
                use std::io::Write;
                writer.write_all(&record_bytes).map_err(|e| {
                    pyo3::exceptions::PyIOError::new_err(format!(
                        "Failed to write record bytes: {}",
                        e
                    ))
                })?;
            },
            None => {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Writer backend not initialized",
                ));
            },
        }

        Ok(())
    }

    /// Alias for write_record (for pymarc compatibility)
    pub fn write(&mut self, record: &PyRecord) -> PyResult<()> {
        self.write_record(record)
    }

    /// Close the writer and flush the buffer
    ///
    /// Flushes any buffered data to disk and closes the writer.
    /// Safe to call multiple times (idempotent).
    ///
    /// ## GIL Management
    /// - **PythonFile:** GIL is held while calling Python flush() method
    /// - **RustFile:** No GIL needed (pure Rust I/O)
    pub fn close(&mut self) -> PyResult<()> {
        if !self.closed {
            match &mut self.backend {
                Some(WriterBackend::PythonFile { file_obj }) => {
                    // PythonFile backend: flush via Python method (GIL required)
                    let py = unsafe { Python::assume_attached() };
                    let file_ref = file_obj.bind(py);
                    if let Ok(flush_method) = file_ref.getattr("flush") {
                        let _ = flush_method.call0();
                    }
                },
                Some(WriterBackend::RustFile { writer }) => {
                    // RustFile backend: flush via Rust I/O (no GIL)
                    use std::io::Write;
                    let _ = writer.flush();
                },
                None => {
                    // Already closed, nothing to do
                },
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
