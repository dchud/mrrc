#![allow(missing_docs, unused_doc_comments, unused_attributes)]
//! Error-handling performance benchmarks.
//!
//! These scenarios are the regression-sensitive measurement points for
//! changes that add detection logic to the hot parse path (per-byte
//! indicator validation, subfield-code validation, directory-walker
//! recharacterization, end-of-record verification, etc.).
//!
//! * `parse_10k_clean_strict` — happy path. No malformations, strict
//!   recovery. The number every user pays. Regressions here are the
//!   most consequential and should be assessed before landing
//!   hot-path changes.
//!
//! * `parse_10k_bad_indicators_lenient` — every record's first
//!   indicator on field 245 is mutated to a non-digit/non-space byte.
//!   The parse path constructs `MarcError::InvalidIndicator` (E201)
//!   per record; the bench runs in lenient mode with `max_errors(0)`
//!   so the per-record detection runs against all 10k records and
//!   the offending fields are dropped via `cap.note` rather than
//!   short-circuiting. Delta between this scenario and the clean
//!   one measures the cost of per-byte indicator validation plus
//!   the cap.note bookkeeping. Fixture is pre-mutated in setup so
//!   the per-iteration work is just parsing.
//!
//! The `parse_10k_bad_indicators_lenient` scenario disables the
//! recovered-error cap (`with_max_errors(0)`) so iteration runs to
//! completion over the fully-malformed stream and produces a stable
//! signal for lenient recovery cost.
//!
//! Run with `cargo bench --bench error_handling_benchmarks` for local
//! profiling. Codspeed exercises the same scenarios in CI for general
//! drift awareness.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use mrrc::{MarcReader, RecoveryMode};
use std::io::Cursor;

const FIXTURE_PATH: &str = "tests/data/fixtures/10k_records.mrc";

fn load_fixture() -> Vec<u8> {
    std::fs::read(FIXTURE_PATH)
        .unwrap_or_else(|e| panic!("read {FIXTURE_PATH}: {e}; run scripts to generate fixtures"))
}

/// Locate every record in `bytes` (assumes well-formed input) and
/// return their absolute byte offsets. Each record begins after the
/// previous one's `RECORD_TERMINATOR` (`0x1D`).
fn record_offsets(bytes: &[u8]) -> Vec<usize> {
    let mut offsets = vec![0usize];
    for (i, &b) in bytes.iter().enumerate() {
        if b == 0x1D && i + 1 < bytes.len() {
            offsets.push(i + 1);
        }
    }
    offsets
}

/// For each record at the given offsets, parse the leader to find the
/// data area and the directory entry for tag 245. Mutate the first
/// indicator byte of 245 to ':' (a byte the documented E201 trigger
/// will reject). Records without a 245 field are left untouched.
fn mutate_245_first_indicator(bytes: &mut [u8], offsets: &[usize]) {
    for &start in offsets {
        // Leader is 24 bytes; parse the base-address-of-data field at
        // bytes 12-16 (5 ASCII digits).
        if start + 24 > bytes.len() {
            continue;
        }
        let Ok(base_addr_str) = std::str::from_utf8(&bytes[start + 12..start + 17]) else {
            continue;
        };
        let Ok(base_addr) = base_addr_str.parse::<usize>() else {
            continue;
        };
        // Walk the directory (12 bytes per entry, terminated by 0x1E)
        // looking for tag 245.
        let dir_start = start + 24;
        let dir_end = start + base_addr;
        if dir_end > bytes.len() {
            continue;
        }
        let mut pos = dir_start;
        while pos + 12 <= dir_end {
            if bytes[pos] == 0x1E {
                break;
            }
            let tag = &bytes[pos..pos + 3];
            if tag == b"245" {
                // Field start within the data area.
                let Ok(field_start_str) = std::str::from_utf8(&bytes[pos + 7..pos + 12]) else {
                    break;
                };
                let Ok(field_offset) = field_start_str.parse::<usize>() else {
                    break;
                };
                let field_byte = start + base_addr + field_offset;
                if field_byte < bytes.len() {
                    bytes[field_byte] = b':';
                }
                break;
            }
            pos += 12;
        }
    }
}

fn benchmark_parse_10k_clean_strict(c: &mut Criterion) {
    let fixture = black_box(load_fixture());
    c.bench_function("parse_10k_clean_strict", |b| {
        b.iter(|| {
            let mut reader =
                MarcReader::new(Cursor::new(&fixture[..])).with_recovery_mode(RecoveryMode::Strict);
            let mut count = 0usize;
            while let Ok(Some(_)) = reader.read_record() {
                count += 1;
            }
            count
        });
    });
}

fn benchmark_parse_10k_bad_indicators_lenient(c: &mut Criterion) {
    let mut fixture = load_fixture();
    let offsets = record_offsets(&fixture);
    mutate_245_first_indicator(&mut fixture, &offsets);
    let fixture = black_box(fixture);
    c.bench_function("parse_10k_bad_indicators_lenient", |b| {
        b.iter(|| {
            let mut reader = MarcReader::new(Cursor::new(&fixture[..]))
                .with_recovery_mode(RecoveryMode::Lenient)
                .with_max_errors(0);
            let mut count = 0usize;
            while let Ok(Some(_)) = reader.read_record() {
                count += 1;
            }
            count
        });
    });
}

criterion_group!(
    error_handling_benches,
    benchmark_parse_10k_clean_strict,
    benchmark_parse_10k_bad_indicators_lenient,
);
criterion_main!(error_handling_benches);
