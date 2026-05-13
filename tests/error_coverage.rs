//! Error-handling coverage harness.
//!
//! Reads `tests/error_coverage.toml` and runs one assertion bundle per
//! `[[case]]` entry. For each `wired = true` case whose `trigger_kind`
//! the harness supports, asserts that the documented
//! [`MarcError`](mrrc::MarcError) variant, code, slug, and positional
//! context fire when the parser exercises the trigger. For each
//! `wired = false` case (or one whose `trigger_kind` the harness does
//! not yet implement), prints a skip reason so the gap between
//! documentation and implementation is visible in CI output.
//!
//! The harness emits `wired: X/Y (skipped: Z)` on every run. The
//! numbers are monotonic over time: when detection lands for a
//! previously-unwired trigger, flipping `wired = false` →
//! `wired = true` in the manifest is sufficient and the assertion
//! picks up the new case automatically.
//!
//! Currently supported `trigger_kind` values:
//!   * `parse_iso2709` — feed bytes to [`MarcReader`](mrrc::MarcReader)
//!     in strict mode and capture the first error
//!   * `parse_marcxml` — feed UTF-8 text to
//!     [`mrrc::marcxml::marcxml_to_record`] and capture the error
//!   * `accessor` — parse the fixture cleanly, then call a hard-coded
//!     accessor whose lookup is documented to raise the case's variant
//!     (currently only `e105_field_not_found` →
//!     `Record::get_field_or_err("999")`)
//!   * `io_error` — wrap a `Read` impl that returns `std::io::Error`
//!     from the first read in a [`MarcReader`](mrrc::MarcReader) and
//!     capture the resulting [`MarcError::IoError`](mrrc::MarcError)
//!   * `recovery_cap` — drive a stream of malformed records past
//!     [`MarcReader::with_max_errors`](mrrc::MarcReader::with_max_errors)
//!     in lenient mode and capture the
//!     [`MarcError::FatalReaderError`](mrrc::MarcError) that fires
//!     when the cap trips
//!   * `writer` — construct a record whose serialized length exceeds
//!     the ISO 2709 99999-byte limit and call
//!     [`MarcWriter::write_record`](mrrc::MarcWriter::write_record),
//!     capturing the resulting
//!     [`MarcError::WriterError`](mrrc::MarcError)
//!
//! The remaining kind (`parse_marcjson`) skips with a per-kind reason
//! — there is no public Rust `str → Record` API for MARCJSON, so the
//! case is exercised on the Python side instead.

use std::fs;
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};

use mrrc::{
    Field, Leader, MarcReader, MarcWriter, Record, RecoveryMode, Subfield, ValidationLevel,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Manifest {
    #[allow(dead_code)]
    schema_version: u32,
    case: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    code: String,
    variant: String,
    slug: String,
    #[serde(default = "default_trigger_kind")]
    trigger_kind: String,
    #[serde(default)]
    trigger_fixture: Option<String>,
    #[allow(dead_code)]
    description: String,
    expected_context: Vec<String>,
    recovery_modes: Vec<String>,
    /// Validation level the harness should use when exercising the
    /// trigger. Optional; defaults to `"structural"` (mrrc's
    /// `MarcReader` default). Cases that require strict-MARC byte
    /// validation (E201, E202, E301) should set this to
    /// `"strict_marc"`.
    #[serde(default)]
    validation_level: Option<String>,
    wired: bool,
    #[serde(default)]
    skip_reason: Option<String>,
}

fn parse_validation_level(case: &Case) -> ValidationLevel {
    match case.validation_level.as_deref() {
        None | Some("structural") => ValidationLevel::Structural,
        Some("strict_marc") => ValidationLevel::StrictMarc,
        Some(other) => panic!(
            "case {}: unknown validation_level {:?} (expected \"structural\" or \"strict_marc\")",
            case.id, other
        ),
    }
}

fn default_trigger_kind() -> String {
    "parse_iso2709".to_string()
}

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/error_coverage.toml")
}

fn load_manifest() -> Manifest {
    let text = fs::read_to_string(manifest_path()).expect("read error_coverage.toml");
    toml::from_str(&text).expect("parse error_coverage.toml")
}

