//! Memory-safety workload for the `AddressSanitizer` run.
//!
//! These tests exercise the library's real allocation patterns — parsing,
//! recovery from malformed input, multi-threaded reads of shared buffers,
//! and writer round-trips for all three record types — so that a build
//! with `RUSTFLAGS="-Z sanitizer=address"` observes actual mrrc code
//! paths rather than stdlib behavior. Under a regular (non-ASAN) build
//! they still run as ordinary integration tests.

use std::io::Cursor;
use std::sync::Arc;
use std::thread;

use mrrc::{
    AuthorityMarcReader, AuthorityMarcWriter, HoldingsMarcReader, HoldingsMarcWriter, MarcReader,
    MarcWriter, RecoveryMode,
};

const SIMPLE_BOOK: &[u8] = include_bytes!("data/simple_book.mrc");
const SIMPLE_AUTHORITY: &[u8] = include_bytes!("data/simple_authority.mrc");
const SIMPLE_HOLDINGS: &[u8] = include_bytes!("data/simple_holdings.mrc");

/// Parse a bibliographic fixture and walk every field and subfield so the
/// sanitizer sees the full per-record allocation and access pattern.
#[test]
fn parse_and_traverse_bibliographic_record() {
    let mut reader = MarcReader::new(Cursor::new(SIMPLE_BOOK));
    let record = reader
        .read_record()
        .expect("read should succeed")
        .expect("fixture contains one record");

    assert_eq!(record.leader.record_type, 'a');
    let mut subfield_count = 0;
    for field in record.fields() {
        assert_eq!(field.tag.len(), 3);
        for subfield in &field.subfields {
            assert!(subfield.code.is_ascii_graphic());
            subfield_count += 1;
        }
    }
    assert!(subfield_count > 0, "fixture has at least one subfield");
    assert!(reader.read_record().expect("read").is_none());
}

/// Round-trip a multi-record stream: parse, re-serialize, re-parse. The
/// writer and parser exchange ownership of buffers; ASAN verifies no
/// use-after-free or out-of-bounds access along the way.
#[test]
fn bibliographic_write_read_roundtrip_stream() {
    let mut stream = Vec::new();
    for _ in 0..50 {
        stream.extend_from_slice(SIMPLE_BOOK);
    }

    let mut reader = MarcReader::new(Cursor::new(&stream[..]));
    let mut buffer = Vec::new();
    let mut count = 0;
    {
        let mut writer = MarcWriter::new(&mut buffer);
        while let Some(record) = reader.read_record().expect("read should succeed") {
            writer.write_record(&record).expect("write should succeed");
            count += 1;
        }
    }
    assert_eq!(count, 50);

    let mut reread = MarcReader::new(Cursor::new(&buffer[..]));
    let mut reread_count = 0;
    while let Some(record) = reread.read_record().expect("re-read should succeed") {
        assert_eq!(record.leader.record_type, 'a');
        reread_count += 1;
    }
    assert_eq!(reread_count, 50);
}

/// Authority round-trip through its own writer and reader.
#[test]
fn authority_write_read_roundtrip() {
    let mut reader = AuthorityMarcReader::new(Cursor::new(SIMPLE_AUTHORITY));
    let record = reader
        .read_record()
        .expect("read should succeed")
        .expect("fixture contains one record");
    assert_eq!(record.leader.record_type, 'z');

    let mut buffer = Vec::new();
    {
        let mut writer = AuthorityMarcWriter::new(&mut buffer);
        writer.write_record(&record).expect("write should succeed");
    }

    let mut reread = AuthorityMarcReader::new(Cursor::new(&buffer[..]));
    let restored = reread
        .read_record()
        .expect("re-read should succeed")
        .expect("round-tripped record present");
    assert_eq!(restored.get_control_field("001"), Some("n79021800"));
}

