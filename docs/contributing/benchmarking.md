# Benchmarking

mrrc has three permanent benchmark layers. Each catches a different class of
regression and runs in a different place; the table at the bottom of this
page summarizes when to reach for which.

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

## Layer summary

| Layer | Catches | Runs | Authoritative for |
|---|---|---|---|
| `cargo bench` (any) | absolute Rust hot-path cost | local + Codspeed | per-PR signal, exploration |
| `pytest --benchmark-only` | FFI overhead | local + Codspeed | Python-binding regressions |
| Codspeed | broad PR-vs-main drift | CI dashboard | continuous awareness |

## Scoped perf gates

When a multi-PR effort needs a tighter, locked-baseline gate than Codspeed
provides (typically because per-PR cumulative drift matters for the cycle's
budget), we add an effort-scoped gate as a short-lived artifact rather than
generalizing this page. Each such gate ships with:

- a dedicated workflow under `.github/workflows/`
- a baseline JSON under `benches/baselines/`
- a comparison script under `scripts/`
- thresholds and methodology documented inline in those artifacts

Look there for the in-flight specifics. Generalizing one of these gates into
permanent infrastructure is a deliberate decision, not a default.