fn fixture_path(case: &Case) -> PathBuf {
    let rel = case.trigger_fixture.as_ref().unwrap_or_else(|| {
        panic!(
            "case {} has trigger_kind requiring a fixture but none set",
            case.id
        )
    });
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn fixture_bytes(case: &Case) -> Vec<u8> {
    let path = fixture_path(case);
    fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
}

fn fixture_text(case: &Case) -> String {
    let path = fixture_path(case);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
}

/// Outcome of attempting to fire a case's documented trigger:
/// either an error fired (and we can assert against it) or the
/// harness chose not to exercise this trigger and produced a reason
/// to record as a skip.
enum TriggerOutcome {
    Fired(mrrc::MarcError),
    NoError,
    UnsupportedKind(String),
}

/// Exercise an accessor case: parse the fixture cleanly, then invoke
/// the accessor whose lookup is documented to fire the case's variant.
/// New `accessor` cases need their case-id branch wired here — accessor
/// names and arguments aren't yet expressed in the manifest schema.
fn exercise_accessor(case: &Case) -> TriggerOutcome {
    let bytes = fixture_bytes(case);
    let mut reader = MarcReader::new(Cursor::new(bytes)).with_recovery_mode(RecoveryMode::Strict);
    let record = match reader.read_record() {
        Ok(Some(r)) => r,
        Ok(None) => {
            return TriggerOutcome::UnsupportedKind(format!(
                "{}: fixture parsed to no records; accessor cannot be exercised",
                case.id
            ))
        },
        Err(e) => {
            return TriggerOutcome::UnsupportedKind(format!(
                "{}: fixture failed to parse cleanly ({e}); accessor cannot be exercised",
                case.id
            ))
        },
    };

    match case.id.as_str() {
        "e105_field_not_found" => match record.get_field_or_err("999") {
            Ok(_) => TriggerOutcome::NoError,
            Err(e) => TriggerOutcome::Fired(e),
        },
        other => TriggerOutcome::UnsupportedKind(format!(
            "trigger_kind=accessor: case {other} has no harness branch; add one in exercise_accessor"
        )),
    }
}

/// Drive `MarcReader` over `bytes` in `mode` at `level` and return
/// the first error encountered. Successful records and EOF
/// (`Ok(None)`) both resolve to `None`.
fn first_iso2709_error(
    bytes: &[u8],
    mode: RecoveryMode,
    level: ValidationLevel,
) -> Option<mrrc::MarcError> {
    let mut reader = MarcReader::new(Cursor::new(bytes))
        .with_recovery_mode(mode)
        .with_validation_level(level);
    loop {
        match reader.read_record() {
            Ok(Some(_)) => {},
            Ok(None) => return None,
            Err(e) => return Some(e),
        }
    }
}

/// `Read` impl that returns `std::io::Error` on the first read.
/// Used to exercise the parser's underlying-stream-failure path
/// for E007 `IoError` without touching the filesystem.
struct FailingReader;

impl Read for FailingReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "synthetic read failure",
        ))
    }
}

/// Build a single record with a structurally invalid directory entry
/// (non-digit field-length bytes). In lenient mode each such record
/// increments the recovery error counter once; concatenating N+1 of
/// these and configuring `with_max_errors(N)` trips the cap on the
/// (N+1)th read. Mirrors `build_bad_record` in `src/reader.rs`'s
/// unit-test module.
fn build_bad_record() -> Vec<u8> {
    const FIELD_TERMINATOR: u8 = 0x1E;
    const RECORD_TERMINATOR: u8 = 0x1D;

    let mut directory = Vec::new();
    directory.extend_from_slice(b"245ABCD00000");
    directory.push(FIELD_TERMINATOR);

    let base_address = 24 + directory.len();
    let record_length = base_address + 1;

    let mut leader = Vec::new();
    leader.extend_from_slice(format!("{record_length:05}").as_bytes());
    leader.extend_from_slice(b"nam a22");
    leader.extend_from_slice(format!("{base_address:05}").as_bytes());
    leader.extend_from_slice(b" i 4500");

    let mut out = Vec::new();
    out.extend_from_slice(&leader);
    out.extend_from_slice(&directory);
    out.push(RECORD_TERMINATOR);
    out
}

