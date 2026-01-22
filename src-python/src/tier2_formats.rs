// Python bindings for Tier 2 format support (Arrow, FlatBuffers, MessagePack)
//
// This module provides Python bindings for reading and writing MARC records
// in high-value binary formats. These are Tier 2 formats in mrrc, providing
// specialized capabilities beyond the core ISO 2709 and Protobuf formats.
//
// Components:
// - Arrow: Columnar format for analytics (DuckDB, Polars integration)
// - FlatBuffers: Zero-copy format for memory-efficient streaming
// - MessagePack: Compact binary with 50+ language support

use crate::error::marc_error_to_py_err;
use crate::wrappers::PyRecord;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Write};

// ============================================================================
// Arrow Format Support
// ============================================================================

use mrrc::arrow_impl::{ArrowReader, ArrowWriter};

/// Internal enum for Arrow reader backends
enum ArrowReaderBackend {
    Bytes(ArrowReader<Cursor<Vec<u8>>>),
    File(ArrowReader<BufReader<File>>),
}

/// Python wrapper for reading MARC records from Arrow IPC format.
///
/// ArrowReader provides streaming access to MARC records stored in Apache Arrow
/// IPC stream format. This format is ideal for analytics integration with tools
/// like DuckDB, Polars, and DataFusion.
///
/// ## Usage
///
/// ```python
/// import mrrc
///
/// # Read from file path
/// reader = mrrc.ArrowReader("records.arrow")
/// for record in reader:
///     print(record.title())
///
/// # Read from bytes
/// reader = mrrc.ArrowReader(arrow_bytes)
/// records = list(reader)
/// ```
#[pyclass(name = "ArrowReader")]
pub struct PyArrowReader {
    backend: Option<ArrowReaderBackend>,
}

