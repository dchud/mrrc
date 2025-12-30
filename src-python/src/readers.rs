// Python wrapper for MARCReader with Python file-like object support

use crate::error::marc_error_to_py_err;
use crate::wrappers::PyRecord;
use mrrc::MarcReader;
use pyo3::prelude::*;
use std::io::Read;

/// Wrapper around a Python file-like object to implement Rust's Read trait
struct PyFileWrapper {
    file_obj: Py<PyAny>,
}

impl PyFileWrapper {
    fn new(file_obj: Py<PyAny>) -> Self {
        PyFileWrapper { file_obj }
    }
}

impl Read for PyFileWrapper {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Python::attach(|py| {
            let file_ref = self.file_obj.bind(py);
            let read_method = file_ref
                .getattr("read")
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "No read method"))?;

            let n = buf.len();
            let result = read_method
                .call1((n,))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

            let bytes = result
                .extract::<Vec<u8>>()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

            let len = bytes.len();
            if len > 0 {
                buf[..len].copy_from_slice(&bytes);
            }
            Ok(len)
        })
    }
}

/// Python wrapper for MarcReader
#[pyclass(name = "MARCReader")]
pub struct PyMARCReader {
    reader: Option<MarcReader<PyFileWrapper>>,
}

#[pymethods]
impl PyMARCReader {
    /// Create a new MARCReader from a Python file-like object
    #[new]
    pub fn new(file: Py<PyAny>) -> PyResult<Self> {
        let wrapper = PyFileWrapper::new(file);
        let reader = MarcReader::new(wrapper);
        Ok(PyMARCReader {
            reader: Some(reader),
        })
    }

    /// Read the next record from the file
    pub fn read_record(&mut self) -> PyResult<Option<PyRecord>> {
        if let Some(reader) = &mut self.reader {
            match reader.read_record().map_err(marc_error_to_py_err)? {
                Some(record) => Ok(Some(PyRecord { inner: record })),
                None => Ok(None),
            }
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Reader has been consumed",
            ))
        }
    }

    /// Iterate over all records in the file
    fn __iter__(mut slf: PyRefMut<'_, Self>) -> PyResult<Self> {
        if slf.reader.is_none() {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Reader has been consumed",
            ));
        }
        Ok(PyMARCReader {
            reader: slf.reader.take(),
        })
    }

    /// Get the next record during iteration
    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRecord> {
        if let Some(reader) = &mut slf.reader {
            match reader.read_record().map_err(marc_error_to_py_err)? {
                Some(record) => Ok(PyRecord { inner: record }),
                None => Err(pyo3::exceptions::PyStopIteration::new_err(())),
            }
        } else {
            Err(pyo3::exceptions::PyStopIteration::new_err(()))
        }
    }

    fn __repr__(&self) -> String {
        if self.reader.is_some() {
            "<MARCReader active>".to_string()
        } else {
            "<MARCReader consumed>".to_string()
        }
    }
}
