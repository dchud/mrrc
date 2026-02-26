// Python bindings for MARC format conversion functions
//
// This module exposes the format conversion capabilities of mrrc to Python:
// - JSON serialization/deserialization
// - XML serialization/deserialization
// - MARCJSON serialization/deserialization
// - Dublin Core conversion
// - MODS XML conversion
// - CSV export

use crate::error::marc_error_to_py_err;
use crate::wrappers::PyRecord;
use mrrc::{csv, dublin_core, json, marcjson, marcxml, mods};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde_json::Value;

/// Convert a MARC record to JSON.
///
/// # Arguments
/// * `record` - A PyRecord instance
///
/// # Returns
/// A JSON string representation of the record
///
/// # Example
/// ```python
/// import mrrc
/// record = mrrc.Record(mrrc.Leader())
/// json_str = mrrc.record_to_json(record)
/// ```
#[pyfunction]
pub fn record_to_json(record: &PyRecord) -> PyResult<String> {
    json::record_to_json(&record.inner)
        .map(|v| v.to_string())
        .map_err(marc_error_to_py_err)
}

/// Convert JSON back to a MARC record.
///
/// # Arguments
/// * `json_str` - A JSON string representing a MARC record
///
/// # Returns
/// A PyRecord instance
///
/// # Example
/// ```python
/// import mrrc
/// json_str = '...'  # JSON representation of a record
/// record = mrrc.json_to_record(json_str)
/// ```
#[pyfunction]
pub fn json_to_record(json_str: &str) -> PyResult<PyRecord> {
    let json_value: Value = serde_json::from_str(json_str)
        .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;
    json::json_to_record(&json_value)
        .map(|inner| PyRecord { inner })
        .map_err(marc_error_to_py_err)
}

/// Convert a MARC record to MARCXML.
///
/// # Arguments
/// * `record` - A PyRecord instance
///
/// # Returns
/// A MARCXML string representation of the record
///
/// # Example
/// ```python
/// import mrrc
/// record = mrrc.Record(mrrc.Leader())
/// xml_str = mrrc.record_to_xml(record)
/// ```
#[pyfunction]
pub fn record_to_xml(record: &PyRecord) -> PyResult<String> {
    marcxml::record_to_marcxml(&record.inner).map_err(marc_error_to_py_err)
}

/// Convert MARCXML back to a MARC record.
///
/// # Arguments
/// * `xml_str` - A MARCXML string representing a MARC record
///
/// # Returns
/// A PyRecord instance
///
/// # Example
/// ```python
/// import mrrc
/// xml_str = '...'  # MARCXML representation of a record
/// record = mrrc.xml_to_record(xml_str)
/// ```
#[pyfunction]
pub fn xml_to_record(xml_str: &str) -> PyResult<PyRecord> {
    marcxml::marcxml_to_record(xml_str)
        .map(|inner| PyRecord { inner })
        .map_err(marc_error_to_py_err)
}

/// Convert a MARCXML collection to multiple MARC records.
///
/// # Arguments
/// * `xml_str` - A MARCXML string containing a `<collection>` element
///
/// # Returns
/// A list of PyRecord instances
///
/// # Example
/// ```python
/// import mrrc
/// records = mrrc.xml_to_records(xml_collection_str)
/// ```
#[pyfunction]
pub fn xml_to_records(xml_str: &str) -> PyResult<Vec<PyRecord>> {
    marcxml::marcxml_to_records(xml_str)
        .map(|records| {
            records
                .into_iter()
                .map(|inner| PyRecord { inner })
                .collect()
        })
        .map_err(marc_error_to_py_err)
}

/// Convert a MARC record to MARCJSON (JSON-LD format).
///
/// # Arguments
/// * `record` - A PyRecord instance
///
/// # Returns
/// A JSON string in MARCJSON format
///
/// # Example
/// ```python
/// import mrrc
/// record = mrrc.Record(mrrc.Leader())
/// marcjson_str = mrrc.record_to_marcjson(record)
/// ```
#[pyfunction]
pub fn record_to_marcjson(record: &PyRecord) -> PyResult<String> {
    marcjson::record_to_marcjson(&record.inner)
        .map(|v| v.to_string())
        .map_err(marc_error_to_py_err)
}