#[pymethods]
impl PyArrowReader {
    /// Create a new ArrowReader from bytes or a file path.
    ///
    /// # Arguments
    /// * `source` - Either bytes/bytearray containing Arrow IPC data, or a file path (str/Path)
    #[new]
    pub fn new(source: &Bound<'_, PyAny>) -> PyResult<Self> {
        // Try to extract as bytes first
        if let Ok(bytes) = source.extract::<Vec<u8>>() {
            let cursor = Cursor::new(bytes);
            let reader = ArrowReader::new(cursor).map_err(marc_error_to_py_err)?;
            return Ok(PyArrowReader {
                backend: Some(ArrowReaderBackend::Bytes(reader)),
            });
        }

        // Try as string file path
        if let Ok(path_str) = source.extract::<String>() {
            let file = File::open(&path_str).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!(
                    "Failed to open file '{}': {}",
                    path_str, e
                ))
            })?;
            let buf_reader = BufReader::new(file);
            let reader = ArrowReader::new(buf_reader).map_err(marc_error_to_py_err)?;
            return Ok(PyArrowReader {
                backend: Some(ArrowReaderBackend::File(reader)),
            });
        }

        // Try pathlib.Path via __fspath__
        if let Ok(fspath) = source.getattr("__fspath__") {
            if fspath.is_callable() {
                if let Ok(path_obj) = fspath.call0() {
                    if let Ok(path_str) = path_obj.extract::<String>() {
                        let file = File::open(&path_str).map_err(|e| {
                            pyo3::exceptions::PyIOError::new_err(format!(
                                "Failed to open file '{}': {}",
                                path_str, e
                            ))
                        })?;
                        let buf_reader = BufReader::new(file);
                        let reader = ArrowReader::new(buf_reader).map_err(marc_error_to_py_err)?;
                        return Ok(PyArrowReader {
                            backend: Some(ArrowReaderBackend::File(reader)),
                        });
                    }
                }
            }
        }

        Err(pyo3::exceptions::PyTypeError::new_err(
            "ArrowReader() argument must be bytes, bytearray, or a file path (str/Path)",
        ))
    }

    /// Read the next record from the Arrow stream.
    pub fn read_record(&mut self) -> PyResult<Option<PyRecord>> {
        let result = match &mut self.backend {
            Some(ArrowReaderBackend::Bytes(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            Some(ArrowReaderBackend::File(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            None => {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Reader has been consumed",
                ))
            },
        };
        Ok(result.map(|inner| PyRecord { inner }))
    }

    /// Get the number of records read so far.
    pub fn records_read(&self) -> Option<usize> {
        match &self.backend {
            Some(ArrowReaderBackend::Bytes(reader)) => Some(reader.records_read()),
            Some(ArrowReaderBackend::File(reader)) => Some(reader.records_read()),
            None => None,
        }
    }

    fn __iter__(slf: PyRefMut<'_, Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRecord> {
        let result = match &mut slf.backend {
            Some(ArrowReaderBackend::Bytes(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            Some(ArrowReaderBackend::File(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            None => return Err(pyo3::exceptions::PyStopIteration::new_err(())),
        };

        match result {
            Some(record) => Ok(PyRecord { inner: record }),
            None => {
                slf.backend = None;
                Err(pyo3::exceptions::PyStopIteration::new_err(()))
            },
        }
    }

    fn __repr__(&self) -> String {
        match &self.backend {
            Some(ArrowReaderBackend::Bytes(reader)) => {
                format!("<ArrowReader records_read={}>", reader.records_read())
            },
            Some(ArrowReaderBackend::File(reader)) => {
                format!("<ArrowReader records_read={}>", reader.records_read())
            },
            None => "<ArrowReader consumed>".to_string(),
        }
    }
}

/// Internal enum for Arrow writer backends
enum ArrowWriterBackend {
    Buffer {
        writer: ArrowWriter<Cursor<Vec<u8>>>,
    },
    File(ArrowWriter<BufWriter<File>>),
}

/// Python wrapper for writing MARC records to Arrow IPC format.
///
/// ArrowWriter provides streaming output of MARC records to Apache Arrow IPC
/// stream format. Records are batched for efficient columnar storage.
///
/// ## Usage
///
/// ```python
/// import mrrc
///
/// # Write to file path
/// writer = mrrc.ArrowWriter("output.arrow")
/// writer.write_record(record1)
/// writer.write_record(record2)
/// writer.close()
///
/// # Write to memory buffer and get bytes
/// writer = mrrc.ArrowWriter()  # No path = memory buffer
/// writer.write_record(record)
/// arrow_bytes = writer.get_bytes()
/// ```
#[pyclass(name = "ArrowWriter")]
pub struct PyArrowWriter {
    backend: Option<ArrowWriterBackend>,
    closed: bool,
}

#[pymethods]
impl PyArrowWriter {
    /// Create a new ArrowWriter.
    ///
    /// # Arguments
    /// * `path` - Optional file path (str/Path). If None, writes to memory buffer.
    #[new]
    #[pyo3(signature = (path=None))]
    pub fn new(path: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        match path {
            None => {
                // In-memory buffer using Cursor<Vec<u8>>
                let buffer = Vec::new();
                let cursor = Cursor::new(buffer);
                let writer = ArrowWriter::new(cursor);
                Ok(PyArrowWriter {
                    backend: Some(ArrowWriterBackend::Buffer { writer }),
                    closed: false,
                })
            },
            Some(path_obj) => {
                if let Ok(path_str) = path_obj.extract::<String>() {
                    let file = File::create(&path_str).map_err(|e| {
                        pyo3::exceptions::PyIOError::new_err(format!(
                            "Failed to create file '{}': {}",
                            path_str, e
                        ))
                    })?;
                    let buf_writer = BufWriter::new(file);
                    let writer = ArrowWriter::new(buf_writer);
                    return Ok(PyArrowWriter {
                        backend: Some(ArrowWriterBackend::File(writer)),
                        closed: false,
                    });
                }

                if let Ok(fspath) = path_obj.getattr("__fspath__") {
                    if fspath.is_callable() {
                        if let Ok(path_result) = fspath.call0() {
                            if let Ok(path_str) = path_result.extract::<String>() {
                                let file = File::create(&path_str).map_err(|e| {
                                    pyo3::exceptions::PyIOError::new_err(format!(
                                        "Failed to create file '{}': {}",
                                        path_str, e
                                    ))
                                })?;
                                let buf_writer = BufWriter::new(file);
                                let writer = ArrowWriter::new(buf_writer);
                                return Ok(PyArrowWriter {
                                    backend: Some(ArrowWriterBackend::File(writer)),
                                    closed: false,
                                });
                            }
                        }
                    }
                }

                Err(pyo3::exceptions::PyTypeError::new_err(
                    "ArrowWriter() path argument must be a file path (str/Path) or None",
                ))
            },
        }
    }

    /// Write a single record to the Arrow stream.
    pub fn write_record(&mut self, record: &PyRecord) -> PyResult<()> {
        if self.closed {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer has been closed",
            ));
        }

        match &mut self.backend {
            Some(ArrowWriterBackend::Buffer { writer }) => writer
                .write_record(&record.inner)
                .map_err(marc_error_to_py_err),
            Some(ArrowWriterBackend::File(writer)) => writer
                .write_record(&record.inner)
                .map_err(marc_error_to_py_err),
            None => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer backend not initialized",
            )),
        }
    }

    /// Write alias for consistency.
    pub fn write(&mut self, record: &PyRecord) -> PyResult<()> {
        self.write_record(record)
    }

    /// Get the number of records written so far.
    pub fn records_written(&self) -> Option<usize> {
        match &self.backend {
            Some(ArrowWriterBackend::Buffer { writer }) => Some(writer.records_written()),
            Some(ArrowWriterBackend::File(writer)) => Some(writer.records_written()),
            None => None,
        }
    }

    /// Get the serialized bytes (only for memory buffer writers).
    ///
    /// This method finalizes the writer and returns the complete Arrow IPC data.
    /// After calling this method, the writer is closed.
    pub fn get_bytes<'py>(&mut self, _py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        if self.closed {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer has been closed",
            ));
        }

        match self.backend.take() {
            Some(ArrowWriterBackend::Buffer { mut writer }) => {
                writer.finish().map_err(marc_error_to_py_err)?;
                self.closed = true;
                // Get the inner cursor and extract its buffer
                // Unfortunately ArrowWriter doesn't expose inner, so we need a workaround
                // For now, return an error - users should use file-based output
                Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "get_bytes() is not yet supported for Arrow format. Use file-based output instead.",
                ))
            },
            Some(ArrowWriterBackend::File(_)) => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "get_bytes() is only available for memory buffer writers",
            )),
            None => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer backend not initialized",
            )),
        }
    }

    /// Close the writer and flush all data.
    pub fn close(&mut self) -> PyResult<()> {
        if !self.closed {
            match &mut self.backend {
                Some(ArrowWriterBackend::Buffer { writer }) => {
                    writer.finish().map_err(marc_error_to_py_err)?;
                },
                Some(ArrowWriterBackend::File(writer)) => {
                    writer.finish().map_err(marc_error_to_py_err)?;
                },
                None => {},
            }
            self.closed = true;
        }
        Ok(())
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

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
            "<ArrowWriter closed>".to_string()
        } else {
            match &self.backend {
                Some(ArrowWriterBackend::Buffer { writer }) => {
                    format!(
                        "<ArrowWriter memory records_written={}>",
                        writer.records_written()
                    )
                },
                Some(ArrowWriterBackend::File(writer)) => {
                    format!(
                        "<ArrowWriter file records_written={}>",
                        writer.records_written()
                    )
                },
                None => "<ArrowWriter uninitialized>".to_string(),
            }
        }
    }
}

