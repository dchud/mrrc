// Map Rust [`MarcError`] values into typed Python exception instances.
//
// For each variant, this constructs the corresponding Python exception class
// from the `mrrc` module and passes positional context as keyword arguments
// (record_index, byte_offset, source, etc.). If the typed-exception
// construction fails for any reason — `mrrc` not importable, class missing,
// kwargs rejected — the mapping falls back to a bare `PyValueError` with the
// Rust `Display` output as its message, so an error never gets dropped.

use mrrc::MarcError;
use pyo3::PyErr;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

/// Map a Rust [`MarcError`] to a Python exception.
pub fn marc_error_to_py_err(err: MarcError) -> PyErr {
    Python::attach(|py| match build_typed(py, &err) {
        Ok(typed) => PyErr::from_value(typed),
        Err(construction_err) => fallback_with_cause(py, err, Some(construction_err)),
    })
}

/// Build a typed Python exception *instance* from a [`MarcError`] without
/// raising it. Used to populate `record.errors` lists where errors are
/// observed rather than thrown. Falls back to a `PyValueError` instance
/// (carrying the construction failure as its cause) on the same shapes that
/// [`marc_error_to_py_err`] handles via fallback.
pub fn marc_error_to_py_object(py: Python<'_>, err: &MarcError) -> Py<PyAny> {
    match build_typed(py, err) {
        Ok(obj) => obj.into(),
        Err(construction_err) => {
            let py_err = match err {
                MarcError::IoError { cause: io, .. } => PyIOError::new_err(io.to_string()),
                other => PyValueError::new_err(other.to_string()),
            };
            py_err.set_cause(py, Some(construction_err));
            py_err.into_value(py).into()
        },
    }
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
///
/// The positional kwargs (`record_index`, `byte_offset`, `found`, ...) are
/// driven uniformly from [`MarcError::metadata`] — every mrrc exception
/// class accepts the full positional set with `None` defaults, so absent
/// fields simply pass through as `None`. Only the Python class name and the
/// per-class extra kwargs (`message`, `expected_length`/`actual_length`,
/// `cap`/`errors_seen` — rejected by classes that don't declare them) need
/// the per-variant `match`.
fn describe<'py>(py: Python<'py>, err: &MarcError) -> PyResult<(&'static str, Bound<'py, PyDict>)> {
    let md = err.metadata();
    let kwargs = PyDict::new(py);
    kwargs.set_item("record_index", md.record_index)?;
    kwargs.set_item("record_control_number", md.record_control_number)?;
    kwargs.set_item("field_tag", md.field_tag)?;
    kwargs.set_item("source", md.source_name)?;
    kwargs.set_item("byte_offset", md.byte_offset)?;
    kwargs.set_item("record_byte_offset", md.record_byte_offset)?;
    kwargs.set_item("indicator_position", md.indicator_position)?;
    kwargs.set_item("subfield_code", md.subfield_code)?;
    kwargs.set_item("expected", md.expected)?;
    match md.found {
        Some(bytes) => kwargs.set_item("found", PyBytes::new(py, bytes))?,
        None => kwargs.set_item("found", py.None())?,
    }
    match md.bytes_near {
        Some(window) => {
            kwargs.set_item("bytes_near", PyBytes::new(py, &window.bytes))?;
            kwargs.set_item("bytes_near_offset", window.start_offset)?;
        },
        None => {
            kwargs.set_item("bytes_near", py.None())?;
            kwargs.set_item("bytes_near_offset", py.None())?;
        },
    }

    let class_name: &'static str = match err {
        MarcError::InvalidLeader { message, .. } => {
            kwargs.set_item("message", message)?;
            "RecordLeaderInvalid"
        },
        MarcError::RecordLengthInvalid { .. } => "RecordLengthInvalid",
        MarcError::BaseAddressInvalid { .. } => "BaseAddressInvalid",
        MarcError::BaseAddressNotFound { .. } => "BaseAddressNotFound",
        MarcError::DirectoryInvalid { .. } => "RecordDirectoryInvalid",
        MarcError::TruncatedRecord {
            expected_length,
            actual_length,
            ..
        } => {
            kwargs.set_item("expected_length", *expected_length)?;
            kwargs.set_item("actual_length", *actual_length)?;
            "TruncatedRecord"
        },
        MarcError::EndOfRecordNotFound { .. } => "EndOfRecordNotFound",
        MarcError::InvalidIndicator { .. } => "InvalidIndicator",
        MarcError::BadSubfieldCode { .. } => "BadSubfieldCode",
        MarcError::InvalidField { message, .. } => {
            kwargs.set_item("message", message)?;
            "InvalidField"
        },
        MarcError::EncodingError { message, .. } => {
            kwargs.set_item("message", message)?;
            "EncodingError"
        },
        MarcError::FieldNotFound { .. } => "FieldNotFound",
        MarcError::XmlError { cause, .. } => {
            kwargs.set_item("message", cause.to_string())?;
            "XmlError"
        },
        MarcError::JsonError { cause, .. } => {
            kwargs.set_item("message", cause.to_string())?;
            "JsonError"
        },
        MarcError::WriterError { message, .. } => {
            kwargs.set_item("message", message)?;
            "WriterError"
        },
        MarcError::FatalReaderError {
            cap, errors_seen, ..
        } => {
            kwargs.set_item("cap", *cap)?;
            kwargs.set_item("errors_seen", *errors_seen)?;
            "FatalReaderError"
        },
        // I/O errors map to Python's built-in OSError via PyIOError, which
        // matches pymarc's behavior. Force the caller into the fallback.
        MarcError::IoError { .. } => {
            return Err(PyValueError::new_err("io error: use fallback"));
        },
        // MarcError is #[non_exhaustive]: a variant added in the core crate
        // before this mapping learns about it routes through the fallback
        // (PyValueError carrying the Display text) instead of being dropped.
        _ => {
            return Err(PyValueError::new_err(
                "unmapped error variant: use fallback",
            ));
        },
    };
    Ok((class_name, kwargs))
}