/// Convert MARCJSON back to a MARC record.
///
/// # Arguments
/// * `marcjson_str` - A JSON string in MARCJSON format
///
/// # Returns
/// A PyRecord instance
///
/// # Example
/// ```python
/// import mrrc
/// marcjson_str = '...'  # MARCJSON representation of a record
/// record = mrrc.marcjson_to_record(marcjson_str)
/// ```
#[pyfunction]
pub fn marcjson_to_record(marcjson_str: &str) -> PyResult<PyRecord> {
    let json_value: Value = serde_json::from_str(marcjson_str)
        .map_err(|e| PyValueError::new_err(format!("Invalid MARCJSON: {}", e)))?;
    marcjson::marcjson_to_record(&json_value)
        .map(|inner| PyRecord { inner })
        .map_err(marc_error_to_py_err)
}

/// Convert a MARC record to Dublin Core metadata.
///
/// Returns a dictionary with Dublin Core elements extracted from the record.
///
/// # Arguments
/// * `record` - A PyRecord instance
///
/// # Returns
/// A dictionary mapping Dublin Core field names to values
///
/// # Example
/// ```python
/// import mrrc
/// record = mrrc.Record(mrrc.Leader())
/// dc = mrrc.record_to_dublin_core(record)
/// print(dc['title'])
/// ```
#[pyfunction]
pub fn record_to_dublin_core(
    record: &PyRecord,
) -> PyResult<std::collections::HashMap<String, Vec<String>>> {
    dublin_core::record_to_dublin_core(&record.inner)
        .map(|dc| {
            let mut map = std::collections::HashMap::new();
            map.insert("title".to_string(), dc.title);
            map.insert("creator".to_string(), dc.creator);
            map.insert("subject".to_string(), dc.subject);
            map.insert("description".to_string(), dc.description);
            map.insert("publisher".to_string(), dc.publisher);
            map.insert("contributor".to_string(), dc.contributor);
            map.insert("date".to_string(), dc.date);
            map.insert("type".to_string(), dc.dc_type);
            map.insert("format".to_string(), dc.format);
            map.insert("identifier".to_string(), dc.identifier);
            map.insert("source".to_string(), dc.source);
            map.insert("language".to_string(), dc.language);
            map.insert("relation".to_string(), dc.relation);
            map.insert("coverage".to_string(), dc.coverage);
            map.insert("rights".to_string(), dc.rights);
            map
        })
        .map_err(marc_error_to_py_err)
}

/// Convert a MARC record to MODS XML.
///
/// MODS (Metadata Object Description Schema) is a more detailed
/// XML representation than Dublin Core.
///
/// # Arguments
/// * `record` - A PyRecord instance
///
/// # Returns
/// An XML string in MODS format
///
/// # Example
/// ```python
/// import mrrc
/// record = mrrc.Record(mrrc.Leader())
/// mods_xml = mrrc.record_to_mods(record)
/// ```
#[pyfunction]
pub fn record_to_mods(record: &PyRecord) -> PyResult<String> {
    mods::record_to_mods_xml(&record.inner).map_err(marc_error_to_py_err)
}

/// Parse a MODS XML string into a MARC record.
///
/// # Arguments
/// * `xml_str` - A MODS XML string
///
/// # Returns
/// A PyRecord instance
///
/// # Example
/// ```python
/// import mrrc
/// record = mrrc.mods_to_record(mods_xml)
/// ```
#[pyfunction]
pub fn mods_to_record(xml_str: &str) -> PyResult<PyRecord> {
    mods::mods_xml_to_record(xml_str)
        .map(|inner| PyRecord { inner })
        .map_err(marc_error_to_py_err)
}

