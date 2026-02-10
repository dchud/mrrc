//! BIBFRAME conversion for MARC records.
//!
//! This module provides bidirectional conversion between MARC records and BIBFRAME,
//! the Library of Congress's linked data model for bibliographic description.
//!
//! # What is BIBFRAME?
//!
//! BIBFRAME (Bibliographic Framework) is a modern linked data replacement for MARC,
//! using RDF to represent bibliographic data. A single MARC record typically becomes
//! multiple BIBFRAME entities:
//!
//! - **Work**: The intellectual content (authors, subjects, language)
//! - **Instance**: A material embodiment (publisher, format, ISBN)
//! - **Item**: A specific copy (location, barcode)
//!
//! # Quick Start
//!
//! ## MARC to BIBFRAME
//!
//! ```ignore
//! use mrrc::{MarcReader, bibframe::{marc_to_bibframe, BibframeConfig}};
//!
//! let reader = MarcReader::from_path("records.mrc")?;
//! for record in reader {
//!     let graph = marc_to_bibframe(&record?, &BibframeConfig::default())?;
//!     println!("{}", graph.to_string()?);
//! }
//! ```
//!
//! ## BIBFRAME to MARC
//!
//! ```ignore
//! use mrrc::bibframe::{bibframe_to_marc, RdfGraph, RdfFormat};
//!
//! let rdf_data = std::fs::read_to_string("record.rdf")?;
//! let graph = RdfGraph::parse(&rdf_data, RdfFormat::RdfXml)?;
//! let record = bibframe_to_marc(&graph)?;
//! ```
//!
//! # Configuration
//!
//! Use [`BibframeConfig`] to control conversion behavior:
//!
//! ```ignore
//! use mrrc::bibframe::{BibframeConfig, RdfFormat};
//!
//! let config = BibframeConfig::new()
//!     .with_base_uri("http://example.org/")
//!     .with_output_format(RdfFormat::Turtle)
//!     .with_authority_linking(true);
//! ```
//!
//! # Modules
//!
//! - `config`: Configuration options for BIBFRAME conversion
//! - `namespaces`: BIBFRAME namespace prefixes and vocabulary terms
//! - `rdf`: RDF graph representation and serialization

mod config;
mod converter;
mod namespaces;
mod rdf;
mod reverse_converter;

pub use config::{BibframeConfig, RdfFormat};
pub use namespaces::{
    bflc, classes, properties, BF, BFLC, CARRIER_TYPES, CONTENT_TYPES, COUNTRIES, LANGUAGES,
    LC_NAMES, LC_SUBJECTS, MADSRDF, MEDIA_TYPES, RDF, RDFS, RELATORS, XSD,
};
pub use rdf::{RdfGraph, RdfNode, RdfTriple};

use crate::error::Result;
use crate::record::Record;

/// Converts a MARC record to a BIBFRAME RDF graph.
///
/// This function transforms a MARC bibliographic record into a BIBFRAME 2.0
/// RDF graph containing Work, Instance, and optionally Item entities.
///
/// # Arguments
///
/// * `record` - The MARC record to convert
/// * `config` - Configuration options for the conversion
///
/// # Returns
///
/// An RDF graph containing the BIBFRAME representation.
///
/// # Examples
///
/// ```ignore
/// use mrrc::{Record, Leader, bibframe::{marc_to_bibframe, BibframeConfig}};
///
/// let record = Record::new(/* leader */);
/// let graph = marc_to_bibframe(&record, &BibframeConfig::default());
/// println!("Created {} triples", graph.len());
/// ```
#[must_use]
pub fn marc_to_bibframe(record: &Record, config: &BibframeConfig) -> RdfGraph {
    converter::convert_marc_to_bibframe(record, config)
}

/// Converts a BIBFRAME RDF graph to a MARC record.
///
/// This function transforms a BIBFRAME 2.0 RDF graph back into a MARC
/// bibliographic record. Note that some information loss is inherent
/// because BIBFRAME is semantically richer than MARC.
///
/// # Arguments
///
/// * `graph` - The BIBFRAME RDF graph to convert
///
/// # Returns
///
/// A MARC Record representing the BIBFRAME data.
///
/// # Errors
///
/// Returns an error if the graph cannot be converted (e.g., missing Work entity).
///
/// # Examples
///
/// ```ignore
/// use mrrc::bibframe::{bibframe_to_marc, RdfGraph, RdfFormat};
///
/// let rdf_data = std::fs::read_to_string("record.jsonld")?;
/// let graph = RdfGraph::parse(&rdf_data, RdfFormat::JsonLd)?;
/// let record = bibframe_to_marc(&graph)?;
/// ```
pub fn bibframe_to_marc(graph: &RdfGraph) -> Result<Record> {
    reverse_converter::convert_bibframe_to_marc(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leader::Leader;

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
    fn test_marc_to_bibframe_basic() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "12345".to_string());

        let config = BibframeConfig::default();
        let graph = marc_to_bibframe(&record, &config);

        // Should have at least Work and Instance with types and relationships
        assert!(graph.len() >= 4);
    }

    #[test]
    fn test_marc_to_bibframe_with_base_uri() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test123".to_string());

        let config = BibframeConfig::new().with_base_uri("http://example.org/");
        let graph = marc_to_bibframe(&record, &config);

        // Serialize and check for expected URIs
        let nt = graph
            .serialize(RdfFormat::NTriples)
            .expect("serialization failed");
        assert!(nt.contains("http://example.org/work/test123"));
        assert!(nt.contains("http://example.org/instance/test123"));
    }

    #[test]
    fn test_bibframe_to_marc_stub() {
        let graph = RdfGraph::new();
        let record = bibframe_to_marc(&graph).expect("conversion failed");
        // Currently just returns empty record
        assert!(record.fields.is_empty());
    }

    #[test]
    fn test_rdf_format_serialization() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "12345".to_string());

        let config = BibframeConfig::default();
        let graph = marc_to_bibframe(&record, &config);

        // Test different output formats
        let _ = graph
            .serialize(RdfFormat::NTriples)
            .expect("N-Triples serialization failed");
        let _ = graph
            .serialize(RdfFormat::Turtle)
            .expect("Turtle serialization failed");
    }
}
