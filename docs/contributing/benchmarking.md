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

| Target | Covers |
|---|---|
| `marc_benchmarks` | ISO 2709 reading of every fixture below (and of the 1k and 10k fixtures from a path), writing, tag lookups (`get_field`, `fields_by_tag`, `get_control_field`), JSON and MARCXML serialization, MARCXML parsing |
| `format_benchmarks` | CSV, MARC-in-JSON, MODS, Dublin Core, and BIBFRAME conversion, including MARCXML, MODS, Dublin Core, and BIBFRAME conversion of the realistic and large-record fixtures; the single-thread parser-pool path |
| `error_handling_benchmarks` | Strict parsing of clean records and lenient parsing of malformed ones |
| `parallel_benchmarks` | Multi-threaded reading, wall-clock only (see [Parallel throughput](#parallel-throughput-wall-clock)) |
| `memory_profile` | Heap per record; prints a table rather than timing (see [Heap per record](#heap-per-record)) |

The benches read fixtures from `tests/data/fixtures/`, built by the scripts
in `scripts/` named below:

| Fixture | Records | Shape | Generator |
|---|---|---|---|
| `1k_records.mrc`, `10k_records.mrc` | 1,000 and 10,000 | About 6 fields of short text per record | `generate_benchmark_fixtures.py` |
| `5k_records.mrc` | 5,000 | The same shape | None (committed as is) |
| `realistic.mrc` | 2,000 | 12-30 fields with real-length text | `generate_realistic_fixture.py` |
| `realistic_marc8.mrc` | 1,000 | Realistic records in MARC-8 (leader position 09 blank) with ANSEL diacritics and Cyrillic and Greek 880 fields | `generate_marc8_fixture.py` |
| `large_records.mrc` | 100 | 150-250 fields; repeated tags interleaved within the 5XX, 6XX, and 7XX blocks | `generate_large_fixture.py` |
| `very_large_records.mrc` | 10 | 800-1,000 fields, interleaved the same way | `generate_large_fixture.py --preset very-large` |

The realistic, MARC-8, and large-record generators use a fixed seed, so
re-running one reproduces its committed file byte for byte. The MARC-8
generator encodes with pymarc's MARC-8 tables and stops unless pymarc decodes
every value back unchanged.

## Python pytest-benchmark suites

Located under `tests/python/test_benchmark_*.py`. Run with
`uv run python -m pytest tests/python/ --benchmark-only -m benchmark`. These
exercise the FFI surface and surface cross-binding overhead.
`test_benchmark_field_access.py` times `record["245"]`,
`record.get_fields("650")`, and the field-handle loop
`for f in record.get_fields(): f["a"]` over the realistic and large-record
fixtures.

`test_memory_benchmarks.py` measures peak memory with `tracemalloc`, which
sees allocations made through Python's allocator (the record and field
wrapper objects) but not the Rust heap behind each record. `memory_profile`
measures that.

## Codspeed regression detection

The `marc_benchmarks`, `error_handling_benchmarks`, and `format_benchmarks`
targets and the Python benchmarks (except the parallel ones) run under
`cargo codspeed` and `pytest-codspeed` in CI, as two jobs of one workflow
(`.github/workflows/benchmarks.yml`): Codspeed combines the results of a
workflow run into one report, but not results from separate workflows. Both
jobs run in simulation mode, which counts instructions under Valgrind and so
varies far less between runs than wall-clock time. Codspeed compares each PR
against the project's main branch and reports deltas in a PR comment and on
the dashboard. It does not block merges. For a
change to a hot path, read its deltas together with a local
[before/after comparison](#comparing-before-and-after-a-change), which also
covers the parallel and heap measurements that Codspeed does not run.

## Heap per record

`cargo bench --bench memory_profile` reads each fixture with a counting global
allocator and prints, per record: the heap a parsed record owns (not counting
the `Record` struct itself, whose size it prints first), the number of live
allocations behind it, and the allocation calls made while reading it. For a
given build the table is the same on every run, so any difference between two
builds is real.

## Comparing before and after a change

For a change to a hot path (record storage, the ISO 2709 codec, a
converter), compare the whole benchmark profile on the change's base commit
and on the branch: release builds, the same machine, run back to back.

```bash
# On the base commit
cargo bench --bench marc_benchmarks --bench format_benchmarks \
    --bench error_handling_benchmarks -- --save-baseline before
cargo bench --bench memory_profile > before-heap.txt
uv run maturin develop --release
uv run python -m pytest tests/python/ -m "benchmark and not slow" --benchmark-only \
    --ignore=tests/python/test_benchmark_parallel.py \
    --ignore=tests/python/test_benchmark_pipeline_parallel.py \
    --benchmark-warmup=on --benchmark-disable-gc --benchmark-min-rounds=30 \
    --benchmark-save=before

# On the branch: the same commands, comparing instead of saving
cargo bench --bench marc_benchmarks --bench format_benchmarks \
    --bench error_handling_benchmarks -- --baseline before
cargo bench --bench memory_profile > after-heap.txt
uv run maturin develop --release
uv run python -m pytest tests/python/ -m "benchmark and not slow" --benchmark-only \
    --ignore=tests/python/test_benchmark_parallel.py \
    --ignore=tests/python/test_benchmark_pipeline_parallel.py \
    --benchmark-warmup=on --benchmark-disable-gc --benchmark-min-rounds=30 \
    --benchmark-compare
```

Run `maturin develop --release` right before the Python benchmarks: the
`check.sh` commit and push hooks rebuild the extension in dev mode. For a
change that affects threading, also run `parallel_benchmarks` and
`scripts/parallel_throughput.py` on both builds.

**Know the noise floor first.** Wall-clock benchmarks on a laptop vary
between runs of the same code. Run the base twice, comparing the second run
against the first (`--baseline` for criterion, `--benchmark-compare` for
pytest-benchmark), and note each benchmark's spread. Most criterion benches
stay within about 1%, but a few allocation-heavy ones (MARC-in-JSON) move by
several percent, and the Python benchmarks vary more than the Rust ones even
with warm-up, garbage collection off, and 30 rounds, as above. criterion
reports every change with a confidence interval and says when a change is
within its noise threshold. A delta inside the noise floor is not evidence
either way; re-run it, or use Codspeed's instruction counts for the
benchmarks Codspeed covers, which include every Python benchmark above.

**Thresholds.** Beyond the noise floor, any benchmark slower by more than
2.5% needs an explanation in the PR, and a slowdown of more than 10% on a
user-facing path (Python reading, field access, a converter) is fixed before
the change merges. Put the comparison in the PR description: every delta over
2.5%, and the heap tables if record storage changed.

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

Worked example — Apple M4 (10 logical / 4 performance cores), Python 3.14,
release build, 10k fixture, 4 copies/thread, median of 7 runs:

| threads | records/s | speedup |
|--:|--:|--:|
| 1 | 430,999 | 1.00x |
| 2 | 631,165 | 1.46x |
| 3 | 622,760 | 1.44x |
| 4 | 615,328 | 1.43x |
| 6 | 574,051 | 1.33x |
| 8 | 560,982 | 1.30x |
| 10 | 532,438 | 1.24x |

**Interpreting the curve.** Speedup peaks near the performance-core count and
then declines. `MARCReader.__next__` releases the GIL for the batch parse but
re-acquires it to hand each record back to Python, so a workload that
materializes every record in Python (as this one does) serializes on that
handoff — capping the gain well below linear. Workloads that do more
Rust-side work per GIL crossing scale better; treat this curve as the floor,
not the ceiling, and re-run it on the target deployment hardware for a
representative number.

**Free-threaded CPython lifts that ceiling.** The cap is the GIL, not the
algorithm. Rebuilding the same extension for a free-threaded interpreter
(3.14t) and re-running on the same machine, the speedup against the 3.14
baseline above climbs to a ~2.4x plateau instead of stalling near ~1.5x —
each column is relative to its own single-thread baseline:

| threads | 3.14 GIL on | 3.14t GIL off |
|--:|--:|--:|
| 1 | 1.00x | 1.00x |
| 2 | 1.46x | 1.44x |
| 4 | 1.43x | 2.24x |
| 8 | 1.30x | 2.39x |

That is not free, though: the no-GIL build starts ~23% slower single-threaded
(331,516 vs 430,999 records/sec — the biased-reference-counting tax), so in
absolute throughput it only pulls ahead past ~3 threads. Free-threading is not
built into the shipped wheels; to measure it, declare the module
free-thread-safe with `#[pymodule(gil_used = false)]` and build against a
free-threaded interpreter (`uv python install 3.14t`). PyO3 0.29 supports
free-threading only on Python 3.14+, so 3.13t will not build.

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
| `cargo bench --bench memory_profile` | heap use per record | local only | memory comparisons |
| before/after comparison | whole-profile drift from one change | local, alongside Codspeed | merging a hot-path change |
| `parallel_throughput.py` | multi-thread GIL-release scaling | local only (wall-clock) | concurrency claims |
