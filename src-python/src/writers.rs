// Python wrapper for MARCWriter with Python file-like object support

use crate::error::marc_error_to_py_err;
use crate::wrappers::PyRecord;
use mrrc::MarcWriter;
use pyo3::prelude::*;
use std::io::Write;

/// Wrapper around a Python file-like object to implement Rust's Write trait
struct PyFileWriteWrapper {
    file_obj: Py<PyAny>,
}

impl PyFileWriteWrapper {
    fn new(file_obj: Py<PyAny>) -> Self {
        PyFileWriteWrapper { file_obj }
    }
}

impl Write for PyFileWriteWrapper {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Python::attach(|py| {
            let file_ref = self.file_obj.bind(py);
            let write_method = file_ref
                .getattr("write")
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "No write method"))?;

            // Create Python bytes object from slice
            let result = write_method
                .call1((buf,))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

            let written = result
                .extract::<usize>()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

            Ok(written)
        })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Python::attach(|py| {
            let file_ref = self.file_obj.bind(py);
            if let Ok(flush_method) = file_ref.getattr("flush") {
                flush_method
                    .call0()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            }
            Ok(())
        })
    }
}

/// Python wrapper for MarcWriter
#[pyclass(name = "MARCWriter")]
pub struct PyMARCWriter {
    writer: Option<MarcWriter<PyFileWriteWrapper>>,
}

#[pymethods]
impl PyMARCWriter {
    /// Create a new MARCWriter for a Python file-like object
    #[new]
    pub fn new(file: Py<PyAny>) -> PyResult<Self> {
        let wrapper = PyFileWriteWrapper::new(file);
        let writer = MarcWriter::new(wrapper);
        Ok(PyMARCWriter {
            writer: Some(writer),
        })
    }

    /// Write a record to the file
    pub fn write_record(&mut self, record: &PyRecord) -> PyResult<()> {
        if let Some(writer) = &mut self.writer {
            writer
                .write_record(&record.inner)
                .map_err(marc_error_to_py_err)?;
            Ok(())
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer has been closed",
            ))
        }
    }

    /// Close the writer and flush the buffer
    pub fn close(&mut self) -> PyResult<()> {
        if self.writer.take().is_some() {
            // Writer is dropped, which should flush
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
        if self.writer.is_some() {
            "<MARCWriter active>".to_string()
        } else {
            "<MARCWriter closed>".to_string()
        }
    }
}
