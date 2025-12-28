// Python wrapper classes for core MARC data structures

use crate::error::marc_error_to_py_err;
use mrrc::{Field, Leader, Record, Subfield};
use pyo3::prelude::*;

/// Python wrapper for a MARC Leader (24-byte record header)
#[pyclass(name = "Leader")]
#[derive(Clone)]
pub struct PyLeader {
    pub inner: Leader,
}

#[pymethods]
impl PyLeader {
    /// Create a new Leader with default values
    #[new]
    pub fn new() -> Self {
        PyLeader {
            inner: Leader {
                record_length: 0,
                record_status: 'n',
                record_type: 'a',
                bibliographic_level: 'm',
                control_record_type: ' ',
                character_coding: ' ',
                indicator_count: 2,
                subfield_code_count: 2,
                data_base_address: 0,
                encoding_level: ' ',
                cataloging_form: ' ',
                multipart_level: ' ',
                reserved: "4500".to_string(),
            },
        }
    }

    /// Record length (5 digits)
    #[getter]
    pub fn record_length(&self) -> u32 {
        self.inner.record_length
    }

    #[setter]
    pub fn set_record_length(&mut self, value: u32) {
        self.inner.record_length = value;
    }

    /// Record status (1 char)
    #[getter]
    pub fn record_status(&self) -> String {
        self.inner.record_status.to_string()
    }

    #[setter]
    pub fn set_record_status(&mut self, value: &str) {
        if let Some(ch) = value.chars().next() {
            self.inner.record_status = ch;
        }
    }

    /// Type of record (1 char)
    #[getter]
    pub fn record_type(&self) -> String {
        self.inner.record_type.to_string()
    }

    #[setter]
    pub fn set_record_type(&mut self, value: &str) {
        if let Some(ch) = value.chars().next() {
            self.inner.record_type = ch;
        }
    }

    /// Bibliographic level (1 char)
    #[getter]
    pub fn bibliographic_level(&self) -> String {
        self.inner.bibliographic_level.to_string()
    }

    #[setter]
    pub fn set_bibliographic_level(&mut self, value: &str) {
        if let Some(ch) = value.chars().next() {
            self.inner.bibliographic_level = ch;
        }
    }

    /// Control record type (1 char)
    #[getter]
    pub fn control_record_type(&self) -> String {
        self.inner.control_record_type.to_string()
    }

    #[setter]
    pub fn set_control_record_type(&mut self, value: &str) {
        if let Some(ch) = value.chars().next() {
            self.inner.control_record_type = ch;
        }
    }

    /// Character coding scheme (1 char)
    #[getter]
    pub fn character_coding(&self) -> String {
        self.inner.character_coding.to_string()
    }

    #[setter]
    pub fn set_character_coding(&mut self, value: &str) {
        if let Some(ch) = value.chars().next() {
            self.inner.character_coding = ch;
        }
    }

    /// Indicator count
    #[getter]
    pub fn indicator_count(&self) -> u8 {
        self.inner.indicator_count
    }

    #[setter]
    pub fn set_indicator_count(&mut self, value: u8) {
        self.inner.indicator_count = value;
    }

    /// Subfield code count
    #[getter]
    pub fn subfield_code_count(&self) -> u8 {
        self.inner.subfield_code_count
    }

    #[setter]
    pub fn set_subfield_code_count(&mut self, value: u8) {
        self.inner.subfield_code_count = value;
    }

    /// Base address of data (5 digits)
    #[getter]
    pub fn data_base_address(&self) -> u32 {
        self.inner.data_base_address
    }

    #[setter]
    pub fn set_data_base_address(&mut self, value: u32) {
        self.inner.data_base_address = value;
    }

    /// Encoding level (1 char)
    #[getter]
    pub fn encoding_level(&self) -> String {
        self.inner.encoding_level.to_string()
    }

    #[setter]
    pub fn set_encoding_level(&mut self, value: &str) {
        if let Some(ch) = value.chars().next() {
            self.inner.encoding_level = ch;
        }
    }

    /// Cataloging form (1 char)
    #[getter]
    pub fn cataloging_form(&self) -> String {
        self.inner.cataloging_form.to_string()
    }

