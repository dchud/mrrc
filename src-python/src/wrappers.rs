// Python wrapper classes for core MARC data structures

use mrrc::{AuthorityRecord, Field, HoldingsRecord, Leader, Record, RecordHelpers, Subfield};
use pyo3::prelude::*;

/// Python wrapper for a MARC Leader (24-byte record header)
///
/// The MARC leader is a 24-byte fixed-length field at the start of every MARC record.
/// It contains metadata describing the record's structure, content type, and encoding.
///
/// # Examples
///
/// ```python
/// import mrrc
/// leader = mrrc.Leader()
/// leader.record_type = 'a'  # Language material
/// leader.bibliographic_level = 'm'  # Monograph
/// ```
#[pyclass(name = "Leader", from_py_object)]
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

    /// Get valid values for a specific leader position (MARC 21 spec reference).
    ///
    /// Returns a dictionary mapping valid character values to their descriptions
    /// for the given position.
    ///
    /// # Arguments
    ///
    /// * `position` - The leader position (5-19)
    ///
    /// # Returns
    ///
    /// A dictionary mapping values to descriptions, or empty dict for unknown positions
    ///
    /// # Example
    ///
    /// ```python
    /// leader = Leader()
    /// valid = leader.get_valid_values(5)
    /// # Returns: {'a': 'increase in encoding level', 'c': 'corrected or revised', ...}
    /// ```
    #[staticmethod]
    pub fn get_valid_values(position: usize, py: Python) -> Option<Py<pyo3::PyAny>> {
        use pyo3::types::PyDict;

        match Leader::valid_values_at_position(position) {
            Some(values) => {
                let dict = PyDict::new(py);
                for (code, desc) in values {
                    dict.set_item(code, desc).expect("Failed to set dict item");
                }
                Some(dict.into())
            },
            None => None,
        }
    }

    /// Get description for a specific value at a leader position.
    ///
    /// # Arguments
    ///
    /// * `position` - The leader position (5-19)
    /// * `value` - The character value to look up
    ///
    /// # Returns
    ///
    /// The description if found, or None if the value is invalid for the position
    ///
    /// # Example
    ///
    /// ```python
    /// leader = Leader()
    /// desc = leader.describe_value(5, "a")
    /// # Returns: "increase in encoding level"
    /// ```
    #[staticmethod]
    pub fn describe_value(position: usize, value: &str) -> Option<String> {
        Leader::describe_value(position, value).map(|s| s.to_string())
    }

    /// Check if a value is valid for a specific leader position.
    ///
    /// Positions without defined valid values accept any value.
    ///
    /// # Arguments
    ///
    /// * `position` - The leader position (5-19)
    /// * `value` - The character value to validate
    ///
    /// # Returns
    ///
    /// True if the value is valid for the position, false otherwise
    ///
    /// # Example
    ///
    /// ```python
    /// leader = Leader()
    /// is_valid = leader.is_valid_value(5, "a")
    /// # Returns: True
    /// ```
    #[staticmethod]
    pub fn is_valid_value(position: usize, value: &str) -> bool {
        Leader::is_valid_value(position, value)
    }

    /// Get description for a specific value at a leader position (alias for describe_value).
    ///
    /// # Arguments
    ///
    /// * `position` - The leader position (5-19)
    /// * `value` - The character value to look up
    ///
    /// # Returns
    ///
    /// The description if found, or None if the value is invalid for the position
    #[staticmethod]
    pub fn get_value_description(position: usize, value: &str) -> Option<String> {
        Leader::describe_value(position, value).map(|s| s.to_string())
    }
}

impl Default for PyLeader {
    fn default() -> Self {
        Self::new()
    }
}

