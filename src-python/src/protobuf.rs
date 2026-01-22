// Python bindings for Protobuf format support (Tier 1)
//
// This module provides Python bindings for reading and writing MARC records
// in Protocol Buffers binary format. Protobuf is a Tier 1 format in mrrc,
// meaning it's always available without feature flags.
//
// Components:
// - PyProtobufReader: Streaming reader from bytes/files
// - PyProtobufWriter: Streaming writer to bytes/files
// - record_to_protobuf: Single-record serialization function
// - protobuf_to_record: Single-record deserialization function

use crate::error::marc_error_to_py_err;
use crate::wrappers::PyRecord;
use mrrc::protobuf::{ProtobufDeserializer, ProtobufReader, ProtobufSerializer, ProtobufWriter};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Write};

/// Internal enum for different reader backends
enum ProtobufReaderBackend {
    /// Reading from in-memory bytes
    Bytes(ProtobufReader<Cursor<Vec<u8>>>),
    /// Reading from a Rust file
    File(ProtobufReader<BufReader<File>>),
}

/// Python wrapper for reading MARC records from Protobuf format.
///
/// ProtobufReader provides streaming access to MARC records stored in
/// Protocol Buffers binary format. Records are read one at a time using
/// length-delimited encoding.
///
/// ## Usage
///
/// ```python
/// import mrrc
///
/// # Read from file path
/// reader = mrrc.ProtobufReader("records.pb")
/// for record in reader:
///     print(record.title())
///
/// # Read from bytes
/// with open("records.pb", "rb") as f:
///     data = f.read()
/// reader = mrrc.ProtobufReader(data)
/// for record in reader:
///     print(record.title())
/// ```
#[pyclass(name = "ProtobufReader")]
pub struct PyProtobufReader {
    backend: Option<ProtobufReaderBackend>,
}

