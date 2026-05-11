//! Regression tests for bugs found by coverage-guided fuzzing. Each
//! fixture under `tests/data/fuzz-regressions/<target>/` is a minimized
//! reproducer committed to guard against reintroduction on every PR.
//!
//! Adding a fixture for an existing target is a single-file change —
//! the per-target test function below auto-discovers any new fixture.
//!
//! Adding a fixture for a NEW target requires adding a new `#[test]`
//! function that mirrors the relevant fuzz target's contract.

use mrrc::{MarcReader, MarcWriter, RecoveryMode, ValidationLevel};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

fn fixtures_dir(target: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/fuzz-regressions")
        .join(target)
}

/// Walks every fixture under the given target directory and yields its
/// bytes. Returns the empty iterator if the directory does not exist
/// (no regressions filed yet for this target).
fn fixtures(target: &str) -> Vec<(PathBuf, Vec<u8>)> {
    let dir = fixtures_dir(target);
    if !dir.exists() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).expect("read fuzz-regressions dir") {
        let path = entry.expect("dir entry").path();
        if !path.is_file() {
            continue;
        }
        let bytes = fs::read(&path).expect("read fixture");
        out.push((path, bytes));
    }
    out
}

/// `parse_record` asserts only that the reader does not panic on
/// arbitrary input. Mirrors `fuzz/fuzz_targets/parse_record.rs`.
#[test]
fn parse_record_regressions() {
    for (_path, bytes) in fixtures("parse_record") {
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]));
        while let Ok(Some(_)) = reader.read_record() {}
    }
}

/// `roundtrip_binary` asserts only that the reader → writer → reader
/// pipeline does not panic. Mirrors
/// `fuzz/fuzz_targets/roundtrip_binary.rs`.
#[test]
fn roundtrip_binary_regressions() {
    for (_path, bytes) in fixtures("roundtrip_binary") {
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]));
        while let Ok(Some(record)) = reader.read_record() {
            let mut buf = Vec::new();
            if MarcWriter::new(&mut buf).write_record(&record).is_err() {
                continue;
            }
            let mut second = MarcReader::new(Cursor::new(&buf[..]));
            while let Ok(Some(_)) = second.read_record() {}
        }
    }
}

/// `recovery_mode_consistency` asserts no panics when driving the
/// same input through strict / lenient / permissive at `strict_marc`
/// validation. Mirrors `fuzz/fuzz_targets/recovery_mode_consistency.rs`.
#[test]
fn recovery_mode_consistency_regressions() {
    for (_path, bytes) in fixtures("recovery_mode_consistency") {
        for mode in [
            RecoveryMode::Strict,
            RecoveryMode::Lenient,
            RecoveryMode::Permissive,
        ] {
            let mut reader = MarcReader::new(Cursor::new(&bytes[..]))
                .with_recovery_mode(mode)
                .with_validation_level(ValidationLevel::StrictMarc);
            // Discard outcomes — only panics are regressions.
            let _ = reader.read_record();
        }
    }
}

/// `error_classification` asserts the strict-marc reader either rejects
/// the input outright or yields records the writer can faithfully
/// round-trip. Mirrors `fuzz/fuzz_targets/error_classification.rs`.
#[test]
fn error_classification_regressions() {
    for (path, bytes) in fixtures("error_classification") {
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]))
            .with_validation_level(ValidationLevel::StrictMarc);
        while let Ok(Some(record)) = reader.read_record() {
            let mut buf = Vec::new();
            if MarcWriter::new(&mut buf).write_record(&record).is_err() {
                // Writer rejected — correct for records outside the
                // representable range.
                continue;
            }
            let mut second = MarcReader::new(Cursor::new(&buf[..]))
                .with_validation_level(ValidationLevel::StrictMarc);
            match second.read_record() {
                Ok(Some(_)) => {},
                other => panic!(
                    "{}: writer-emitted bytes failed to re-parse to a record: {other:?}",
                    path.display()
                ),
            }
        }
    }
}
