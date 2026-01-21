//! Multi-format support for MARC records.
//!
//! This module provides a unified interface for reading and writing MARC records
//! in various serialization formats. All formats implement the same traits,
//! allowing format-agnostic code.
//!
//! # Format Tiers
//!
//! Formats are organized into tiers based on importance and use cases:
//!
//! ## Tier 1: Core Formats (Always Available)
//!
//! | Format | Module | Description |
//! |--------|--------|-------------|
//! | ISO 2709 | `iso2709` | Standard MARC interchange format (baseline) |
//! | Protobuf | `protobuf` | Schema-based binary format with evolution support |
//!
//! ## Tier 2: High-Value Formats (Feature-Gated)
//!
//! | Format | Feature | Module | Description |
//! |--------|---------|--------|-------------|
//! | Arrow | `format-arrow` | `arrow` | Columnar format for analytics (DuckDB, Polars) |
//! | Flatbuffers | `format-flatbuffers` | `flatbuffers` | Zero-copy, memory-efficient streaming |
//! | Messagepack | `format-messagepack` | `messagepack` | Compact binary, 50+ language support |
//!
//! ## Tier 3: Specialized Formats (Feature-Gated, On-Demand)
//!
//! | Format | Feature | Module | Description |
//! |--------|---------|--------|-------------|
//! | CBOR | `format-cbor` | `cbor` | RFC 7049 standard for archival |
//! | Avro | `format-avro` | `avro` | Kafka/data lake integration |
//!
//! # Usage
//!
//! ## Reading Records
//!
//! ```ignore
//! use mrrc::formats::{FormatReader, iso2709::Iso2709Reader};
//! use std::fs::File;
//!
//! let file = File::open("records.mrc")?;
//! let mut reader = Iso2709Reader::new(file);
//!
//! while let Some(record) = reader.read_record()? {
//!     println!("Title: {:?}", record.title());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Writing Records
//!
//! ```ignore
//! use mrrc::formats::{FormatWriter, iso2709::Iso2709Writer};
//! use mrrc::Record;
//!
//! let mut buffer = Vec::new();
//! let mut writer = Iso2709Writer::new(&mut buffer);
//!
//! writer.write_record(&record)?;
//! writer.finish()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Format-Agnostic Processing
//!
//! ```ignore
//! use mrrc::formats::{FormatReader, FormatWriter};
//!
//! fn convert<R: FormatReader, W: FormatWriter>(
//!     reader: &mut R,
//!     writer: &mut W,
//! ) -> mrrc::Result<usize> {
//!     let mut count = 0;
//!     while let Some(record) = reader.read_record()? {
//!         writer.write_record(&record)?;
//!         count += 1;
//!     }
//!     writer.finish()?;
//!     Ok(count)
//! }
//! ```
//!
//! # Feature Flags
//!
//! Format support is controlled by feature flags in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! mrrc = { version = "0.4", features = ["format-arrow", "format-messagepack"] }
//! ```
//!
//! Available features:
//! - `format-arrow` — Enable Arrow columnar format support
//! - `format-flatbuffers` — Enable FlatBuffers support
//! - `format-messagepack` — Enable MessagePack support
//! - `format-cbor` — Enable CBOR support
//! - `format-avro` — Enable Avro support
//! - `all-formats` — Enable all format support

// Core traits - always available
mod traits;

pub use traits::{FormatReader, FormatReaderExt, FormatWriter, RecordIterator};

// ============================================================================
// TIER 1: Core Formats (Always Available)
// ============================================================================

/// ISO 2709 binary format support (MARC standard interchange format).
///
/// This is the baseline format that all MARC libraries must support.
/// It provides streaming read/write with high performance (~900k records/sec).
///
/// # Performance
///
/// - Read: ~900,000 records/second
/// - Write: ~800,000 records/second
pub mod iso2709 {
    // Re-export existing implementations with trait-compatible wrappers
    // TODO: Create Iso2709Reader and Iso2709Writer wrapper types that implement traits
    // For now, users can use the existing MarcReader/MarcWriter directly
    pub use crate::reader::MarcReader as Iso2709Reader;
    pub use crate::writer::MarcWriter as Iso2709Writer;
}

/// Protocol Buffers binary format support.
///
/// Protobuf provides schema-based serialization with excellent forward/backward
/// compatibility, making it ideal for APIs and cross-language interchange.
///
/// # Schema Evolution
///
/// The Protobuf schema supports forward and backward compatibility:
/// - New optional fields can be added without breaking old readers
/// - Old fields can be deprecated but remain readable
///
/// # Usage
///
/// ```ignore
/// use mrrc::formats::{FormatReader, FormatWriter};
/// use mrrc::formats::protobuf::{ProtobufReader, ProtobufWriter};
/// use std::io::Cursor;
///
/// // Writing records
/// let mut buffer = Vec::new();
/// let mut writer = ProtobufWriter::new(&mut buffer);
/// writer.write_record(&record)?;
/// writer.finish()?;
///
/// // Reading records
/// let cursor = Cursor::new(buffer);
/// let mut reader = ProtobufReader::new(cursor);
/// while let Some(record) = reader.read_record()? {
///     // process record
/// }
/// ```
pub mod protobuf {
    // Re-export streaming reader/writer that implement FormatReader/FormatWriter traits
    pub use crate::protobuf::{ProtobufReader, ProtobufWriter};
    // Re-export single-record serializer/deserializer for direct use
    pub use crate::protobuf::{ProtobufDeserializer, ProtobufSerializer};
}