/// Parse a MODS collection XML string into multiple MARC records.
///
/// # Arguments
/// * `xml_str` - A MODS collection XML string containing `<modsCollection>`
///
/// # Returns
/// A list of PyRecord instances
///
/// # Example
/// ```python
/// import mrrc
/// records = mrrc.mods_collection_to_records(mods_collection_xml)
/// ```
#[pyfunction]
pub fn mods_collection_to_records(xml_str: &str) -> PyResult<Vec<PyRecord>> {
    mods::mods_xml_to_records(xml_str)
        .map(|records| {
            records
                .into_iter()
                .map(|inner| PyRecord { inner })
                .collect()
        })
        .map_err(marc_error_to_py_err)
}

/// Convert a MARC record directly to Dublin Core XML format.
///
/// Convenience function that combines record_to_dublin_core() and dublin_core_to_xml()
/// in a single call.
///
/// # Arguments
/// * `record` - A PyRecord instance
///
/// # Returns
/// An XML string in Dublin Core RDF/XML format
///
/// # Example
/// ```python
/// import mrrc
/// record = mrrc.Record(mrrc.Leader())
/// dc_xml = mrrc.record_to_dublin_core_xml(record)
/// print(dc_xml)
/// ```
#[pyfunction]
pub fn record_to_dublin_core_xml(record: &PyRecord) -> PyResult<String> {
    dublin_core::record_to_dublin_core_xml(&record.inner).map_err(marc_error_to_py_err)
}

/// Convert Dublin Core metadata to XML.
///
/// # Arguments
/// * `dublin_core` - A dictionary with Dublin Core elements
///
/// # Returns
/// An XML string in Dublin Core XML format
///
/// # Example
/// ```python
/// import mrrc
/// dc = {'title': ['Example'], 'creator': ['Author']}
/// dc_xml = mrrc.dublin_core_to_xml(dc)
/// ```
#[pyfunction]
pub fn dublin_core_to_xml(
    dublin_core: std::collections::HashMap<String, Vec<String>>,
) -> PyResult<String> {
    // Helper to extract list from map, defaulting to empty vec
    let get_string_list =
        |key: &str| -> Vec<String> { dublin_core.get(key).cloned().unwrap_or_default() };

    let dc_record = mrrc::dublin_core::DublinCoreRecord {
        title: get_string_list("title"),
        creator: get_string_list("creator"),
        subject: get_string_list("subject"),
        description: get_string_list("description"),
        publisher: get_string_list("publisher"),
        contributor: get_string_list("contributor"),
        date: get_string_list("date"),
        dc_type: get_string_list("type"),
        format: get_string_list("format"),
        identifier: get_string_list("identifier"),
        source: get_string_list("source"),
        language: get_string_list("language"),
        relation: get_string_list("relation"),
        coverage: get_string_list("coverage"),
        rights: get_string_list("rights"),
    };

    Ok(dublin_core::dublin_core_to_xml(&dc_record))
}

/// Convert a single MARC record to CSV format.
///
/// Produces a CSV with one row per field/subfield combination, with columns:
/// - `tag`: The MARC field tag
/// - `ind1`: First indicator (or empty for control fields)
/// - `ind2`: Second indicator (or empty for control fields)
/// - `subfield_code`: The subfield code (or empty for control fields)
/// - `value`: The field or subfield value
///
/// # Arguments
/// * `record` - A PyRecord instance (can be the internal _mrrc.Record or wrapped Record)
///
/// # Returns
/// A CSV string representation of the record
///
/// # Example
/// ```python
/// import mrrc
/// record = mrrc.Record(mrrc.Leader())
/// csv_str = mrrc.record_to_csv(record)
/// ```
#[pyfunction]
pub fn record_to_csv(record: &pyo3::Bound<'_, pyo3::PyAny>) -> PyResult<String> {
    // Try to extract as PyRecord directly first
    if let Ok(py_record) = record.extract::<pyo3::PyRef<'_, PyRecord>>() {
        return csv::record_to_csv(&py_record.inner).map_err(marc_error_to_py_err);
    }

    // Otherwise, try to get the _inner attribute (wrapped Record)
    if let Ok(inner) = record.getattr("_inner") {
        if let Ok(py_record) = inner.extract::<pyo3::PyRef<'_, PyRecord>>() {
            return csv::record_to_csv(&py_record.inner).map_err(marc_error_to_py_err);
        }
    }

    Err(pyo3::exceptions::PyTypeError::new_err(
        "record must be a PyRecord or wrapped Record",
    ))
}