// ============================================================================
// FlatBuffers Format Support
// ============================================================================

use mrrc::flatbuffers_impl::{
    FlatbuffersDeserializer, FlatbuffersReader, FlatbuffersSerializer, FlatbuffersWriter,
};

/// Internal enum for FlatBuffers reader backends
enum FlatbuffersReaderBackend {
    Bytes(FlatbuffersReader<Cursor<Vec<u8>>>),
    File(FlatbuffersReader<BufReader<File>>),
}

/// Python wrapper for reading MARC records from FlatBuffers format.
///
/// FlatbuffersReader provides streaming access to MARC records stored in
/// FlatBuffers binary format. This format offers zero-copy access and
/// memory-efficient streaming.
///
/// ## Usage
///
/// ```python
/// import mrrc
///
/// reader = mrrc.FlatbuffersReader("records.fb")
/// for record in reader:
///     print(record.title())
/// ```
#[pyclass(name = "FlatbuffersReader")]
pub struct PyFlatbuffersReader {
    backend: Option<FlatbuffersReaderBackend>,
}

#[pymethods]
impl PyFlatbuffersReader {
    /// Create a new FlatbuffersReader from bytes or a file path.
    #[new]
    pub fn new(source: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(bytes) = source.extract::<Vec<u8>>() {
            let cursor = Cursor::new(bytes);
            let reader = FlatbuffersReader::new(cursor);
            return Ok(PyFlatbuffersReader {
                backend: Some(FlatbuffersReaderBackend::Bytes(reader)),
            });
        }

        if let Ok(path_str) = source.extract::<String>() {
            let file = File::open(&path_str).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!(
                    "Failed to open file '{}': {}",
                    path_str, e
                ))
            })?;
            let buf_reader = BufReader::new(file);
            let reader = FlatbuffersReader::new(buf_reader);
            return Ok(PyFlatbuffersReader {
                backend: Some(FlatbuffersReaderBackend::File(reader)),
            });
        }

        if let Ok(fspath) = source.getattr("__fspath__") {
            if fspath.is_callable() {
                if let Ok(path_obj) = fspath.call0() {
                    if let Ok(path_str) = path_obj.extract::<String>() {
                        let file = File::open(&path_str).map_err(|e| {
                            pyo3::exceptions::PyIOError::new_err(format!(
                                "Failed to open file '{}': {}",
                                path_str, e
                            ))
                        })?;
                        let buf_reader = BufReader::new(file);
                        let reader = FlatbuffersReader::new(buf_reader);
                        return Ok(PyFlatbuffersReader {
                            backend: Some(FlatbuffersReaderBackend::File(reader)),
                        });
                    }
                }
            }
        }

        Err(pyo3::exceptions::PyTypeError::new_err(
            "FlatbuffersReader() argument must be bytes, bytearray, or a file path (str/Path)",
        ))
    }

    /// Read the next record from the FlatBuffers stream.
    pub fn read_record(&mut self) -> PyResult<Option<PyRecord>> {
        let result = match &mut self.backend {
            Some(FlatbuffersReaderBackend::Bytes(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            Some(FlatbuffersReaderBackend::File(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            None => {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Reader has been consumed",
                ))
            },
        };
        Ok(result.map(|inner| PyRecord { inner }))
    }

    /// Get the number of records read so far.
    pub fn records_read(&self) -> Option<usize> {
        match &self.backend {
            Some(FlatbuffersReaderBackend::Bytes(reader)) => Some(reader.records_read()),
            Some(FlatbuffersReaderBackend::File(reader)) => Some(reader.records_read()),
            None => None,
        }
    }

    fn __iter__(slf: PyRefMut<'_, Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRecord> {
        let result = match &mut slf.backend {
            Some(FlatbuffersReaderBackend::Bytes(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            Some(FlatbuffersReaderBackend::File(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            None => return Err(pyo3::exceptions::PyStopIteration::new_err(())),
        };

        match result {
            Some(record) => Ok(PyRecord { inner: record }),
            None => {
                slf.backend = None;
                Err(pyo3::exceptions::PyStopIteration::new_err(()))
            },
        }
    }

    fn __repr__(&self) -> String {
        match &self.backend {
            Some(FlatbuffersReaderBackend::Bytes(reader)) => {
                format!("<FlatbuffersReader records_read={}>", reader.records_read())
            },
            Some(FlatbuffersReaderBackend::File(reader)) => {
                format!("<FlatbuffersReader records_read={}>", reader.records_read())
            },
            None => "<FlatbuffersReader consumed>".to_string(),
        }
    }
}

/// Internal enum for FlatBuffers writer backends
enum FlatbuffersWriterBackend {
    Buffer {
        buffer: Vec<u8>,
        records_written: usize,
    },
    File(FlatbuffersWriter<BufWriter<File>>),
}

/// Python wrapper for writing MARC records to FlatBuffers format.
///
/// ## Usage
///
/// ```python
/// import mrrc
///
/// writer = mrrc.FlatbuffersWriter("output.fb")
/// writer.write_record(record)
/// writer.close()
/// ```
#[pyclass(name = "FlatbuffersWriter")]
pub struct PyFlatbuffersWriter {
    backend: Option<FlatbuffersWriterBackend>,
    closed: bool,
}

#[pymethods]
impl PyFlatbuffersWriter {
    /// Create a new FlatbuffersWriter.
    #[new]
    #[pyo3(signature = (path=None))]
    pub fn new(path: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        match path {
            None => Ok(PyFlatbuffersWriter {
                backend: Some(FlatbuffersWriterBackend::Buffer {
                    buffer: Vec::new(),
                    records_written: 0,
                }),
                closed: false,
            }),
            Some(path_obj) => {
                if let Ok(path_str) = path_obj.extract::<String>() {
                    let file = File::create(&path_str).map_err(|e| {
                        pyo3::exceptions::PyIOError::new_err(format!(
                            "Failed to create file '{}': {}",
                            path_str, e
                        ))
                    })?;
                    let buf_writer = BufWriter::new(file);
                    let writer = FlatbuffersWriter::new(buf_writer);
                    return Ok(PyFlatbuffersWriter {
                        backend: Some(FlatbuffersWriterBackend::File(writer)),
                        closed: false,
                    });
                }

                if let Ok(fspath) = path_obj.getattr("__fspath__") {
                    if fspath.is_callable() {
                        if let Ok(path_result) = fspath.call0() {
                            if let Ok(path_str) = path_result.extract::<String>() {
                                let file = File::create(&path_str).map_err(|e| {
                                    pyo3::exceptions::PyIOError::new_err(format!(
                                        "Failed to create file '{}': {}",
                                        path_str, e
                                    ))
                                })?;
                                let buf_writer = BufWriter::new(file);
                                let writer = FlatbuffersWriter::new(buf_writer);
                                return Ok(PyFlatbuffersWriter {
                                    backend: Some(FlatbuffersWriterBackend::File(writer)),
                                    closed: false,
                                });
                            }
                        }
                    }
                }

                Err(pyo3::exceptions::PyTypeError::new_err(
                    "FlatbuffersWriter() path argument must be a file path (str/Path) or None",
                ))
            },
        }
    }

    /// Write a single record to the FlatBuffers stream.
    pub fn write_record(&mut self, record: &PyRecord) -> PyResult<()> {
        if self.closed {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer has been closed",
            ));
        }

        match &mut self.backend {
            Some(FlatbuffersWriterBackend::Buffer {
                buffer,
                records_written,
            }) => {
                let record_bytes = FlatbuffersSerializer::serialize_size_prefixed(&record.inner)
                    .map_err(marc_error_to_py_err)?;
                buffer.write_all(&record_bytes).map_err(|e| {
                    pyo3::exceptions::PyIOError::new_err(format!("Failed to write record: {}", e))
                })?;
                *records_written += 1;
                Ok(())
            },
            Some(FlatbuffersWriterBackend::File(writer)) => writer
                .write_record(&record.inner)
                .map_err(marc_error_to_py_err),
            None => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer backend not initialized",
            )),
        }
    }

    /// Write alias for consistency.
    pub fn write(&mut self, record: &PyRecord) -> PyResult<()> {
        self.write_record(record)
    }

    /// Get the number of records written so far.
    pub fn records_written(&self) -> Option<usize> {
        match &self.backend {
            Some(FlatbuffersWriterBackend::Buffer {
                records_written, ..
            }) => Some(*records_written),
            Some(FlatbuffersWriterBackend::File(writer)) => Some(writer.records_written()),
            None => None,
        }
    }

    /// Get the serialized bytes (only for memory buffer writers).
    pub fn get_bytes<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        if self.closed {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer has been closed",
            ));
        }

        match self.backend.take() {
            Some(FlatbuffersWriterBackend::Buffer { buffer, .. }) => {
                self.closed = true;
                Ok(PyBytes::new(py, &buffer))
            },
            Some(FlatbuffersWriterBackend::File(_)) => {
                Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "get_bytes() is only available for memory buffer writers",
                ))
            },
            None => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer backend not initialized",
            )),
        }
    }

    /// Close the writer and flush all data.
    pub fn close(&mut self) -> PyResult<()> {
        if !self.closed {
            match &mut self.backend {
                Some(FlatbuffersWriterBackend::Buffer { .. }) => {},
                Some(FlatbuffersWriterBackend::File(writer)) => {
                    writer.finish().map_err(marc_error_to_py_err)?;
                },
                None => {},
            }
            self.closed = true;
        }
        Ok(())
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

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
            "<FlatbuffersWriter closed>".to_string()
        } else {
            match &self.backend {
                Some(FlatbuffersWriterBackend::Buffer {
                    records_written, ..
                }) => format!(
                    "<FlatbuffersWriter memory records_written={}>",
                    records_written
                ),
                Some(FlatbuffersWriterBackend::File(writer)) => format!(
                    "<FlatbuffersWriter file records_written={}>",
                    writer.records_written()
                ),
                None => "<FlatbuffersWriter uninitialized>".to_string(),
            }
        }
    }
}

