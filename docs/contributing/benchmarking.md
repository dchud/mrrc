# Benchmarking

mrrc has three permanent benchmark layers. Each catches a different class of
regression and runs in a different place; the table at the bottom of this
page summarizes when to reach for which.

These layers measure *how fast* the code is. To find out *where* the time
goes, see the [Profiling Guide](profiling.md), which covers local CPU
profiling of the same bench targets with `cargo flamegraph`.

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

## Parallel throughput (wall-clock)

The Codspeed gate runs in simulation mode (Valgrind instruction counts),
which serializes threads — so it cannot measure mrrc's headline concurrency
feature: releasing the GIL while parsing in pure Rust. That needs a real
wall-clock run on controlled hardware, which `scripts/parallel_throughput.py`
provides. Build a release extension first, then sweep thread counts:

```bash
uv run maturin develop --release
uv run python scripts/parallel_throughput.py
```

Each task parses an in-memory `bytes` copy of a fixture with `MARCReader`
(the CursorBackend path, which holds no GIL during parsing) across a
`ThreadPoolExecutor` of T = 1..cores workers; the script reports the median
records/sec and the speedup over the single-thread baseline.

**Record the hardware.** The script prints what the stdlib can see
(`platform`, logical-core count); add the physical CPU model, the
performance/efficiency core split, and RAM by hand. The numbers are
wall-clock and machine-specific — **not** a portable records/sec claim.

**Fixtures** are the synthetic `tests/data/fixtures/*.mrc` files (default
`10k_records.mrc`); they exercise parsing throughput, not real-world MARC
variety. Pass `--fixture` to change.

Worked example — Apple M4 (10 logical / 4 performance cores), release build,
10k fixture, 4 copies/thread, median of 7 runs:

| threads | records/s | speedup |
|--:|--:|--:|
| 1 | 459,454 | 1.00x |
| 2 | 663,510 | 1.44x |
| 3 | 668,586 | 1.46x |
| 4 | 653,850 | 1.42x |
| 6 | 637,955 | 1.39x |
| 8 | 624,790 | 1.36x |
| 10 | 600,506 | 1.31x |

**Interpreting the curve.** Speedup peaks near the performance-core count and
then declines. `MARCReader.__next__` releases the GIL for the batch parse but
re-acquires it to hand each record back to Python, so a workload that
materializes every record in Python (as this one does) serializes on that
handoff — capping the gain well below linear. Workloads that do more
Rust-side work per GIL crossing scale better; treat this curve as the floor,
not the ceiling, and re-run it on the target deployment hardware for a
representative number.

**Free-threaded CPython lifts that ceiling.** The cap is the GIL, not the
algorithm — re-run the same harness under a free-threaded interpreter and the
scaling roughly doubles. Measured on the same M4, Python 3.14 with the GIL
versus 3.14t without it (same version, so the GIL is the only variable):

| threads | 3.14 GIL on | 3.14t GIL off |
|--:|--:|--:|
| 1 | 1.00x | 1.00x |
| 2 | 1.41x | 1.71x |
| 4 | 1.30x | 2.33x |
| 8 | 1.21x | 2.39x |

Removing the GIL raises the plateau from ~1.4x to ~2.4x and stops the decline
past the performance-core count. It is not free, though: the no-GIL build
starts ~22% slower single-threaded (304,766 vs 388,643 records/sec — the
biased-reference-counting tax), so it only pulls ahead past ~3 threads.
Free-threading is not built into the shipped wheels; to measure it, declare
the module free-thread-safe with `#[pymodule(gil_used = false)]` and build
against a free-threaded interpreter (`uv python install 3.14t`). PyO3 0.29
supports free-threading only on Python 3.14+, so 3.13t will not build.

To confirm the GIL is actually released — a yes/no detector, not a throughput
number — run:

```bash
uv run python scripts/parallel_throughput.py --gil-check
```

One thread parses in a loop while a second spins a pure-Python counter; a high
counter total means the counter ran *during* the parses. (A "GIL held"
baseline would require reverting the GIL-release implementation, so treat this
as a sanity check that expects a high count.)

## Layer summary

| Layer | Catches | Runs | Authoritative for |
|---|---|---|---|
| `cargo bench` (any) | absolute Rust hot-path cost | local + Codspeed | per-PR signal, exploration |
| `pytest --benchmark-only` | FFI overhead | local + Codspeed | Python-binding regressions |
| Codspeed | broad PR-vs-main drift | CI dashboard | continuous awareness |
| `parallel_throughput.py` | multi-thread GIL-release scaling | local only (wall-clock) | concurrency claims |