fn fire_trigger(case: &Case) -> TriggerOutcome {
    match case.trigger_kind.as_str() {
        "parse_iso2709" => match first_iso2709_error(
            &fixture_bytes(case),
            RecoveryMode::Strict,
            parse_validation_level(case),
        ) {
            Some(e) => TriggerOutcome::Fired(e),
            None => TriggerOutcome::NoError,
        },
        "parse_marcxml" => {
            let text = fixture_text(case);
            match mrrc::marcxml::marcxml_to_record(&text) {
                Ok(_) => TriggerOutcome::NoError,
                Err(e) => TriggerOutcome::Fired(e),
            }
        },
        "parse_marcjson" => TriggerOutcome::UnsupportedKind(
            "no public Rust str-to-Record API for MARCJSON; case is exercised in the Python harness".to_string(),
        ),
        "io_error" => {
            let mut reader = MarcReader::new(FailingReader).with_recovery_mode(RecoveryMode::Strict);
            match reader.read_record() {
                Ok(_) => TriggerOutcome::NoError,
                Err(e) => TriggerOutcome::Fired(e),
            }
        },
        "recovery_cap" => {
            const CAP: usize = 1;
            let mut stream = Vec::new();
            for _ in 0..=(CAP + 1) {
                stream.extend_from_slice(&build_bad_record());
            }
            let mut reader = MarcReader::new(Cursor::new(stream))
                .with_recovery_mode(RecoveryMode::Lenient)
                .with_max_errors(CAP);
            loop {
                match reader.read_record() {
                    Ok(Some(_)) => {},
                    Ok(None) => return TriggerOutcome::NoError,
                    Err(e) => return TriggerOutcome::Fired(e),
                }
            }
        },
        "accessor" => exercise_accessor(case),
        "writer" => {
            let leader = Leader::from_bytes(b"00000nam a2200000 i 4500")
                .expect("synthetic minimal leader parses");
            let mut record = Record::new(leader);
            // ~100k subfield value forces the writer's serialized record
            // length past the ISO 2709 99999-byte ceiling.
            let big_value = "x".repeat(100_000);
            let field = Field {
                tag: "999".to_string(),
                indicator1: ' ',
                indicator2: ' ',
                subfields: smallvec::smallvec![Subfield {
                    code: 'a',
                    value: big_value,
                }],
            };
            record.add_field(field);

            let mut buf = Vec::new();
            let mut writer = MarcWriter::new(&mut buf);
            match writer.write_record(&record) {
                Ok(()) => TriggerOutcome::NoError,
                Err(e) => TriggerOutcome::Fired(e),
            }
        },
        other => TriggerOutcome::UnsupportedKind(format!("unknown trigger_kind {other:?}")),
    }
}

fn json_probe_key(field: &str) -> &str {
    match field {
        "bytes_near" => "bytes_near_hex",
        "found" => "found_hex",
        other => other,
    }
}

/// Per-case outcome the harness reports up to the top-level test.
enum WiredOutcome {
    Asserted,
    SkippedByHarness(String),
    Failed(String),
}