/// Serialize a MARC record to FlatBuffers binary format.
#[pyfunction]
pub fn record_to_flatbuffers<'py>(
    py: Python<'py>,
    record: &PyRecord,
) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = FlatbuffersSerializer::serialize(&record.inner).map_err(marc_error_to_py_err)?;
    Ok(PyBytes::new(py, &bytes))
}

/// Deserialize a MARC record from FlatBuffers binary format.
#[pyfunction]
pub fn flatbuffers_to_record(data: &[u8]) -> PyResult<PyRecord> {
    let record = FlatbuffersDeserializer::deserialize(data).map_err(marc_error_to_py_err)?;
    Ok(PyRecord { inner: record })
}

// ============================================================================
// MessagePack Format Support
// ============================================================================

use mrrc::messagepack::{MessagePackReader, MessagePackWriter};

/// Internal enum for MessagePack reader backends
enum MessagePackReaderBackend {
    Bytes(MessagePackReader<Cursor<Vec<u8>>>),
    File(MessagePackReader<BufReader<File>>),
}

/// Python wrapper for reading MARC records from MessagePack format.
///
/// MessagePackReader provides streaming access to MARC records stored in
/// MessagePack binary format. This format offers compact serialization with
/// broad language support (50+ languages).
///
/// ## Usage
///
/// ```python
/// import mrrc
///
/// reader = mrrc.MessagePackReader("records.msgpack")
/// for record in reader:
///     print(record.title())
/// ```
#[pyclass(name = "MessagePackReader")]
pub struct PyMessagePackReader {
    backend: Option<MessagePackReaderBackend>,
}

