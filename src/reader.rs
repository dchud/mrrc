//! Reading MARC records from binary streams.
//!
//! This module provides [`MarcReader`] for reading ISO 2709 formatted MARC records
//! from any source that implements [`std::io::Read`].
//!
//! # Examples
//!
//! Reading records from a file:
//!
//! ```no_run
//! use mrrc::MarcReader;
//! use std::fs::File;
//!
//! let file = File::open("records.mrc")?;
//! let mut reader = MarcReader::new(file);
//!
//! while let Some(record) = reader.read_record()? {
//!     println!("Record type: {}", record.leader.record_type);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Reading from a buffer:
//!
//! ```
//! use mrrc::MarcReader;
//! use std::io::Cursor;
//!
//! let data = b"...binary MARC data...";
//! let cursor = Cursor::new(data.to_vec());
//! let mut reader = MarcReader::new(cursor);
//!
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::error::Result;
use crate::formats::FormatReader;
use crate::iso2709::{self, DataFieldParseConfig, ParseContext, FIELD_TERMINATOR, LEADER_LEN};
use crate::leader::Leader;
use crate::record::Record;
use crate::recovery::RecoveryMode;
use std::io::Read;

/// Reader for ISO 2709 binary MARC format.
///
/// `MarcReader` reads one MARC record at a time from any source implementing [`std::io::Read`].
/// Records are fully parsed and returned as [`Record`] instances.
///
/// # Examples
///
/// ```
/// use mrrc::MarcReader;
/// use std::io::Cursor;
///
/// let binary_data = vec![]; // MARC binary data
/// let cursor = Cursor::new(binary_data);
/// let mut reader = MarcReader::new(cursor);
///
/// match reader.read_record() {
///     Ok(Some(record)) => println!("Record type: {}", record.leader.record_type),
///     Ok(None) => println!("End of file"),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
#[derive(Debug)]
pub struct MarcReader<R: Read> {
    reader: R,
    recovery_mode: RecoveryMode,
    records_read: usize,
    ctx: ParseContext,
}

impl<R: Read> MarcReader<R> {
    /// Create a new MARC reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - Any source implementing [`std::io::Read`]
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::MarcReader;
    /// use std::io::Cursor;
    ///
    /// let data = vec![];
    /// let cursor = Cursor::new(data);
    /// let reader = MarcReader::new(cursor);
    /// ```
    pub fn new(reader: R) -> Self {
        MarcReader {
            reader,
            recovery_mode: RecoveryMode::Strict,
            records_read: 0,
            ctx: ParseContext::new(),
        }
    }

    /// Set the recovery mode for handling malformed records.
    ///
    /// The recovery mode determines how the reader handles truncated or
    /// malformed MARC records:
    /// - `Strict`: Return errors immediately (default)
    /// - `Lenient`: Attempt to recover and salvage valid data
    /// - `Permissive`: Be very lenient, accepting partial data
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::{MarcReader, RecoveryMode};
    /// use std::io::Cursor;
    ///
    /// let data = vec![];
    /// let cursor = Cursor::new(data);
    /// let mut reader = MarcReader::new(cursor)
    ///     .with_recovery_mode(RecoveryMode::Lenient);
    /// ```
    #[must_use]
    pub fn with_recovery_mode(mut self, mode: RecoveryMode) -> Self {
        self.recovery_mode = mode;
        self
    }

    /// Attach a source identifier (filename or stream id) to errors raised by
    /// this reader. Populates `source_name` on every emitted error where
    /// applicable. Use [`MarcReader::from_path`] when constructing from a
    /// filesystem path to set this automatically.
    #[must_use]
    pub fn with_source(mut self, name: impl Into<String>) -> Self {
        self.ctx.source_name = Some(name.into());
        self
    }
}

impl MarcReader<std::fs::File> {
    /// Open `path` for reading and create a [`MarcReader`] whose errors
    /// include the path as their `source_name`.
    ///
    /// # Errors
    ///
    /// Returns the underlying [`std::io::Error`] if the file cannot be opened.
    pub fn from_path(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        let file = std::fs::File::open(path)?;
        Ok(Self::new(file).with_source(path.display().to_string()))
    }
}

