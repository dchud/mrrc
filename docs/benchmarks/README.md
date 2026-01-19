# MRRC Benchmarking

This directory contains benchmarking documentation, infrastructure, and results.

## Quick Links

- **[RESULTS](RESULTS.md)** - Detailed performance measurements and all four-way comparisons
- **[FAQ](FAQ.md)** - Quick Q&A: "Is it faster?", "Do I need threading?", "How much speedup?"
- **[BENCHMARKING GUIDE](../../BENCHMARKING.md)** - How to run benchmarks, caching system, CI integration
- **[Benchmark Scripts](../../scripts/)** - `benchmark_comparison.py` and `criterion_extractor.py`
- **[Rust Benchmarks](../../benches/)** - Criterion.rs source in `benches/marc_benchmarks.rs`

**For design details:** See [docs/ARCHITECTURE.md](../ARCHITECTURE.md) for GIL release strategy and [docs/PERFORMANCE.md](../PERFORMANCE.md) for usage patterns.

## Overview

MRRC's performance is evaluated across three implementations:

1. **Rust (mrrc)** - Pure Rust library (maximum performance baseline)
2. **Python (pymrrc)** - PyO3-based Python wrapper (production wrapper)
3. **Pure Python (pymarc)** - Baseline Python library (for comparison)

### Key Findings

**1. Python wrapper is ~4x faster than pymarc (single-threaded, default behavior, after warm-up):**
- Rust: ~1,000,000 rec/s (baseline)
- Python wrapper (pymrrc): ~300,000 rec/s (after warm-up; ~30% of Rust, ~4x faster than pymarc)
- Pure Python (pymarc): ~72,700 rec/s
- **GIL is released automatically during record parsing** — no code changes needed
- **Cold-start:** ~20% slower due to JIT/caching, but warms up automatically in real workloads

**2. Multi-threaded parallelism with explicit concurrency (opt-in):**
- Requires: Use `concurrent.futures.ThreadPoolExecutor` to spawn threads
- 2-thread speedup: ~2.0x vs sequential processing
- 4-thread speedup: ~3.74x vs sequential processing
- Each thread needs its own `MARCReader` instance
- GIL released during parsing in each thread simultaneously

**Key Difference:** Single-threaded pymrrc (default) automatically gets faster parsing via GIL release but stays single-threaded. Multi-threaded (explicit) requires ThreadPoolExecutor and gives linear scaling on multi-core systems.

**Methodology Note:** Benchmarks use pytest-benchmark which automatically performs warm-up iterations to stabilize measurements. The ~300k rec/s warm-up number is what matters for real-world performance; cold-start measurements are ~20% slower.

See [RESULTS.md](RESULTS.md) for detailed measurements and [docs/PERFORMANCE.md](../PERFORMANCE.md) for threading guidance.

## Benchmark Infrastructure

### Three Test Systems

| System | Framework | Location | Overhead |
|--------|-----------|----------|----------|
| **Rust** | Criterion.rs | `benches/marc_benchmarks.rs` | None (baseline) |
| **Python** | pytest-benchmark | `tests/python/test_benchmark*.py` | PyO3 wrapper (~10-15%) |
| **Comparison** | Custom Python script | `scripts/benchmark_comparison.py` | Caching + CI-aware |

### Quick Start

```bash
# Generate fresh Rust benchmarks and run all comparisons
cargo bench --release
python3 scripts/benchmark_comparison.py

# Check benchmark cache status
python3 scripts/criterion_extractor.py

# Run fast CI-mode tests (skips slow 100k)
CI=1 python3 scripts/benchmark_comparison.py
```

## Key Features (New in 2025-12-31)

### ✅ Fast Caching
- Criterion.rs results parsed from `target/criterion/` (~100ms, no recompilation)
- Benchmark comparison runs instantly with cached data

### ✅ Staleness Detection
- Auto-detects if benchmarks are >24h old or source changed
- Warns users to refresh with `cargo bench --release`

### ✅ CI Optimization
- Detects CI environment (GitHub Actions, GitLab, CircleCI, etc.)
- Full suite locally: 1k, 10k, 100k records
- Fast CI: 1k, 10k only (saves ~4.5 minutes per run)

## Test Fixtures

Located in `tests/data/fixtures/`:
- `1k_records.mrc` (257 KB) - Quick tests
- `10k_records.mrc` (2.5 MB) - Standard benchmarks
- `100k_records.mrc` (25 MB) - Comprehensive tests (local-only)

## Running Benchmarks

See [BENCHMARKING.md](../../BENCHMARKING.md) for complete workflows.

### Quick Commands

```bash
# Rust benchmarks (baseline)
cargo bench --release

# Python wrapper benchmarks
pytest tests/python/ --benchmark-only -v

# Three-way comparison (requires pymarc)
pip install pymarc
python scripts/benchmark_comparison.py
```

## Next Steps

- See [BENCHMARKING.md](../../BENCHMARKING.md) for detailed guides on workflows, caching, and CI integration
- See [RESULTS.md](RESULTS.md) for detailed performance analysis and measurements
- See [Rust benchmarks](../../benches/marc_benchmarks.rs) for Criterion.rs implementation
- Contribute new benchmarks in `tests/python/test_benchmark_*.py`
