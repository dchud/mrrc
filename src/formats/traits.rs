//! Format reader and writer traits for MARC records.
//!
//! This module defines the core traits that all format implementations must implement,
//! providing a uniform interface for reading and writing MARC records regardless of
//! the underlying serialization format.
//!
//! # Design Rationale
//!
//! The traits are designed to:
//! - Support both streaming (one record at a time) and batch operations
//! - Work with the existing `Record` type
//! - Allow format-specific optimizations while maintaining API consistency
//! - Be object-safe for dynamic dispatch when needed
//!
//! # Example
//!
//! ```ignore
//! use mrrc::formats::{FormatReader, FormatWriter};
//! use mrrc::Record;
//!
//! fn process_records<R: FormatReader, W: FormatWriter>(
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

use crate::error::Result;
use crate::record::Record;

/// Trait for readers that can produce MARC records from a source.
///
/// This trait abstracts over different serialization formats, allowing
/// uniform access to MARC records regardless of the underlying format.
///
/// # Streaming vs Batch
///
/// The trait supports both streaming and batch reading patterns:
/// - Use [`read_record`](Self::read_record) for streaming one record at a time
/// - Use [`read_all`](Self::read_all) to read all records into memory
///
/// # Implementation Notes
///
/// Implementations should:
/// - Return `Ok(None)` when the source is exhausted (not an error)
/// - Preserve field ordering and all record metadata exactly
/// - Handle encoding consistently (typically UTF-8 normalized)
pub trait FormatReader: std::fmt::Debug {
    /// Read the next record from the source.
    ///
    /// Returns:
    /// - `Ok(Some(record))` if a record was read successfully
    /// - `Ok(None)` if the end of the source was reached
    /// - `Err(_)` if reading failed due to malformed data or I/O errors
    ///
    /// # Errors
    ///
    /// Returns an error if the source contains malformed data or I/O fails.
    ///
    /// # Fidelity Requirements
    ///
    /// The returned record MUST preserve:
    /// - Exact field ordering (fields appear in same sequence as source)
    /// - Exact subfield ordering within each field
    /// - All indicator values (including blank indicators)
    /// - All whitespace in field/subfield values
    /// - Leader data exactly as encoded in the source format
    fn read_record(&mut self) -> Result<Option<Record>>;

    /// Read all remaining records into a vector.
    ///
    /// This is a convenience method that repeatedly calls [`read_record`](Self::read_record)
    /// until the source is exhausted. For large files, prefer streaming with
    /// `read_record` to avoid memory pressure.
    ///
    /// # Errors
    ///
    /// Returns an error if any record fails to read. On error, previously
    /// read records are discarded.
    fn read_all(&mut self) -> Result<Vec<Record>> {
        let mut records = Vec::new();
        while let Some(record) = self.read_record()? {
            records.push(record);
        }
        Ok(records)
    }

    /// Returns the number of records read so far.
    ///
    /// This is useful for progress reporting and debugging.
    /// The default implementation returns `None` if tracking is not supported.
    fn records_read(&self) -> Option<usize> {
        None
    }
}

/// Trait for writers that can serialize MARC records to a format.
///
/// This trait provides a uniform interface for writing MARC records to
/// different serialization formats. All format-specific writers implement
/// this trait.
///
/// # Usage Pattern
///
/// Writers follow a standard pattern:
/// 1. Create the writer with format-specific configuration
/// 2. Write records using [`write_record`](Self::write_record) or [`write_batch`](Self::write_batch)
/// 3. Call [`finish`](Self::finish) to flush and finalize output
///
/// # Important: Always Call `finish`
///
/// The [`finish`](Self::finish) method MUST be called to ensure all data is written.
/// Some formats buffer data for efficiency and only write on finish.
/// Dropping a writer without calling `finish` may result in data loss.
///
/// # Example
///
/// ```ignore
/// use mrrc::formats::FormatWriter;
///
/// fn write_records<W: FormatWriter>(writer: &mut W, records: &[Record]) -> mrrc::Result<()> {
///     writer.write_batch(records)?;
///     writer.finish()?; // Always call finish!
///     Ok(())
/// }
/// ```
pub trait FormatWriter: std::fmt::Debug {
    /// Write a single record to the output.
    ///
    /// # Fidelity Requirements
    ///
    /// The written record MUST preserve:
    /// - Exact field ordering (fields written in same sequence as input)
    /// - Exact subfield ordering within each field
    /// - All indicator values (including blank indicators)
    /// - All whitespace in field/subfield values
    /// - Leader data exactly as provided
    ///
    /// # Errors
    ///
    /// Returns an error if the record cannot be serialized (e.g., invalid
    /// structure) or if writing to the underlying output fails.
    fn write_record(&mut self, record: &Record) -> Result<()>;

    /// Write multiple records to the output.
    ///
    /// This method may be more efficient than calling `write_record` repeatedly
    /// for formats that benefit from batch operations.
    ///
    /// The default implementation calls `write_record` for each record.
    ///
    /// # Errors
    ///
    /// Returns an error if any record cannot be written.
    fn write_batch(&mut self, records: &[Record]) -> Result<()> {
        for record in records {
            self.write_record(record)?;
        }
        Ok(())
    }

