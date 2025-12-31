# Benchmarking MRRC

This directory contains benchmarking infrastructure and results for MRRC.

## Overview

MRRC's performance is evaluated across multiple dimensions:

1. **Pure Rust library** (`mrrc`) - High-performance baseline
2. **Python wrapper** (`mrrc` Python package) - PyO3/Maturin bindings to Rust
3. **pymarc comparison** - Performance vs. pure Python `pymarc` library

## Benchmark Types

### 1. Rust Benchmarks (`cargo bench`)

Located in `benches/marc_benchmarks.rs` and executed with `cargo bench --release`.

Measures raw Rust performance (baseline):
- Record parsing (1k, 10k, 100k records)
- Field access patterns
- Writing/serialization (MARC21, JSON, XML)
- Round-trip operations (read + write)

**No PyO3 overhead; measures pure Rust performance.**

### 2. Python Wrapper Benchmarks (`pytest-benchmark`)

Located in `tests/python/` and executed with `pytest tests/python/ --benchmark-only`.

Measures Python wrapper performance (PyO3 overhead included):
- Record reading via `MARCReader` (1k, 10k, 100k records)
- Record writing via `MARCWriter`
- Field extraction patterns (title, author, subjects)
- Round-trip read/write cycles
- Streaming patterns

**Includes PyO3 wrapper overhead (expected 10-15%).**

### 3. Comparative Benchmarks (`scripts/benchmark_comparison.py`)

Compares `mrrc` (Python wrapper) against `pymarc`:
- Pure reading performance
- Field extraction (common workflow)
- Memory usage patterns

## Running Benchmarks

### Quick benchmark (1k/10k records, skip 100k)
```bash
# Fast Python benchmarks (used in CI)
pytest tests/python/ -v --benchmark-only -m "not slow"
```

### Full benchmark suite (1k, 10k, 100k records - local only)
```bash
# All Python benchmarks including slow 100k tests
pytest tests/python/ -v --benchmark-only
```

### Rust benchmarks (baseline performance)
```bash
# Raw Rust performance (no wrapper overhead)
cargo bench --release
```

### Three-tier comparison (requires pymarc)
```bash
# Install dependencies
pip install pymarc

# Run comparison: Rust vs Python wrapper vs pymarc
cargo bench --release
pytest tests/python/ -v --benchmark-only -m "not slow"
python scripts/benchmark_comparison.py

# View detailed analysis in docs/benchmarks/RESULTS.md
```

## Benchmark Fixtures

Test fixtures in `tests/data/fixtures/`:
- `1k_records.mrc` (0.25 MB) - Quick tests
- `10k_records.mrc` (2.5 MB) - Standard benchmarks
- `100k_records.mrc` (25 MB) - Comprehensive tests

Generate new fixtures:
```bash
python scripts/generate_benchmark_fixtures.py
```

## Results & Historical Tracking

### Current Results

See `.benchmarks/comparison.json` for latest comparative results.

### CI Integration

GitHub Actions runs benchmarks on:
- **Python benchmarks** - All pushes to main, all PRs
- **Codspeed** - Optional continuous performance regression detection (requires CODSPEED_TOKEN secret)

View results in GitHub Actions artifacts for each run.

## Performance Expectations

### Three-Tier Performance

```
Rust (mrrc)       ~ 70,000 rec/s    (baseline - no overhead)
                     ↓ +13-15% overhead
Python (pymrrc)   ~ 60,000 rec/s    (wrapper adds PyO3 cost)
Python (pymarc)   ~ 70,000 rec/s    (pure Python reference)
```

**Key Insights:**
- Rust and pure Python (pymarc) perform similarly (~70k rec/s)
- PyO3 wrapper adds 13-15% overhead (object allocation, type conversion)
- Overhead is **consistent and acceptable** for production use
- GIL release in I/O operations enables multi-core parallelism

### Trade-offs by Library

| Library | Speed | Memory Safety | Maintainability | Format Support |
|---|---|---|---|---|
| Rust (mrrc) | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ (Rust) | ⭐⭐⭐⭐⭐ |
| Python (pymrrc) | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Pure Python (pymarc) | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |

## Interpreting Results

Benchmark output from pytest-benchmark:

```
test_read_1k_records          PASSED [ 16.42 ms/iter ]
test_read_10k_records         PASSED [ 161.02 ms/iter ]
```

This means:
- Reading 1,000 records takes ~16.4 milliseconds
- Throughput: ~61,000 records/second
- Reading 10,000 records takes ~161 milliseconds
- Throughput: ~62,000 records/second

## Performance Tuning

### For CI/CD

To speed up CI benchmarks:
- Use `pytest tests/python/ -m "not slow"` to skip 100k record tests
- Set `benchmark_min_rounds = 1` in `pyproject.toml` for rapid iteration
- Use `--benchmark-skip` to disable benchmarks in regular test runs

### For Development

Build with optimizations:
```bash
# Development build (unoptimized, fast compilation)
maturin develop

# Release build (optimized, slow compilation)
maturin develop --release
```

Benchmarks automatically use the installed wheel, so release builds are faster.

## Regression Detection

### Manual Detection

Compare against a baseline:
```bash
pytest tests/python/ --benchmark-json=baseline.json --benchmark-only
# ... make changes ...
pytest tests/python/ --benchmark-json=current.json --benchmark-only
pytest-benchmark compare baseline.json current.json
```

### Continuous (Codspeed)

Set up Codspeed for automatic regression detection:

1. Create account at https://codspeed.io
2. Add `CODSPEED_TOKEN` secret to GitHub repo
3. Codspeed runs on each PR and main branch push
4. Detects regressions >5% vs main

## Adding New Benchmarks

1. Create test in `tests/python/test_benchmark_*.py`
2. Use `fixture_*` from `conftest.py`
3. Mark with `@pytest.mark.benchmark`
4. Mark slow tests with `@pytest.mark.slow`

Example:
```python
@pytest.mark.benchmark
def test_my_benchmark(self, benchmark, fixture_1k):
    def work():
        reader = MARCReader(io.BytesIO(fixture_1k))
        # ... do something ...
    
    benchmark(work)
```

## References

- [pytest-benchmark Documentation](https://pytest-benchmark.readthedocs.io/)
- [Codspeed Documentation](https://codspeed.io/docs/)
- [Rust benchmarking best practices](https://docs.rs/criterion/latest/criterion/)
