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
//!
//! Other kinds (`parse_marcjson`, `io_error`, `recovery_cap`,
//! `accessor`, `writer`) skip with a per-kind reason; their cases
//! remain in the manifest so the docs-vs-implementation gap is tracked.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use mrrc::{MarcReader, RecoveryMode};
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
    wired: bool,
    #[serde(default)]
    skip_reason: Option<String>,
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

/// Drive `MarcReader` over `bytes` in `mode` and return the first
/// error encountered. Successful records and EOF (`Ok(None)`) both
/// resolve to `None`.
fn first_iso2709_error(bytes: &[u8], mode: RecoveryMode) -> Option<mrrc::MarcError> {
    let mut reader = MarcReader::new(Cursor::new(bytes)).with_recovery_mode(mode);
    loop {
        match reader.read_record() {
            Ok(Some(_)) => {},
            Ok(None) => return None,
            Err(e) => return Some(e),
        }
    }
}

fn fire_trigger(case: &Case) -> TriggerOutcome {
    match case.trigger_kind.as_str() {
        "parse_iso2709" => match first_iso2709_error(&fixture_bytes(case), RecoveryMode::Strict) {
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
        "io_error" => TriggerOutcome::UnsupportedKind(
            "trigger_kind=io_error requires injecting a custom Read source from test code".to_string(),
        ),
        "recovery_cap" => TriggerOutcome::UnsupportedKind(
            "trigger_kind=recovery_cap requires building a multi-record malformed stream and configuring max_errors".to_string(),
        ),
        "accessor" => TriggerOutcome::UnsupportedKind(
            "trigger_kind=accessor requires constructing the accessor call from test code".to_string(),
        ),
        "writer" => TriggerOutcome::UnsupportedKind(
            "trigger_kind=writer requires constructing an oversize record from test code".to_string(),
        ),
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
    if !case.recovery_modes.iter().any(|m| m == "strict") {
        // Strict mode is the only mode the harness asserts today.
        // Cases without strict in their contract are exercised when
        // per-record diagnostic surfaces land for lenient/permissive.
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
            "parse_iso2709" | "parse_marcxml" | "parse_marcjson"
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
