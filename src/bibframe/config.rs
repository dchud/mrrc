//! Configuration options for BIBFRAME conversion.
//!
//! This module provides the [`BibframeConfig`] struct which controls how MARC records
//! are converted to BIBFRAME and how the resulting RDF is serialized.

use std::fmt;

/// Output format for RDF serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RdfFormat {
    /// RDF/XML format (application/rdf+xml) - Most compatible with legacy systems
    RdfXml,
    /// JSON-LD format (application/ld+json) - Modern, readable, web-friendly
    #[default]
    JsonLd,
    /// Turtle format (text/turtle) - Compact, human-friendly
    Turtle,
    /// N-Triples format (application/n-triples) - Simple, line-based
    NTriples,
}

impl fmt::Display for RdfFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RdfXml => write!(f, "RDF/XML"),
            Self::JsonLd => write!(f, "JSON-LD"),
            Self::Turtle => write!(f, "Turtle"),
            Self::NTriples => write!(f, "N-Triples"),
        }
    }
}

impl RdfFormat {
    /// Returns the MIME type for this RDF format.
    #[must_use]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::RdfXml => "application/rdf+xml",
            Self::JsonLd => "application/ld+json",
            Self::Turtle => "text/turtle",
            Self::NTriples => "application/n-triples",
        }
    }

    /// Returns the typical file extension for this RDF format.
    #[must_use]
    pub const fn file_extension(&self) -> &'static str {
        match self {
            Self::RdfXml => "rdf",
            Self::JsonLd => "jsonld",
            Self::Turtle => "ttl",
            Self::NTriples => "nt",
        }
    }
}

/// Configuration for BIBFRAME conversion.
///
/// Controls how MARC records are converted to BIBFRAME entities and how the
/// resulting RDF graph is serialized.
///
/// # Examples
///
/// ```ignore
/// use mrrc::bibframe::{BibframeConfig, RdfFormat};
///
/// // Default configuration (blank nodes, JSON-LD output)
/// let config = BibframeConfig::default();
///
/// // Custom configuration with minted URIs
/// let config = BibframeConfig {
///     base_uri: Some("http://example.org/".into()),
///     output_format: RdfFormat::Turtle,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct BibframeConfig {
    // === URI Generation ===
    /// Base URI for generated resources.
    ///
    /// When `Some`, entities are given minted URIs like `{base}/work/{id}`.
    /// When `None` (default), blank nodes are used (`_:work1`, etc.).
    pub base_uri: Option<String>,

    /// Use MARC 001 control number in generated URIs.
    ///
    /// When true, the control number from field 001 is used as the identifier
    /// in generated URIs. When false, a hash-based identifier is generated.
    pub use_control_number: bool,

    /// Link to external authority URIs when identifiable.
    ///
    /// When true, agents and subjects with identifiable authority control
    /// numbers will link to external URIs like `http://id.loc.gov/authorities/names/`.
    /// When false, all entities use local URIs or blank nodes.
    pub link_authorities: bool,

    // === Output Control ===
    /// Output format for RDF serialization.
    pub output_format: RdfFormat,

    /// Include BFLC (BIBFRAME Library of Congress) extensions.
    ///
    /// BFLC extensions are required for practical LOC compatibility and
    /// provide properties like `bflc:aap` (authorized access point) that
    /// enable better MARC round-trip fidelity.
    pub include_bflc: bool,

    /// Include source MARC in `bf:AdminMetadata`.
    ///
    /// When true, the original MARC record is embedded in the output
    /// for debugging and provenance tracking. Increases output size.
    pub include_source: bool,

    // === Error Handling ===
    /// Stop on first conversion error.
    ///
    /// When true, conversion stops at the first error encountered.
    /// When false (default), errors are collected and conversion continues.
    pub fail_fast: bool,

    /// Strict validation mode.
    ///
    /// When true, questionable data (e.g., invalid indicators) causes errors.
    /// When false (default), best-effort conversion is attempted with warnings.
    pub strict: bool,
}

impl Default for BibframeConfig {
    fn default() -> Self {
        Self {
            base_uri: None,
            use_control_number: true,
            link_authorities: false,
            output_format: RdfFormat::default(),
            include_bflc: true,
            include_source: false,
            fail_fast: false,
            strict: false,
        }
    }
}

impl BibframeConfig {
    /// Creates a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the base URI for generated resources.
    #[must_use]
    pub fn with_base_uri(mut self, uri: impl Into<String>) -> Self {
        self.base_uri = Some(uri.into());
        self
    }

    /// Sets the output format.
    #[must_use]
    pub const fn with_output_format(mut self, format: RdfFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Enables linking to external authority URIs.
    #[must_use]
    pub const fn with_authority_linking(mut self, enabled: bool) -> Self {
        self.link_authorities = enabled;
        self
    }

    /// Enables strict validation mode.
    #[must_use]
    pub const fn with_strict_mode(mut self, enabled: bool) -> Self {
        self.strict = enabled;
        self
    }

    /// Enables fail-fast error handling.
    #[must_use]
    pub const fn with_fail_fast(mut self, enabled: bool) -> Self {
        self.fail_fast = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BibframeConfig::default();
        assert!(config.base_uri.is_none());
        assert!(config.use_control_number);
        assert!(!config.link_authorities);
        assert_eq!(config.output_format, RdfFormat::JsonLd);
        assert!(config.include_bflc);
        assert!(!config.include_source);
        assert!(!config.fail_fast);
        assert!(!config.strict);
    }

    #[test]
    fn test_builder_pattern() {
        let config = BibframeConfig::new()
            .with_base_uri("http://example.org/")
            .with_output_format(RdfFormat::Turtle)
            .with_authority_linking(true)
            .with_strict_mode(true);

        assert_eq!(config.base_uri, Some("http://example.org/".into()));
        assert_eq!(config.output_format, RdfFormat::Turtle);
        assert!(config.link_authorities);
        assert!(config.strict);
    }

    #[test]
    fn test_rdf_format_display() {
        assert_eq!(format!("{}", RdfFormat::RdfXml), "RDF/XML");
        assert_eq!(format!("{}", RdfFormat::JsonLd), "JSON-LD");
        assert_eq!(format!("{}", RdfFormat::Turtle), "Turtle");
        assert_eq!(format!("{}", RdfFormat::NTriples), "N-Triples");
    }

    #[test]
    fn test_rdf_format_mime_types() {
        assert_eq!(RdfFormat::RdfXml.mime_type(), "application/rdf+xml");
        assert_eq!(RdfFormat::JsonLd.mime_type(), "application/ld+json");
        assert_eq!(RdfFormat::Turtle.mime_type(), "text/turtle");
        assert_eq!(RdfFormat::NTriples.mime_type(), "application/n-triples");
    }

    #[test]
    fn test_rdf_format_extensions() {
        assert_eq!(RdfFormat::RdfXml.file_extension(), "rdf");
        assert_eq!(RdfFormat::JsonLd.file_extension(), "jsonld");
        assert_eq!(RdfFormat::Turtle.file_extension(), "ttl");
        assert_eq!(RdfFormat::NTriples.file_extension(), "nt");
    }
}
