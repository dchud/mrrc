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
//! - [`leader`] — MARC record leader (24-byte header)
//! - [`json`] — JSON serialization/deserialization
//! - [`marcjson`] — MARCJSON format (standard JSON-LD format for MARC)
//! - [`xml`] — XML serialization/deserialization
//! - [`encoding`] — Character encoding support (MARC-8 and UTF-8)
//! - [`error`] — Error types and result type
//!
//! ## Format Support
//!
//! The library supports:
//! - **ISO 2709 Binary Format** — The standard MARC interchange format
//! - **JSON** — Generic JSON representation of records
//! - **MARCJSON** — Standard JSON-LD format for MARC records
//! - **XML** — XML representation with proper field/subfield structure
//! - **Character Encodings** — MARC-8 and UTF-8 with automatic detection

pub mod encoding;
pub mod error;
pub mod json;
pub mod leader;
pub mod marc8_tables;
pub mod marcjson;
pub mod reader;
/// Core MARC record structures (`Record`, `Field`, `Subfield`)
pub mod record;
pub mod writer;
pub mod xml;

pub use error::{MarcError, Result};
pub use leader::Leader;
pub use reader::MarcReader;
pub use record::{Field, FieldBuilder, Record, RecordBuilder, Subfield};
pub use writer::MarcWriter;
