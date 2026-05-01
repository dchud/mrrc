# Benchmarking and the perf budget

mrrc has three benchmark layers. Each catches a different class of regression
and runs in a different place; the table at the bottom of this page summarizes
when to reach for which.

## Rust criterion benchmarks

Located in `benches/`. Run with `cargo bench --bench <name>`. Output lands
under `target/criterion/<scenario>/new/` as JSON estimates suitable for
programmatic comparison.

The benches rely on fixtures in `tests/data/fixtures/` (1k / 5k / 10k record
files; the 100k fixture is gitignored and regenerated locally with
`scripts/generate_benchmark_fixtures.py` when needed).

## Python pytest-benchmark suites

Located under `tests/python/test_benchmark_*.py`. Run with
`uv run python -m pytest tests/python/ --benchmark-only -m benchmark`. These
exercise the FFI surface and surface cross-binding overhead.

## Codspeed regression detection

Both the Rust and Python suites are exercised under `pytest-codspeed` /
`cargo codspeed` in the `Codspeed Performance Regression Detection` CI job.
Codspeed compares each PR against the project's main branch and reports
deltas on the dashboard. Use it as a generic "is anything weird" signal —
**not** as a hard gate, because cross-machine variance on the public CI
runner makes precise threshold enforcement (2-5% range) unreliable.

## The error-handling perf budget (bd-0x73 epic)

The bd-0x73 epic adds per-byte detection logic to the hot ISO 2709 parse
path. We track its cost as a cumulative budget against a v0.8.0-equivalent
baseline:

- **+2%** cumulative regression on any scenario → pause and assess the
  trade-off before merging
- **+5%** cumulative regression on any scenario → block until recovered or
  the trade-off is explicitly accepted (and the baseline file updated as a
  deliberate decision, not as a routine refresh)

Two tools enforce this budget; they answer different questions.

### Local cumulative-budget check

`scripts/check_error_handling_perf.py` runs the error-handling benches
locally and compares each scenario's mean against the numbers in
`benches/baselines/error_handling_v080.json`. Use this on the **same machine
class** that captured the committed baseline (Apple Silicon for the
file in this repo) before claiming a Phase B PR is ready. Cross-machine
comparison against a checked-in baseline is not statistically valid at
the 2%/5% threshold — that is what the CI workflow is for.

```sh
# Run benches and report deltas vs the checked-in baseline:
python scripts/check_error_handling_perf.py

# Re-use existing target/criterion output without re-running benches:
python scripts/check_error_handling_perf.py --no-bench
```

Exit codes: `0` clean, `1` warn (>2%), `2` fail (>5%), `3` missing output.

### CI same-runner before/after gate

`.github/workflows/error-handling-perf-gate.yml` runs on every PR that
touches `src/`, `src-python/`, the bench file, the comparison script, or
the workflow itself. The job:

1. Checks out PR HEAD on the runner; saves the bench file, the comparison
   script, and the baseline JSON to a temporary overlay.
2. Switches to `main` HEAD; restores the overlay so the bench definitions
   match across both runs; runs benches and saves
   `target/criterion → baseline-criterion`.
3. Switches back to PR HEAD; runs benches again; the new
   `target/criterion` reflects the PR's parser code.
4. Invokes the comparison script in `--baseline-criterion-dir` mode. Both
   measurements come from the same physical CI runner in the same workflow
   run, so the noise floor is single-digit percent and the 2%/5% threshold
   has real signal.

The gate's exit code propagates: warn fails the step at code 1, fail at
code 2.

## Refreshing the baseline

Update `benches/baselines/error_handling_v080.json` deliberately, not
opportunistically. Reasons that justify a refresh:

- An accepted +2-5% trade-off has shipped (record the rationale in the
  CHANGELOG entry that introduced it).
- A genuine improvement has shipped and the new lower numbers should be
  the basis for future Phase B comparisons.
- Hardware change (rare; document the new machine in the file's
  `machine` field).

To refresh: run `cargo bench --bench error_handling_benchmarks` on the
recorded machine class, then update the `mean_ns`, `std_dev_ns`, and
`median_ns` fields per scenario from
`target/criterion/<scenario>/new/estimates.json`. Bump the
`captured_against_commit` SHA and `captured_at` date.

## Layer summary

| Layer | Catches | Runs | Authoritative for |
|---|---|---|---|
| `cargo bench` (any) | absolute Rust hot-path cost | local + Codspeed | per-PR signal, exploration |
| `pytest --benchmark-only` | FFI overhead | local + Codspeed | Python-binding regressions |
| Codspeed | broad PR-vs-main drift | CI dashboard | continuous awareness |
| `scripts/check_error_handling_perf.py` | cumulative budget vs v0.8.0 | local (same-machine) | Phase B "ready to ship" |
| `error-handling-perf-gate.yml` | PR-vs-main on the same runner | CI gate | merge gate at 2%/5% |
