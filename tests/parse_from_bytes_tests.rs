//! Equivalence of the slice-based parse entry point with the reader path.
//!
//! `parse_record_from_bytes` exists so callers that already hold a
//! complete record's bytes (the Python bindings, batch pipelines) can
//! parse without reader I/O or per-record copies. These tests pin that it
//! yields exactly what `MarcReader` yields for the same bytes — including
//! error behavior on truncated input.

use mrrc::{
    MarcReader, RecoveryMode, ValidationLevel, parse_record_from_bytes,
    parse_record_from_shared_bytes,
};
use std::io::Cursor;
use std::sync::Arc;

fn fixture_1k() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/fixtures/1k_records.mrc"
    );
    std::fs::read(path).expect("1k_records.mrc fixture must exist")
}

/// Split a stream of complete records into per-record byte chunks using
/// the leader's 5-digit record length.
fn split_records(stream: &[u8]) -> Vec<Vec<u8>> {
    let mut records = Vec::new();
    let mut pos = 0;
    while pos + 24 <= stream.len() {
        let len: usize = std::str::from_utf8(&stream[pos..pos + 5])
            .expect("record length must be ASCII")
            .parse()
            .expect("record length must be digits");
        records.push(stream[pos..pos + len].to_vec());
        pos += len;
    }
    assert_eq!(pos, stream.len(), "stream must split exactly");
    records
}

#[test]
fn slice_parse_matches_reader_parse_over_the_corpus() {
    let stream = fixture_1k();
    let mut reader = MarcReader::new(Cursor::new(stream.clone()));

    let mut count = 0;
    for record_bytes in split_records(&stream) {
        let from_reader = reader
            .read_record()
            .expect("reader parse")
            .expect("reader record");
        let from_slice = parse_record_from_bytes(
            record_bytes,
            RecoveryMode::Strict,
            ValidationLevel::Structural,
        )
        .expect("slice parse")
        .expect("slice record");
        assert_eq!(
            format!("{from_reader:?}"),
            format!("{from_slice:?}"),
            "record {count} diverged between reader and slice parse"
        );
        count += 1;
    }
    assert_eq!(count, 1000);
}

#[test]
fn shared_bytes_parse_matches_owned_bytes_parse_over_the_corpus() {
    // `parse_record_from_shared_bytes` is the zero-copy entry point the
    // Python bindings use so the `current_chunk` stash can share one
    // allocation with the parser. It must yield exactly what the owned-bytes
    // entry point yields for the same bytes.
    let stream = fixture_1k();
    let mut count = 0;
    for record_bytes in split_records(&stream) {
        let shared = Arc::new(record_bytes.clone());
        let from_owned = parse_record_from_bytes(
            record_bytes,
            RecoveryMode::Strict,
            ValidationLevel::Structural,
        )
        .expect("owned parse")
        .expect("owned record");
        let from_shared = parse_record_from_shared_bytes(
            &shared,
            RecoveryMode::Strict,
            ValidationLevel::Structural,
        )
        .expect("shared parse")
        .expect("shared record");
        assert_eq!(
            format!("{from_owned:?}"),
            format!("{from_shared:?}"),
            "record {count} diverged between owned and shared parse"
        );
        // The caller still holds the buffer after parsing — the whole point
        // of the shared entry point.
        assert!(!shared.is_empty());
        count += 1;
    }
    assert_eq!(count, 1000);
}

#[test]
fn empty_bytes_yield_none() {
    let result = parse_record_from_bytes(
        Vec::new(),
        RecoveryMode::Strict,
        ValidationLevel::Structural,
    )
    .expect("empty parse is not an error");
    assert!(result.is_none());
}

#[test]
fn truncated_record_errors_in_strict_like_the_reader() {
    let stream = fixture_1k();
    let first = split_records(&stream).remove(0);
    let truncated = first[..first.len() - 10].to_vec();

    let slice_err = parse_record_from_bytes(
        truncated.clone(),
        RecoveryMode::Strict,
        ValidationLevel::Structural,
    )
    .expect_err("strict truncated parse must error");
    let mut reader = MarcReader::new(Cursor::new(truncated));
    let reader_err = reader
        .read_record()
        .expect_err("strict truncated read must error");

    // Same error variant (E005 TruncatedRecord) from both entry points.
    assert_eq!(slice_err.code(), reader_err.code());
}

#[test]
fn truncated_record_recovers_in_permissive_like_the_reader() {
    let stream = fixture_1k();
    let first = split_records(&stream).remove(0);
    let truncated = first[..first.len() - 10].to_vec();

    let from_slice = parse_record_from_bytes(
        truncated.clone(),
        RecoveryMode::Permissive,
        ValidationLevel::Structural,
    )
    .expect("permissive truncated parse");
    let mut reader =
        MarcReader::new(Cursor::new(truncated)).with_recovery_mode(RecoveryMode::Permissive);
    let from_reader = reader.read_record().expect("permissive truncated read");

    match (from_slice, from_reader) {
        (Some(a), Some(b)) => {
            assert_eq!(format!("{a:?}"), format!("{b:?}"));
            assert!(!a.errors.is_empty(), "truncation must be diagnosed");
        },
        (a, b) => panic!(
            "entry points diverged on truncated permissive parse: slice={:?} reader={:?}",
            a.map(|_| "record"),
            b.map(|_| "record")
        ),
    }
}
