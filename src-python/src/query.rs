// Python wrapper classes for MARC field query patterns
//
// This module exposes the Rust Query DSL to Python users, providing
// a powerful way to search for fields based on complex criteria like
// indicators, tag ranges, subfield presence, and regex patterns.

use mrrc::field_query::{FieldQuery, SubfieldPatternQuery, SubfieldValueQuery, TagRangeQuery};
use pyo3::prelude::*;

/// Python wrapper for FieldQuery - a builder for complex field matching.
///
/// `FieldQuery` uses the builder pattern to construct queries that can match
/// on tags, indicators, and subfield presence. This provides functionality
/// beyond pymarc's simple `get_fields(*tags)` method.
///
/// # Examples
///
/// ```python
/// import mrrc
///
/// # Find all 650 fields with indicator2='0' (LCSH) that have subfield 'a'
/// query = mrrc.FieldQuery().tag("650").indicator2("0").has_subfield("a")
/// for field in record.fields_matching(query):
///     print(f"LCSH heading: {field}")
///
/// # Match any field with specific indicators
/// query = mrrc.FieldQuery().indicator1("1").indicator2("0")
/// ```
#[pyclass(name = "FieldQuery")]
#[derive(Clone)]
pub struct PyFieldQuery {
    pub inner: FieldQuery,
}

#[pymethods]
impl PyFieldQuery {
    /// Create a new query that matches all fields.
    ///
    /// Returns:
    ///     FieldQuery: A new query builder with no restrictions.
    ///
    /// Example:
    ///     >>> query = mrrc.FieldQuery()  # matches all fields
    #[new]
    pub fn new() -> Self {
        PyFieldQuery {
            inner: FieldQuery::new(),
        }
    }

    /// Restrict query to fields with a specific tag.
    ///
    /// Args:
    ///     tag: The 3-character field tag (e.g., "650", "245").
    ///
    /// Returns:
    ///     FieldQuery: Self, for method chaining.
    ///
    /// Example:
    ///     >>> query = mrrc.FieldQuery().tag("650")
    pub fn tag(&self, tag: &str) -> Self {
        PyFieldQuery {
            inner: self.inner.clone().tag(tag),
        }
    }

    /// Restrict query to fields with a specific first indicator.
    ///
    /// Args:
    ///     indicator: Single character indicator value, or None to match any.
    ///
    /// Returns:
    ///     FieldQuery: Self, for method chaining.
    ///
    /// Example:
    ///     >>> query = mrrc.FieldQuery().indicator1("1")  # match ind1='1'
    ///     >>> query = mrrc.FieldQuery().indicator1(None)  # match any ind1
    #[pyo3(signature = (indicator=None))]
    pub fn indicator1(&self, indicator: Option<&str>) -> Self {
        let ind = indicator.and_then(|s| s.chars().next());
        PyFieldQuery {
            inner: self.inner.clone().indicator1(ind),
        }
    }

    /// Restrict query to fields with a specific second indicator.
    ///
    /// Args:
    ///     indicator: Single character indicator value, or None to match any.
    ///
    /// Returns:
    ///     FieldQuery: Self, for method chaining.
    ///
    /// Example:
    ///     >>> query = mrrc.FieldQuery().indicator2("0")  # match ind2='0' (LCSH)
    ///     >>> query = mrrc.FieldQuery().indicator2(None)  # match any ind2
    #[pyo3(signature = (indicator=None))]
    pub fn indicator2(&self, indicator: Option<&str>) -> Self {
        let ind = indicator.and_then(|s| s.chars().next());
        PyFieldQuery {
            inner: self.inner.clone().indicator2(ind),
        }
    }

