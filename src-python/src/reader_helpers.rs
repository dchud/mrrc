//! Shared helpers for MARC reader backends (authority, holdings).
//!
//! Eliminates duplication between authority_readers.rs and holdings_readers.rs
//! for common operations: recovery mode parsing, source file opening, and
//! reading raw record bytes from Python file objects.

use mrrc::recovery::RecoveryMode;
use pyo3::prelude::*;
use std::fs::File;

/// Parse a recovery_mode string into a RecoveryMode enum.
///
/// Returns `PyValueError` for invalid values.
pub fn parse_recovery_mode(mode: &str) -> PyResult<RecoveryMode> {
    match mode {
        "lenient" => Ok(RecoveryMode::Lenient),
        "permissive" => Ok(RecoveryMode::Permissive),
        "strict" => Ok(RecoveryMode::Strict),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Invalid recovery_mode '{}': must be 'strict', 'lenient', or 'permissive'",
            mode
        ))),
    }
}

/// Try to open a source as a file path (str or pathlib.Path).
///
/// Returns `Ok(Some(file))` if the source is a path, `Ok(None)` if it's not
/// a path type, or `Err` for file I/O errors.
pub fn try_open_as_path(source: &Bound<'_, PyAny>) -> PyResult<Option<File>> {
    // Try str path first
    if let Ok(path_str) = source.extract::<String>() {
        return open_file_path(&path_str).map(Some);
    }

    // Try pathlib.Path via __fspath__
    if let Ok(method) = source.getattr("__fspath__") {
        if method.is_callable() {
            if let Ok(path_obj) = method.call0() {
                if let Ok(path_str) = path_obj.extract::<String>() {
                    return open_file_path(&path_str).map(Some);
                }
            }
        }
    }

    Ok(None)
}

/// Open a file path with proper Python error mapping.
fn open_file_path(path: &str) -> PyResult<File> {
    match File::open(path) {
        Ok(file) => Ok(file),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(pyo3::exceptions::PyFileNotFoundError::new_err(format!(
                "No such file or directory: '{}'",
                path
            )))
        },
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => Err(
            pyo3::exceptions::PyPermissionError::new_err(format!("Permission denied: '{}'", path)),
        ),
        Err(e) => Err(pyo3::exceptions::PyIOError::new_err(format!(
            "Failed to open file '{}': {}",
            path, e
        ))),
    }
}

/// Try to extract bytes/bytearray from a source.
///
/// Returns `Ok(Some(bytes))` if the source is bytes, `Ok(None)` otherwise.
pub fn try_extract_bytes(source: &Bound<'_, PyAny>) -> PyResult<Option<Vec<u8>>> {
    match source.extract::<Vec<u8>>() {
        Ok(bytes) => Ok(Some(bytes)),
        Err(_) => Ok(None),
    }
}

/// Try to get a Python file-like object (has .read() method).
///
/// Returns `Ok(Some(obj))` if the source has a callable .read(), `Ok(None)` otherwise.
pub fn try_as_python_file(source: &Bound<'_, PyAny>) -> PyResult<Option<Py<PyAny>>> {
    if let Ok(method) = source.getattr("read") {
        if method.is_callable() {
            return Ok(Some(source.clone().unbind()));
        }
    }
    Ok(None)
}

/// Return a type error for unsupported input types.
pub fn unsupported_source_error(source: &Bound<'_, PyAny>) -> PyResult<()> {
    let type_name = source.get_type().name()?;
    Err(pyo3::exceptions::PyTypeError::new_err(format!(
        "Unsupported input type: {}. Supported types: str (file path), pathlib.Path, \
         bytes, bytearray, or file-like object (with .read() method)",
        type_name
    )))
}

/// Read one complete MARC record as raw bytes from a Python file object.
///
/// Returns `Ok(Some(bytes))` for a complete record, `Ok(None)` at EOF,
/// or `Err` for malformed/truncated data.
pub fn read_record_bytes_from_python_file(py_obj: &Bound<'_, PyAny>) -> PyResult<Option<Vec<u8>>> {
    let read_method = py_obj.getattr("read").map_err(|e| {
        pyo3::exceptions::PyIOError::new_err(format!("Object missing .read() method: {}", e))
    })?;

    // Read leader (24 bytes)
    let leader_result = read_method.call1((24usize,)).map_err(|e| {
        pyo3::exceptions::PyIOError::new_err(format!("Failed to read leader: {}", e))
    })?;

    let leader: Vec<u8> = leader_result
        .extract()
        .map_err(|_| pyo3::exceptions::PyValueError::new_err("Leader must be bytes"))?;

    if leader.is_empty() {
        return Ok(None); // EOF
    }

    if leader.len() != 24 {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Incomplete leader: expected 24 bytes, got {}",
            leader.len()
        )));
    }

    // Parse record length from leader
    let record_length_str = String::from_utf8_lossy(&leader[0..5]);
    let record_length: usize = record_length_str.trim().parse().map_err(|_| {
        pyo3::exceptions::PyValueError::new_err(format!(
            "Invalid record length in leader: '{}'",
            record_length_str
        ))
    })?;

    if record_length < 24 {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Record length too small: {} (minimum 24)",
            record_length
        )));
    }

    // Read remainder
    let record_data_bytes = read_method.call1((record_length - 24,)).map_err(|e| {
        pyo3::exceptions::PyIOError::new_err(format!("Failed to read record data: {}", e))
    })?;

    let record_data: Vec<u8> = record_data_bytes
        .extract()
        .map_err(|_| pyo3::exceptions::PyValueError::new_err("Record data must be bytes"))?;

    if record_data.len() != record_length - 24 {
        return Err(crate::parse_error::ParseError::truncated_record(
            record_length - 24,
            record_data.len(),
        )
        .to_py_err());
    }

    // Assemble complete record
    let mut complete_record = Vec::with_capacity(record_length);
    complete_record.extend_from_slice(&leader);
    complete_record.extend_from_slice(&record_data);

    Ok(Some(complete_record))
}
