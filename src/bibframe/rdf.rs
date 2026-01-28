//! RDF serialization layer for BIBFRAME.
//!
//! This module provides RDF parsing and serialization using the oxrdfio library.
//! It wraps the library's functionality in a higher-level API tailored for
//! BIBFRAME conversion.

use std::io::{Read, Write};

use oxrdf::{BlankNode, Literal, NamedNode, NamedOrBlankNode, Quad, Term, Triple};
use oxrdfio::{JsonLdProfileSet, RdfFormat as OxRdfFormat, RdfParser, RdfSerializer};

use crate::error::{MarcError, Result};

use super::config::RdfFormat;
use super::namespaces;

/// A single RDF triple (subject, predicate, object).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RdfTriple {
    /// The subject of the triple.
    pub subject: RdfNode,
    /// The predicate (property) of the triple.
    pub predicate: String,
    /// The object of the triple.
    pub object: RdfNode,
}

impl RdfTriple {
    /// Creates a new RDF triple.
    #[must_use]
    pub fn new(subject: RdfNode, predicate: impl Into<String>, object: RdfNode) -> Self {
        Self {
            subject,
            predicate: predicate.into(),
            object,
        }
    }
}

/// An RDF node (subject or object in a triple).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RdfNode {
    /// A named node (IRI/URI).
    Uri(String),
    /// A blank node with a local identifier.
    BlankNode(String),
    /// A literal value with optional language tag or datatype.
    Literal {
        /// The literal value.
        value: String,
        /// Optional language tag (e.g., "en", "ja").
        language: Option<String>,
        /// Optional datatype URI.
        datatype: Option<String>,
    },
}

impl RdfNode {
    /// Creates a new URI node.
    #[must_use]
    pub fn uri(uri: impl Into<String>) -> Self {
        Self::Uri(uri.into())
    }

    /// Creates a new blank node.
    #[must_use]
    pub fn blank(id: impl Into<String>) -> Self {
        Self::BlankNode(id.into())
    }

    /// Creates a new plain literal.
    #[must_use]
    pub fn literal(value: impl Into<String>) -> Self {
        Self::Literal {
            value: value.into(),
            language: None,
            datatype: None,
        }
    }

    /// Creates a new literal with a language tag.
    #[must_use]
    pub fn literal_with_lang(value: impl Into<String>, lang: impl Into<String>) -> Self {
        Self::Literal {
            value: value.into(),
            language: Some(lang.into()),
            datatype: None,
        }
    }

    /// Creates a new typed literal.
    #[must_use]
    pub fn typed_literal(value: impl Into<String>, datatype: impl Into<String>) -> Self {
        Self::Literal {
            value: value.into(),
            language: None,
            datatype: Some(datatype.into()),
        }
    }

    /// Returns true if this is a URI node.
    #[must_use]
    pub const fn is_uri(&self) -> bool {
        matches!(self, Self::Uri(_))
    }

    /// Returns true if this is a blank node.
    #[must_use]
    pub const fn is_blank(&self) -> bool {
        matches!(self, Self::BlankNode(_))
    }

    /// Returns true if this is a literal.
    #[must_use]
    pub const fn is_literal(&self) -> bool {
        matches!(self, Self::Literal { .. })
    }

    /// Creates a BIBFRAME class URI.
    #[must_use]
    pub fn bf_class(class_name: &str) -> Self {
        Self::Uri(format!("{}{}", namespaces::BF, class_name))
    }

    /// Creates a BFLC class URI.
    #[must_use]
    pub fn bflc_class(class_name: &str) -> Self {
        Self::Uri(format!("{}{}", namespaces::BFLC, class_name))
    }
}

/// An RDF graph containing triples.
#[derive(Debug, Clone, Default)]
pub struct RdfGraph {
    /// The triples in this graph.
    triples: Vec<RdfTriple>,
    /// Counter for generating unique blank node IDs.
    blank_node_counter: usize,
}

impl RdfGraph {
    /// Creates a new empty RDF graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a triple to the graph.
    pub fn add_triple(&mut self, triple: RdfTriple) {
        self.triples.push(triple);
    }

    /// Adds a triple from components.
    pub fn add(&mut self, subject: RdfNode, predicate: impl Into<String>, object: RdfNode) {
        self.add_triple(RdfTriple::new(subject, predicate, object));
    }

