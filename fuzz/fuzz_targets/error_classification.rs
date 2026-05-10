//! Error-classification fuzz harness — strengthens `parse_record`'s
//! panic-only contract with per-input behavioral assertions.
//!
//! Driven at [`ValidationLevel::StrictMarc`]: every byte the parser
//! accepts must be well-formed per the documented codec rules
//! (indicator and subfield-code byte sets, UTF-8 strictness). At the
//! default `Structural` level the parser is deliberately lossy
//! (replaces invalid UTF-8 with U+FFFD, accepts non-graphic subfield
//! codes), so round-trip equivalence isn't a guaranteed invariant
//! there.
//!
//! Asserted contract:
//!
//! - `Ok(record)` → the writer can round-trip the record back through
//!   the reader. If [`MarcWriter::write_record`] rejects the parsed
//!   record (e.g., total length exceeds the ISO 2709 99999-byte limit),
//!   we discard; otherwise re-parsing the writer's output must yield a
//!   record. A divergence (re-parse fails or yields no record) means
//!   strict-marc parsing silently accepted bytes that don't represent a
//!   stable record.
//! - `Err(e)` → `e.code()` returns one of the documented `Exxx`
//!   identifiers (per `tests/error_coverage.toml`). If a future
//!   `MarcError` variant lands without a `code()` arm or with an
//!   off-spec identifier, the assertion fires.
//!
//! What this catches that `parse_record` and `roundtrip_binary` do not:
//!
//! - Silent acceptance of bytes the documented codec considers invalid
//!   — the parser returns `Ok` for input the writer can't faithfully
//!   represent. `parse_record` only flags panics; `roundtrip_binary`
//!   discards re-parse failures.
//! - Future `MarcError` variants that ship without a documented code,
//!   surfacing docs-vs-code drift before release.
//!
//! See `docs/contributing/fuzzing.md` for triage.

#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::{MarcReader, MarcWriter, ValidationLevel};
use std::io::Cursor;

/// Documented error codes in `tests/error_coverage.toml`. A code outside
/// this set means either a new variant landed without a manifest entry,
/// or an existing variant's `code()` arm was changed without updating
/// the manifest. Either way: docs-vs-code drift that needs triage.
const DOCUMENTED_CODES: &[&str] = &[
    "E001", "E002", "E003", "E004", "E005", "E006", "E007", "E099", "E101", "E105", "E106",
    "E201", "E202", "E301", "E401", "E402", "E404",
];

fuzz_target!(|data: &[u8]| {
    let mut reader =
        MarcReader::new(Cursor::new(data)).with_validation_level(ValidationLevel::StrictMarc);
    loop {
        match reader.read_record() {
            Ok(Some(record)) => {
                let mut buf = Vec::new();
                if MarcWriter::new(&mut buf).write_record(&record).is_err() {
                    // Writer rejected the record (representable-range
                    // violation, etc.). Not a parser-validity bug.
                    continue;
                }
                let mut second = MarcReader::new(Cursor::new(&buf[..]))
                    .with_validation_level(ValidationLevel::StrictMarc);
                match second.read_record() {
                    Ok(Some(_)) => {},
                    Ok(None) => {
                        panic!("writer emitted bytes that re-parse to no record")
                    },
                    Err(e) => panic!("writer emitted bytes that fail to re-parse: {e:?}"),
                }
            },
            Ok(None) => break,
            Err(e) => {
                let code = e.code();
                assert!(
                    DOCUMENTED_CODES.contains(&code),
                    "MarcError code {code:?} not in documented set ({e:?})"
                );
                break;
            },
        }
    }
});