impl<R: Read> MarcReader<R> {
    /// Read a single MARC record.
    ///
    /// Returns `Ok(Some(record))` if a record was successfully read, `Ok(None)` if EOF
    /// was reached, or `Err` if a parsing error occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use mrrc::MarcReader;
    /// use std::io::Cursor;
    ///
    /// # let data = vec![];
    /// # let cursor = Cursor::new(data);
    /// let mut reader = MarcReader::new(cursor);
    ///
    /// match reader.read_record() {
    ///     Ok(Some(record)) => { /* process record */ },
    ///     Ok(None) => println!("End of file"),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The binary data is malformed
    /// - The record structure is invalid
    /// - An I/O error occurs
    #[allow(clippy::too_many_lines, clippy::redundant_else)]
    pub fn read_record(&mut self) -> Result<Option<Record>> {
        // Read the leader (24 bytes). EOF here is a clean stream end.
        let Some(leader_bytes) = iso2709::read_leader_bytes(&mut self.reader)? else {
            return Ok(None);
        };

        // We have a record — initialize the per-record positional context.
        self.ctx.begin_record();

        let leader = Leader::from_bytes(&leader_bytes)?;
        leader.validate_for_reading()?;

        // Calculate the size of the directory and data areas
        let record_length = leader.record_length as usize;
        let base_address = leader.data_base_address as usize;

        // Directory starts after leader, ends at base_address
        let directory_size = base_address - 24;

        // The leader has been consumed.
        self.ctx.advance(LEADER_LEN);

        // Read the full record data. In non-Strict modes a short read returns
        // `(buffer, true)`; salvage from a partial buffer is not implemented,
        // so the recovery dispatch below is unreachable in practice (the read
        // primitive returns a buffer of full length even on a short read).
        let (record_data, _was_truncated) = iso2709::read_record_data(
            &mut self.reader,
            record_length,
            self.recovery_mode,
            &self.ctx,
        )?;

        // Hand the loaded record buffer to the context so `err_*` helpers
        // raised during directory/field parsing can capture a byte-window
        // for hex-dump rendering. The record data starts at the stream
        // offset just past the leader.
        let record_data_offset = self.ctx.stream_byte_offset;
        self.ctx.set_parse_buffer(&record_data, record_data_offset);

        if record_data.len() < (record_length - 24) && self.recovery_mode != RecoveryMode::Strict {
            return crate::recovery::try_recover_record(
                leader,
                &record_data,
                base_address,
                self.recovery_mode,
                &self.ctx,
            )
            .map(Some);
        }

        // Parse directory
        let directory_end = std::cmp::min(directory_size, record_data.len());
        let directory = if directory_end > 0 {
            &record_data[..directory_end]
        } else {
            &[]
        };
        let data_start = std::cmp::min(base_address - 24, record_data.len());
        let data = if data_start < record_data.len() {
            &record_data[data_start..]
        } else {
            &[]
        };

        let mut record = Record::new(leader);

        // Parse directory entries (12 bytes each: tag(3) + length(4) + start position(5))
        // Directory is terminated with FIELD_TERMINATOR
        let mut pos = 0;
        while pos < directory.len() {
            if directory[pos] == FIELD_TERMINATOR {
                // End of directory
                break;
            }

            if pos + 12 > directory.len() {
                if self.recovery_mode == RecoveryMode::Strict {
                    return Err(self.ctx.err_directory_invalid(
                        Some(&directory[pos..]),
                        "complete 12-byte directory entry",
                    ));
                } else {
                    break;
                }
            }

            let entry_chunk = &directory[pos..pos + 12];
            let tag = String::from_utf8_lossy(&entry_chunk[0..3]).to_string();
            let field_length = match iso2709::parse_4digits(&entry_chunk[3..7]) {
                Ok(len) => len,
                Err(e) => {
                    if self.recovery_mode == RecoveryMode::Strict {
                        return Err(e);
                    } else {
                        pos += 12;
                        continue;
                    }
                },
            };
            let start_position = match iso2709::parse_5digits(&entry_chunk[7..12]) {
                Ok(pos) => pos,
                Err(e) => {
                    if self.recovery_mode == RecoveryMode::Strict {
                        return Err(e);
                    } else {
                        pos += 12;
                        continue;
                    }
                },
            };
            pos += 12;

            let end_position = start_position + field_length;
            if end_position > data.len() {
                if self.recovery_mode == RecoveryMode::Strict {
                    self.ctx.current_field_tag = Some(tag.clone());
                    return Err(self.ctx.err_invalid_field(format!(
                        "Field {tag} exceeds data area (end {end_position} > {})",
                        data.len()
                    )));
                } else {
                    // Try to read what we have
                    let available_end = std::cmp::min(end_position, data.len());
                    if available_end > start_position {
                        let field_data = &data[start_position..available_end];
                        if tag != "LDR" {
                            if iso2709::is_control_field_tag(&tag) {
                                let value = String::from_utf8_lossy(
                                    &field_data[..field_data.len().saturating_sub(1)],
                                )
                                .to_string();
                                if tag == "001" {
                                    self.ctx.record_control_number = Some(value.clone());
                                }
                                record.add_control_field(tag, value);
                            } else {
                                self.ctx.current_field_tag = Some(tag.clone());
                                if let Ok(field) = iso2709::parse_data_field(
                                    field_data,
                                    &tag,
                                    DataFieldParseConfig::BIBLIOGRAPHIC,
                                    &self.ctx,
                                ) {
                                    record.add_field(field);
                                }
                                self.ctx.current_field_tag = None;
                            }
                        }
                    }
                    continue;
                }
            }

            let field_data = &data[start_position..end_position];

            // Parse field (skip LDR as it's already parsed)
            if tag != "LDR" {
                if iso2709::is_control_field_tag(&tag) {
                    // Control field (001-009): strip the trailing field terminator
                    let value =
                        String::from_utf8_lossy(&field_data[..field_data.len().saturating_sub(1)])
                            .to_string();
                    // Opportunistic 001 capture for downstream error context.
                    if tag == "001" {
                        self.ctx.record_control_number = Some(value.clone());
                    }
                    record.add_control_field(tag, value);
                } else {
                    // Data field (010+)
                    self.ctx.current_field_tag = Some(tag.clone());
                    let parsed = iso2709::parse_data_field(
                        field_data,
                        &tag,
                        DataFieldParseConfig::BIBLIOGRAPHIC,
                        &self.ctx,
                    );
                    self.ctx.current_field_tag = None;
                    match parsed {
                        Ok(field) => record.add_field(field),
                        Err(e) => {
                            if self.recovery_mode == RecoveryMode::Strict {
                                return Err(e);
                            }
                            // In lenient/permissive mode, skip this field and continue
                        },
                    }
                }
            }
        }

        // Advance past the rest of the record (everything after the leader).
        self.ctx.advance(record_length.saturating_sub(LEADER_LEN));
        self.records_read += 1;
        Ok(Some(record))
    }
}

