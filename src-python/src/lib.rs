// MRRC Python wrapper using PyO3
// This module provides Python bindings to the Rust MARC library

mod authority_readers;
mod backend;
mod batched_reader;
mod batched_unified_reader;
mod bibframe;
mod boundary_scanner_wrapper;
mod buffered_reader;
mod error;
mod formats;
mod holdings_readers;
mod parse_error;
mod producer_consumer_pipeline_wrapper;
mod query;
mod rayon_parser_pool_wrapper;
mod readers;
mod unified_reader;
mod wrappers;
mod writers;

use authority_readers::PyAuthorityMARCReader;
use bibframe::{PyBibframeConfig, PyRdfGraph};
use boundary_scanner_wrapper::PyRecordBoundaryScanner;
use holdings_readers::PyHoldingsMARCReader;
use producer_consumer_pipeline_wrapper::PyProducerConsumerPipeline;
use pyo3::prelude::*;
use query::{PyFieldQuery, PySubfieldPatternQuery, PySubfieldValueQuery, PyTagRangeQuery};
use rayon_parser_pool_wrapper::{parse_batch_parallel, parse_batch_parallel_limited};
use readers::PyMARCReader;
use wrappers::{PyAuthorityRecord, PyField, PyHoldingsRecord, PyLeader, PyRecord, PySubfield};
use writers::PyMARCWriter;

/// Initialize the Python module
#[pymodule]
fn _mrrc(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLeader>()?;
    m.add_class::<PySubfield>()?;
    m.add_class::<PyField>()?;
    m.add_class::<PyRecord>()?;
    m.add_class::<PyAuthorityRecord>()?;
    m.add_class::<PyHoldingsRecord>()?;
    m.add_class::<PyMARCReader>()?;
    m.add_class::<PyAuthorityMARCReader>()?;
    m.add_class::<PyHoldingsMARCReader>()?;
    m.add_class::<PyMARCWriter>()?;
    m.add_class::<PyRecordBoundaryScanner>()?;
    m.add_class::<PyProducerConsumerPipeline>()?;

    // Query DSL classes
    m.add_class::<PyFieldQuery>()?;
    m.add_class::<PyTagRangeQuery>()?;
    m.add_class::<PySubfieldPatternQuery>()?;
    m.add_class::<PySubfieldValueQuery>()?;

    // Format conversion functions
    m.add_function(wrap_pyfunction!(formats::record_to_json, m)?)?;
    m.add_function(wrap_pyfunction!(formats::json_to_record, m)?)?;
    m.add_function(wrap_pyfunction!(formats::record_to_xml, m)?)?;
    m.add_function(wrap_pyfunction!(formats::xml_to_record, m)?)?;
    m.add_function(wrap_pyfunction!(formats::xml_to_records, m)?)?;
    m.add_function(wrap_pyfunction!(formats::record_to_marcjson, m)?)?;
    m.add_function(wrap_pyfunction!(formats::marcjson_to_record, m)?)?;
    m.add_function(wrap_pyfunction!(formats::record_to_dublin_core, m)?)?;
    m.add_function(wrap_pyfunction!(formats::record_to_dublin_core_xml, m)?)?;
    m.add_function(wrap_pyfunction!(formats::record_to_mods, m)?)?;
    m.add_function(wrap_pyfunction!(formats::mods_to_record, m)?)?;
    m.add_function(wrap_pyfunction!(formats::mods_collection_to_records, m)?)?;
    m.add_function(wrap_pyfunction!(formats::dublin_core_to_xml, m)?)?;
    m.add_function(wrap_pyfunction!(formats::record_to_csv, m)?)?;
    m.add_function(wrap_pyfunction!(formats::records_to_csv, m)?)?;
    m.add_function(wrap_pyfunction!(formats::records_to_csv_filtered, m)?)?;

    // BIBFRAME conversion (LOC linked data format)
    m.add_class::<PyBibframeConfig>()?;
    m.add_class::<PyRdfGraph>()?;
    m.add_function(wrap_pyfunction!(bibframe::py_marc_to_bibframe, m)?)?;
    m.add_function(wrap_pyfunction!(bibframe::py_bibframe_to_marc, m)?)?;

    // Rayon parser pool functions
    m.add_function(wrap_pyfunction!(parse_batch_parallel, m)?)?;
    m.add_function(wrap_pyfunction!(parse_batch_parallel_limited, m)?)?;

    m.add(
        "__doc__",
        "MRRC: A fast MARC library written in Rust with Python bindings",
    )?;
    m.add("__version__", "0.1.0")?;

    Ok(())
}