    /// Generates a new unique blank node ID.
    pub fn new_blank_node(&mut self) -> RdfNode {
        self.blank_node_counter += 1;
        RdfNode::blank(format!("b{}", self.blank_node_counter))
    }

    /// Returns the number of triples in the graph.
    #[must_use]
    pub fn len(&self) -> usize {
        self.triples.len()
    }

    /// Returns true if the graph is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.triples.is_empty()
    }

    /// Returns an iterator over the triples.
    pub fn triples(&self) -> impl Iterator<Item = &RdfTriple> {
        self.triples.iter()
    }

    /// Serializes the graph to a string in the specified format.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn serialize(&self, format: RdfFormat) -> Result<String> {
        let mut output = Vec::new();
        self.serialize_to_writer(&mut output, format)?;
        String::from_utf8(output).map_err(|e| MarcError::ParseError(e.to_string()))
    }

    /// Serializes the graph to a writer in the specified format.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn serialize_to_writer<W: Write>(&self, writer: W, format: RdfFormat) -> Result<()> {
        let ox_format = to_oxrdf_format(format);
        let mut serializer = RdfSerializer::from_format(ox_format).for_writer(writer);

        for triple in &self.triples {
            let ox_triple = to_oxrdf_triple(triple)?;
            serializer.serialize_triple(&ox_triple).map_err(|e| {
                MarcError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;
        }

        serializer.finish().map_err(|e| {
            MarcError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        Ok(())
    }

    /// Parses an RDF graph from a reader in the specified format.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn parse_from_reader<R: Read>(reader: R, format: RdfFormat) -> Result<Self> {
        let ox_format = to_oxrdf_format(format);
        let parser = RdfParser::from_format(ox_format).for_reader(reader);

        let mut graph = Self::new();

        for result in parser {
            let quad = result.map_err(|e| MarcError::ParseError(e.to_string()))?;
            let triple = from_oxrdf_quad(&quad)?;
            graph.add_triple(triple);
        }

        Ok(graph)
    }

    /// Parses an RDF graph from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn parse(input: &str, format: RdfFormat) -> Result<Self> {
        Self::parse_from_reader(input.as_bytes(), format)
    }
}

/// Converts our [`RdfFormat`] to oxrdfio's format.
fn to_oxrdf_format(format: RdfFormat) -> OxRdfFormat {
    match format {
        RdfFormat::RdfXml => OxRdfFormat::RdfXml,
        RdfFormat::JsonLd => OxRdfFormat::JsonLd {
            profile: JsonLdProfileSet::default(),
        },
        RdfFormat::Turtle => OxRdfFormat::Turtle,
        RdfFormat::NTriples => OxRdfFormat::NTriples,
    }
}

/// Converts an [`RdfTriple`] to an oxrdf Triple.
fn to_oxrdf_triple(triple: &RdfTriple) -> Result<Triple> {
    let subject = match &triple.subject {
        RdfNode::Uri(uri) => NamedOrBlankNode::NamedNode(
            NamedNode::new(uri).map_err(|e| MarcError::ParseError(format!("Invalid URI: {e}")))?,
        ),
        RdfNode::BlankNode(id) => NamedOrBlankNode::BlankNode(
            BlankNode::new(id)
                .map_err(|e| MarcError::ParseError(format!("Invalid blank node ID: {e}")))?,
        ),
        RdfNode::Literal { .. } => {
            return Err(MarcError::ParseError(
                "Literals cannot be triple subjects".into(),
            ));
        },
    };

    let predicate = NamedNode::new(&triple.predicate)
        .map_err(|e| MarcError::ParseError(format!("Invalid predicate URI: {e}")))?;

    let object = match &triple.object {
        RdfNode::Uri(uri) => Term::NamedNode(
            NamedNode::new(uri).map_err(|e| MarcError::ParseError(format!("Invalid URI: {e}")))?,
        ),
        RdfNode::BlankNode(id) => Term::BlankNode(
            BlankNode::new(id)
                .map_err(|e| MarcError::ParseError(format!("Invalid blank node ID: {e}")))?,
        ),
        RdfNode::Literal {
            value,
            language,
            datatype,
        } => {
            let lit = if let Some(lang) = language {
                Literal::new_language_tagged_literal(value, lang)
                    .map_err(|e| MarcError::ParseError(format!("Invalid language tag: {e}")))?
            } else if let Some(dt) = datatype {
                let dt_node = NamedNode::new(dt)
                    .map_err(|e| MarcError::ParseError(format!("Invalid datatype URI: {e}")))?;
                Literal::new_typed_literal(value, dt_node)
            } else {
                Literal::new_simple_literal(value)
            };
            Term::Literal(lit)
        },
    };

    Ok(Triple::new(subject, predicate, object))
}

