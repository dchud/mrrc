//! Recovery strategies for malformed and truncated MARC records.
//!
//! This module provides tools for gracefully handling MARC records that are
//! truncated, malformed, or otherwise incomplete. The recovery mechanism attempts
//! to extract as much valid data as possible while maintaining data integrity.

use crate::error::{MarcError, Result};
use crate::leader::Leader;
use crate::record::Record;

/// Strategy for handling malformed or truncated records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecoveryMode {
    /// Strict mode: return errors for any malformation (default)
    #[default]
    Strict,
    /// Lenient mode: attempt to recover and salvage valid data
    Lenient,
    /// Permissive mode: be very lenient with recovery, accepting partial data
    Permissive,
}

/// Recovery context for handling malformed data
#[derive(Debug)]
pub struct RecoveryContext {
    /// Current recovery mode
    pub mode: RecoveryMode,
    /// Whether warnings/recoveries were needed
    pub has_errors: bool,
    /// List of recovery messages
    pub recovery_messages: Vec<String>,
}

impl Default for RecoveryContext {
    fn default() -> Self {
        RecoveryContext {
            mode: RecoveryMode::Strict,
            has_errors: false,
            recovery_messages: Vec::new(),
        }
    }
}

impl RecoveryContext {
    /// Create a new recovery context with the given mode
    #[must_use]
    pub fn new(mode: RecoveryMode) -> Self {
        RecoveryContext {
            mode,
            has_errors: false,
            recovery_messages: Vec::new(),
        }
    }

    /// Record a recovery message
    fn add_message(&mut self, message: String) {
        self.has_errors = true;
        self.recovery_messages.push(message);
    }

    /// Try to recover from an error based on the recovery mode
    ///
    /// # Errors
    ///
    /// Returns an error if in strict mode, otherwise records the error and returns Ok(None).
    pub fn recover<T>(&mut self, error: MarcError, context: &str) -> Result<Option<T>> {
        match self.mode {
            RecoveryMode::Strict => Err(error),
            RecoveryMode::Lenient | RecoveryMode::Permissive => {
                self.add_message(format!("{context}: {error}"));
                Ok(None)
            },
        }
    }
}

const FIELD_TERMINATOR: u8 = 0x1E;
const SUBFIELD_DELIMITER: u8 = 0x1F;

/// Recover a record from a truncated or malformed state
///
/// # Errors
///
/// Returns an error if the record cannot be recovered, depending on the recovery mode.
#[allow(clippy::too_many_lines)]
pub fn try_recover_record(
    leader: Leader,
    partial_data: &[u8],
    base_address: usize,
    mode: RecoveryMode,
) -> Result<Record> {
    let mut context = RecoveryContext::new(mode);
    let mut record = Record::new(leader);

    // Try to extract whatever valid directory entries we can find
    let directory_size = base_address.saturating_sub(24);

    if directory_size == 0 {
        return Err(MarcError::TruncatedRecord(
            "No directory found in record".to_string(),
        ));
    }

    // Attempt to parse directory entries with recovery
    let directory_end = std::cmp::min(directory_size, partial_data.len());
    let directory = &partial_data[..directory_end];

    let mut pos = 0;
    while pos < directory.len() {
        if directory[pos] == FIELD_TERMINATOR {
            break;
        }

        // Check if we have enough bytes for a complete entry
        if pos + 12 > directory.len() {
            if mode != RecoveryMode::Strict {
                context.add_message("Incomplete directory entry at end of record".to_string());
            }
            break;
        }

        // Try to parse directory entry
        let entry_chunk = &directory[pos..pos + 12];
        let tag = String::from_utf8_lossy(&entry_chunk[0..3]).to_string();

        // Parse field length and position with error handling
        let field_length = if mode == RecoveryMode::Strict {
            parse_4digits(&entry_chunk[3..7])?
        } else if let Ok(len) = parse_4digits(&entry_chunk[3..7]) {
            len
        } else {
            context.add_message(format!("Invalid field length for tag {tag}"));
            pos += 12;
            continue;
        };

        let start_position = if mode == RecoveryMode::Strict {
            parse_digits(&entry_chunk[7..12])?
        } else if let Ok(p) = parse_digits(&entry_chunk[7..12]) {
            p
        } else {
            context.add_message(format!("Invalid start position for tag {tag}"));
            pos += 12;
            continue;
        };

        pos += 12;

        // Check if field data is accessible
        let end_position = start_position + field_length;
        let data_start = directory_size;

        if start_position < data_start || end_position > partial_data.len() {
            if mode == RecoveryMode::Strict {
                return Err(MarcError::TruncatedRecord(format!(
                    "Field {tag} data not available"
                )));
            }
            context.add_message(format!("Field {tag} data truncated"));

            // Try to read what we have
            let available_end = std::cmp::min(end_position, partial_data.len());
            if available_end > data_start {
                if let Ok(field) = try_parse_field(
                    &partial_data[start_position..available_end],
                    &tag,
                    SUBFIELD_DELIMITER,
                    FIELD_TERMINATOR,
                ) {
                    record.add_field(field);
                }
            }
            continue;
        }

        if tag != "LDR" {
            if tag.starts_with('0') && tag.chars().all(char::is_numeric) && tag.as_str() < "010" {
                let value = String::from_utf8_lossy(
                    &partial_data[start_position..end_position.saturating_sub(1)],
                )
                .to_string();
                record.add_control_field(tag, value);
            } else if let Ok(field) = try_parse_field(
                &partial_data[start_position..end_position],
                &tag,
                SUBFIELD_DELIMITER,
                FIELD_TERMINATOR,
            ) {
                record.add_field(field);
            } else if mode != RecoveryMode::Strict {
                context.add_message(format!("Failed to parse field {tag}"));
            }
        }
    }

    Ok(record)
}

