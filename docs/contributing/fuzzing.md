# Fuzzing

Coverage-guided fuzzing exercises mrrc's parsers with mutated byte streams
that random property testing misses. It targets the kinds of bugs a parser
is uniquely exposed to: panics on malformed input, infinite loops on
pathological structures, and memory issues in the dependency chain.

This guide covers installing cargo-fuzz, running targets locally, and the
playbook for investigating CI findings.

## What is tested

| Target | Entry point | Status |
|--------|-------------|--------|
| `parse_record` | `MarcReader::read_record` over the full ISO 2709 reader | Active |
| `roundtrip_binary` | Parse → serialize → parse-again coupling | Active |
| `parse_leader` | 24-byte leader parsing | Planned (bd-gbgx) |
| `decode_marc8` | MARC-8 encoding state machine | Planned (bd-2dia) |
| `parse_marcxml` | MARCXML reader | Planned (bd-3t62) |
| `parse_json` / `parse_marcjson` | JSON readers | Planned (bd-uss1) |

`parse_record` is the first target and the highest-value one — any bytes
passing through mrrc eventually hit its code paths. The other targets
narrow the mutator's focus to smaller state spaces (faster convergence)
or cross different axes of behavior (writer path, JSON/XML parsers).

`roundtrip_binary` couples the reader and writer: every record the reader
extracts is serialized via `MarcWriter` and re-parsed. mrrc does not
guarantee byte-for-byte round-trip stability — the writer canonicalizes
the leader and regenerates the directory — so the only assertion is that
neither the writer path nor the second reader panics. `Err(MarcError)`
returns from the writer (e.g., records exceeding the 4 GiB representable
limit) or from the second reader are correct behavior and discarded. A
stronger structural-equality variant (same field tags, subfield codes,
and values across the round trip) can be layered later once the
guarantees are documented.

## Installing cargo-fuzz

cargo-fuzz requires the nightly Rust toolchain because `libfuzzer-sys`
uses compiler features (`-C passes=sancov-*`) that are only available on
nightly.

```bash
# Install the nightly toolchain (rustup)
rustup toolchain install nightly

# Install cargo-fuzz into ~/.cargo/bin/
cargo install cargo-fuzz
```

No project-level install is needed — `fuzz/` is a standalone Cargo
workspace with its own `rust-toolchain.toml` pinning nightly, so the root
stable pin (1.95.0) is unaffected.

## Running a target locally

Run from the repo root. cargo-fuzz resolves `./fuzz/Cargo.toml` from the
current directory, and `+nightly` overrides the root toolchain pin so the
fuzz crate compiles against nightly.

```bash
# Short 60-second smoke run
cargo +nightly fuzz run parse_record -- -max_total_time=60

# Overnight coverage hunt
cargo +nightly fuzz run parse_record -- -max_total_time=28800

# Run with a known input to reproduce a crash
cargo +nightly fuzz run parse_record fuzz/artifacts/parse_record/crash-<hash>
```

libfuzzer flags go after the `--` separator. Common ones:

- `-max_total_time=<seconds>` — stop after N seconds of fuzzing
- `-runs=<N>` — stop after N inputs
- `-max_len=<N>` — cap input size in bytes
- `-dict=<file>` — load a dictionary of interesting tokens

The full libfuzzer flag reference is at <https://llvm.org/docs/LibFuzzer.html#options>.

## Seed corpus

`fuzz/corpus/parse_record/` is seeded from small binary MARC fixtures
under `tests/data/*.mrc`. These give the mutator a realistic starting
distribution (valid leaders, valid directory entries, typical subfield
patterns) so it can focus on exploring what happens when those pieces
get broken.

Seed files are tracked in git. Mutator-discovered corpus entries
(SHA-named additions libfuzzer creates during a run) are gitignored —
they are local-only and can grow into the GBs on a long fuzz session.
The gitignore at `fuzz/.gitignore` allows each curated seed explicitly
by name.

**Adding a new seed:** drop the file into `fuzz/corpus/parse_record/`,
add an explicit `!corpus/parse_record/<filename>` line to
`fuzz/.gitignore`, and commit both.