/// Convert multiple MARC records to CSV format.
///
/// Produces a CSV with one row per field/subfield combination across all records, with columns:
/// - `tag`: The MARC field tag
/// - `ind1`: First indicator (or empty for control fields)
/// - `ind2`: Second indicator (or empty for control fields)
/// - `subfield_code`: The subfield code (or empty for control fields)
/// - `value`: The field or subfield value
///
/// # Arguments
/// * `records` - A list of PyRecord instances
///
/// # Returns
/// A CSV string representation of all records
///
/// # Example
/// ```python
/// import mrrc
/// records = [mrrc.Record(mrrc.Leader()), mrrc.Record(mrrc.Leader())]
/// csv_str = mrrc.records_to_csv(records)
/// ```
#[pyfunction]
pub fn records_to_csv(records: &pyo3::Bound<'_, pyo3::types::PyList>) -> PyResult<String> {
    let mut rust_records = Vec::new();
    for item in records.iter() {
        // Try PyRecord first
        if let Ok(record) = item.extract::<pyo3::PyRef<'_, PyRecord>>() {
            rust_records.push(record.inner.clone());
            continue;
        }

        // Try wrapped Record with _inner attribute
        if let Ok(inner) = item.getattr("_inner") {
            if let Ok(record) = inner.extract::<pyo3::PyRef<'_, PyRecord>>() {
                rust_records.push(record.inner.clone());
                continue;
            }
        }

        return Err(pyo3::exceptions::PyTypeError::new_err(
            "All items must be PyRecord or wrapped Record",
        ));
    }
    csv::records_to_csv(&rust_records).map_err(marc_error_to_py_err)
}

/// Convert MARC records to CSV format using a custom field filter.
///
/// Allows filtering which fields are exported to CSV. Only fields matching the
/// provided filter function will be included.
///
/// # Arguments
/// * `records` - A list of PyRecord instances
/// * `filter_fn` - A callable that takes a field tag (str) and returns True to include it
///
/// # Returns
/// A CSV string with only the filtered fields
///
/// # Example
/// ```python
/// import mrrc
/// records = [mrrc.Record(mrrc.Leader())]
/// # Only export 245 field (title)
/// csv_str = mrrc.records_to_csv_filtered(records, lambda tag: tag == "245")
/// ```
#[pyfunction]
pub fn records_to_csv_filtered(
    records: &pyo3::Bound<'_, pyo3::types::PyList>,
    filter_fn: pyo3::Py<pyo3::PyAny>,
) -> PyResult<String> {
    let mut rust_records = Vec::new();
    for item in records.iter() {
        // Try PyRecord first
        if let Ok(record) = item.extract::<pyo3::PyRef<'_, PyRecord>>() {
            rust_records.push(record.inner.clone());
            continue;
        }

        // Try wrapped Record with _inner attribute
        if let Ok(inner) = item.getattr("_inner") {
            if let Ok(record) = inner.extract::<pyo3::PyRef<'_, PyRecord>>() {
                rust_records.push(record.inner.clone());
                continue;
            }
        }

        return Err(pyo3::exceptions::PyTypeError::new_err(
            "All items must be PyRecord or wrapped Record",
        ));
    }

    // Create a closure that calls the Python filter function
    Python::attach(|py| {
        csv::records_to_csv_filtered(&rust_records, |tag| {
            let result = filter_fn
                .call1(py, (tag,))
                .ok()
                .and_then(|obj| obj.extract::<bool>(py).ok())
                .unwrap_or(false);
            result
        })
        .map_err(marc_error_to_py_err)
    })
}