    #[setter]
    pub fn set_cataloging_form(&mut self, value: &str) {
        if let Some(ch) = value.chars().next() {
            self.inner.cataloging_form = ch;
        }
    }

    /// Multipart resource level (1 char)
    #[getter]
    pub fn multipart_level(&self) -> String {
        self.inner.multipart_level.to_string()
    }

    #[setter]
    pub fn set_multipart_level(&mut self, value: &str) {
        if let Some(ch) = value.chars().next() {
            self.inner.multipart_level = ch;
        }
    }

    /// Reserved (4 chars)
    #[getter]
    pub fn reserved(&self) -> String {
        self.inner.reserved.clone()
    }

    #[setter]
    pub fn set_reserved(&mut self, value: &str) {
        self.inner.reserved = value.to_string();
    }

    fn __repr__(&self) -> String {
        format!(
            "<Leader record_type={} bib_level={}>",
            self.inner.record_type, self.inner.bibliographic_level
        )
    }

    fn __str__(&self) -> String {
        format!(
            "Leader(status={}, type={}, level={})",
            self.inner.record_status, self.inner.record_type, self.inner.bibliographic_level
        )
    }

    fn __eq__(&self, other: &PyLeader) -> bool {
        self.inner == other.inner
    }
}

impl Default for PyLeader {
    fn default() -> Self {
        Self::new()
    }
}

/// Python wrapper for a Subfield (code + value pair)
#[pyclass(name = "Subfield")]
#[derive(Clone)]
pub struct PySubfield {
    pub inner: Subfield,
}

#[pymethods]
impl PySubfield {
    /// Create a new Subfield
    #[new]
    pub fn new(code: &str, value: &str) -> PyResult<Self> {
        if code.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Subfield code cannot be empty",
            ));
        }
        let code_char = code.chars().next().unwrap();
        Ok(PySubfield {
            inner: Subfield {
                code: code_char,
                value: value.to_string(),
            },
        })
    }

    /// Subfield code (single character)
    #[getter]
    pub fn code(&self) -> String {
        self.inner.code.to_string()
    }

    /// Subfield value
    #[getter]
    pub fn value(&self) -> String {
        self.inner.value.clone()
    }

    #[setter]
    pub fn set_value(&mut self, value: &str) {
        self.inner.value = value.to_string();
    }

    fn __repr__(&self) -> String {
        format!(
            "<Subfield code={} value={}>",
            self.inner.code, self.inner.value
        )
    }

    fn __str__(&self) -> String {
        format!("${}{}", self.inner.code, self.inner.value)
    }

    fn __eq__(&self, other: &PySubfield) -> bool {
        self.inner == other.inner
    }
}

/// Python wrapper for a Field
#[pyclass(name = "Field")]
#[derive(Clone)]
pub struct PyField {
    pub inner: Field,
}

