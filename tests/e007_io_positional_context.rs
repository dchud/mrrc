//! Regression coverage for E007 (`MarcError::IoError`) positional context.
//!
//! When the underlying `Read` source fails partway through a record's data
//! area, the surfaced `IoError` must carry the positional context of the
//! record being read — `record_index`, `byte_offset`, and `source_name` —
//! rather than the context-free `From<std::io::Error>` fallback. Without
//! this, a failure midway through a multi-record stream surfaces as a bare
//! "IO error" with no indication of which record or byte position failed.

use std::io::{self, Read};

use mrrc::{MarcError, MarcReader};

/// Build a minimal structurally valid ISO 2709 record carrying a single
/// `001` control field with the given value. Mirrors the leader shape used
/// across the test suite (`nam a22 ... i 4500`) with a correct directory.
fn build_valid_record(control_001: &str) -> Vec<u8> {
    const FIELD_TERMINATOR: u8 = 0x1E;
    const RECORD_TERMINATOR: u8 = 0x1D;

    let mut field_data = Vec::new();
    field_data.extend_from_slice(control_001.as_bytes());
    field_data.push(FIELD_TERMINATOR);

    let mut directory = Vec::new();
    directory.extend_from_slice(b"001");
    directory.extend_from_slice(format!("{:04}", field_data.len()).as_bytes());
    directory.extend_from_slice(b"00000");
    directory.push(FIELD_TERMINATOR);

    let base_address = 24 + directory.len();
    let record_length = base_address + field_data.len() + 1;

    let mut leader = Vec::new();
    leader.extend_from_slice(format!("{record_length:05}").as_bytes());
    leader.extend_from_slice(b"nam a22");
    leader.extend_from_slice(format!("{base_address:05}").as_bytes());
    leader.extend_from_slice(b" i 4500");

    let mut out = Vec::new();
    out.extend_from_slice(&leader);
    out.extend_from_slice(&directory);
    out.extend_from_slice(&field_data);
    out.push(RECORD_TERMINATOR);
    out
}

/// `Read` impl that serves a prepared byte buffer and then, once the buffer
/// is exhausted, returns an `io::Error` instead of a clean `Ok(0)` EOF. This
/// drives the parser into the underlying-read-failure path of
/// `read_record_data` rather than the clean end-of-stream path.
struct FailAfterBuffer {
    data: Vec<u8>,
    pos: usize,
}

impl Read for FailAfterBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.data.len() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "synthetic mid-record read failure",
            ));
        }
        let n = std::cmp::min(buf.len(), self.data.len() - self.pos);
        buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

#[test]
fn io_error_midstream_carries_record_index_and_source() {
    // A complete first record, followed by only the 24-byte leader of a
    // second record. The reader serves all of that, then errors when the
    // parser tries to read record 2's data area.
    let rec1 = build_valid_record("rec1");
    let rec2 = build_valid_record("rec2");
    let rec1_len = rec1.len();

    let mut stream = rec1;
    stream.extend_from_slice(&rec2[..24]); // leader of record 2 only

    let reader_src = FailAfterBuffer {
        data: stream,
        pos: 0,
    };
    let mut reader = MarcReader::new(reader_src).with_source("synthetic-stream.mrc");

    // Record 1 parses cleanly.
    let first = reader.read_record().expect("record 1 should parse");
    assert!(first.is_some(), "record 1 should be present");

    // Record 2's data read fails: the IoError must carry positional context.
    let err = reader
        .read_record()
        .expect_err("record 2's data read should surface an IoError");

    match err {
        MarcError::IoError {
            record_index,
            byte_offset,
            source_name,
            ..
        } => {
            assert_eq!(
                record_index,
                Some(2),
                "IoError should be attributed to the failing record (record 2)"
            );
            assert_eq!(
                source_name.as_deref(),
                Some("synthetic-stream.mrc"),
                "IoError should carry the reader's source name"
            );
            assert!(
                byte_offset.is_some_and(|off| off >= rec1_len),
                "IoError byte_offset should point past record 1 (got {byte_offset:?}, rec1 len {rec1_len})"
            );
        },
        other => panic!("expected MarcError::IoError, got {other:?}"),
    }
}

/// `Read` impl that errors on the very first read, before any leader bytes
/// are delivered.
struct FailOnFirstRead;

impl Read for FailOnFirstRead {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "synthetic boundary read failure",
        ))
    }
}

#[test]
fn io_error_at_leader_boundary_uses_contextfree_fallback() {
    // A read failure at the record boundary (the leader read, before
    // `begin_record` runs) cannot be attributed to a specific in-progress
    // record, so it surfaces via the context-free `From<io::Error>`
    // fallback: still an IoError, but with no `record_index`.
    let mut reader = MarcReader::new(FailOnFirstRead).with_source("synthetic-stream.mrc");

    let err = reader
        .read_record()
        .expect_err("a boundary read failure should surface an IoError");

    match err {
        MarcError::IoError { record_index, .. } => {
            assert_eq!(
                record_index, None,
                "a boundary-read IoError has no in-progress record to attribute to"
            );
        },
        other => panic!("expected MarcError::IoError, got {other:?}"),
    }
}
