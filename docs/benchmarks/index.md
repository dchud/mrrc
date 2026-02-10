# Benchmarking

This directory contains benchmarking documentation, infrastructure, and results.

## Contents

- [Results](results.md) - Detailed performance measurements and comparisons
- [FAQ](faq.md) - Common questions about performance and threading
- [Benchmark Scripts](../../scripts/) - `benchmark_comparison.py` and `criterion_extractor.py`
- [Rust Benchmarks](../../benches/) - Criterion.rs source in `benches/marc_benchmarks.rs`

Related documentation:
- [Threading Guide](../guides/threading-python.md) - GIL release strategy and threading patterns
- [Performance Tuning](../guides/performance-tuning.md) - Usage patterns and optimization

## Overview

mrrc performance is evaluated across three implementations:

1. **Rust (mrrc)** - Pure Rust library (baseline)
2. **Python (pymrrc)** - PyO3-based Python wrapper
3. **Pure Python (pymarc)** - Baseline Python library (for comparison)

### Summary

**Single-threaded performance (default behavior, after warm-up):**
- Rust: ~1,000,000 rec/s (baseline)
- Python wrapper (pymrrc): ~300,000 rec/s (~30% of Rust, ~4x faster than pymarc)
- Pure Python (pymarc): ~70,000 rec/s

**Multi-threaded performance (explicit opt-in):**
- Requires `concurrent.futures.ThreadPoolExecutor` or `ProducerConsumerPipeline`
- 2-thread speedup: ~2x vs sequential
- 4-thread speedup: ~3-4x vs sequential
- Each thread needs its own `MARCReader` instance
- GIL released during parsing in each thread

**Methodology:** Benchmarks use pytest-benchmark which performs warm-up iterations to stabilize measurements. Cold-start performance is ~20% slower due to JIT/caching effects.

See [results.md](results.md) for detailed measurements and [threading-python.md](../guides/threading-python.md) for threading guidance.

## Benchmark Infrastructure

### Test Systems

| System | Framework | Location | Notes |
|--------|-----------|----------|-------|
| Rust | Criterion.rs | `benches/marc_benchmarks.rs` | Baseline |
| Python | pytest-benchmark | `tests/python/test_benchmark*.py` | PyO3 wrapper (~10-15% overhead) |
| Comparison | Custom script | `scripts/benchmark_comparison.py` | Caching + CI-aware |

### Running Benchmarks

```bash
# Rust benchmarks
cargo bench --release

# Python benchmarks
pytest tests/python/ --benchmark-only -v

# Three-way comparison (requires pymarc)
pip install pymarc
python scripts/benchmark_comparison.py

# Check benchmark cache status
python scripts/criterion_extractor.py

# CI-mode
CI=1 python scripts/benchmark_comparison.py
```

## Caching and Staleness Detection

The benchmark infrastructure includes:

- **Caching**: Criterion.rs results parsed from `target/criterion/` (~100ms, no recompilation)
- **Staleness detection**: Auto-detects if benchmarks are >24h old or source changed; warns to refresh with `cargo bench --release`
- **CI optimization**: Detects CI environment and runs reduced test suite (1k, 10k)

## Test Fixtures

Located in `tests/data/fixtures/`:
- `1k_records.mrc` (257 KB) - Quick tests
- `10k_records.mrc` (2.5 MB) - Standard benchmarks