#[pymethods]
impl PyField {
    /// Create a new Field
    #[new]
    pub fn new(tag: &str, indicator1: &str, indicator2: &str) -> PyResult<Self> {
        if tag.len() != 3 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Tag must be exactly 3 characters",
            ));
        }

        let ind1 = indicator1.chars().next().unwrap_or(' ');
        let ind2 = indicator2.chars().next().unwrap_or(' ');

        Ok(PyField {
            inner: Field {
                tag: tag.to_string(),
                indicator1: ind1,
                indicator2: ind2,
                subfields: vec![],
            },
        })
    }

    /// Field tag (3 digits)
    #[getter]
    pub fn tag(&self) -> String {
        self.inner.tag.clone()
    }

    /// First indicator
    #[getter]
    pub fn indicator1(&self) -> String {
        self.inner.indicator1.to_string()
    }

    #[setter]
    pub fn set_indicator1(&mut self, value: &str) {
        if let Some(ch) = value.chars().next() {
            self.inner.indicator1 = ch;
        }
    }

    /// Second indicator
    #[getter]
    pub fn indicator2(&self) -> String {
        self.inner.indicator2.to_string()
    }

    #[setter]
    pub fn set_indicator2(&mut self, value: &str) {
        if let Some(ch) = value.chars().next() {
            self.inner.indicator2 = ch;
        }
    }

    /// Add a subfield
    pub fn add_subfield(&mut self, code: &str, value: &str) -> PyResult<()> {
        if code.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Subfield code cannot be empty",
            ));
        }
        let code_char = code.chars().next().unwrap();
        self.inner.subfields.push(Subfield {
            code: code_char,
            value: value.to_string(),
        });
        Ok(())
    }

    /// Get all subfields
    pub fn subfields(&self) -> Vec<PySubfield> {
        self.inner
            .subfields
            .iter()
            .map(|sf| PySubfield { inner: sf.clone() })
            .collect()
    }

    /// Get subfields by code
    pub fn subfields_by_code(&self, code: &str) -> PyResult<Vec<String>> {
        if code.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Code cannot be empty",
            ));
        }
        let code_char = code.chars().next().unwrap();
        Ok(self
            .inner
            .subfields
            .iter()
            .filter(|sf| sf.code == code_char)
            .map(|sf| sf.value.clone())
            .collect())
    }

    fn __repr__(&self) -> String {
        format!(
            "<Field tag={} ind1={} ind2={} subfields={}>",
            self.inner.tag,
            self.inner.indicator1,
            self.inner.indicator2,
            self.inner.subfields.len()
        )
    }

    fn __str__(&self) -> String {
        format!("Field({})", self.inner.tag)
    }

    fn __eq__(&self, other: &PyField) -> bool {
        self.inner == other.inner
    }
}

/// Python wrapper for a Record
#[pyclass(name = "Record")]
pub struct PyRecord {
    pub inner: Record,
}

#[pymethods]
impl PyRecord {
    /// Create a new Record with a given Leader
    #[new]
    pub fn new(leader: &PyLeader) -> Self {
        PyRecord {
            inner: Record::new(leader.inner.clone()),
        }
    }

    /// Get the leader
    pub fn leader(&self) -> PyLeader {
        PyLeader {
            inner: self.inner.leader.clone(),
        }
    }

    /// Add a control field (000-009)
    pub fn add_control_field(&mut self, tag: &str, value: &str) -> PyResult<()> {
        if tag.len() != 3 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Tag must be exactly 3 characters",
            ));
        }
        self.inner.add_control_field_str(tag, value);
        Ok(())
    }

    /// Get a control field value
    pub fn control_field(&self, tag: &str) -> Option<String> {
        self.inner.get_control_field(tag).map(|s| s.to_string())
    }

    /// Add a data field
    pub fn add_field(&mut self, field: &PyField) {
        self.inner.add_field(field.inner.clone());
    }

    /// Get fields by tag
    pub fn fields_by_tag(&self, tag: &str) -> Vec<PyField> {
        self.inner
            .fields_by_tag(tag)
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get all fields
    pub fn fields(&self) -> Vec<PyField> {
        let mut result = vec![];
        for (_, fields) in &self.inner.fields {
            for field in fields {
                result.push(PyField {
                    inner: field.clone(),
                });
            }
        }
        result
    }

    /// Get all control fields as a dict-like structure
    pub fn control_fields(&self) -> Vec<(String, String)> {
        self.inner
            .control_fields
            .iter()
            .map(|(tag, value)| (tag.clone(), value.clone()))
            .collect()
    }

    /// Get title from 245 field (first subfield $a)
    pub fn title(&self) -> Option<String> {
        self.inner.title().map(|s| s.to_string())
    }

    /// Get author from 100/110/111 field
    pub fn author(&self) -> Option<String> {
        self.inner.author().map(|s| s.to_string())
    }

    /// Get ISBN from 020 field
    pub fn isbn(&self) -> Option<String> {
        self.inner.isbn().map(|s| s.to_string())
    }

    fn __repr__(&self) -> String {
        format!(
            "<Record type={} fields={}>",
            self.inner.leader.record_type,
            self.inner.fields.len()
        )
    }

    fn __str__(&self) -> String {
        format!("Record(type={})", self.inner.leader.record_type)
    }

    fn __eq__(&self, other: &PyRecord) -> bool {
        self.inner.leader == other.inner.leader
            && self.inner.control_fields == other.inner.control_fields
            && self.inner.fields == other.inner.fields
    }
}
