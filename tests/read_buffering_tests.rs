//! Reads-per-record behavior of the MARC read loop.
//!
//! The per-record read loop issues at least two reads per record (leader,
//! then body). On an unbuffered `File` each read is a syscall, so
//! `from_path` wraps the file in a 64 KiB `BufReader`. These tests pin
//! both halves of that claim by counting the reads that reach the
//! underlying reader.

use mrrc::MarcReader;
use std::cell::Cell;
use std::io::{BufReader, Cursor, Read};
use std::rc::Rc;

/// `Read` wrapper that counts calls reaching the underlying reader.
///
/// The counter is shared via `Rc<Cell<_>>` so it stays observable after
/// the wrapper is moved into a `BufReader` or `MarcReader`.
struct CountingReader<R> {
    inner: R,
    reads: Rc<Cell<usize>>,
}

impl<R: Read> Read for CountingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reads.set(self.reads.get() + 1);
        self.inner.read(buf)
    }
}

fn fixture_1k() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/fixtures/1k_records.mrc"
    );
    std::fs::read(path).expect("1k_records.mrc fixture must exist")
}

fn drain<R: Read>(reader: &mut MarcReader<R>) -> usize {
    let mut count = 0;
    while let Ok(Some(_)) = reader.read_record() {
        count += 1;
    }
    count
}

#[test]
fn unbuffered_read_loop_issues_at_least_two_reads_per_record() {
    let data = fixture_1k();
    let reads = Rc::new(Cell::new(0));
    let counting = CountingReader {
        inner: Cursor::new(data),
        reads: reads.clone(),
    };

    let mut reader = MarcReader::new(counting);
    assert_eq!(drain(&mut reader), 1000);
    assert!(
        reads.get() >= 2000,
        "expected >= 2 reads per record unbuffered, got {} for 1000 records",
        reads.get()
    );
}

#[test]
fn buffered_read_loop_issues_one_read_per_buffer_fill() {
    let data = fixture_1k();
    let len = data.len();
    let reads = Rc::new(Cell::new(0));
    let counting = CountingReader {
        inner: Cursor::new(data),
        reads: reads.clone(),
    };

    // Same capacity from_path uses. Expected reads: one per 64 KiB
    // buffer fill, plus one zero-byte read detecting EOF.
    let mut reader = MarcReader::new(BufReader::with_capacity(64 * 1024, counting));
    assert_eq!(drain(&mut reader), 1000);
    let max_expected = len.div_ceil(64 * 1024) + 1;
    assert!(
        reads.get() <= max_expected,
        "expected <= {} underlying reads ({} bytes through a 64 KiB buffer), got {}",
        max_expected,
        len,
        reads.get()
    );
}

#[test]
fn from_path_reads_the_same_records_as_an_unbuffered_reader() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/fixtures/1k_records.mrc"
    );
    let mut from_path = MarcReader::from_path(path).expect("fixture must open");
    let mut unbuffered = MarcReader::new(Cursor::new(fixture_1k()));

    loop {
        let a = from_path.read_record().expect("from_path read");
        let b = unbuffered.read_record().expect("cursor read");
        match (a, b) {
            (None, None) => break,
            (Some(ra), Some(rb)) => assert_eq!(format!("{ra:?}"), format!("{rb:?}")),
            (a, b) => panic!(
                "readers diverged: from_path yielded {:?}, cursor yielded {:?}",
                a.map(|_| "record"),
                b.map(|_| "record")
            ),
        }
    }
}
