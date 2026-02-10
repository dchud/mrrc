#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! # MRRC: MARC Rust Crate
//!
//! A high-performance Rust library for reading, writing, and manipulating MARC bibliographic
//! records in the ISO 2709 binary format.
//!
//! ## Quick Start
//!
//! ### Reading MARC Records
//!
//! ```ignore
//! use mrrc::MarcReader;
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let file = File::open("records.mrc")?;
//! let mut reader = MarcReader::new(file);
//!
//! while let Some(record) = reader.read_record()? {
//!     if let Some(title) = record.title() {
//!         println!("Title: {}", title);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Creating and Writing MARC Records
//!
//! ```ignore
//! use mrrc::{MarcWriter, Record, Field, Leader};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut record = Record::new(Leader::default());
//! record.add_control_field("001".to_string(), "12345".to_string());
//!
//! let mut field = Field::new("245".to_string(), '1', '0');
//! field.add_subfield('a', "Test Title".to_string());
//! record.add_field(field);
//!
//! let mut buffer = Vec::new();
//! {
//!     let mut writer = MarcWriter::new(&mut buffer);
//!     writer.write_record(&record)?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Using Helper Methods
//!
//! ```ignore
//! use mrrc::{Record, Leader, Field};
//!
//! let mut record = Record::new(Leader::default());
//!
//! let mut field_245 = Field::new("245".to_string(), '1', '0');
//! field_245.add_subfield('a', "The Great Gatsby".to_string());
//! field_245.add_subfield('c', "F. Scott Fitzgerald".to_string());
//! record.add_field(field_245);
//!
//! // Use convenience methods
//! assert_eq!(record.title(), Some("The Great Gatsby"));
//! assert_eq!(record.is_book(), true);
//! ```
//!
//! ## Modules
//!
//! - [`record`] — Core MARC record structures (`Record`, `Field`, `Subfield`)
//! - [`reader`] — Reading MARC records from binary data streams
//! - [`writer`] — Writing MARC records to binary format
//! - [`formats`] — Format traits and ISO 2709 support
//! - [`bibframe`] — BIBFRAME linked data conversion
//! - [`boundary_scanner`] — Record boundary detection for parallel processing
//! - [`leader`] — MARC record leader (24-byte header)
//! - [`json`] — JSON serialization/deserialization
//! - [`marcjson`] — MARCJSON format (standard JSON-LD format for MARC)
//! - [`xml`] — XML serialization/deserialization
//! - [`csv`] — CSV (Comma-Separated Values) export format
//! - [`dublin_core`] — Dublin Core metadata serialization
//! - [`mods`] — MODS (Metadata Object Description Schema) serialization
//! - [`encoding`] — Character encoding support (MARC-8 and UTF-8)
//! - [`error`] — Error types and result type
//!
//! ## Format Support
//!
//! - **ISO 2709 Binary Format** — The standard MARC interchange format
//! - **BIBFRAME** — LOC linked data format for bibliographic description
//! - **JSON** — Generic JSON representation of records
//! - **MARCJSON** — Standard JSON-LD format for MARC records
//! - **XML** — XML representation with proper field/subfield structure
//! - **CSV** — Tabular export format for spreadsheet applications
//! - **Dublin Core** — Simplified metadata schema for discovery
//! - **MODS** — Detailed metadata description schema for libraries
//! - **Character Encodings** — MARC-8 and UTF-8 with automatic detection

pub mod authority_queries;
pub mod authority_reader;
pub mod authority_record;
pub mod authority_writer;
pub mod bibframe;
pub mod bibliographic_helpers;
pub mod boundary_scanner;
pub mod csv;
pub mod dublin_core;
pub mod encoding;
pub mod encoding_validation;
pub mod error;
pub mod field_collection;
pub mod field_linkage;
pub mod field_query;
pub mod field_query_helpers;
pub mod format_queries;
/// Multi-format support with unified Reader/Writer traits.
///
/// See the [`formats`] module documentation for details on supported formats
/// and how to use format-agnostic code.
pub mod formats;
pub mod holdings_reader;
pub mod holdings_record;
pub mod holdings_writer;
pub mod json;
pub mod leader;
pub mod macros;
pub mod marc8_tables;
pub mod marc_record;
pub mod marcjson;
pub mod mods;
pub mod producer_consumer_pipeline;
pub mod rayon_parser_pool;
pub mod reader;
/// Core MARC record structures (`Record`, `Field`, `Subfield`)
pub mod record;
pub mod record_builder_generic;
pub mod record_helpers;
pub mod record_validation;
pub mod recovery;
pub mod validation;
pub mod writer;
pub mod xml;

pub use authority_queries::AuthorityQueries;
pub use authority_reader::AuthorityMarcReader;
pub use authority_record::{
    AuthorityRecord, AuthorityRecordBuilder, HeadingType, KindOfRecord, LevelOfEstablishment,
};
pub use authority_writer::AuthorityMarcWriter;
pub use bibliographic_helpers::{IsbnValidator, PublicationInfo};
pub use encoding_validation::{EncodingAnalysis, EncodingValidator};
pub use error::{MarcError, Result};
pub use field_linkage::LinkageInfo;
pub use field_query::{FieldQuery, SubfieldPatternQuery, SubfieldValueQuery, TagRangeQuery};
pub use field_query_helpers::FieldQueryHelpers;
pub use format_queries::{AuthoritySpecificQueries, BibliographicQueries, HoldingsSpecificQueries};
pub use holdings_reader::HoldingsMarcReader;
pub use holdings_record::{
    AcquisitionStatus, Completeness, HoldingsRecord, HoldingsRecordBuilder, HoldingsType,
    MethodOfAcquisition,
};
pub use holdings_writer::HoldingsMarcWriter;
pub use leader::Leader;
pub use marc_record::MarcRecord;
pub use producer_consumer_pipeline::{PipelineConfig, PipelineError, ProducerConsumerPipeline};
pub use reader::MarcReader;
pub use record::{Field, FieldBuilder, Record, RecordBuilder, Subfield};
pub use record_builder_generic::GenericRecordBuilder;
pub use record_helpers::RecordHelpers;
pub use record_validation::RecordStructureValidator;
pub use recovery::{RecoveryContext, RecoveryMode};
pub use validation::IndicatorValidator;
pub use writer::MarcWriter;
