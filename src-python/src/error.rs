// Map Rust [`MarcError`] values into typed Python exception instances.
//
// For each variant, this constructs the corresponding Python exception class
// from the `mrrc` module and passes positional context as keyword arguments
// (record_index, byte_offset, source, etc.). If the typed-exception
// construction fails for any reason — `mrrc` not importable, class missing,
// kwargs rejected — the mapping falls back to a bare `PyValueError` with the
// Rust `Display` output as its message, so an error never gets dropped.

use mrrc::{BytesNear, MarcError};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use pyo3::PyErr;

/// Map a Rust [`MarcError`] to a Python exception.
pub fn marc_error_to_py_err(err: MarcError) -> PyErr {
    Python::attach(|py| match build_typed(py, &err) {
        Ok(typed) => PyErr::from_value(typed),
        Err(construction_err) => fallback_with_cause(py, err, Some(construction_err)),
    })
}

fn fallback_with_cause(py: Python<'_>, err: MarcError, cause: Option<PyErr>) -> PyErr {
    let py_err = match err {
        MarcError::IoError { cause: io, .. } => PyIOError::new_err(io.to_string()),
        other => PyValueError::new_err(other.to_string()),
    };
    // Chain the construction failure as __cause__ so a broken install (mrrc
    // missing, class shape changed, kwargs rejected) is debuggable instead
    // of silently swallowed. Skipped when the variant is IoError, where we
    // intentionally route through the fallback path.
    if let Some(cause) = cause {
        py_err.set_cause(py, Some(cause));
    }
    py_err
}

/// Construct the typed Python exception instance corresponding to `err`.
///
/// Returns the Python exception object (a `Bound<PyAny>`) or any error
/// raised during attribute lookup, dictionary construction, or class call.
fn build_typed<'py>(py: Python<'py>, err: &MarcError) -> PyResult<Bound<'py, PyAny>> {
    let mrrc_module = py.import("mrrc")?;
    let (class_name, kwargs) = describe(py, err)?;
    let cls = mrrc_module.getattr(class_name)?;
    cls.call((), Some(&kwargs))
}