/// Try to parse a data field, returning None if it fails (for recovery mode)
fn try_parse_field(
    data: &[u8],
    tag: &str,
    subfield_delim: u8,
    field_term: u8,
) -> Result<crate::record::Field> {
    use crate::record::Field;

    if data.is_empty() {
        return Err(MarcError::InvalidField("Empty field data".to_string()));
    }

    if data.len() < 2 {
        return Err(MarcError::InvalidField(
            "Data field too short (needs indicators)".to_string(),
        ));
    }

    let indicator1 = data[0] as char;
    let indicator2 = data[1] as char;
    let mut field = Field::new(tag.to_string(), indicator1, indicator2);

    let subfield_data = &data[2..];
    let mut current_position = 0;

    while current_position < subfield_data.len() {
        if subfield_data[current_position] == field_term {
            break;
        }

        if subfield_data[current_position] == subfield_delim {
            current_position += 1;
            if current_position >= subfield_data.len() {
                break;
            }

            let code = subfield_data[current_position] as char;
            current_position += 1;

            let mut end = current_position;
            while end < subfield_data.len()
                && subfield_data[end] != subfield_delim
                && subfield_data[end] != field_term
            {
                end += 1;
            }

            let value = String::from_utf8_lossy(&subfield_data[current_position..end]).to_string();
            field.add_subfield(code, value);
            current_position = end;
        } else {
            return Err(MarcError::InvalidField(
                "Expected subfield delimiter".to_string(),
            ));
        }
    }

    Ok(field)
}

/// Parse a 4-digit ASCII number from bytes
fn parse_4digits(bytes: &[u8]) -> Result<usize> {
    if bytes.len() != 4 {
        return Err(MarcError::InvalidRecord(format!(
            "Expected 4-digit field, got {} bytes",
            bytes.len()
        )));
    }

    let s = String::from_utf8_lossy(bytes);
    s.parse::<usize>()
        .map_err(|_| MarcError::InvalidRecord(format!("Invalid numeric field: '{s}'")))
}

/// Parse a 5-digit ASCII number from bytes
fn parse_digits(bytes: &[u8]) -> Result<usize> {
    if bytes.len() != 5 {
        return Err(MarcError::InvalidRecord(format!(
            "Expected 5-digit field, got {} bytes",
            bytes.len()
        )));
    }

    let s = String::from_utf8_lossy(bytes);
    s.parse::<usize>()
        .map_err(|_| MarcError::InvalidRecord(format!("Invalid numeric field: '{s}'")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_context_default() {
        let ctx = RecoveryContext::default();
        assert_eq!(ctx.mode, RecoveryMode::Strict);
        assert!(!ctx.has_errors);
        assert!(ctx.recovery_messages.is_empty());
    }

    #[test]
    fn test_recovery_mode_lenient() {
        let mut ctx = RecoveryContext::new(RecoveryMode::Lenient);
        let error = MarcError::InvalidField("test".to_string());
        let result: Result<Option<()>> = ctx.recover(error, "test context");
        assert!(result.is_ok());
        assert!(ctx.has_errors);
        assert!(!ctx.recovery_messages.is_empty());
    }

    #[test]
    fn test_recovery_mode_strict() {
        let mut ctx = RecoveryContext::new(RecoveryMode::Strict);
        let error = MarcError::InvalidField("test".to_string());
        let result: Result<Option<()>> = ctx.recover(error, "test context");
        assert!(result.is_err());
    }
}