#[pymethods]
impl PyProtobufReader {
    /// Create a new ProtobufReader from bytes or a file path.
    ///
    /// # Arguments
    /// * `source` - Either bytes/bytearray containing protobuf data, or a file path (str/Path)
    ///
    /// # Example
    /// ```python
    /// # From file path
    /// reader = mrrc.ProtobufReader("records.pb")
    ///
    /// # From bytes
    /// reader = mrrc.ProtobufReader(protobuf_bytes)
    /// ```
    #[new]
    pub fn new(source: &Bound<'_, PyAny>) -> PyResult<Self> {
        // Try to extract as bytes first
        if let Ok(bytes) = source.extract::<Vec<u8>>() {
            let cursor = Cursor::new(bytes);
            let reader = ProtobufReader::new(cursor);
            return Ok(PyProtobufReader {
                backend: Some(ProtobufReaderBackend::Bytes(reader)),
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
            let reader = ProtobufReader::new(buf_reader);
            return Ok(PyProtobufReader {
                backend: Some(ProtobufReaderBackend::File(reader)),
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
                        let reader = ProtobufReader::new(buf_reader);
                        return Ok(PyProtobufReader {
                            backend: Some(ProtobufReaderBackend::File(reader)),
                        });
                    }
                }
            }
        }

        Err(pyo3::exceptions::PyTypeError::new_err(
            "ProtobufReader() argument must be bytes, bytearray, or a file path (str/Path)",
        ))
    }

    /// Read the next record from the protobuf stream.
    ///
    /// Returns the next record, or None if end of stream is reached.
    ///
    /// # Example
    /// ```python
    /// reader = mrrc.ProtobufReader("records.pb")
    /// while (record := reader.read_record()) is not None:
    ///     process(record)
    /// ```
    pub fn read_record(&mut self) -> PyResult<Option<PyRecord>> {
        let result = match &mut self.backend {
            Some(ProtobufReaderBackend::Bytes(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            Some(ProtobufReaderBackend::File(reader)) => {
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
            Some(ProtobufReaderBackend::Bytes(reader)) => Some(reader.records_read()),
            Some(ProtobufReaderBackend::File(reader)) => Some(reader.records_read()),
            None => None,
        }
    }

    fn __iter__(slf: PyRefMut<'_, Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRecord> {
        let result = match &mut slf.backend {
            Some(ProtobufReaderBackend::Bytes(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            Some(ProtobufReaderBackend::File(reader)) => {
                reader.read_record().map_err(marc_error_to_py_err)?
            },
            None => {
                return Err(pyo3::exceptions::PyStopIteration::new_err(()));
            },
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
            Some(ProtobufReaderBackend::Bytes(reader)) => {
                format!("<ProtobufReader records_read={}>", reader.records_read())
            },
            Some(ProtobufReaderBackend::File(reader)) => {
                format!("<ProtobufReader records_read={}>", reader.records_read())
            },
            None => "<ProtobufReader consumed>".to_string(),
        }
    }
}

/// Internal enum for different writer backends
enum ProtobufWriterBackend {
    /// Writing to in-memory buffer (manual management for get_bytes() support)
    Buffer {
        buffer: Vec<u8>,
        records_written: usize,
    },
    /// Writing to a Rust file
    File(ProtobufWriter<BufWriter<File>>),
}

/// Python wrapper for writing MARC records to Protobuf format.
///
/// ProtobufWriter provides streaming output of MARC records to Protocol Buffers
/// binary format. Records are written with length-delimited encoding for
/// efficient streaming.
///
/// ## Usage
///
/// ```python
/// import mrrc
///
/// # Write to file path
/// writer = mrrc.ProtobufWriter("output.pb")
/// writer.write_record(record1)
/// writer.write_record(record2)
/// writer.close()
///
/// # Write to memory buffer and get bytes
/// writer = mrrc.ProtobufWriter()  # No path = memory buffer
/// writer.write_record(record)
/// protobuf_bytes = writer.get_bytes()
/// ```
#[pyclass(name = "ProtobufWriter")]
pub struct PyProtobufWriter {
    backend: Option<ProtobufWriterBackend>,
    closed: bool,
}

#[pymethods]
impl PyProtobufWriter {
    /// Create a new ProtobufWriter.
    ///
    /// # Arguments
    /// * `path` - Optional file path (str/Path). If None, writes to memory buffer.
    ///
    /// # Example
    /// ```python
    /// # Write to file
    /// writer = mrrc.ProtobufWriter("output.pb")
    ///
    /// # Write to memory (get bytes later with get_bytes())
    /// writer = mrrc.ProtobufWriter()
    /// ```
    #[new]
    #[pyo3(signature = (path=None))]
    pub fn new(path: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        match path {
            None => {
                // In-memory buffer - managed directly for get_bytes() support
                Ok(PyProtobufWriter {
                    backend: Some(ProtobufWriterBackend::Buffer {
                        buffer: Vec::new(),
                        records_written: 0,
                    }),
                    closed: false,
                })
            },
            Some(path_obj) => {
                // Try as string path
                if let Ok(path_str) = path_obj.extract::<String>() {
                    let file = File::create(&path_str).map_err(|e| {
                        pyo3::exceptions::PyIOError::new_err(format!(
                            "Failed to create file '{}': {}",
                            path_str, e
                        ))
                    })?;
                    let buf_writer = BufWriter::new(file);
                    let writer = ProtobufWriter::new(buf_writer);
                    return Ok(PyProtobufWriter {
                        backend: Some(ProtobufWriterBackend::File(writer)),
                        closed: false,
                    });
                }

                // Try pathlib.Path via __fspath__
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
                                let writer = ProtobufWriter::new(buf_writer);
                                return Ok(PyProtobufWriter {
                                    backend: Some(ProtobufWriterBackend::File(writer)),
                                    closed: false,
                                });
                            }
                        }
                    }
                }

                Err(pyo3::exceptions::PyTypeError::new_err(
                    "ProtobufWriter() path argument must be a file path (str/Path) or None",
                ))
            },
        }
    }

    /// Write a single record to the protobuf stream.
    ///
    /// # Arguments
    /// * `record` - The MARC record to write
    ///
    /// # Example
    /// ```python
    /// writer = mrrc.ProtobufWriter("output.pb")
    /// writer.write_record(record)
    /// ```
    pub fn write_record(&mut self, record: &PyRecord) -> PyResult<()> {
        if self.closed {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer has been closed",
            ));
        }

        match &mut self.backend {
            Some(ProtobufWriterBackend::Buffer {
                buffer,
                records_written,
            }) => {
                // Serialize to single-record bytes
                let record_bytes =
                    ProtobufSerializer::serialize(&record.inner).map_err(marc_error_to_py_err)?;

                // Write length-delimited format (varint length prefix + data)
                // This matches ProtobufWriter's streaming format
                write_length_delimited(buffer, &record_bytes).map_err(|e| {
                    pyo3::exceptions::PyIOError::new_err(format!("Failed to write record: {}", e))
                })?;
                *records_written += 1;
                Ok(())
            },
            Some(ProtobufWriterBackend::File(writer)) => writer
                .write_record(&record.inner)
                .map_err(marc_error_to_py_err),
            None => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer backend not initialized",
            )),
        }
    }

    /// Alias for write_record (for consistency with MARCWriter).
    pub fn write(&mut self, record: &PyRecord) -> PyResult<()> {
        self.write_record(record)
    }

    /// Get the number of records written so far.
    pub fn records_written(&self) -> Option<usize> {
        match &self.backend {
            Some(ProtobufWriterBackend::Buffer {
                records_written, ..
            }) => Some(*records_written),
            Some(ProtobufWriterBackend::File(writer)) => Some(writer.records_written()),
            None => None,
        }
    }

    /// Get the serialized bytes (only for memory buffer writers).
    ///
    /// This method finalizes the writer and returns the complete protobuf data.
    /// After calling this method, the writer is closed and cannot be used further.
    ///
    /// # Returns
    /// bytes containing the serialized protobuf data (length-delimited format)
    ///
    /// # Raises
    /// RuntimeError if the writer was created with a file path
    ///
    /// # Example
    /// ```python
    /// writer = mrrc.ProtobufWriter()  # Memory buffer
    /// writer.write_record(record)
    /// data = writer.get_bytes()  # Returns bytes
    /// ```
    pub fn get_bytes<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        if self.closed {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer has been closed",
            ));
        }

        match self.backend.take() {
            Some(ProtobufWriterBackend::Buffer { buffer, .. }) => {
                self.closed = true;
                Ok(PyBytes::new(py, &buffer))
            },
            Some(ProtobufWriterBackend::File(_)) => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "get_bytes() is only available for memory buffer writers (created without a path)",
            )),
            None => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Writer backend not initialized",
            )),
        }
    }

    /// Close the writer and flush all data.
    ///
    /// This must be called to ensure all data is written. The writer
    /// cannot be used after closing.
    pub fn close(&mut self) -> PyResult<()> {
        if !self.closed {
            match &mut self.backend {
                Some(ProtobufWriterBackend::Buffer { .. }) => {
                    // Buffer is already complete, nothing to flush
                },
                Some(ProtobufWriterBackend::File(writer)) => {
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
            "<ProtobufWriter closed>".to_string()
        } else {
            match &self.backend {
                Some(ProtobufWriterBackend::Buffer {
                    records_written, ..
                }) => {
                    format!(
                        "<ProtobufWriter memory records_written={}>",
                        records_written
                    )
                },
                Some(ProtobufWriterBackend::File(writer)) => {
                    format!(
                        "<ProtobufWriter file records_written={}>",
                        writer.records_written()
                    )
                },
                None => "<ProtobufWriter uninitialized>".to_string(),
            }
        }
    }
}

/// Write data with a varint length prefix (protobuf length-delimited format).
fn write_length_delimited<W: Write>(writer: &mut W, data: &[u8]) -> std::io::Result<()> {
    // Write varint length prefix
    let mut len = data.len();
    while len >= 0x80 {
        writer.write_all(&[(len as u8) | 0x80])?;
        len >>= 7;
    }
    writer.write_all(&[len as u8])?;

    // Write data
    writer.write_all(data)
}

/// Serialize a MARC record to Protocol Buffers binary format.
///
/// This function serializes a single MARC record to protobuf bytes.
/// For writing multiple records, use ProtobufWriter instead.
///
/// # Arguments
/// * `record` - The MARC record to serialize
///
/// # Returns
/// bytes containing the serialized protobuf data
///
/// # Example
/// ```python
/// import mrrc
///
/// record = mrrc.Record(mrrc.Leader())
/// protobuf_bytes = mrrc.record_to_protobuf(record)
/// ```
#[pyfunction]
pub fn record_to_protobuf<'py>(
    py: Python<'py>,
    record: &PyRecord,
) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = ProtobufSerializer::serialize(&record.inner).map_err(marc_error_to_py_err)?;
    Ok(PyBytes::new(py, &bytes))
}

/// Deserialize a MARC record from Protocol Buffers binary format.
///
/// This function deserializes a single MARC record from protobuf bytes.
/// For reading multiple records, use ProtobufReader instead.
///
/// # Arguments
/// * `data` - bytes/bytearray containing the protobuf-encoded record
///
/// # Returns
/// A MARC record
///
/// # Example
/// ```python
/// import mrrc
///
/// record = mrrc.protobuf_to_record(protobuf_bytes)
/// print(record.title())
/// ```
#[pyfunction]
pub fn protobuf_to_record(data: &[u8]) -> PyResult<PyRecord> {
    let record = ProtobufDeserializer::deserialize(data).map_err(marc_error_to_py_err)?;
    Ok(PyRecord { inner: record })
}