    /// Require the field to have a subfield with the given code.
    ///
    /// Multiple calls add additional required subfields (AND logic).
    ///
    /// Args:
    ///     code: Single character subfield code (e.g., "a", "x").
    ///
    /// Returns:
    ///     FieldQuery: Self, for method chaining.
    ///
    /// Example:
    ///     >>> query = mrrc.FieldQuery().tag("650").has_subfield("a")
    ///     >>> query = query.has_subfield("x")  # must have both 'a' AND 'x'
    pub fn has_subfield(&self, code: &str) -> PyResult<Self> {
        if code.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Subfield code cannot be empty",
            ));
        }
        let code_char = code.chars().next().unwrap();
        Ok(PyFieldQuery {
            inner: self.inner.clone().has_subfield(code_char),
        })
    }

    /// Require the field to have all of the given subfield codes.
    ///
    /// Args:
    ///     codes: List of single-character subfield codes.
    ///
    /// Returns:
    ///     FieldQuery: Self, for method chaining.
    ///
    /// Example:
    ///     >>> query = mrrc.FieldQuery().tag("650").has_subfields(["a", "x", "v"])
    pub fn has_subfields(&self, codes: Vec<String>) -> PyResult<Self> {
        let chars: Vec<char> = codes.into_iter().filter_map(|s| s.chars().next()).collect();
        if chars.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "At least one subfield code is required",
            ));
        }
        Ok(PyFieldQuery {
            inner: self.inner.clone().has_subfields(&chars),
        })
    }

    /// Convert this query to a tag range query.
    ///
    /// This allows matching fields within a tag range while preserving
    /// any indicator and subfield requirements already set.
    ///
    /// Args:
    ///     start_tag: Start of tag range (inclusive), e.g., "600".
    ///     end_tag: End of tag range (inclusive), e.g., "699".
    ///
    /// Returns:
    ///     TagRangeQuery: A new range-based query.
    ///
    /// Example:
    ///     >>> query = mrrc.FieldQuery().indicator2("0").tag_range("600", "699")
    ///     >>> # Matches all 6XX fields with ind2='0'
    pub fn tag_range(&self, start_tag: &str, end_tag: &str) -> PyTagRangeQuery {
        PyTagRangeQuery {
            inner: self.inner.clone().tag_range(start_tag, end_tag),
        }
    }

    fn __repr__(&self) -> String {
        let tag = self
            .inner
            .tag
            .as_ref()
            .map_or("*".to_string(), |t| t.clone());
        let ind1 = self
            .inner
            .indicator1
            .map_or("*".to_string(), |c| c.to_string());
        let ind2 = self
            .inner
            .indicator2
            .map_or("*".to_string(), |c| c.to_string());
        format!(
            "<FieldQuery tag={} ind1={} ind2={} required_subfields={:?}>",
            tag, ind1, ind2, self.inner.required_subfields
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// Python wrapper for TagRangeQuery - query fields within a tag range.
///
/// This query type matches fields whose tags fall within a specified range,
/// useful for querying groups of related fields (e.g., all 6XX subject fields).
///
/// # Examples
///
/// ```python
/// import mrrc
///
/// # Find all subject fields (600-699) with indicator2='0' (LCSH)
/// query = mrrc.TagRangeQuery("600", "699", indicator2="0")
/// for field in record.fields_matching_range(query):
///     print(f"Subject: {field}")
/// ```
#[pyclass(name = "TagRangeQuery")]
#[derive(Clone)]
pub struct PyTagRangeQuery {
    pub inner: TagRangeQuery,
}

#[pymethods]
impl PyTagRangeQuery {
    /// Create a new tag range query.
    ///
    /// Args:
    ///     start_tag: Start of tag range (inclusive), e.g., "600".
    ///     end_tag: End of tag range (inclusive), e.g., "699".
    ///     indicator1: Optional first indicator filter (None = match any).
    ///     indicator2: Optional second indicator filter (None = match any).
    ///     required_subfields: Optional list of required subfield codes.
    ///
    /// Example:
    ///     >>> query = mrrc.TagRangeQuery("600", "699", indicator2="0")
    ///     >>> query = mrrc.TagRangeQuery("100", "199")  # all 1XX fields
    #[new]
    #[pyo3(signature = (start_tag, end_tag, *, indicator1=None, indicator2=None, required_subfields=None))]
    pub fn new(
        start_tag: &str,
        end_tag: &str,
        indicator1: Option<&str>,
        indicator2: Option<&str>,
        required_subfields: Option<Vec<String>>,
    ) -> Self {
        let ind1 = indicator1.and_then(|s| s.chars().next());
        let ind2 = indicator2.and_then(|s| s.chars().next());
        let subfields: Vec<char> = required_subfields
            .unwrap_or_default()
            .into_iter()
            .filter_map(|s| s.chars().next())
            .collect();

        PyTagRangeQuery {
            inner: TagRangeQuery {
                start_tag: start_tag.to_string(),
                end_tag: end_tag.to_string(),
                indicator1: ind1,
                indicator2: ind2,
                required_subfields: subfields,
            },
        }
    }

    /// Start of tag range (inclusive).
    #[getter]
    pub fn start_tag(&self) -> String {
        self.inner.start_tag.clone()
    }

    /// End of tag range (inclusive).
    #[getter]
    pub fn end_tag(&self) -> String {
        self.inner.end_tag.clone()
    }

    /// First indicator filter (None = match any).
    #[getter]
    pub fn indicator1(&self) -> Option<String> {
        self.inner.indicator1.map(|c| c.to_string())
    }

    /// Second indicator filter (None = match any).
    #[getter]
    pub fn indicator2(&self) -> Option<String> {
        self.inner.indicator2.map(|c| c.to_string())
    }

    /// Check if a tag is within this range.
    ///
    /// Args:
    ///     tag: The 3-character tag to check.
    ///
    /// Returns:
    ///     bool: True if the tag is within the range (inclusive).
    pub fn tag_in_range(&self, tag: &str) -> bool {
        self.inner.tag_in_range(tag)
    }

    fn __repr__(&self) -> String {
        let ind1 = self
            .inner
            .indicator1
            .map_or("*".to_string(), |c| c.to_string());
        let ind2 = self
            .inner
            .indicator2
            .map_or("*".to_string(), |c| c.to_string());
        format!(
            "<TagRangeQuery range={}-{} ind1={} ind2={}>",
            self.inner.start_tag, self.inner.end_tag, ind1, ind2
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// Python wrapper for SubfieldPatternQuery - regex matching on subfield values.
///
/// This query type finds fields where a specific subfield's value matches
/// a regular expression pattern.
///
/// # Examples
///
/// ```python
/// import mrrc
///
/// # Find all ISBNs starting with 978 (ISBN-13)
/// query = mrrc.SubfieldPatternQuery("020", "a", r"^978-")
/// for field in record.fields_matching_pattern(query):
///     isbn = field.get_subfield("a")
///     print(f"ISBN-13: {isbn}")
///
/// # Find personal names with date ranges
/// query = mrrc.SubfieldPatternQuery("100", "d", r"\d{4}-\d{4}")
/// ```
#[pyclass(name = "SubfieldPatternQuery")]
#[derive(Clone)]
pub struct PySubfieldPatternQuery {
    pub inner: SubfieldPatternQuery,
}

#[pymethods]
impl PySubfieldPatternQuery {
    /// Create a new subfield pattern query.
    ///
    /// Args:
    ///     tag: The 3-character field tag to search in.
    ///     subfield_code: The subfield code to match against.
    ///     pattern: A regex pattern string.
    ///
    /// Raises:
    ///     ValueError: If the pattern is not a valid regular expression.
    ///
    /// Example:
    ///     >>> query = mrrc.SubfieldPatternQuery("020", "a", r"^978-")
    ///     >>> query = mrrc.SubfieldPatternQuery("100", "d", r"\d{4}-\d{4}")
    #[new]
    pub fn new(tag: &str, subfield_code: &str, pattern: &str) -> PyResult<Self> {
        if subfield_code.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Subfield code cannot be empty",
            ));
        }
        let code_char = subfield_code.chars().next().unwrap();

        match SubfieldPatternQuery::new(tag, code_char, pattern) {
            Ok(query) => Ok(PySubfieldPatternQuery { inner: query }),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid regex pattern: {}",
                e
            ))),
        }
    }

    /// The field tag to search in.
    #[getter]
    pub fn tag(&self) -> String {
        self.inner.tag.clone()
    }

    /// The subfield code to match against.
    #[getter]
    pub fn subfield_code(&self) -> String {
        self.inner.subfield_code.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "<SubfieldPatternQuery tag={} subfield={}>",
            self.inner.tag, self.inner.subfield_code
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// Python wrapper for SubfieldValueQuery - exact or partial string matching.
///
/// This query type finds fields where a specific subfield's value matches
/// a string, either exactly or as a substring.
///
/// # Examples
///
/// ```python
/// import mrrc
///
/// # Find exact subject heading "History"
/// query = mrrc.SubfieldValueQuery("650", "a", "History")
///
/// # Find subjects containing "History" anywhere
/// query = mrrc.SubfieldValueQuery("650", "a", "History", partial=True)
/// ```
#[pyclass(name = "SubfieldValueQuery")]
#[derive(Clone)]
pub struct PySubfieldValueQuery {
    pub inner: SubfieldValueQuery,
}

#[pymethods]
impl PySubfieldValueQuery {
    /// Create a new subfield value query.
    ///
    /// Args:
    ///     tag: The 3-character field tag to search in.
    ///     subfield_code: The subfield code to match against.
    ///     value: The value to match.
    ///     partial: If True, match substrings. If False (default), exact match.
    ///
    /// Example:
    ///     >>> query = mrrc.SubfieldValueQuery("650", "a", "History")
    ///     >>> query = mrrc.SubfieldValueQuery("650", "a", "History", partial=True)
    #[new]
    #[pyo3(signature = (tag, subfield_code, value, *, partial=false))]
    pub fn new(tag: &str, subfield_code: &str, value: &str, partial: bool) -> PyResult<Self> {
        if subfield_code.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Subfield code cannot be empty",
            ));
        }
        let code_char = subfield_code.chars().next().unwrap();

        let inner = if partial {
            SubfieldValueQuery::partial(tag, code_char, value)
        } else {
            SubfieldValueQuery::new(tag, code_char, value)
        };

        Ok(PySubfieldValueQuery { inner })
    }

    /// The field tag to search in.
    #[getter]
    pub fn tag(&self) -> String {
        self.inner.tag.clone()
    }

    /// The subfield code to match against.
    #[getter]
    pub fn subfield_code(&self) -> String {
        self.inner.subfield_code.to_string()
    }

    /// The value to match.
    #[getter]
    pub fn value(&self) -> String {
        self.inner.value.clone()
    }

    /// Whether this is a partial (substring) match.
    #[getter]
    pub fn partial(&self) -> bool {
        self.inner.partial
    }

    fn __repr__(&self) -> String {
        let match_type = if self.inner.partial {
            "partial"
        } else {
            "exact"
        };
        format!(
            "<SubfieldValueQuery tag={} subfield={} value={:?} match={}>",
            self.inner.tag, self.inner.subfield_code, self.inner.value, match_type
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}
