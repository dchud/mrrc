// MRRC Python wrapper using PyO3
// This module provides Python bindings to the Rust MARC library

mod error;
mod readers;
mod wrappers;
mod writers;

use pyo3::prelude::*;
use readers::PyMARCReader;
use wrappers::{PyField, PyLeader, PyRecord, PySubfield};
use writers::PyMARCWriter;

/// Initialize the Python module
#[pymodule]
fn _mrrc(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLeader>()?;
    m.add_class::<PySubfield>()?;
    m.add_class::<PyField>()?;
    m.add_class::<PyRecord>()?;
    m.add_class::<PyMARCReader>()?;
    m.add_class::<PyMARCWriter>()?;

    m.add(
        "__doc__",
        "MRRC: A fast MARC library written in Rust with Python bindings",
    )?;
    m.add("__version__", "0.1.0")?;

    Ok(())
}
