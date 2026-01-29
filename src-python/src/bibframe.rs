// Python bindings for BIBFRAME conversion functions
//
// This module exposes BIBFRAME conversion capabilities to Python:
// - BibframeConfig for conversion configuration
// - RdfGraph for RDF graph representation
// - marc_to_bibframe for MARC → BIBFRAME conversion
// - bibframe_to_marc for BIBFRAME → MARC conversion

use crate::error::marc_error_to_py_err;
use crate::wrappers::PyRecord;
use mrrc::bibframe::{
    bibframe_to_marc, marc_to_bibframe, BibframeConfig, RdfFormat, RdfGraph, RdfNode,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Configuration for BIBFRAME conversion.
///
/// Controls how MARC records are converted to BIBFRAME entities and how
/// the resulting RDF graph is serialized.
///
/// # Example
///
/// ```python
/// import mrrc
///
/// # Default configuration
/// config = mrrc.BibframeConfig()
///
/// # Custom configuration
/// config = mrrc.BibframeConfig()
/// config.set_base_uri("http://example.org/")
/// config.set_output_format("turtle")
/// config.set_authority_linking(True)
/// ```
#[pyclass(name = "BibframeConfig")]
#[derive(Clone)]
pub struct PyBibframeConfig {
    pub(crate) inner: BibframeConfig,
}

#[pymethods]
impl PyBibframeConfig {
    /// Create a new BibframeConfig with default settings.
    #[new]
    fn new() -> Self {
        Self {
            inner: BibframeConfig::default(),
        }
    }

    /// Set the base URI for generated resources.
    ///
    /// When set, entities are given minted URIs like `{base}/work/{id}`.
    /// When None (default), blank nodes are used.
    ///
    /// # Arguments
    /// * `uri` - The base URI string
    fn set_base_uri(&mut self, uri: &str) {
        self.inner.base_uri = Some(uri.to_string());
    }

    /// Get the current base URI.
    ///
    /// # Returns
    /// The base URI or None if not set
    #[getter]
    fn base_uri(&self) -> Option<String> {
        self.inner.base_uri.clone()
    }

    /// Set the output format for RDF serialization.
    ///
    /// # Arguments
    /// * `format` - One of: "rdf-xml", "jsonld", "turtle", "ntriples"
    ///
    /// # Raises
    /// ValueError: If format is not recognized
    fn set_output_format(&mut self, format: &str) -> PyResult<()> {
        self.inner.output_format = parse_rdf_format(format)?;
        Ok(())
    }

    /// Get the current output format.
    ///
    /// # Returns
    /// The format name as a string
    #[getter]
    fn output_format(&self) -> &'static str {
        match self.inner.output_format {
            RdfFormat::RdfXml => "rdf-xml",
            RdfFormat::JsonLd => "jsonld",
            RdfFormat::Turtle => "turtle",
            RdfFormat::NTriples => "ntriples",
        }
    }

    /// Enable or disable linking to external authority URIs.
    ///
    /// When True, agents and subjects with identifiable authority control
    /// numbers link to external URIs like <http://id.loc.gov/authorities/names/>.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable authority linking
    fn set_authority_linking(&mut self, enabled: bool) {
        self.inner.link_authorities = enabled;
    }

    /// Get the current authority linking setting.
    #[getter]
    fn authority_linking(&self) -> bool {
        self.inner.link_authorities
    }

    /// Enable or disable BFLC extensions.
    ///
    /// BFLC extensions are required for practical LOC compatibility.
    ///
    /// # Arguments
    /// * `enabled` - Whether to include BFLC extensions
    fn set_include_bflc(&mut self, enabled: bool) {
        self.inner.include_bflc = enabled;
    }

    /// Get the current BFLC extension setting.
    #[getter]
    fn include_bflc(&self) -> bool {
        self.inner.include_bflc
    }

    /// Enable or disable strict validation mode.
    ///
    /// When True, questionable data causes errors.
    /// When False (default), best-effort conversion is attempted.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable strict mode
    fn set_strict(&mut self, enabled: bool) {
        self.inner.strict = enabled;
    }

    /// Get the current strict mode setting.
    #[getter]
    fn strict(&self) -> bool {
        self.inner.strict
    }

    /// Enable or disable fail-fast error handling.
    ///
    /// When True, conversion stops at the first error.
    /// When False (default), errors are collected and conversion continues.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable fail-fast mode
    fn set_fail_fast(&mut self, enabled: bool) {
        self.inner.fail_fast = enabled;
    }

    /// Get the current fail-fast setting.
    #[getter]
    fn fail_fast(&self) -> bool {
        self.inner.fail_fast
    }

    fn __repr__(&self) -> String {
        format!(
            "BibframeConfig(base_uri={:?}, output_format={:?}, authority_linking={})",
            self.inner.base_uri,
            self.output_format(),
            self.inner.link_authorities
        )
    }
}