// Implement the FormatReader trait for MarcReader
impl<R: Read + std::fmt::Debug> FormatReader for MarcReader<R> {
    fn read_record(&mut self) -> Result<Option<Record>> {
        // Delegate to the existing implementation
        MarcReader::read_record(self)
    }

    fn records_read(&self) -> Option<usize> {
        Some(self.records_read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    use crate::iso2709::{RECORD_TERMINATOR, SUBFIELD_DELIMITER};

    #[test]
    fn test_read_simple_record() {
        // Manually build a valid MARC record
        let mut record_bytes = Vec::new();

        // Data area: field 245
        let mut field_245 = Vec::new();
        field_245.extend_from_slice(b"10"); // Indicators
        field_245.push(SUBFIELD_DELIMITER);
        field_245.push(b'a');
        field_245.extend_from_slice(b"Test title");
        field_245.push(FIELD_TERMINATOR);

        // Directory (without terminator yet)
        let mut directory = Vec::new();
        directory.extend_from_slice(b"245");
        directory.extend_from_slice(format!("{:04}", field_245.len()).as_bytes());
        directory.extend_from_slice(b"00000");

        // Base address is after leader + directory + directory terminator
        let base_address = 24 + directory.len() + 1; // +1 for directory terminator
        directory.push(FIELD_TERMINATOR);
        let record_length = base_address + field_245.len() + 1;

        // Leader (must be exactly 24 bytes)
        let mut leader = Vec::new();
        leader.extend_from_slice(format!("{record_length:05}").as_bytes()); // 0-4
        leader.push(b'n'); // 5: status
        leader.push(b'a'); // 6: type
        leader.push(b'm'); // 7: bib level
        leader.push(b' '); // 8: control type
        leader.push(b'a'); // 9: character coding
        leader.push(b'2'); // 10: indicator count
        leader.push(b'2'); // 11: subfield code count
        leader.extend_from_slice(format!("{base_address:05}").as_bytes()); // 12-16
        leader.push(b' '); // 17: encoding level
        leader.push(b' '); // 18: cataloging form
        leader.push(b' '); // 19: multipart level
        leader.extend_from_slice(b"4500"); // 20-23: reserved

        // Assemble
        record_bytes.extend_from_slice(&leader);
        record_bytes.extend_from_slice(&directory);
        record_bytes.extend_from_slice(&field_245);
        record_bytes.push(RECORD_TERMINATOR);

        let cursor = Cursor::new(record_bytes);
        let mut reader = MarcReader::new(cursor);

        let record = reader.read_record().unwrap().unwrap();

        assert_eq!(record.leader.record_type, 'a');
        let fields = record.get_fields("245");
        assert!(fields.is_some());
        let field = &fields.unwrap()[0];
        assert_eq!(field.indicator1, '1');
        assert_eq!(field.indicator2, '0');

        let title = field.get_subfield('a');
        assert_eq!(title, Some("Test title"));
    }

    #[test]
    fn test_eof_returns_none() {
        let data = vec![];
        let cursor = Cursor::new(data);
        let mut reader = MarcReader::new(cursor);

        let result = reader.read_record().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_read_multiple_records() {
        // Build two records
        let mut all_bytes = Vec::new();

        for _ in 0..2 {
            let mut field_245 = Vec::new();
            field_245.extend_from_slice(b"10");
            field_245.push(SUBFIELD_DELIMITER);
            field_245.push(b'a');
            field_245.extend_from_slice(b"Test title");
            field_245.push(FIELD_TERMINATOR);

            let mut directory = Vec::new();
            directory.extend_from_slice(b"245");
            directory.extend_from_slice(format!("{:04}", field_245.len()).as_bytes());
            directory.extend_from_slice(b"00000");

            let base_address = 24 + directory.len() + 1;
            directory.push(FIELD_TERMINATOR);
            let record_length = base_address + field_245.len() + 1;

            let mut leader = Vec::new();
            leader.extend_from_slice(format!("{record_length:05}").as_bytes()); // 0-4
            leader.push(b'n'); // 5
            leader.push(b'a'); // 6
            leader.push(b'm'); // 7
            leader.push(b' '); // 8
            leader.push(b'a'); // 9
            leader.push(b'2'); // 10
            leader.push(b'2'); // 11
            leader.extend_from_slice(format!("{base_address:05}").as_bytes()); // 12-16
            leader.push(b' '); // 17
            leader.push(b' '); // 18
            leader.push(b' '); // 19
            leader.extend_from_slice(b"4500"); // 20-23

            all_bytes.extend_from_slice(&leader);
            all_bytes.extend_from_slice(&directory);
            all_bytes.extend_from_slice(&field_245);
            all_bytes.push(RECORD_TERMINATOR);
        }

        let cursor = Cursor::new(all_bytes);
        let mut reader = MarcReader::new(cursor);

        let record1 = reader.read_record().unwrap();
        assert!(record1.is_some());

        let record2 = reader.read_record().unwrap();
        assert!(record2.is_some());

        let record3 = reader.read_record().unwrap();
        assert!(record3.is_none());
    }

    #[test]
    fn test_format_reader_trait() {
        // Build two records
        let mut all_bytes = Vec::new();

        for _ in 0..2 {
            let mut field_245 = Vec::new();
            field_245.extend_from_slice(b"10");
            field_245.push(SUBFIELD_DELIMITER);
            field_245.push(b'a');
            field_245.extend_from_slice(b"Test title");
            field_245.push(FIELD_TERMINATOR);

            let mut directory = Vec::new();
            directory.extend_from_slice(b"245");
            directory.extend_from_slice(format!("{:04}", field_245.len()).as_bytes());
            directory.extend_from_slice(b"00000");

            let base_address = 24 + directory.len() + 1;
            directory.push(FIELD_TERMINATOR);
            let record_length = base_address + field_245.len() + 1;

            let mut leader = Vec::new();
            leader.extend_from_slice(format!("{record_length:05}").as_bytes());
            leader.push(b'n');
            leader.push(b'a');
            leader.push(b'm');
            leader.push(b' ');
            leader.push(b'a');
            leader.push(b'2');
            leader.push(b'2');
            leader.extend_from_slice(format!("{base_address:05}").as_bytes());
            leader.push(b' ');
            leader.push(b' ');
            leader.push(b' ');
            leader.extend_from_slice(b"4500");

            all_bytes.extend_from_slice(&leader);
            all_bytes.extend_from_slice(&directory);
            all_bytes.extend_from_slice(&field_245);
            all_bytes.push(RECORD_TERMINATOR);
        }

        let cursor = Cursor::new(all_bytes);
        let mut reader = MarcReader::new(cursor);

        // Verify records_read starts at 0
        assert_eq!(reader.records_read(), Some(0));

        // Use the FormatReader trait method read_all
        let records = FormatReader::read_all(&mut reader).unwrap();
        assert_eq!(records.len(), 2);

        // Verify records_read counter
        assert_eq!(reader.records_read(), Some(2));
    }

    #[test]
    fn test_format_reader_iterator() {
        use crate::formats::FormatReaderExt;

        // Build two records
        let mut all_bytes = Vec::new();

        for _ in 0..3 {
            let mut field_245 = Vec::new();
            field_245.extend_from_slice(b"10");
            field_245.push(SUBFIELD_DELIMITER);
            field_245.push(b'a');
            field_245.extend_from_slice(b"Test title");
            field_245.push(FIELD_TERMINATOR);

            let mut directory = Vec::new();
            directory.extend_from_slice(b"245");
            directory.extend_from_slice(format!("{:04}", field_245.len()).as_bytes());
            directory.extend_from_slice(b"00000");

            let base_address = 24 + directory.len() + 1;
            directory.push(FIELD_TERMINATOR);
            let record_length = base_address + field_245.len() + 1;

            let mut leader = Vec::new();
            leader.extend_from_slice(format!("{record_length:05}").as_bytes());
            leader.push(b'n');
            leader.push(b'a');
            leader.push(b'm');
            leader.push(b' ');
            leader.push(b'a');
            leader.push(b'2');
            leader.push(b'2');
            leader.extend_from_slice(format!("{base_address:05}").as_bytes());
            leader.push(b' ');
            leader.push(b' ');
            leader.push(b' ');
            leader.extend_from_slice(b"4500");

            all_bytes.extend_from_slice(&leader);
            all_bytes.extend_from_slice(&directory);
            all_bytes.extend_from_slice(&field_245);
            all_bytes.push(RECORD_TERMINATOR);
        }

        let cursor = Cursor::new(all_bytes);
        let mut reader = MarcReader::new(cursor);

        // Use the FormatReaderExt iterator
        let mut count = 0;
        for result in reader.records() {
            result.unwrap();
            count += 1;
        }
        assert_eq!(count, 3);
        assert_eq!(reader.records_read(), Some(3));
    }

    #[test]
    fn test_malformed_leader_record_length_too_small() {
        // Build a 24-byte leader where record_length (bytes 0-4) = 00010 (< 24)
        let leader = b"00010nam a2200025 i 4500";
        let cursor = Cursor::new(leader.to_vec());
        let mut reader = MarcReader::new(cursor);
        let result = reader.read_record();
        assert!(result.is_err(), "expected Err for record_length < 24");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Record length must be at least 24"),
            "got: {err}"
        );
    }

    #[test]
    fn test_malformed_leader_base_address_too_small() {
        // Build a 24-byte leader where base_address (bytes 12-16) = 00010 (< 24)
        let leader = b"00050nam a2200010 i 4500";
        let cursor = Cursor::new(leader.to_vec());
        let mut reader = MarcReader::new(cursor);
        let result = reader.read_record();
        assert!(result.is_err(), "expected Err for base_address < 24");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Base address of data must be at least 24"),
            "got: {err}"
        );
    }
}