/// Converts an oxrdf Quad back to our [`RdfTriple`].
fn from_oxrdf_quad(quad: &Quad) -> Result<RdfTriple> {
    let subject = match &quad.subject {
        NamedOrBlankNode::NamedNode(n) => RdfNode::Uri(n.as_str().to_string()),
        NamedOrBlankNode::BlankNode(b) => RdfNode::BlankNode(b.as_str().to_string()),
        #[allow(unreachable_patterns)]
        _ => {
            return Err(MarcError::ParseError("Unsupported subject type".into()));
        },
    };

    let predicate = quad.predicate.as_str().to_string();

    let object = match &quad.object {
        Term::NamedNode(n) => RdfNode::Uri(n.as_str().to_string()),
        Term::BlankNode(b) => RdfNode::BlankNode(b.as_str().to_string()),
        Term::Literal(lit) => {
            let value = lit.value().to_string();
            let language = lit.language().map(String::from);
            let datatype = if language.is_none() && lit.datatype().as_str() != namespaces::XSD {
                Some(lit.datatype().as_str().to_string())
            } else {
                None
            };
            RdfNode::Literal {
                value,
                language,
                datatype,
            }
        },
        #[allow(unreachable_patterns)]
        _ => {
            return Err(MarcError::ParseError("Unsupported object type".into()));
        },
    };

    Ok(RdfTriple::new(subject, predicate, object))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdf_node_construction() {
        let uri = RdfNode::uri("http://example.org/foo");
        assert!(uri.is_uri());

        let blank = RdfNode::blank("b1");
        assert!(blank.is_blank());

        let lit = RdfNode::literal("hello");
        assert!(lit.is_literal());

        let lang_lit = RdfNode::literal_with_lang("hello", "en");
        assert!(lang_lit.is_literal());
    }

    #[test]
    fn test_bf_class_uri() {
        let work = RdfNode::bf_class("Work");
        assert_eq!(
            work,
            RdfNode::Uri("http://id.loc.gov/ontologies/bibframe/Work".into())
        );
    }

    #[test]
    fn test_rdf_graph_operations() {
        let mut graph = RdfGraph::new();
        assert!(graph.is_empty());

        let subj = graph.new_blank_node();
        graph.add(
            subj.clone(),
            format!("{}type", namespaces::RDF),
            RdfNode::bf_class("Work"),
        );
        graph.add(
            subj,
            format!("{}{}", namespaces::RDFS, "label"),
            RdfNode::literal("Test Work"),
        );

        assert_eq!(graph.len(), 2);
        assert!(!graph.is_empty());
    }

    #[test]
    fn test_serialize_ntriples() {
        let mut graph = RdfGraph::new();
        let subj = RdfNode::uri("http://example.org/work1");
        graph.add(
            subj.clone(),
            format!("{}type", namespaces::RDF),
            RdfNode::bf_class("Work"),
        );
        graph.add(
            subj,
            format!("{}{}", namespaces::RDFS, "label"),
            RdfNode::literal("Test"),
        );

        let nt = graph
            .serialize(RdfFormat::NTriples)
            .expect("serialization failed");
        assert!(nt.contains("<http://example.org/work1>"));
        assert!(nt.contains("bibframe/Work"));
        assert!(nt.contains("\"Test\""));
    }

    #[test]
    fn test_roundtrip_ntriples() {
        let mut graph = RdfGraph::new();
        let subj = RdfNode::uri("http://example.org/work1");
        graph.add(
            subj,
            format!("{}type", namespaces::RDF),
            RdfNode::bf_class("Work"),
        );

        let nt = graph
            .serialize(RdfFormat::NTriples)
            .expect("serialization failed");
        let parsed = RdfGraph::parse(&nt, RdfFormat::NTriples).expect("parsing failed");

        assert_eq!(parsed.len(), graph.len());
    }
}