/// Python wrapper for a Subfield (code + value pair)
///
/// Subfields are named data elements within fields, consisting of a
/// single-character code and a value. For example, subfield 'a' in a
/// 245 field typically contains the main title.
///
/// # Examples
///
/// ```python
/// import mrrc
/// sf = mrrc.Subfield('a', 'The Great Gatsby')
/// print(f"Code: {sf.code}, Value: {sf.value}")
/// ```
#[pyclass(name = "Subfield", from_py_object)]
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
///
/// A MARC field consists of a 3-character tag, two indicators, and one or more subfields.
/// Fields can represent bibliographic data (like 245 for title) or subject headings (like 650).
///
/// # Examples
///
/// ```python
/// import mrrc
/// field = mrrc.Field('245', '1', '0')
/// field.add_subfield('a', 'The Great Gatsby')
/// field.add_subfield('c', 'F. Scott Fitzgerald')
/// ```
#[pyclass(name = "Field", from_py_object)]
#[derive(Clone)]
pub struct PyField {
    pub inner: Field,
}

#[pymethods]
impl PyField {
    /// Create a new Field
    ///
    /// # Arguments
    /// * `tag` - 3-character field tag (e.g., '245')
    /// * `indicator1` - First indicator (default: '0')
    /// * `indicator2` - Second indicator (default: '0')
    /// * `subfields` - Optional list of Subfield objects to initialize
    /// * `indicators` - Optional list [ind1, ind2] (alternative to positional args)
    #[new]
    #[pyo3(signature = (tag, indicator1=None, indicator2=None, *, subfields=None, indicators=None))]
    pub fn new(
        py: Python,
        tag: &str,
        indicator1: Option<&str>,
        indicator2: Option<&str>,
        subfields: Option<Vec<PySubfield>>,
        indicators: Option<Py<PyAny>>,
    ) -> PyResult<Self> {
        if tag.len() != 3 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Tag must be exactly 3 characters",
            ));
        }

        // Determine indicators: prefer explicit indicators list, then positional args
        let (ind1, ind2) = if let Some(inds) = indicators {
            // Try to extract [ind1, ind2] from Python list
            if let Ok(list) = inds.extract::<Vec<String>>(py) {
                if list.len() < 2 {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "indicators must have at least 2 elements",
                    ));
                }
                (
                    list[0].chars().next().unwrap_or(' '),
                    list[1].chars().next().unwrap_or(' '),
                )
            } else {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "indicators must be a list of strings",
                ));
            }
        } else {
            (
                indicator1.and_then(|s| s.chars().next()).unwrap_or('0'),
                indicator2.and_then(|s| s.chars().next()).unwrap_or('0'),
            )
        };

        // Convert PySubfield objects to inner Subfield objects
        let sfs: smallvec::SmallVec<[_; 4]> = if let Some(sfs) = subfields {
            sfs.iter().map(|psf| psf.inner.clone()).collect()
        } else {
            smallvec::SmallVec::new()
        };

        Ok(PyField {
            inner: Field {
                tag: tag.to_string(),
                indicator1: ind1,
                indicator2: ind2,
                subfields: sfs,
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
///
/// A MARC bibliographic record is the fundamental unit of MARC data.
/// It consists of a leader (24-byte header), control fields (000-009),
/// and data fields (010 and above).
///
/// # Examples
///
/// ```python
/// import mrrc
/// leader = mrrc.Leader()
/// leader.record_type = 'a'
/// leader.bibliographic_level = 'm'
///
/// record = mrrc.Record(leader)
/// record.add_control_field('001', '12345')
///
/// field = mrrc.Field('245', '1', '0')
/// field.add_subfield('a', 'Example Title')
/// record.add_field(field)
///
/// print(f"Title: {record.title()}")
/// ```
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

    /// Set the leader (for record modification)
    pub fn set_leader(&mut self, leader: &PyLeader) {
        self.inner.leader = leader.inner.clone();
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

    /// Get the first field with a given tag (pymarc compatibility)
    pub fn get_field(&self, tag: &str) -> Option<PyField> {
        self.inner
            .get_field(tag)
            .map(|f| PyField { inner: f.clone() })
    }

    /// Get all fields with a given tag (pymarc compatibility)
    pub fn get_fields(&self, tag: &str) -> Vec<PyField> {
        self.inner
            .get_fields(tag)
            .map(|fields| {
                fields
                    .iter()
                    .map(|f| PyField { inner: f.clone() })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all fields
    pub fn fields(&self) -> Vec<PyField> {
        let mut result = vec![];
        for fields in self.inner.fields.values() {
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

    /// Remove all fields with a given tag
    ///
    /// Returns the removed fields.
    pub fn remove_field(&mut self, tag: &str) -> Vec<PyField> {
        self.inner
            .remove_fields_by_tag(tag)
            .into_iter()
            .map(|f| PyField { inner: f })
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

    /// Get all subject headings from 6XX subject fields
    pub fn subjects(&self) -> Vec<String> {
        self.inner
            .subjects()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get all location fields (852)
    pub fn location(&self) -> Vec<String> {
        self.inner
            .location()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get all notes from 5xx fields
    pub fn notes(&self) -> Vec<String> {
        self.inner.notes().iter().map(|s| s.to_string()).collect()
    }

    /// Get publisher from 260 or 264 (RDA) field
    pub fn publisher(&self) -> Option<String> {
        self.inner.publisher().map(|s| s.to_string())
    }

    /// Get uniform title from 130 field
    pub fn uniform_title(&self) -> Option<String> {
        self.inner.uniform_title().map(|s| s.to_string())
    }

    /// Get SuDoc (government document classification) from 086 field
    pub fn sudoc(&self) -> Option<String> {
        self.inner.sudoc().map(|s| s.to_string())
    }

    /// Get ISSN title from 222 field
    pub fn issn_title(&self) -> Option<String> {
        self.inner.issn_title().map(|s| s.to_string())
    }

    /// Get ISSN-L from 024 field
    pub fn issnl(&self) -> Option<String> {
        self.inner.issnl().map(|s| s.to_string())
    }

    /// Get publication year (alias for publication_year)
    pub fn pubyear(&self) -> Option<u32> {
        self.inner.pubyear()
    }

    /// Get ISSN from 022 field
    pub fn issn(&self) -> Option<String> {
        self.inner.issn().map(|s| s.to_string())
    }

    /// Get series from 490 field
    pub fn series(&self) -> Option<String> {
        self.inner.series().map(|s| s.to_string())
    }

    /// Get physical description from 300 field
    pub fn physical_description(&self) -> Option<String> {
        self.inner.physical_description().map(|s| s.to_string())
    }

    /// Check if this record is a book (record type 'a' + bibliographic level 'm')
    pub fn is_book(&self) -> bool {
        self.inner.is_book()
    }

    /// Check if this record is a serial (bibliographic level 's')
    pub fn is_serial(&self) -> bool {
        self.inner.is_serial()
    }

    /// Check if this record is music (record type 'c' or 'd')
    pub fn is_music(&self) -> bool {
        self.inner.is_music()
    }

    /// Check if this record is audiovisual material (record type 'g')
    pub fn is_audiovisual(&self) -> bool {
        self.inner.is_audiovisual()
    }

    // =========================================================================
    // Query DSL Methods - Advanced field searching beyond pymarc's get_fields()
    // =========================================================================

    /// Get fields matching indicator values.
    ///
    /// This is a convenience method for the common case of filtering by indicators.
    /// For more complex queries, use `fields_matching()` with a `FieldQuery`.
    ///
    /// Args:
    ///     tag: The 3-character field tag to search.
    ///     indicator1: Optional first indicator value (None = match any).
    ///     indicator2: Optional second indicator value (None = match any).
    ///
    /// Returns:
    ///     List of Field objects matching the criteria.
    ///
    /// Example:
    ///     >>> # Find all 650 fields with indicator2='0' (Library of Congress Subject Headings)
    ///     >>> lcsh_subjects = record.fields_by_indicator("650", indicator2="0")
    ///     >>> for field in lcsh_subjects:
    ///     ...     print(field.get_subfield("a"))
    #[pyo3(signature = (tag, *, indicator1=None, indicator2=None))]
    pub fn fields_by_indicator(
        &self,
        tag: &str,
        indicator1: Option<&str>,
        indicator2: Option<&str>,
    ) -> Vec<PyField> {
        let ind1 = indicator1.and_then(|s| s.chars().next());
        let ind2 = indicator2.and_then(|s| s.chars().next());
        self.inner
            .fields_by_indicator(tag, ind1, ind2)
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get fields within a tag range (inclusive).
    ///
    /// Useful for querying groups of related fields, such as all subject fields
    /// (600-699) or all added entry fields (700-799).
    ///
    /// Args:
    ///     start_tag: Start of range (inclusive), e.g., "600".
    ///     end_tag: End of range (inclusive), e.g., "699".
    ///
    /// Returns:
    ///     List of Field objects within the tag range.
    ///
    /// Example:
    ///     >>> # Find all subject fields (600-699)
    ///     >>> subjects = record.fields_in_range("600", "699")
    ///     >>> for field in subjects:
    ///     ...     print(f"{field.tag}: {field.get_subfield('a')}")
    pub fn fields_in_range(&self, start_tag: &str, end_tag: &str) -> Vec<PyField> {
        self.inner
            .fields_in_range(start_tag, end_tag)
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    // =========================================================================
    // Linked field navigation (880 alternate graphic representation)
    // =========================================================================

    /// Find all 880 fields linked to a given field via subfield $6.
    ///
    /// Given a non-880 field that has a $6 linkage subfield, returns all 880
    /// fields whose $6 occurrence number matches. This is the pymarc-compatible
    /// API for navigating alternate graphic representations (Hebrew, Arabic,
    /// CJK, Cyrillic, etc.).
    ///
    /// Args:
    ///     field: A Field object with a $6 linkage subfield.
    ///
    /// Returns:
    ///     List of linked 880 Field objects (empty if no linkage or no match).
    ///
    /// Example:
    ///     >>> f245 = record.get_fields('245')\[0\]
    ///     >>> linked = record.get_linked_fields(f245)
    ///     >>> if linked:
    ///     ...     print(linked\[0\]\['a'\])
    pub fn get_linked_fields(&self, field: &PyField) -> Vec<PyField> {
        self.inner
            .get_linked_fields(&field.inner)
            .into_iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Find the single 880 field linked to a given field via subfield $6.
    ///
    /// Like get_linked_fields() but returns only the first match. Use this
    /// when you expect exactly one linked 880 (the common case).
    ///
    /// Args:
    ///     field: A Field object with a $6 linkage subfield.
    ///
    /// Returns:
    ///     The linked 880 Field, or None if no linkage or no match.
    pub fn get_linked_field(&self, field: &PyField) -> Option<PyField> {
        self.inner
            .get_linked_field(&field.inner)
            .map(|f| PyField { inner: f.clone() })
    }

    /// Find the original field linked from a given 880 field.
    ///
    /// Given an 880 field, finds its linked original field by parsing the
    /// tag and occurrence from its $6 subfield.
    ///
    /// Args:
    ///     field_880: An 880 Field object.
    ///
    /// Returns:
    ///     The linked original Field, or None.
    pub fn get_original_field(&self, field_880: &PyField) -> Option<PyField> {
        self.inner
            .get_original_field(&field_880.inner)
            .map(|f| PyField { inner: f.clone() })
    }

    /// Get field pairs of original fields with their linked 880 counterparts.
    ///
    /// For a given tag, returns tuples of (original_field, linked_880_or_None).
    ///
    /// Args:
    ///     tag: The field tag to pair (e.g., '245', '100').
    ///
    /// Returns:
    ///     List of (Field, Optional[Field]) tuples.
    ///
    /// Example:
    ///     >>> for orig, linked in record.get_field_pairs('245'):
    ///     ...     print(orig\['a'\])
    ///     ...     if linked:
    ///     ...         print(linked\['a'\])
    pub fn get_field_pairs(&self, tag: &str) -> Vec<(PyField, Option<PyField>)> {
        self.inner
            .get_field_pairs(tag)
            .into_iter()
            .map(|(orig, linked)| {
                (
                    PyField {
                        inner: orig.clone(),
                    },
                    linked.map(|f| PyField { inner: f.clone() }),
                )
            })
            .collect()
    }

    /// Get fields matching a FieldQuery.
    ///
    /// This method enables complex field matching using the Query DSL.
    /// A FieldQuery can combine tag, indicator, and subfield requirements.
    ///
    /// Args:
    ///     query: A FieldQuery object with the matching criteria.
    ///
    /// Returns:
    ///     List of Field objects matching all query criteria.
    ///
    /// Example:
    ///     >>> query = mrrc.FieldQuery().tag("650").indicator2("0").has_subfield("a")
    ///     >>> lcsh = record.fields_matching(query)
    ///     >>> for field in lcsh:
    ///     ...     print(field.get_subfield("a"))
    pub fn fields_matching(&self, query: &crate::query::PyFieldQuery) -> Vec<PyField> {
        self.inner
            .fields_matching(&query.inner)
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get fields matching a TagRangeQuery.
    ///
    /// This method finds fields within a tag range that also match indicator
    /// and subfield requirements.
    ///
    /// Args:
    ///     query: A TagRangeQuery object with range and filter criteria.
    ///
    /// Returns:
    ///     List of Field objects matching all query criteria.
    ///
    /// Example:
    ///     >>> # Find all 6XX subjects with indicator2='0' (LCSH) that have subfield 'a'
    ///     >>> query = mrrc.TagRangeQuery("600", "699", indicator2="0", required_subfields=["a"])
    ///     >>> subjects = record.fields_matching_range(query)
    pub fn fields_matching_range(&self, query: &crate::query::PyTagRangeQuery) -> Vec<PyField> {
        self.inner
            .fields_matching_range(&query.inner)
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get fields matching a SubfieldPatternQuery (regex matching).
    ///
    /// This method finds fields where a specific subfield's value matches
    /// a regular expression pattern.
    ///
    /// Args:
    ///     query: A SubfieldPatternQuery object with tag, subfield, and regex.
    ///
    /// Returns:
    ///     List of Field objects where the subfield matches the pattern.
    ///
    /// Example:
    ///     >>> # Find all ISBN-13s (start with 978 or 979)
    ///     >>> query = mrrc.SubfieldPatternQuery("020", "a", r"^97\[89\]-")
    ///     >>> isbn13_fields = record.fields_matching_pattern(query)
    pub fn fields_matching_pattern(
        &self,
        query: &crate::query::PySubfieldPatternQuery,
    ) -> Vec<PyField> {
        self.inner
            .fields_matching_pattern(&query.inner)
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get fields matching a SubfieldValueQuery (exact or partial string matching).
    ///
    /// This method finds fields where a specific subfield's value matches
    /// a string exactly or as a substring.
    ///
    /// Args:
    ///     query: A SubfieldValueQuery object with tag, subfield, value, and match type.
    ///
    /// Returns:
    ///     List of Field objects where the subfield matches the value.
    ///
    /// Example:
    ///     >>> # Find exact subject heading "History"
    ///     >>> query = mrrc.SubfieldValueQuery("650", "a", "History")
    ///     >>> history_fields = record.fields_matching_value(query)
    ///
    ///     >>> # Find subjects containing "History" anywhere
    ///     >>> query = mrrc.SubfieldValueQuery("650", "a", "History", partial=True)
    ///     >>> related_fields = record.fields_matching_value(query)
    pub fn fields_matching_value(
        &self,
        query: &crate::query::PySubfieldValueQuery,
    ) -> Vec<PyField> {
        self.inner
            .fields
            .values()
            .flatten()
            .filter(|field| query.inner.matches(field))
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Convert record to JSON string
    ///
    /// # Example
    /// ```python
    /// json_str = record.to_json()
    /// ```
    pub fn to_json(&self) -> PyResult<String> {
        use mrrc::json;
        json::record_to_json(&self.inner)
            .map(|v| v.to_string())
            .map_err(crate::error::marc_error_to_py_err)
    }

    /// Convert record to XML string
    ///
    /// # Example
    /// ```python
    /// xml_str = record.to_xml()
    /// ```
    pub fn to_xml(&self) -> PyResult<String> {
        use mrrc::xml;
        xml::record_to_xml(&self.inner).map_err(crate::error::marc_error_to_py_err)
    }

    /// Convert record to MARCJSON format
    ///
    /// # Example
    /// ```python
    /// marcjson_str = record.to_marcjson()
    /// ```
    pub fn to_marcjson(&self) -> PyResult<String> {
        use mrrc::marcjson;
        marcjson::record_to_marcjson(&self.inner)
            .map(|v| v.to_string())
            .map_err(crate::error::marc_error_to_py_err)
    }

    /// Convert record to Dublin Core metadata
    ///
    /// # Returns
    /// A dictionary mapping Dublin Core field names to lists of values
    ///
    /// # Example
    /// ```python
    /// dc = record.to_dublin_core()
    /// print(dc['title'])
    /// ```
    pub fn to_dublin_core(&self) -> PyResult<std::collections::HashMap<String, Vec<String>>> {
        use mrrc::dublin_core;
        dublin_core::record_to_dublin_core(&self.inner)
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
            .map_err(crate::error::marc_error_to_py_err)
    }

    /// Convert record to MODS XML format
    ///
    /// MODS (Metadata Object Description Schema) is a more detailed
    /// XML representation than Dublin Core.
    ///
    /// # Example
    /// ```python
    /// mods_xml = record.to_mods()
    /// ```
    pub fn to_mods(&self) -> PyResult<String> {
        use mrrc::mods;
        mods::record_to_mods_xml(&self.inner).map_err(crate::error::marc_error_to_py_err)
    }

    /// Convert record to MARC21 binary format (ISO 2709)
    ///
    /// Returns the record as bytes in MARC21 format following the ISO 2709 standard.
    /// This is the standard MARC binary interchange format used by libraries worldwide.
    ///
    /// # Returns
    /// Bytes object containing the MARC21-encoded record
    ///
    /// # Example
    /// ```python
    /// marc_bytes = record.to_marc21()
    /// with open('record.mrc', 'wb') as f:
    ///     f.write(marc_bytes)
    /// ```
    pub fn to_marc21(&self) -> PyResult<Vec<u8>> {
        use mrrc::MarcWriter;

        let mut buffer = Vec::new();
        let mut writer = MarcWriter::new(&mut buffer);
        writer
            .write_record(&self.inner)
            .map_err(crate::error::marc_error_to_py_err)?;

        Ok(buffer)
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

/// Python wrapper for a MARC Authority Record (Type Z)
///
/// Authority records are used to maintain authorized access points (names, subjects, etc.)
/// They use the same ISO 2709 binary format as bibliographic records but are organized
/// by functional role (heading, tracings, notes, etc.).
#[pyclass(name = "AuthorityRecord", from_py_object)]
#[derive(Clone)]
pub struct PyAuthorityRecord {
    pub inner: AuthorityRecord,
}

#[pymethods]
impl PyAuthorityRecord {
    /// Get the leader
    #[getter]
    pub fn leader(&self) -> PyLeader {
        PyLeader {
            inner: self.inner.leader.clone(),
        }
    }

    /// Get record type (single character)
    pub fn record_type(&self) -> String {
        self.inner.leader.record_type.to_string()
    }

    /// Get the main heading (1XX field)
    pub fn heading(&self) -> Option<PyField> {
        self.inner.heading().map(|f| PyField { inner: f.clone() })
    }

    /// Get the heading text (from the main heading field)
    pub fn heading_text(&self) -> Option<String> {
        self.inner
            .heading()
            .and_then(|f| f.get_subfield('a'))
            .map(|s| s.to_string())
    }

    /// Get all see-from tracings (4XX fields)
    pub fn see_from_tracings(&self) -> Vec<PyField> {
        self.inner
            .see_from_tracings()
            .into_iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get all see-also-from tracings (5XX fields)
    pub fn see_also_tracings(&self) -> Vec<PyField> {
        self.inner
            .see_also_tracings()
            .into_iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get all notes (66X-68X fields)
    pub fn notes(&self) -> Vec<PyField> {
        self.inner
            .notes()
            .into_iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get all heading linking entries (7XX fields)
    pub fn linking_entries(&self) -> Vec<PyField> {
        self.inner
            .linking_entries()
            .into_iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get fields by tag
    pub fn get_fields(&self, tag: &str) -> Option<Vec<PyField>> {
        self.inner.get_fields(tag).map(|fields: &[Field]| {
            fields
                .iter()
                .map(|f: &Field| PyField { inner: f.clone() })
                .collect()
        })
    }

    /// Get control field by tag
    pub fn get_control_field(&self, tag: &str) -> Option<String> {
        self.inner.get_control_field(tag).map(|s| s.to_string())
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> PyResult<String> {
        // Authority records can be serialized like bibliographic records
        // by wrapping them in a Record structure
        // For now, convert the heading field to JSON
        let heading_json = self
            .heading()
            .map(|f| format!("{{\"heading\": {}}}", f.inner.tag))
            .unwrap_or_else(|| "{}".to_string());
        Ok(heading_json)
    }

    fn __repr__(&self) -> String {
        format!(
            "<AuthorityRecord type={} heading={}>",
            self.inner.leader.record_type,
            self.heading_text().unwrap_or_else(|| "Unknown".to_string())
        )
    }

    fn __str__(&self) -> String {
        format!(
            "AuthorityRecord({})",
            self.heading_text().unwrap_or_else(|| "Unknown".to_string())
        )
    }
}

/// Python wrapper for a MARC Holdings Record (Type x/y/v/u)
///
/// Holdings records are used to maintain inventory and location information.
/// They use the same ISO 2709 binary format as bibliographic records but organize
/// fields by functional role (locations, enumeration, notes, etc.).
#[pyclass(name = "HoldingsRecord", from_py_object)]
#[derive(Clone)]
pub struct PyHoldingsRecord {
    pub inner: HoldingsRecord,
}

#[pymethods]
impl PyHoldingsRecord {
    /// Get the leader
    #[getter]
    pub fn leader(&self) -> PyLeader {
        PyLeader {
            inner: self.inner.leader.clone(),
        }
    }

    /// Get record type (single character: x, y, v, or u)
    pub fn record_type(&self) -> String {
        self.inner.leader.record_type.to_string()
    }

    /// Get all location fields (852)
    pub fn locations(&self) -> Vec<PyField> {
        self.inner
            .locations()
            .iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get all basic captions (853)
    pub fn captions_basic(&self) -> Vec<PyField> {
        self.inner
            .captions_basic()
            .iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get all supplement captions (854)
    pub fn captions_supplements(&self) -> Vec<PyField> {
        self.inner
            .captions_supplements()
            .iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get all index captions (855)
    pub fn captions_indexes(&self) -> Vec<PyField> {
        self.inner
            .captions_indexes()
            .iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get all basic enumeration (863)
    pub fn enumeration_basic(&self) -> Vec<PyField> {
        self.inner
            .enumeration_basic()
            .iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get all supplement enumeration (864)
    pub fn enumeration_supplements(&self) -> Vec<PyField> {
        self.inner
            .enumeration_supplements()
            .iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get all index enumeration (865)
    pub fn enumeration_indexes(&self) -> Vec<PyField> {
        self.inner
            .enumeration_indexes()
            .iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get all basic textual holdings (866)
    pub fn textual_holdings_basic(&self) -> Vec<PyField> {
        self.inner
            .textual_holdings_basic()
            .iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get all supplement textual holdings (867)
    pub fn textual_holdings_supplements(&self) -> Vec<PyField> {
        self.inner
            .textual_holdings_supplements()
            .iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get all index textual holdings (868)
    pub fn textual_holdings_indexes(&self) -> Vec<PyField> {
        self.inner
            .textual_holdings_indexes()
            .iter()
            .map(|f| PyField { inner: f.clone() })
            .collect()
    }

    /// Get fields by tag
    pub fn get_fields(&self, tag: &str) -> Option<Vec<PyField>> {
        self.inner.get_fields(tag).map(|fields: &[Field]| {
            fields
                .iter()
                .map(|f: &Field| PyField { inner: f.clone() })
                .collect()
        })
    }

    /// Get control field by tag
    pub fn get_control_field(&self, tag: &str) -> Option<String> {
        self.inner.get_control_field(tag).map(|s| s.to_string())
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> PyResult<String> {
        // Holdings records can include location and enumeration information
        let loc_count = self.locations().len();
        Ok(format!("{{\"locations\": {}}}", loc_count))
    }

    fn __repr__(&self) -> String {
        format!(
            "<HoldingsRecord type={} locations={}>",
            self.inner.leader.record_type,
            self.locations().len()
        )
    }

    fn __str__(&self) -> String {
        format!(
            "HoldingsRecord(type={}, locations={})",
            self.inner.leader.record_type,
            self.locations().len()
        )
    }
}