**Complementary corpus from the testbed:** the
[mrrc-testbed](https://github.com/dchud/mrrc-testbed) repo curates MARC
fixtures from real-world public datasets (LoC BIBFRAME samples, OCLC
samples, etc.). Its fixtures are an excellent source of additional seed
inputs. The testbed's `formal-methods-implementation-plan.md` anticipates
a `just fuzz-seed` recipe that exports fixture data for mrrc's fuzz
corpus — see the testbed repo for current status. Fuzzing in mrrc and
fixture curation in mrrc-testbed are complementary; neither replaces the
other.

## Managing the local corpus

Each local `cargo fuzz run` appends new coverage-expanding inputs to
`fuzz/corpus/parse_record/`. They are gitignored so they never enter the
repo, but they do accumulate on disk. Over many runs the corpus can reach
tens or hundreds of MB, which slows fuzz startup (libfuzzer reads every
input on launch). Two cleanup commands handle it:

**Minimize in place** — keeps coverage, sheds redundant inputs. Usually
shrinks the corpus 50-90%. Run after a long session when startup feels
slow:

```bash
cargo +nightly fuzz cmin parse_record
```

**Full reset** — removes only mutator-discovered files, keeps curated
seeds (the `-X` flag means "only ignored files"):

```bash
git clean -fdX fuzz/corpus/parse_record/
```

CI runners start fresh each nightly run and throw away mutator adds when
the runner tears down, so no cleanup is needed there.

## Playbook: investigating a CI failure

This section is an executable runbook for turning a red nightly run into
a committed regression test and bug fix. It works for both a human
developer and an agent operating from a cold start. Each step has the
exact command to run and a clear success/failure signal.

**Prerequisites:** `gh` CLI authenticated against this repo; nightly
toolchain and cargo-fuzz installed (see the install section above).

### Step 1 — Find and fetch the failing run

```bash
# Most recent failing fuzz runs (one per line: time, URL, run ID)
gh run list --workflow=fuzz.yml --status=failure --limit=5

# Download the artifact for a specific run, landing into the local
# fuzz/artifacts/ tree (same layout libfuzzer uses locally).
gh run download <run-id> --name fuzz-artifacts-parse_record --dir fuzz/artifacts/
```

After the download, list the files to pick up the exact crash filename:

```bash
ls fuzz/artifacts/parse_record/
```

Each `crash-<sha1>` file is a standalone reproducer.

### Step 2 — Reproduce locally with a backtrace

```bash
RUST_BACKTRACE=1 cargo +nightly fuzz run parse_record \
    fuzz/artifacts/parse_record/crash-<sha1>
```

Three possible outcomes:

1. **Rust panic.** Look for `thread '<unnamed>' panicked at ...` followed
   by stack frames. The deepest `src/...` frame is the first suspect.
2. **libFuzzer OOM / timeout.** Look for `ERROR: libFuzzer:
   out-of-memory` or `timeout`. The input size and the slowest loop in
   the hot parse path are the suspects.
3. **No crash.** Re-run twice more. If it never reproduces, the finding
   may be platform-specific or timing-sensitive. Skip ahead to step 8
   and file a bead with the failing CI run URL in the description (the
   artifact is retrievable from the Actions UI for 30 days). Do not
   silently discard; do not proceed through steps 4-7 since there is
   nothing to minimize or regression-test.

### Step 3 — Classify the finding

| Symptom | Meaning | First suspect |
|---------|---------|----------------|
| `thread '...' panicked at src/...` | Unchecked indexing, unwrap, arithmetic overflow, slice bounds | Deepest `src/` frame in the backtrace |
| `thread '...' panicked at <dep>/...` | A dependency panics on an input shape we should have rejected earlier | Our caller of the dep; fix by validating the input before the call, not by wrapping the panic |
| `libFuzzer: out-of-memory` | Unbounded allocation fed by input-controlled length | Allocation sites in the hot parse path; directory-length and record-length fields |
| `libFuzzer: timeout` | Infinite loop or super-linear algorithm | Loops over input-controlled counters / offsets; fallthrough branches that never advance the cursor |
| Doesn't reproduce | Non-determinism | File anyway (see step 2 outcome 3) |

### Step 4 — Minimize the reproducer

```bash
cargo +nightly fuzz tmin parse_record \
    fuzz/artifacts/parse_record/crash-<sha1>
```

The minimized file lands in `fuzz/artifacts/parse_record/` — exact
filename varies by cargo-fuzz version (typically starts with
`minimized-from-` or is the smallest new file). List the directory to
find it, then verify it still reproduces the same crash (step 2 again
on the minimized file).

### Step 5 — Write the regression test FIRST

Test-driven: confirm the reproducer fails before fixing, so the fix has a
witness.

Copy the minimized file into the regressions tree. Pick a descriptive
slug (`truncated-leader-panic`, `zero-length-directory-oom`,
`indicator-byte-underflow`) — never reuse the sha1 filename, it is not
readable. Binary content: use `cp`, not a heredoc.

```bash
mkdir -p tests/data/fuzz-regressions/parse_record
cp fuzz/artifacts/parse_record/<minimized-filename> \
   tests/data/fuzz-regressions/parse_record/<short-slug>.mrc
```

If `tests/fuzz_regressions.rs` does not yet exist, create it with the
harness pattern in [Regression test harness](#regression-test-harness)
below. Within a given target, the harness auto-discovers every fixture
under `tests/data/fuzz-regressions/<target>/` — no test-code edits
needed for subsequent fixtures **in that target**. Adding a finding for
a new target (the first `parse_leader` regression, for example)
requires a new `#[test]` function pointing at that target's fixture
directory; see the harness comment.

Run the test and confirm it **fails**:

```bash
cargo test --package mrrc --test fuzz_regressions
```

### Step 6 — Fix the bug

- Navigate to the panic site from the step-2 backtrace.
- Replace the panicking operation with a recoverable one:
  - `arr[i]` → `arr.get(i).ok_or_else(|| ctx.err_...())?`
  - `x.unwrap()` → `x.ok_or_else(|| ...)?` or `x?`
  - `a - b` → `a.checked_sub(b).ok_or_else(|| ...)?`
- Return `Err(MarcError)` with positional context. The `ctx.err_*`
  helpers live on `ParseContext` in `src/iso2709.rs` — each builds a
  specific `MarcError` variant with stream position, record index,
  byte offset, and (where available) the 001 control number
  auto-populated. Reach for `ctx.err_directory_invalid(...)`,
  `ctx.err_record_length_invalid(...)`, etc., rather than constructing
  `MarcError` variants by hand.
- Do not silently swallow. `Err` returns on malformed input are correct
  behavior; panics are not.

### Step 7 — Verify the fix

```bash
# Regression test now passes
cargo test --package mrrc --test fuzz_regressions

# Nothing else regressed
.cargo/check.sh --quick

# The fuzzer no longer finds this crash (60-second smoke)
cargo +nightly fuzz run parse_record -- -max_total_time=60
```

### Step 8 — File a bead and open the PR

```bash
br create "Fix <short description> in <module>" -t bug -p 2 -d "...description..."
```

Bead description template:

```
## Summary
<one sentence: what panicked and where>

## Reproducer
Regression test: tests/data/fuzz-regressions/parse_record/<slug>.mrc
Original CI run: <URL from step 1>

## Root cause
<one to two sentences>

## Fix
<one sentence>
```

Branch: `fix/fuzz-<slug>`. One finding per PR unless the root cause is
literally identical across multiple artifacts. CHANGELOG entry under
`### Fixed` in `[Unreleased]` citing the bead ID and the CI run URL;
the `[Unreleased]` block must keep Keep-a-Changelog ordering (Breaking,
Added, Changed, Deprecated, Removed, Fixed, Security, Dependencies) —
`scripts/lint-changelog.sh` fails the commit if `### Fixed` appears
before any `### Added` section.

Only close the bead after CI is green on all platforms.

### Regression test harness

If `tests/fuzz_regressions.rs` does not yet exist, the first
crash-finding PR creates it with this pattern. Subsequent fixtures are
added as a single-file change — no test-code edits.

```rust
// tests/fuzz_regressions.rs
// Regression tests for bugs found by coverage-guided fuzzing. Each
// fixture under tests/data/fuzz-regressions/<target>/ is a minimized
// reproducer committed to guard against reintroduction on every PR.
//
// Adding a fixture for an existing target is a single-file change — the
// per-target test function below auto-discovers any new fixture.
//
// Adding a fixture for a NEW target requires adding a new #[test]
// function that mirrors `parse_record_regressions` but calls the
// appropriate public API (e.g., the writer path for roundtrip_binary,
// or the MARCXML reader for parse_marcxml).

use mrrc::MarcReader;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

fn fixtures_dir(target: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/fuzz-regressions")
        .join(target)
}

#[test]
fn parse_record_regressions() {
    let dir = fixtures_dir("parse_record");
    if !dir.exists() {
        return; // No regressions filed yet.
    }
    for entry in fs::read_dir(&dir).expect("read fuzz-regressions dir") {
        let path = entry.expect("dir entry").path();
        if !path.is_file() {
            continue;
        }
        let bytes = fs::read(&path).expect("read fixture");
        // Err returns on malformed input are correct; only panics,
        // OOMs, and timeouts would be regressions. A panic inside
        // read_record unwinds and fails the test.
        let mut reader = MarcReader::new(Cursor::new(&bytes[..]));
        loop {
            match reader.read_record() {
                Ok(Some(_)) => continue,
                Ok(None) | Err(_) => break,
            }
        }
    }
}
```

### What NOT to do

- **Never change the fuzz harness to avoid the crash.** If the harness
  is wrong (e.g., unwrapping a Result it should discard), that is a
  separate bug tracked as its own PR — not a way to silence a finding.
- **Never commit `fuzz/artifacts/*`.** It is gitignored. The permanent
  record is the fixture under `tests/data/fuzz-regressions/`.
- **Never `unwrap_or_default()` your way around it.** A targeted fix
  that returns a default value instead of `Err(MarcError)` masks the
  original bug — legitimate errors start being silently swallowed.
- **Never skip the regression test.** Fixing the bug without a test
  means the same crash can regress silently on the next refactor.
- **Never close the bead before CI is green on every platform.** Local
  check.sh passing is necessary but not sufficient.
- **Never write artifacts or test data to `/tmp` or outside the repo
  tree.** Triage belongs inside the repo so the reproducer, test, and
  fix all live together and survive a session ending.

## CI

The nightly fuzz job lives at `.github/workflows/fuzz.yml`. It:

- Runs daily at 03:00 UTC (offset from the 02:00 memory-safety ASAN job
  so they do not contend for cache).
- Can be triggered on demand via `workflow_dispatch`, with an optional
  `max_total_time` input for longer runs.
- Fails the job on any finding (crash, OOM, or timeout).
- Uploads `fuzz/artifacts/` as a workflow artifact on failure, with 30
  days of retention.
- Does **not** auto-file issues. GitHub's default scheduled-workflow
  failure email plus the red mark on the Actions tab are the
  notifications; triage is manual (see the playbook above).

This is not a PR gate. Fuzzing is inherently open-ended and unreliable
as a blocking check — a flaky random finding should not block a feature
merge. Regressions from fuzz findings live in
`tests/data/fuzz-regressions/` and run on every PR as regular
integration tests via `tests/fuzz_regressions.rs`.

## Why not `cargo fuzz` on stable?

`libfuzzer-sys` uses LLVM's SanitizerCoverage instrumentation, which is
exposed through nightly-only `-C passes=sancov-*` rustc flags. There is
no stable equivalent today. The standalone `fuzz/` workspace with its own
nightly pin isolates this constraint so the rest of the repo stays on
stable 1.95.0.

## Related work

- [Formal Methods](formal-methods.md) — primer on the property-based
  tests (`tests/properties.rs`) that sit underneath fuzzing in the
  5-level verification pyramid; covers the pyramid framing, the
  regression-seed policy, and the relationship to the broader
  mrrc-testbed verification strategy.
- `.github/workflows/memory-safety.yml` — nightly ASAN run, complementary
  to fuzzing (ASAN instruments the test suite; fuzzing instruments a
  dedicated harness).