    /// Finish writing and flush any buffered data.
    ///
    /// This method MUST be called to ensure all data is written to the output.
    /// After calling `finish`, the writer should not be used for further writes.
    ///
    /// # Errors
    ///
    /// Returns an error if flushing fails or if the underlying output
    /// cannot be finalized (e.g., network error, disk full).
    fn finish(&mut self) -> Result<()>;

    /// Returns the number of records written so far.
    ///
    /// This is useful for progress reporting and debugging.
    /// The default implementation returns `None` if tracking is not supported.
    fn records_written(&self) -> Option<usize> {
        None
    }
}

/// Extension trait providing iterator-style access for format readers.
///
/// This trait is automatically implemented for all types implementing [`FormatReader`].
pub trait FormatReaderExt: FormatReader {
    /// Create an iterator over records from this reader.
    ///
    /// The iterator yields `Result<Record>` for each record, allowing
    /// error handling during iteration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use mrrc::formats::{FormatReader, FormatReaderExt};
    ///
    /// fn count_records<R: FormatReader>(mut reader: R) -> mrrc::Result<usize> {
    ///     let mut count = 0;
    ///     for result in reader.records() {
    ///         let _record = result?;
    ///         count += 1;
    ///     }
    ///     Ok(count)
    /// }
    /// ```
    fn records(&mut self) -> RecordIterator<'_, Self>
    where
        Self: Sized,
    {
        RecordIterator { reader: self }
    }
}

impl<T: FormatReader> FormatReaderExt for T {}

/// Iterator adapter for [`FormatReader`].
///
/// Created by the [`records`](FormatReaderExt::records) method.
#[derive(Debug)]
pub struct RecordIterator<'a, R: FormatReader> {
    reader: &'a mut R,
}

impl<R: FormatReader> Iterator for RecordIterator<'_, R> {
    type Item = Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.read_record() {
            Ok(Some(record)) => Some(Ok(record)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Leader;

    /// Create a test leader with valid default values for testing.
    fn test_leader() -> Leader {
        // Standard 24-byte leader for a language material book record
        // "00000nam a2200000 i 4500"
        Leader::from_bytes(b"00000nam a2200000 i 4500").unwrap()
    }

    /// Mock reader for testing trait implementations
    #[derive(Debug)]
    struct MockReader {
        records: Vec<Record>,
        index: usize,
    }

    impl MockReader {
        fn new(records: Vec<Record>) -> Self {
            Self { records, index: 0 }
        }
    }

    impl FormatReader for MockReader {
        fn read_record(&mut self) -> Result<Option<Record>> {
            if self.index < self.records.len() {
                let record = self.records[self.index].clone();
                self.index += 1;
                Ok(Some(record))
            } else {
                Ok(None)
            }
        }

        fn records_read(&self) -> Option<usize> {
            Some(self.index)
        }
    }

    /// Mock writer for testing trait implementations
    #[derive(Debug)]
    struct MockWriter {
        records: Vec<Record>,
        finished: bool,
    }

    impl MockWriter {
        fn new() -> Self {
            Self {
                records: Vec::new(),
                finished: false,
            }
        }
    }

    impl FormatWriter for MockWriter {
        fn write_record(&mut self, record: &Record) -> Result<()> {
            if self.finished {
                return Err(crate::MarcError::InvalidRecord(
                    "Writer already finished".to_string(),
                ));
            }
            self.records.push(record.clone());
            Ok(())
        }

        fn finish(&mut self) -> Result<()> {
            self.finished = true;
            Ok(())
        }

        fn records_written(&self) -> Option<usize> {
            Some(self.records.len())
        }
    }

    #[test]
    fn test_reader_read_all() {
        let records = vec![
            Record::new(test_leader()),
            Record::new(test_leader()),
            Record::new(test_leader()),
        ];
        let mut reader = MockReader::new(records.clone());

        let result = reader.read_all().unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(reader.records_read(), Some(3));
    }

    #[test]
    fn test_reader_empty() {
        let mut reader = MockReader::new(vec![]);

        let result = reader.read_all().unwrap();
        assert!(result.is_empty());
        assert_eq!(reader.records_read(), Some(0));
    }

    #[test]
    fn test_reader_iterator() {
        let records = vec![Record::new(test_leader()), Record::new(test_leader())];
        let mut reader = MockReader::new(records);

        let mut count = 0;
        for result in reader.records() {
            result.unwrap();
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_writer_batch() {
        let records = vec![
            Record::new(test_leader()),
            Record::new(test_leader()),
            Record::new(test_leader()),
        ];
        let mut writer = MockWriter::new();

        writer.write_batch(&records).unwrap();
        assert_eq!(writer.records_written(), Some(3));

        writer.finish().unwrap();
        assert!(writer.finished);
    }

    #[test]
    fn test_writer_single_records() {
        let mut writer = MockWriter::new();

        writer.write_record(&Record::new(test_leader())).unwrap();
        writer.write_record(&Record::new(test_leader())).unwrap();
        assert_eq!(writer.records_written(), Some(2));

        writer.finish().unwrap();
    }

    #[test]
    fn test_writer_cannot_write_after_finish() {
        let mut writer = MockWriter::new();
        writer.finish().unwrap();

        let result = writer.write_record(&Record::new(test_leader()));
        assert!(result.is_err());
    }
}