#[pymethods]
impl PyMessagePackReader {
    /// Create a new MessagePackReader from bytes or a file path.
    #[new]
    pub fn new(source: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(bytes) = source.extract::<Vec<u8>>() {
            let cursor = Cursor::new(bytes);
            let reader = MessagePackReader::new(cursor);
            return Ok(PyMessagePackReader {
                backend: Some(MessagePackReaderBackend::Bytes(reader)),
            });
        }

        if let Ok(path_str) = source.extract::<String>() {
            let file = File::open(&path_str).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!(
                    "Failed to open file '{}': {}",
                    path_str, e
                ))
            })?;
            let buf_reader = BufReader::new(file);
            let reader = MessagePackReader::new(buf_reader);
            return Ok(PyMessagePackReader {
                backend: Some(MessagePackReaderBackend::File(reader)),
            });
        }

        if let Ok(fspath) = source.getattr("__fspath__") {
            if fspath.is_callable() {
                if let Ok(path_obj) = fspath.call0() {
                    if let Ok(path_str) = path_obj.extract::<String>() {
                        let file = File::open(&path_str).map_err(|e| {
                            pyo3::exceptions::PyIOError::new_err(format!(
                                "Failed to open file '{}': {}",
                                path_str, e
                            ))
                        })?;
                        let buf_reader = BufReader::new(file);
                        let reader = MessagePackReader::new(buf_reader);
                        return Ok(PyMessagePackReader {
                            backend: Some(MessagePackReaderBackend::File(reader)),
                        });
                    }
                }
            }
        }

        Err(pyo3::exceptions::PyTypeError::new_err(
            "MessagePackReader() argument must be bytes, bytearray, or a file path (str/Path)",
        ))
    }

    /// Read the next record from the MessagePack stream.
    pub fn read_record(&mut self) -> PyResult<Option<PyRecord>> {
        let result = match &mut self.backend {
            Some(MessagePackReaderBackend::Bytes(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            Some(MessagePackReaderBackend::File(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            None => {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Reader has been consumed",
                ))
            },
        };
        Ok(result.map(|inner| PyRecord { inner }))
    }

    /// Get the number of records read so far.
    pub fn records_read(&self) -> Option<usize> {
        match &self.backend {
            Some(MessagePackReaderBackend::Bytes(reader)) => Some(reader.records_read()),
            Some(MessagePackReaderBackend::File(reader)) => Some(reader.records_read()),
            None => None,
        }
    }

    fn __iter__(slf: PyRefMut<'_, Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRecord> {
        let result = match &mut slf.backend {
            Some(MessagePackReaderBackend::Bytes(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            Some(MessagePackReaderBackend::File(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            None => return Err(pyo3::exceptions::PyStopIteration::new_err(())),
        };

        match result {
            Some(record) => Ok(PyRecord { inner: record }),
            None => {
                slf.backend = None;
                Err(pyo3::exceptions::PyStopIteration::new_err(()))
            },
        }
    }

    fn __repr__(&self) -> String {
        match &self.backend {
            Some(MessagePackReaderBackend::Bytes(reader)) => {
                format!("<MessagePackReader records_read={}>", reader.records_read())
            },
            Some(MessagePackReaderBackend::File(reader)) => {
                format!("<MessagePackReader records_read={}>", reader.records_read())
            },
            None => "<MessagePackReader consumed>".to_string(),
        }
    }
}

/// Internal enum for MessagePack writer backends
enum MessagePackWriterBackend {
    Buffer(MessagePackWriter<Cursor<Vec<u8>>>),
    File(MessagePackWriter<BufWriter<File>>),
}

/// Python wrapper for writing MARC records to MessagePack format.
///
/// ## Usage
///
/// ```python
/// import mrrc
///
/// writer = mrrc.MessagePackWriter("output.msgpack")
/// writer.write_record(record)
/// writer.close()
/// ```
#[pyclass(name = "MessagePackWriter")]
pub struct PyMessagePackWriter {
    backend: Option<MessagePackWriterBackend>,
    closed: bool,
}

#[pymethods]
impl PyMessagePackWriter {
    /// Create a new MessagePackWriter.
    #[new]
    #[pyo3(signature = (path=None))]
    pub fn new(path: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        match path {
            None => {
                let cursor = Cursor::new(Vec::new());
                let writer = MessagePackWriter::new(cursor);
                Ok(PyMessagePackWriter {
                    backend: Some(MessagePackWriterBackend::Buffer(writer)),
                    closed: false,
                })
            },
            Some(path_obj) => {
                if let Ok(path_str) = path_obj.extract::<String>() {
                    let file = File::create(&path_str).map_err(|e| {
                        pyo3::exceptions::PyIOError::new_err(format!(
                            "Failed to create file '{}': {}",
                            path_str, e
                        ))
                    })?;
                    let buf_writer = BufWriter::new(file);
                    let writer = MessagePackWriter::new(buf_writer);
                    return Ok(PyMessagePackWriter {
                        backend: Some(MessagePackWriterBackend::File(writer)),
                        closed: false,
                    });
                }

                if let Ok(fspath) = path_obj.getattr("__fspath__") {
                    if fspath.is_callable() {
                        if let Ok(path_result) = fspath.call0() {
                            if let Ok(path_str) = path_result.extract::<String>() {
                                let file = File::create(&path_str).map_err(|e| {
                                    pyo3::exceptions::PyIOError::new_err(format!(
                                        "Failed to create file '{}': {}",
                                        path_str, e
                                    ))
                                })?;
                                let buf_writer = BufWriter::new(file);
                                let writer = MessagePackWriter::new(buf_writer);
                                return Ok(PyMessagePackWriter {
                                    backend: Some(MessagePackWriterBackend::File(writer)),
                                    closed: false,
                                });
                            }
                        }
                    }
                }

                Err(pyo3::exceptions::PyTypeError::new_err(
                    "MessagePackWriter() path argument must be a file path (str/Path) or None",
                ))
            },
        }
    }

    /// Write a single record to the MessagePack stream.
    pub fn write_record(&mut self, record: &PyRecord) -> PyResult<()> {
        if self.closed {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer has been closed",
            ));
        }

        match &mut self.backend {
            Some(MessagePackWriterBackend::Buffer(writer)) => writer
                .write_record(&record.inner)
                .map_err(marc_error_to_py_err),
            Some(MessagePackWriterBackend::File(writer)) => writer
                .write_record(&record.inner)
                .map_err(marc_error_to_py_err),
            None => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer backend not initialized",
            )),
        }
    }

    /// Write alias for consistency.
    pub fn write(&mut self, record: &PyRecord) -> PyResult<()> {
        self.write_record(record)
    }

    /// Get the number of records written so far.
    pub fn records_written(&self) -> Option<usize> {
        match &self.backend {
            Some(MessagePackWriterBackend::Buffer(writer)) => Some(writer.records_written()),
            Some(MessagePackWriterBackend::File(writer)) => Some(writer.records_written()),
            None => None,
        }
    }

    /// Get the serialized bytes (only for memory buffer writers).
    pub fn get_bytes<'py>(&mut self, _py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        if self.closed {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer has been closed",
            ));
        }

        match self.backend.take() {
            Some(MessagePackWriterBackend::Buffer(mut writer)) => {
                writer.finish().map_err(marc_error_to_py_err)?;
                self.closed = true;
                // Get the inner cursor's buffer
                // Unfortunately MessagePackWriter doesn't expose into_inner, so we can't easily
                // get the bytes. Return an error for now.
                Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "get_bytes() is not yet supported for MessagePack buffer mode. Use file-based output or record_to_messagepack() for single records.",
                ))
            },
            Some(MessagePackWriterBackend::File(_)) => {
                Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "get_bytes() is only available for memory buffer writers",
                ))
            },
            None => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer backend not initialized",
            )),
        }
    }

    /// Close the writer and flush all data.
    pub fn close(&mut self) -> PyResult<()> {
        if !self.closed {
            match &mut self.backend {
                Some(MessagePackWriterBackend::Buffer(writer)) => {
                    writer.finish().map_err(marc_error_to_py_err)?;
                },
                Some(MessagePackWriterBackend::File(writer)) => {
                    writer.finish().map_err(marc_error_to_py_err)?;
                },
                None => {},
            }
            self.closed = true;
        }
        Ok(())
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

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
            "<MessagePackWriter closed>".to_string()
        } else {
            match &self.backend {
                Some(MessagePackWriterBackend::Buffer(writer)) => {
                    format!(
                        "<MessagePackWriter memory records_written={}>",
                        writer.records_written()
                    )
                },
                Some(MessagePackWriterBackend::File(writer)) => format!(
                    "<MessagePackWriter file records_written={}>",
                    writer.records_written()
                ),
                None => "<MessagePackWriter uninitialized>".to_string(),
            }
        }
    }
}

/// Serialize a MARC record to MessagePack binary format.
#[pyfunction]
pub fn record_to_messagepack<'py>(
    py: Python<'py>,
    record: &PyRecord,
) -> PyResult<Bound<'py, PyBytes>> {
    // Serialize using the writer to a buffer
    let mut buffer = Vec::new();
    let mut writer = MessagePackWriter::new(&mut buffer);
    writer
        .write_record(&record.inner)
        .map_err(marc_error_to_py_err)?;
    writer.finish().map_err(marc_error_to_py_err)?;
    Ok(PyBytes::new(py, &buffer))
}

/// Deserialize a MARC record from MessagePack binary format.
#[pyfunction]
pub fn messagepack_to_record(data: &[u8]) -> PyResult<PyRecord> {
    let cursor = Cursor::new(data.to_vec());
    let mut reader = MessagePackReader::new(cursor);
    match reader.read_record().map_err(marc_error_to_py_err)? {
        Some(record) => Ok(PyRecord { inner: record }),
        None => Err(pyo3::exceptions::PyValueError::new_err(
            "No record found in MessagePack data",
        )),
    }
}