/// Pull the Python class name and kwargs out of `err`. Note: `IoError`
/// returns Err so the caller falls through to `PyIOError`, which is the
/// proper Python class for I/O errors (matches built-in `OSError`).
fn describe<'py>(py: Python<'py>, err: &MarcError) -> PyResult<(&'static str, Bound<'py, PyDict>)> {
    let kwargs = PyDict::new(py);
    let class_name: &'static str = match err {
        MarcError::InvalidLeader {
            record_index,
            byte_offset,
            record_byte_offset,
            source_name,
            found,
            expected,
            bytes_near,
            ..
        } => {
            populate_common(py, &kwargs, *record_index, None, None, source_name)?;
            kwargs.set_item("byte_offset", *byte_offset)?;
            kwargs.set_item("record_byte_offset", *record_byte_offset)?;
            set_found(py, &kwargs, found.as_deref())?;
            kwargs.set_item("expected", expected)?;
            set_bytes_near(py, &kwargs, bytes_near.as_ref())?;
            "RecordLeaderInvalid"
        },
        MarcError::RecordLengthInvalid {
            record_index,
            byte_offset,
            source_name,
            found,
            expected,
            bytes_near,
        } => {
            populate_common(py, &kwargs, *record_index, None, None, source_name)?;
            kwargs.set_item("byte_offset", *byte_offset)?;
            set_found(py, &kwargs, found.as_deref())?;
            kwargs.set_item("expected", expected)?;
            set_bytes_near(py, &kwargs, bytes_near.as_ref())?;
            "RecordLengthInvalid"
        },
        MarcError::BaseAddressInvalid {
            record_index,
            byte_offset,
            source_name,
            record_control_number,
            found,
            expected,
            bytes_near,
        } => {
            populate_common(
                py,
                &kwargs,
                *record_index,
                record_control_number.as_deref(),
                None,
                source_name,
            )?;
            kwargs.set_item("byte_offset", *byte_offset)?;
            set_found(py, &kwargs, found.as_deref())?;
            kwargs.set_item("expected", expected)?;
            set_bytes_near(py, &kwargs, bytes_near.as_ref())?;
            "BaseAddressInvalid"
        },
        MarcError::BaseAddressNotFound {
            record_index,
            byte_offset,
            source_name,
            record_control_number,
            bytes_near,
        } => {
            populate_common(
                py,
                &kwargs,
                *record_index,
                record_control_number.as_deref(),
                None,
                source_name,
            )?;
            kwargs.set_item("byte_offset", *byte_offset)?;
            set_bytes_near(py, &kwargs, bytes_near.as_ref())?;
            "BaseAddressNotFound"
        },
        MarcError::DirectoryInvalid {
            record_index,
            byte_offset,
            record_byte_offset,
            source_name,
            record_control_number,
            field_tag,
            found,
            expected,
            bytes_near,
        } => {
            populate_common(
                py,
                &kwargs,
                *record_index,
                record_control_number.as_deref(),
                field_tag.as_deref(),
                source_name,
            )?;
            kwargs.set_item("byte_offset", *byte_offset)?;
            kwargs.set_item("record_byte_offset", *record_byte_offset)?;
            set_found(py, &kwargs, found.as_deref())?;
            kwargs.set_item("expected", expected)?;
            set_bytes_near(py, &kwargs, bytes_near.as_ref())?;
            "RecordDirectoryInvalid"
        },
        MarcError::TruncatedRecord {
            record_index,
            byte_offset,
            record_byte_offset,
            source_name,
            record_control_number,
            expected_length,
            actual_length,
            bytes_near,
        } => {
            populate_common(
                py,
                &kwargs,
                *record_index,
                record_control_number.as_deref(),
                None,
                source_name,
            )?;
            kwargs.set_item("byte_offset", *byte_offset)?;
            kwargs.set_item("record_byte_offset", *record_byte_offset)?;
            kwargs.set_item("expected_length", *expected_length)?;
            kwargs.set_item("actual_length", *actual_length)?;
            set_bytes_near(py, &kwargs, bytes_near.as_ref())?;
            "TruncatedRecord"
        },
        MarcError::EndOfRecordNotFound {
            record_index,
            byte_offset,
            record_byte_offset,
            source_name,
            record_control_number,
            bytes_near,
        } => {
            populate_common(
                py,
                &kwargs,
                *record_index,
                record_control_number.as_deref(),
                None,
                source_name,
            )?;
            kwargs.set_item("byte_offset", *byte_offset)?;
            kwargs.set_item("record_byte_offset", *record_byte_offset)?;
            set_bytes_near(py, &kwargs, bytes_near.as_ref())?;
            "EndOfRecordNotFound"
        },
        MarcError::InvalidIndicator {
            record_index,
            byte_offset,
            record_byte_offset,
            source_name,
            record_control_number,
            field_tag,
            indicator_position,
            found,
            expected,
            bytes_near,
        } => {
            populate_common(
                py,
                &kwargs,
                *record_index,
                record_control_number.as_deref(),
                field_tag.as_deref(),
                source_name,
            )?;
            kwargs.set_item("byte_offset", *byte_offset)?;
            kwargs.set_item("record_byte_offset", *record_byte_offset)?;
            kwargs.set_item("indicator_position", *indicator_position)?;
            set_found(py, &kwargs, found.as_deref())?;
            kwargs.set_item("expected", expected)?;
            set_bytes_near(py, &kwargs, bytes_near.as_ref())?;
            "InvalidIndicator"
        },
        MarcError::BadSubfieldCode {
            record_index,
            byte_offset,
            record_byte_offset,
            source_name,
            record_control_number,
            field_tag,
            subfield_code,
            bytes_near,
        } => {
            populate_common(
                py,
                &kwargs,
                *record_index,
                record_control_number.as_deref(),
                field_tag.as_deref(),
                source_name,
            )?;
            kwargs.set_item("byte_offset", *byte_offset)?;
            kwargs.set_item("record_byte_offset", *record_byte_offset)?;
            kwargs.set_item("subfield_code", *subfield_code)?;
            set_bytes_near(py, &kwargs, bytes_near.as_ref())?;
            "BadSubfieldCode"
        },
        MarcError::InvalidField {
            record_index,
            byte_offset,
            record_byte_offset,
            source_name,
            record_control_number,
            field_tag,
            message,
            bytes_near,
        } => {
            populate_common(
                py,
                &kwargs,
                *record_index,
                record_control_number.as_deref(),
                field_tag.as_deref(),
                source_name,
            )?;
            kwargs.set_item("byte_offset", *byte_offset)?;
            kwargs.set_item("record_byte_offset", *record_byte_offset)?;
            kwargs.set_item("message", message)?;
            set_bytes_near(py, &kwargs, bytes_near.as_ref())?;
            "InvalidField"
        },
        MarcError::EncodingError {
            record_index,
            byte_offset,
            source_name,
            record_control_number,
            field_tag,
            message,
            bytes_near,
        } => {
            populate_common(
                py,
                &kwargs,
                *record_index,
                record_control_number.as_deref(),
                field_tag.as_deref(),
                source_name,
            )?;
            kwargs.set_item("byte_offset", *byte_offset)?;
            kwargs.set_item("message", message)?;
            set_bytes_near(py, &kwargs, bytes_near.as_ref())?;
            "EncodingError"
        },
        MarcError::FieldNotFound {
            record_index,
            record_control_number,
            field_tag,
        } => {
            populate_common(
                py,
                &kwargs,
                *record_index,
                record_control_number.as_deref(),
                Some(field_tag.as_str()),
                &None,
            )?;
            "FieldNotFound"
        },
        MarcError::XmlError {
            cause,
            record_index,
            byte_offset,
            source_name,
        } => {
            populate_common(py, &kwargs, *record_index, None, None, source_name)?;
            kwargs.set_item("byte_offset", *byte_offset)?;
            kwargs.set_item("message", cause.to_string())?;
            "XmlError"
        },
        MarcError::JsonError {
            cause,
            record_index,
            byte_offset,
            source_name,
        } => {
            populate_common(py, &kwargs, *record_index, None, None, source_name)?;
            kwargs.set_item("byte_offset", *byte_offset)?;
            kwargs.set_item("message", cause.to_string())?;
            "JsonError"
        },
        MarcError::WriterError {
            record_index,
            record_control_number,
            message,
        } => {
            populate_common(
                py,
                &kwargs,
                *record_index,
                record_control_number.as_deref(),
                None,
                &None,
            )?;
            kwargs.set_item("message", message)?;
            "WriterError"
        },
        MarcError::FatalReaderError {
            cap,
            errors_seen,
            record_index,
            source_name,
        } => {
            populate_common(py, &kwargs, *record_index, None, None, source_name)?;
            kwargs.set_item("cap", *cap)?;
            kwargs.set_item("errors_seen", *errors_seen)?;
            "FatalReaderError"
        },
        // I/O errors map to Python's built-in OSError via PyIOError, which
        // matches pymarc's behavior. Force the caller into the fallback.
        MarcError::IoError { .. } => {
            return Err(PyValueError::new_err("io error: use fallback"));
        },
    };
    Ok((class_name, kwargs))
}

