//! Error types for MARC operations.
//!
//! This module provides the [`MarcError`] type for all MARC library operations
//! and the [`Result`] convenience type.

use thiserror::Error;

/// Error type for all MARC library operations.
///
/// Represents various error conditions that can occur during MARC record
/// parsing, writing, or manipulation.
#[derive(Error, Debug)]
pub enum MarcError {
    /// Error indicating an invalid or malformed MARC record.
    #[error("Invalid MARC record: {0}")]
    InvalidRecord(String),

    /// Error indicating an invalid leader (24-byte header).
    #[error("Invalid leader: {0}")]
    InvalidLeader(String),

    /// Error indicating an invalid field structure.
    #[error("Invalid field: {0}")]
    InvalidField(String),

    /// Error related to character encoding conversion.
    #[error("Encoding error: {0}")]
    EncodingError(String),

    /// Error during parsing of MARC data.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Error indicating a truncated or incomplete record.
    #[error("Truncated record: {0}")]
    TruncatedRecord(String),

    /// IO error from the underlying source/destination.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Convenience type alias for [`std::result::Result`] with [`MarcError`].
pub type Result<T> = std::result::Result<T, MarcError>;