/// Holdings round-trip through its own writer and reader.
#[test]
fn holdings_write_read_roundtrip() {
    let mut reader = HoldingsMarcReader::new(Cursor::new(SIMPLE_HOLDINGS));
    let record = reader
        .read_record()
        .expect("read should succeed")
        .expect("fixture contains one record");
    assert_eq!(record.leader.record_type, 'x');

    let mut buffer = Vec::new();
    {
        let mut writer = HoldingsMarcWriter::new(&mut buffer);
        writer.write_record(&record).expect("write should succeed");
    }

    let mut reread = HoldingsMarcReader::new(Cursor::new(&buffer[..]));
    let restored = reread
        .read_record()
        .expect("re-read should succeed")
        .expect("round-tripped record present");
    assert_eq!(restored.get_control_field("001"), Some("n79021800"));
}

/// Lenient-mode recovery over corrupted input. Error recovery takes
/// different slicing and buffer-advance paths than the happy path; ASAN
/// checks them for out-of-bounds reads.
#[test]
fn lenient_recovery_over_corrupted_stream() {
    // Three copies of the fixture; corrupt the middle record's directory
    // (its first entry's 4-byte length field starts 27 bytes in).
    let mut stream = Vec::new();
    for _ in 0..3 {
        stream.extend_from_slice(SIMPLE_BOOK);
    }
    let second_start = SIMPLE_BOOK.len();
    for byte in &mut stream[second_start + 27..second_start + 31] {
        *byte = b'X';
    }

    let mut reader =
        MarcReader::new(Cursor::new(&stream[..])).with_recovery_mode(RecoveryMode::Lenient);
    let mut parsed = 0;
    while let Some(_record) = reader.read_record().expect("lenient should not error") {
        parsed += 1;
    }
    assert_eq!(parsed, 3, "lenient mode recovers the corrupted record");
}

/// Lenient-mode parsing of truncated suffixes of a valid record. Every
/// prefix length exercises a different bounds check in the parser.
#[test]
fn lenient_parse_of_all_truncations_never_overreads() {
    for len in 0..SIMPLE_BOOK.len() {
        let slice = &SIMPLE_BOOK[..len];
        let mut reader =
            MarcReader::new(Cursor::new(slice)).with_recovery_mode(RecoveryMode::Lenient);
        // Drain the reader; outcomes vary by prefix length, but no
        // iteration may read outside `slice`.
        while let Ok(Some(_)) = reader.read_record() {}
    }
}

/// Concurrent parsing of a shared buffer from multiple threads. Each
/// thread owns its reader but shares the underlying bytes via Arc; ASAN
/// verifies the parse path performs no invalid accesses under
/// interleaved execution.
#[test]
fn concurrent_parsing_of_shared_buffer() {
    let mut stream = Vec::new();
    for _ in 0..20 {
        stream.extend_from_slice(SIMPLE_BOOK);
    }
    let shared: Arc<Vec<u8>> = Arc::new(stream);

    let mut handles = Vec::new();
    for _ in 0..8 {
        let buffer = Arc::clone(&shared);
        handles.push(thread::spawn(move || {
            let mut reader = MarcReader::new(Cursor::new(&buffer[..]));
            let mut count = 0;
            while let Some(record) = reader.read_record().expect("read should succeed") {
                assert_eq!(record.leader.record_type, 'a');
                count += 1;
            }
            count
        }));
    }

    for handle in handles {
        assert_eq!(handle.join().expect("thread should not panic"), 20);
    }
}

/// Records cloned and dropped across thread boundaries. `Record` clones
/// share their parse-diagnostics allocation by reference count; dropping
/// clones on different threads exercises that shared-ownership path.
#[test]
fn records_clone_and_drop_across_threads() {
    let mut reader = MarcReader::new(Cursor::new(SIMPLE_BOOK));
    let record = reader
        .read_record()
        .expect("read should succeed")
        .expect("fixture contains one record");

    let mut handles = Vec::new();
    for _ in 0..8 {
        let clone = record.clone();
        handles.push(thread::spawn(move || {
            let count = clone.fields().count();
            drop(clone);
            count
        }));
    }

    let expected = record.fields().count();
    for handle in handles {
        assert_eq!(handle.join().expect("thread should not panic"), expected);
    }
}