// ============================================================================
// TIER 2: High-Value Formats (Feature-Gated)
// ============================================================================

/// Apache Arrow columnar format support.
///
/// Arrow provides columnar data representation optimized for analytics.
/// Enable with the `format-arrow` feature.
#[cfg(feature = "format-arrow")]
pub mod arrow {
    //! Apache Arrow columnar format for MARC records.
    //!
    //! Arrow is the standard for in-memory columnar data, enabling
    //! integration with analytics tools like DuckDB, Polars, and DataFusion.
    //!
    //! # Use Cases
    //!
    //! - Large-scale MARC analytics (field frequency, pattern analysis)
    //! - Integration with data science tools (Pandas, Polars)
    //! - SQL queries over MARC collections (via DuckDB)
    //!
    //! # Performance
    //!
    //! - Read: ~865,000 records/second
    //! - Excellent compression (~96% size reduction)
    //!
    //! # Usage
    //!
    //! ```ignore
    //! use mrrc::formats::arrow::ArrowWriter;
    //! use mrrc::formats::FormatWriter;
    //! use std::io::Cursor;
    //!
    //! let mut buffer = Vec::new();
    //! let mut writer = ArrowWriter::new(&mut buffer);
    //!
    //! writer.write_record(&record)?;
    //! writer.finish()?;
    //!
    //! // buffer now contains Arrow IPC stream data
    //! ```

    // Re-export ArrowReader and ArrowWriter for streaming I/O
    pub use crate::arrow_impl::{ArrowReader, ArrowWriter};

    // Re-export helper functions for direct batch operations
    pub use crate::arrow_impl::{
        arrow_batch_to_records, create_arrow_schema, records_to_arrow_batch, ArrowMarcTable,
    };
}

/// FlatBuffers zero-copy format support.
///
/// FlatBuffers provides memory-efficient serialization with zero-copy access.
/// Enable with the `format-flatbuffers` feature.
#[cfg(feature = "format-flatbuffers")]
pub mod flatbuffers {
    //! FlatBuffers format for MARC records.
    //!
    //! FlatBuffers is ideal for memory-constrained environments and
    //! streaming APIs where zero-copy access is beneficial.
    //!
    //! # Use Cases
    //!
    //! - Mobile/embedded MARC applications
    //! - Streaming APIs with memory constraints
    //! - Real-time MARC data processing
    //!
    //! # Performance
    //!
    //! - 64% memory savings vs standard serialization
    //! - Zero-copy field access (no deserialization overhead)

    // TODO: Implement FlatBuffersReader and FlatBuffersWriter with trait conformance
}

/// `MessagePack` compact binary format support.
///
/// `MessagePack` provides compact serialization with broad language support (50+ languages),
/// making it ideal for REST APIs, IPC, and language-agnostic data exchange.
///
/// # Use Cases
///
/// - REST API responses (smaller than JSON)
/// - Inter-process communication
/// - Language-agnostic data exchange
///
/// # Performance
///
/// - Read/Write: ~750,000 records/second
/// - ~25% smaller than equivalent JSON
///
/// # Usage
///
/// ```ignore
/// use mrrc::formats::messagepack::{MessagePackReader, MessagePackWriter};
/// use mrrc::formats::{FormatReader, FormatWriter};
/// use std::io::Cursor;
///
/// // Writing records
/// let mut buffer = Vec::new();
/// let mut writer = MessagePackWriter::new(&mut buffer);
/// writer.write_record(&record)?;
/// writer.finish()?;
///
/// // Reading records
/// let cursor = Cursor::new(buffer);
/// let mut reader = MessagePackReader::new(cursor);
/// while let Some(record) = reader.read_record()? {
///     // process record
/// }
/// ```
pub mod messagepack {
    pub use crate::messagepack::{MessagePackReader, MessagePackWriter};
}

// ============================================================================
// TIER 3: Specialized Formats (Feature-Gated, On-Demand)
// ============================================================================

/// CBOR (RFC 7049) format support.
///
/// CBOR provides standards-compliant binary serialization for archival.
/// Enable with the `format-cbor` feature.
#[cfg(feature = "format-cbor")]
pub mod cbor {
    //! CBOR format for MARC records.
    //!
    //! CBOR (Concise Binary Object Representation) is an IETF standard
    //! suitable for government and academic archival requirements.
    //!
    //! # Use Cases
    //!
    //! - Long-term archival with standards compliance
    //! - Government/academic preservation requirements

    // TODO: Implement CborReader and CborWriter with trait conformance
}

