use thiserror::Error;

#[derive(Error, Debug)]
pub enum MarcError {
    #[error("Invalid MARC record: {0}")]
    InvalidRecord(String),

    #[error("Invalid leader: {0}")]
    InvalidLeader(String),

    #[error("Invalid field: {0}")]
    InvalidField(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, MarcError>;
