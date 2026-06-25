# Benchmark Results

## mrrc vs pymarc (Python)

Measured with `scripts/benchmark_comparison.py` against a pinned pymarc over
the same records. These figures come from a single host — an Apple M4 MacBook
Air, a working development laptop rather than a dedicated benchmark rig — so
read them as the *relative* speedup, not an absolute maximum. The read figures
vary run-to-run on a busy machine; extract and roundtrip are steady.

| Operation | mrrc | pymarc | speedup |
|-----------|-----:|-------:|--------:|
| read — per-record (`for r in reader`) | ~190–255k rec/s | ~30–32k rec/s | **~6–8×** |
| read_bulk — `parse_batch_parallel` (parallel) | ~0.8–1.2M rec/s | ~32k rec/s | **~26–36×** |
| extract — title + every subfield value | ~44k rec/s | ~26k rec/s | **~1.7×** |
| roundtrip — `as_marc()` | ~140k rec/s | ~21k rec/s | **~7×** |

Context: Apple M4 (10 cores, 24 GiB), macOS 26.5, Python 3.14, rustc 1.95, a
**release** build of mrrc 0.8.x, pymarc 5.3.1, over a 2,000-record realistic
fixture (`tests/data/fixtures/realistic.mrc`, ~1.1 KB/record). `read_bulk` pits
mrrc's parallel batch parse against pymarc's per-record read — each library's
fastest read path; pymarc has no batch equivalent.

Two things to know: `read`/`read_bulk` lean hardest on the Rust parser, while
`extract` (the field-handle access path) is the relative weak spot. And the
build profile matters — a debug build is several times slower, so a release
build (`maturin develop --release`, or any published wheel) is required to
reproduce these.

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

- Rust benchmarks: `benches/marc_benchmarks.rs`,
  `benches/error_handling_benchmarks.rs`, `benches/parallel_benchmarks.rs`
- Python benchmarks: `tests/python/test_benchmark_*.py`
- Memory benchmarks: `tests/python/test_memory_benchmarks.py`
- Fixture generator: `scripts/generate_benchmark_fixtures.py`
- CI workflows: `.github/workflows/benchmark-rust.yml`,
  `.github/workflows/benchmark-python.yml`