/// An RDF graph containing BIBFRAME triples.
///
/// This class wraps the RDF graph produced by MARC→BIBFRAME conversion
/// and provides serialization to various RDF formats.
///
/// # Example
///
/// ```python
/// import mrrc
///
/// record = mrrc.Record(leader="00000nam a22000007a 4500")
/// config = mrrc.BibframeConfig()
/// graph = mrrc.marc_to_bibframe(record, config)
///
/// # Get number of triples
/// print(f"Graph has {len(graph)} triples")
///
/// # Serialize to different formats
/// rdf_xml = graph.serialize("rdf-xml")
/// jsonld = graph.serialize("jsonld")
/// turtle = graph.serialize("turtle")
/// ntriples = graph.serialize("ntriples")
/// ```
#[pyclass(name = "RdfGraph")]
pub struct PyRdfGraph {
    pub(crate) inner: RdfGraph,
}

#[pymethods]
impl PyRdfGraph {
    /// Create a new empty RDF graph.
    #[new]
    fn new() -> Self {
        Self {
            inner: RdfGraph::new(),
        }
    }

    /// Get the number of triples in the graph.
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Check if the graph is empty.
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Serialize the graph to a string in the specified format.
    ///
    /// # Arguments
    /// * `format` - One of: "rdf-xml", "jsonld", "turtle", "ntriples"
    ///
    /// # Returns
    /// The serialized RDF as a string
    ///
    /// # Raises
    /// ValueError: If format is not recognized or serialization fails
    fn serialize(&self, format: &str) -> PyResult<String> {
        let rdf_format = parse_rdf_format(format)?;
        self.inner
            .serialize(rdf_format)
            .map_err(marc_error_to_py_err)
    }

    /// Parse an RDF graph from a string.
    ///
    /// # Arguments
    /// * `data` - The RDF data as a string
    /// * `format` - One of: "rdf-xml", "jsonld", "turtle", "ntriples"
    ///
    /// # Returns
    /// A new RdfGraph instance
    ///
    /// # Raises
    /// ValueError: If format is not recognized or parsing fails
    #[staticmethod]
    fn parse(data: &str, format: &str) -> PyResult<PyRdfGraph> {
        let rdf_format = parse_rdf_format(format)?;
        let inner = RdfGraph::parse(data, rdf_format).map_err(marc_error_to_py_err)?;
        Ok(PyRdfGraph { inner })
    }

    /// Get all triples as a list of (subject, predicate, object) tuples.
    ///
    /// # Returns
    /// A list of tuples where each tuple is (subject_str, predicate_str, object_str)
    fn triples(&self) -> Vec<(String, String, String)> {
        self.inner
            .triples()
            .map(|t| {
                (
                    node_to_string(&t.subject),
                    t.predicate.clone(),
                    node_to_string(&t.object),
                )
            })
            .collect()
    }

    fn __repr__(&self) -> String {
        format!("RdfGraph({} triples)", self.inner.len())
    }
}

/// Convert a MARC record to a BIBFRAME RDF graph.
///
/// This function transforms a MARC bibliographic record into a BIBFRAME 2.0
/// RDF graph containing Work, Instance, and optionally Item entities.
///
/// # Arguments
/// * `record` - The MARC record to convert
/// * `config` - Configuration options for the conversion
///
/// # Returns
/// An RdfGraph containing the BIBFRAME representation
///
/// # Example
///
/// ```python
/// import mrrc
///
/// record = mrrc.Record(leader="00000nam a22000007a 4500")
/// record.add_control_field("001", "12345")
/// record.add_control_field("008", "040520s2023    xxu           000 0 eng  ")
///
/// config = mrrc.BibframeConfig()
/// config.set_base_uri("http://example.org/")
///
/// graph = mrrc.marc_to_bibframe(record, config)
/// print(graph.serialize("jsonld"))
/// ```
#[pyfunction]
#[pyo3(name = "marc_to_bibframe")]
pub fn py_marc_to_bibframe(record: &PyRecord, config: &PyBibframeConfig) -> PyRdfGraph {
    let graph = marc_to_bibframe(&record.inner, &config.inner);
    PyRdfGraph { inner: graph }
}

