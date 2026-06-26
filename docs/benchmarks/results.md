# Benchmark Results

## pymarc, the Python wrapper, and native Rust

mrrc has two front doors — a pymarc-compatible Python API and the native Rust
crate — so the honest comparison is three-way: the mature pure-Python library,
the wrapper a Python user actually calls, and the Rust ceiling underneath it.
All three are measured over the same records by
`scripts/benchmark_comparison.py` (the native column via
`examples/benchmark_native`). Every figure is **median records per second**
(rec/s); they come from a single host — an Apple M4 laptop, a working
development machine rather than a dedicated benchmark rig — so read them as
*relative* throughput, not absolute maxima; `read` in particular varies
run-to-run on a busy machine.

| Operation | pymarc (rec/s) | mrrc Python (rec/s) | mrrc Rust (rec/s) | Python vs pymarc | Rust vs pymarc |
|-----------|---------------:|--------------------:|------------------:|-----------------:|---------------:|
| `read` | ~32k | ~225k | ~289k | **~7×** | **~9×** |
| `read_bulk` | ~32k | ~977k | ~1.10M | **~31×** | **~35×** |
| `extract` | ~27k | ~44k | ~255k | **~1.6×** | **~9×** |
| `roundtrip` | ~21k | ~138k | ~184k | **~6.5×** | **~8.5×** |

- `read` — per-record iteration (`for r in reader`), no field access.
- `read_bulk` — mrrc's parallel `parse_batch_parallel` against pymarc's
  per-record read (pymarc has no batch equivalent); each library's fastest read.
- `extract` — `record.title` plus `field.value()` for every field.
- `roundtrip` — re-encode each record with `as_marc()`.

**Both speedup columns use the same pymarc baseline, but the two mrrc columns
do different work.** pymarc and mrrc (Python) both parse and hand back Python
objects, so *Python vs pymarc* is a like-for-like speedup. mrrc (Rust) parses to
Rust records and stops — it never builds Python objects — so *Rust vs pymarc* is
the native ceiling. The distance between the two multipliers (`~7×` vs `~9×` for
`read`) is the cost of the Python binding: crossing the PyO3 boundary and
materializing `Record`/`Field` objects, not extra parsing work.

That distance is the useful part. For `read`, `read_bulk`, and `roundtrip` the
two multipliers sit close together — the wrapper captures roughly 75–88% of the
native throughput, because field handles are lazy and the bulk path is a thin
shim over the same parallel Rust parse, so a Python user gives up little.
`extract` is the exception: touching every field's value crosses the boundary
per field, so the wrapper reaches only `~1.6×` against `~9×` native — pure Rust
pulls roughly 5–6× ahead. If per-field access dominates your workload and you
need the ceiling, that is the case for reaching for the Rust crate directly.

pymarc remains the reference implementation: mature, flexible, pure Python. The
multipliers here reflect native parsing and the absence of per-record Python
object construction, not a verdict on pymarc — for many Python codebases the
wrapper's drop-in compatibility with it is the whole point.

Context: Apple M4 (10 cores, 24 GiB), macOS 26.5, Python 3.14, rustc 1.95, a
**release** build of mrrc 0.9.0, pymarc 5.3.1, over a 2,000-record realistic
fixture (`tests/data/fixtures/realistic.mrc`, ~1.1 KB/record). The build profile
matters — a debug build is several times slower, so a release build
(`maturin develop --release`, or any published wheel) is required to reproduce
these.

### Reproducing this comparison

```bash
uv sync --all-extras                 # installs pymarc (the 'oracle' extra)
uv run maturin develop --release     # release build is essential
# regenerate the realistic fixture if needed:
uv run python scripts/generate_realistic_fixture.py
# run on as quiet a machine as you can manage:
.venv/bin/python scripts/benchmark_comparison.py \
    tests/data/fixtures/realistic.mrc --repeat 9 --output tmp/comparison-results.md
```

The harness builds and runs `examples/benchmark_native` (via `cargo run
--release`) for the native column; pass `--no-native` to drop it where a Rust
toolchain is unavailable.

The sections below describe what CI measures and the broader benchmarking setup.

## What CI measures

Two CodSpeed jobs run on every pull request, both in simulation mode:

- **Rust**: criterion benches (`benches/marc_benchmarks.rs`,
  `benches/error_handling_benchmarks.rs`) via
  `.github/workflows/benchmark-rust.yml`
- **Python**: pytest benchmarks (`tests/python/test_benchmark_reading.py`,
  `test_benchmark_writing.py`, `test_memory_benchmarks.py`) via
  `.github/workflows/benchmark-python.yml`

Simulation mode executes each benchmark once under Valgrind and models its
cost from instruction counts and cache behavior. The result is deterministic:
the same code produces the same number regardless of runner speed, which makes
it reliable for detecting regressions between commits. It is **not wall-clock
time** — simulation results cannot be quoted as records/second.

Parallel-throughput benchmarks are excluded from CI because Valgrind
serializes threads, so multi-threaded speedup cannot be measured under
simulation. Measure parallelism locally instead (below).

## Measuring locally

Local runs use real wall-clock time. For stable numbers: run on AC power, on
a quiet machine, and let the frameworks' warm-up and repeated rounds do their
work.

### Single-threaded

```bash
# Rust (criterion)
cargo bench --bench marc_benchmarks
cargo bench --bench error_handling_benchmarks

# Python (pytest-benchmark)
uv run maturin develop --release
uv run pytest tests/python/ -m "benchmark and not slow" --benchmark-only -v
```

### Parallel throughput

```bash
# Rust (criterion, rayon)
cargo bench --bench parallel_benchmarks

# Python (ThreadPoolExecutor and ProducerConsumerPipeline)
uv run pytest tests/python/test_benchmark_parallel.py \
    tests/python/test_benchmark_pipeline_parallel.py --benchmark-only -v
```

## Producing a citable comparison

Any published figure — especially a comparison against pymarc — must come
from a run that records:

- the date of the run
- hardware: CPU model, core count, memory
- OS name and version
- Rust toolchain version and Python version
- the exact, pinned version of every library measured (including pymarc)
- the harness used, committed to this repository
- the fixture data and its size

A multiplier without this context is not reproducible and does not belong in
the documentation.

## Test fixtures

Benchmark fixtures live in `tests/data/fixtures/`:

- `1k_records.mrc` (~257 KB) / `10k_records.mrc` (~2.5 MB) — tiny synthetic
  records (~6 fields each), from `scripts/generate_benchmark_fixtures.py`.
  Fine for regression detection, but too small for a fair speed comparison:
  per-record Python object overhead dominates and hides the parser.
- `realistic.mrc` (2,000 records, ~2.2 MB) — varied ~18-field / ~1.1 KB
  records, generated deterministically by
  `scripts/generate_realistic_fixture.py`. Used for the comparison above.

Synthetic fixtures are adequate for relative comparison. Figures intended to
describe real-world performance should also be measured against representative
library data (a real corpus).

## References

- Three-way comparison harness: `scripts/benchmark_comparison.py` (Python +
  pymarc) and `examples/benchmark_native.rs` (native Rust ceiling)
- Rust benchmarks: `benches/marc_benchmarks.rs`,
  `benches/error_handling_benchmarks.rs`, `benches/parallel_benchmarks.rs`
- Python benchmarks: `tests/python/test_benchmark_*.py`
- Memory benchmarks: `tests/python/test_memory_benchmarks.py`
- Fixture generator: `scripts/generate_benchmark_fixtures.py`
- CI workflows: `.github/workflows/benchmark-rust.yml`,
  `.github/workflows/benchmark-python.yml`