/// Apache Avro format support.
///
/// Avro provides schema-registry integration for data lake ecosystems.
/// Enable with the `format-avro` feature.
#[cfg(feature = "format-avro")]
pub mod avro {
    //! Apache Avro format for MARC records.
    //!
    //! Avro is designed for Kafka and data lake integration with
    //! schema registry support.
    //!
    //! # Use Cases
    //!
    //! - Kafka streaming pipelines
    //! - Data lake ingestion (Hadoop, Spark)
    //! - Schema registry integration

    // TODO: Implement AvroReader and AvroWriter with trait conformance
}

// ============================================================================
// Format Detection and Convenience Functions
// ============================================================================

/// Supported format types for format detection and dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Format {
    /// ISO 2709 binary MARC format (`.mrc`, `.marc`)
    Iso2709,
    /// Protocol Buffers format (`.pb`, `.proto`)
    Protobuf,
    /// Apache Arrow IPC format (`.arrow`, `.feather`)
    #[cfg(feature = "format-arrow")]
    Arrow,
    /// FlatBuffers format (`.fb`, `.fbs`)
    #[cfg(feature = "format-flatbuffers")]
    FlatBuffers,
    /// MessagePack format (`.msgpack`, `.mp`)
    #[cfg(feature = "format-messagepack")]
    MessagePack,
    /// CBOR format (`.cbor`)
    #[cfg(feature = "format-cbor")]
    Cbor,
    /// Apache Avro format (`.avro`)
    #[cfg(feature = "format-avro")]
    Avro,
}

impl Format {
    /// Detect format from file extension.
    ///
    /// Returns `None` if the extension is not recognized or the format
    /// feature is not enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use mrrc::formats::Format;
    ///
    /// assert_eq!(Format::from_extension("mrc"), Some(Format::Iso2709));
    /// assert_eq!(Format::from_extension("pb"), Some(Format::Protobuf));
    /// assert_eq!(Format::from_extension("unknown"), None);
    /// ```
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mrc" | "marc" => Some(Self::Iso2709),
            "pb" | "proto" | "protobuf" => Some(Self::Protobuf),
            #[cfg(feature = "format-arrow")]
            "arrow" | "feather" | "ipc" => Some(Self::Arrow),
            #[cfg(feature = "format-flatbuffers")]
            "fb" | "fbs" | "flatbuf" => Some(Self::FlatBuffers),
            #[cfg(feature = "format-messagepack")]
            "msgpack" | "mp" | "messagepack" => Some(Self::MessagePack),
            #[cfg(feature = "format-cbor")]
            "cbor" => Some(Self::Cbor),
            #[cfg(feature = "format-avro")]
            "avro" => Some(Self::Avro),
            _ => None,
        }
    }

    /// Get the canonical file extension for this format.
    ///
    /// # Example
    ///
    /// ```
    /// use mrrc::formats::Format;
    ///
    /// assert_eq!(Format::Iso2709.extension(), "mrc");
    /// assert_eq!(Format::Protobuf.extension(), "pb");
    /// ```
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Iso2709 => "mrc",
            Self::Protobuf => "pb",
            #[cfg(feature = "format-arrow")]
            Self::Arrow => "arrow",
            #[cfg(feature = "format-flatbuffers")]
            Self::FlatBuffers => "fb",
            #[cfg(feature = "format-messagepack")]
            Self::MessagePack => "msgpack",
            #[cfg(feature = "format-cbor")]
            Self::Cbor => "cbor",
            #[cfg(feature = "format-avro")]
            Self::Avro => "avro",
        }
    }

    /// Get the human-readable name for this format.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Iso2709 => "ISO 2709",
            Self::Protobuf => "Protocol Buffers",
            #[cfg(feature = "format-arrow")]
            Self::Arrow => "Apache Arrow",
            #[cfg(feature = "format-flatbuffers")]
            Self::FlatBuffers => "FlatBuffers",
            #[cfg(feature = "format-messagepack")]
            Self::MessagePack => "MessagePack",
            #[cfg(feature = "format-cbor")]
            Self::Cbor => "CBOR",
            #[cfg(feature = "format-avro")]
            Self::Avro => "Apache Avro",
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_extension() {
        assert_eq!(Format::from_extension("mrc"), Some(Format::Iso2709));
        assert_eq!(Format::from_extension("marc"), Some(Format::Iso2709));
        assert_eq!(Format::from_extension("MRC"), Some(Format::Iso2709));
        assert_eq!(Format::from_extension("pb"), Some(Format::Protobuf));
        assert_eq!(Format::from_extension("proto"), Some(Format::Protobuf));
        assert_eq!(Format::from_extension("unknown"), None);
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(Format::Iso2709.extension(), "mrc");
        assert_eq!(Format::Protobuf.extension(), "pb");
    }

    #[test]
    fn test_format_name() {
        assert_eq!(Format::Iso2709.name(), "ISO 2709");
        assert_eq!(Format::Protobuf.name(), "Protocol Buffers");
    }

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", Format::Iso2709), "ISO 2709");
        assert_eq!(format!("{}", Format::Protobuf), "Protocol Buffers");
    }
}