/// Exercise one wired case. Returns the outcome so the caller can
/// distinguish assertion failures (manifest claims something the
/// parser doesn't deliver) from harness skips (this harness cannot
/// exercise the case's `trigger_kind`, but the wiring may still be
/// in place and exercised by another harness).
fn run_wired(case: &Case) -> WiredOutcome {
    // The harness's parse_iso2709/parse_marcxml/parse_marcjson/accessor
    // branches all drive the parser in strict mode. Triggers with their
    // own intrinsic mode requirements (recovery_cap drives lenient) carry
    // that mode inside the trigger branch and bypass this gate.
    let strict_only_trigger = !matches!(case.trigger_kind.as_str(), "recovery_cap");
    if strict_only_trigger && !case.recovery_modes.iter().any(|m| m == "strict") {
        return WiredOutcome::SkippedByHarness(
            "case contract does not cover strict mode; non-strict assertions pending".to_string(),
        );
    }

    let err = match fire_trigger(case) {
        TriggerOutcome::Fired(e) => e,
        TriggerOutcome::NoError => {
            return WiredOutcome::Failed(format!(
                "{} ({} / {}): expected {} error, got success",
                case.id, case.code, case.variant, case.code
            ));
        },
        TriggerOutcome::UnsupportedKind(reason) => {
            return WiredOutcome::SkippedByHarness(reason);
        },
    };

    if err.code() != case.code {
        return WiredOutcome::Failed(format!(
            "{} ({}): expected code {}, got {} ({:?})",
            case.id,
            case.variant,
            case.code,
            err.code(),
            err
        ));
    }
    if err.slug() != case.slug {
        return WiredOutcome::Failed(format!(
            "{} ({}): expected slug {:?}, got {:?}",
            case.id,
            case.variant,
            case.slug,
            err.slug()
        ));
    }
    let dict = err.to_json_value();
    for field in &case.expected_context {
        let key = json_probe_key(field);
        let present = dict.get(key).is_some_and(|v| !v.is_null());
        if !present {
            return WiredOutcome::Failed(format!(
                "{} ({}): expected_context field {} not populated (probed via {:?}); error JSON: {}",
                case.id, case.variant, field, key, dict
            ));
        }
    }
    WiredOutcome::Asserted
}

#[test]
fn manifest_is_well_formed() {
    let manifest = load_manifest();
    assert_eq!(manifest.schema_version, 1, "schema_version drift");
    assert!(!manifest.case.is_empty(), "manifest has no cases");

    let mut ids = std::collections::HashSet::new();
    for case in &manifest.case {
        let inserted = ids.insert(case.id.clone());
        assert!(inserted, "duplicate case id {}", case.id);

        if matches!(
            case.trigger_kind.as_str(),
            "parse_iso2709" | "parse_marcxml" | "parse_marcjson" | "accessor"
        ) {
            assert!(
                case.trigger_fixture.is_some(),
                "case {}: trigger_kind {:?} requires a trigger_fixture",
                case.id,
                case.trigger_kind
            );
            let path = fixture_path(case);
            assert!(
                path.exists(),
                "case {}: fixture {} does not exist",
                case.id,
                path.display()
            );
        }

        if !case.wired {
            assert!(
                case.skip_reason.is_some(),
                "case {} is unwired but has no skip_reason",
                case.id
            );
        }
    }
}

#[test]
fn coverage_assertions() {
    let manifest = load_manifest();
    let total = manifest.case.len();
    let mut failures: Vec<String> = Vec::new();
    let mut wired_count = 0usize;
    let mut asserted = 0usize;
    let mut harness_skips: Vec<(String, String)> = Vec::new();
    let mut unwired_skips: Vec<(String, String)> = Vec::new();

    for case in &manifest.case {
        if case.wired {
            wired_count += 1;
            match run_wired(case) {
                WiredOutcome::Asserted => asserted += 1,
                WiredOutcome::SkippedByHarness(reason) => {
                    harness_skips.push((case.id.clone(), reason));
                },
                WiredOutcome::Failed(reason) => failures.push(reason),
            }
        } else {
            let reason = case
                .skip_reason
                .clone()
                .unwrap_or_else(|| "unwired (no reason provided)".into());
            unwired_skips.push((case.id.clone(), reason));
        }
    }

    for (id, reason) in &unwired_skips {
        eprintln!("[error_coverage] UNWIRED {id}: {reason}");
    }
    for (id, reason) in &harness_skips {
        eprintln!("[error_coverage] HARNESS-SKIP {id}: {reason}");
    }
    eprintln!(
        "[error_coverage] wired in manifest: {wired_count}/{total} \
         | harness asserted: {asserted}/{wired_count} \
         | harness skipped: {} (unwired: {}, harness limitations: {})",
        unwired_skips.len() + harness_skips.len(),
        unwired_skips.len(),
        harness_skips.len(),
    );

    assert!(
        failures.is_empty(),
        "wired-case failures:\n  - {}",
        failures.join("\n  - ")
    );
}