/// Convert a BIBFRAME RDF graph to a MARC record.
///
/// This function transforms a BIBFRAME 2.0 RDF graph back into a MARC
/// bibliographic record. Note that some information loss is inherent
/// because BIBFRAME is semantically richer than MARC.
///
/// # Arguments
/// * `graph` - The BIBFRAME RDF graph to convert
///
/// # Returns
/// A MARC Record representing the BIBFRAME data
///
/// # Raises
/// ValueError: If the graph cannot be converted
///
/// # Example
///
/// ```python
/// import mrrc
///
/// # Round-trip conversion
/// record = mrrc.Record(leader="00000nam a22000007a 4500")
/// config = mrrc.BibframeConfig()
/// graph = mrrc.marc_to_bibframe(record, config)
/// recovered = mrrc.bibframe_to_marc(graph)
/// ```
#[pyfunction]
#[pyo3(name = "bibframe_to_marc")]
pub fn py_bibframe_to_marc(graph: &PyRdfGraph) -> PyResult<PyRecord> {
    let record = bibframe_to_marc(&graph.inner).map_err(marc_error_to_py_err)?;
    Ok(PyRecord { inner: record })
}

/// Parse an RDF format string into the enum variant.
fn parse_rdf_format(format: &str) -> PyResult<RdfFormat> {
    match format.to_lowercase().as_str() {
        "rdf-xml" | "rdfxml" | "rdf/xml" | "application/rdf+xml" => Ok(RdfFormat::RdfXml),
        "jsonld" | "json-ld" | "application/ld+json" => Ok(RdfFormat::JsonLd),
        "turtle" | "ttl" | "text/turtle" => Ok(RdfFormat::Turtle),
        "ntriples" | "nt" | "n-triples" | "application/n-triples" => Ok(RdfFormat::NTriples),
        _ => Err(PyValueError::new_err(format!(
            "Unknown RDF format: '{}'. Use one of: rdf-xml, jsonld, turtle, ntriples",
            format
        ))),
    }
}

/// Convert an RdfNode to a string representation.
fn node_to_string(node: &RdfNode) -> String {
    match node {
        RdfNode::Uri(uri) => format!("<{}>", uri),
        RdfNode::BlankNode(id) => format!("_:{}", id),
        RdfNode::Literal {
            value,
            language,
            datatype,
        } => {
            if let Some(lang) = language {
                format!("\"{}\"@{}", value, lang)
            } else if let Some(dt) = datatype {
                format!("\"{}\"^^<{}>", value, dt)
            } else {
                format!("\"{}\"", value)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mrrc::leader::Leader;
    use mrrc::record::Record;

    fn make_test_leader() -> Leader {
        Leader {
            record_length: 1000,
            record_status: 'n',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: ' ',
            character_coding: 'a',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 100,
            encoding_level: ' ',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    #[test]
    fn test_config_default() {
        let config = PyBibframeConfig::new();
        assert!(config.base_uri().is_none());
        assert_eq!(config.output_format(), "jsonld");
        assert!(!config.authority_linking());
    }

    #[test]
    fn test_config_setters() {
        let mut config = PyBibframeConfig::new();
        config.set_base_uri("http://example.org/");
        config.set_output_format("turtle").unwrap();
        config.set_authority_linking(true);

        assert_eq!(config.base_uri(), Some("http://example.org/".into()));
        assert_eq!(config.output_format(), "turtle");
        assert!(config.authority_linking());
    }

    #[test]
    fn test_rdf_graph_basic() {
        let graph = PyRdfGraph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.__len__(), 0);
    }

    #[test]
    fn test_parse_rdf_format() {
        assert!(matches!(parse_rdf_format("rdf-xml"), Ok(RdfFormat::RdfXml)));
        assert!(matches!(parse_rdf_format("jsonld"), Ok(RdfFormat::JsonLd)));
        assert!(matches!(parse_rdf_format("turtle"), Ok(RdfFormat::Turtle)));
        assert!(matches!(
            parse_rdf_format("ntriples"),
            Ok(RdfFormat::NTriples)
        ));
        assert!(parse_rdf_format("invalid").is_err());
    }

    #[test]
    fn test_marc_to_bibframe_conversion() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test123".to_string());

        let py_record = PyRecord { inner: record };
        let config = PyBibframeConfig::new();

        let graph = py_marc_to_bibframe(&py_record, &config);
        assert!(!graph.is_empty());
    }
}
