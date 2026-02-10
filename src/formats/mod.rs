//! Multi-format support for MARC records.
//!
//! This module provides a unified interface for reading and writing MARC records
//! in various serialization formats. All formats implement the same traits,
//! allowing format-agnostic code.
//!
//! # Supported Formats
//!
//! | Format | Module | Description |
//! |--------|--------|-------------|
//! | ISO 2709 | `iso2709` | Standard MARC interchange format (baseline) |
//!
//! BIBFRAME linked data support is available via the [`bibframe`](crate::bibframe) module.
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

// Core traits - always available
mod traits;

pub use traits::{FormatReader, FormatReaderExt, FormatWriter, RecordIterator};

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
    pub use crate::reader::MarcReader as Iso2709Reader;
    pub use crate::writer::MarcWriter as Iso2709Writer;
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
}

impl Format {
    /// Detect format from file extension.
    ///
    /// Returns `None` if the extension is not recognized.
    ///
    /// # Example
    ///
    /// ```
    /// use mrrc::formats::Format;
    ///
    /// assert_eq!(Format::from_extension("mrc"), Some(Format::Iso2709));
    /// assert_eq!(Format::from_extension("unknown"), None);
    /// ```
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mrc" | "marc" => Some(Self::Iso2709),
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
    /// ```
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Iso2709 => "mrc",
        }
    }

    /// Get the human-readable name for this format.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Iso2709 => "ISO 2709",
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
        assert_eq!(Format::from_extension("unknown"), None);
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(Format::Iso2709.extension(), "mrc");
    }

    #[test]
    fn test_format_name() {
        assert_eq!(Format::Iso2709.name(), "ISO 2709");
    }

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", Format::Iso2709), "ISO 2709");
    }
}