fn populate_common<'py>(
    _py: Python<'py>,
    kwargs: &Bound<'py, PyDict>,
    record_index: Option<usize>,
    record_control_number: Option<&str>,
    field_tag: Option<&str>,
    source_name: &Option<String>,
) -> PyResult<()> {
    kwargs.set_item("record_index", record_index)?;
    kwargs.set_item("record_control_number", record_control_number)?;
    kwargs.set_item("field_tag", field_tag)?;
    kwargs.set_item("source", source_name.as_deref())?;
    Ok(())
}

fn set_found<'py>(
    py: Python<'py>,
    kwargs: &Bound<'py, PyDict>,
    found: Option<&[u8]>,
) -> PyResult<()> {
    match found {
        Some(bytes) => kwargs.set_item("found", PyBytes::new(py, bytes)),
        None => kwargs.set_item("found", py.None()),
    }
}

fn set_bytes_near<'py>(
    py: Python<'py>,
    kwargs: &Bound<'py, PyDict>,
    bytes_near: Option<&BytesNear>,
) -> PyResult<()> {
    match bytes_near {
        Some(window) => {
            kwargs.set_item("bytes_near", PyBytes::new(py, &window.bytes))?;
            kwargs.set_item("bytes_near_offset", window.start_offset)?;
        },
        None => {
            kwargs.set_item("bytes_near", py.None())?;
            kwargs.set_item("bytes_near_offset", py.None())?;
        },
    }
    Ok(())
}
