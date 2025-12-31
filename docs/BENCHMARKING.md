# Benchmarking Guide

This document explains how to run and interpret benchmarks for the MRRC project.

## Overview

The MRRC benchmarking system compares performance across three implementations:

1. **Rust (mrrc)** - Pure Rust library (maximum performance baseline)
2. **Python (pymrrc)** - PyO3-based Python wrapper (production wrapper)
3. **Pure Python (pymarc)** - Baseline Python library (for comparison)

## Quick Start

### Run Local Benchmarks (Full Suite)

```bash
# Build Rust benchmarks and cache results
cargo bench --release

# Run comprehensive Python comparison
python3 scripts/benchmark_comparison.py
```

This generates Criterion.rs results and compares all three implementations across:
- 1k records (fast)
- 10k records (moderate)
- 100k records (comprehensive, local-only)

### Run CI-Mode Benchmarks (Fast)

```bash
# Simulates CI environment (skips 100k)
CI=1 python3 scripts/benchmark_comparison.py
```

Useful for quick validation without the 100k test.

## Architecture

### Criterion.rs Benchmarks (`benches/marc_benchmarks.rs`)

- Provides the Rust baseline
- Results cached in `target/criterion/`
- Statistical analysis: 100+ samples per benchmark
- Tests: read, field access, serialization, round-trip

### Python Comparison Script (`scripts/benchmark_comparison.py`)

- Compares pymrrc vs pymarc across common operations
- Extracts Rust results from cached Criterion data
- Outputs results to `.benchmarks/comparison.json`
- Detects CI environment and adjusts test scope

### Criterion Result Extractor (`scripts/criterion_extractor.py`)

- Parses Criterion.rs JSON output (fast, no re-compilation)
- Detects stale benchmarks (age or source changes)
- Provides caching summary and staleness detection
- Can be used standalone to inspect Criterion results

## How Results Are Cached

### Rust Benchmarks

Criterion.rs stores results in `target/criterion/`:

```
target/criterion/
├── read_1k_records/
│   ├── new/
│   │   ├── estimates.json      ← Mean time in nanoseconds
│   │   ├── sample.json
│   │   └── ...
│   └── base/
│       ├── estimates.json
│       └── ...
├── read_10k_records/
│   └── ...
└── ...
```

**Key file:** `benchmark_name/base/estimates.json` contains `mean.point_estimate` in nanoseconds.

### Python Comparison Results

Results saved to `.benchmarks/comparison.json`:

```json
{
  "read_1k": {
    "python": {
      "speedup": 7.34,
      "pymrrc_mean_ms": 1.87,
      "pymarc_mean_ms": 13.76
    },
    "three_way": {
      "pymrrc_vs_pymarc": {...},
      "rust_vs_pymarc": {...}
    }
  }
}
```

## Detecting Stale Benchmarks

The `criterion_extractor.py` module detects staleness in two ways:

### 1. Age Detection

```python
extractor = CriterionExtractor()
is_stale = extractor.is_stale(max_age_hours=24)  # Default: 24 hours
```

### 2. Source File Detection

Automatically detects if benchmark source files (`benches/*.rs`) are newer than cached results.

### Check Status

```bash
python3 scripts/criterion_extractor.py
```

Output shows which benchmarks are available and if any are stale.

## CI Integration

### Environment Detection

The comparison script detects CI via environment variables:
- `CI`
- `GITHUB_ACTIONS`
- `GITLAB_CI`
- `CIRCLECI`
- `TRAVIS`

### CI Behavior

When CI is detected:
- ✅ Runs 1k and 10k benchmarks
- ✅ Runs Rust comparisons (using cached results)
- ❌ Skips 100k benchmark (too slow for CI)
- ❌ Skips regenerating Rust benchmarks (uses cache)

**Rationale:** 100k benchmarks take 2-5 minutes; cache-based comparison is instant.

### CI Configuration (GitHub Actions)

```yaml
- name: Run benchmarks
  run: python3 scripts/benchmark_comparison.py
  env:
    CI: true
```

## Common Workflows

### Workflow 1: Fresh Benchmarks Locally

```bash
# Generate latest Rust benchmarks
cargo bench --release

# Run full Python comparison
python3 scripts/benchmark_comparison.py

# Check cached results
python3 scripts/criterion_extractor.py
```

### Workflow 2: CI Validation

```bash
# Cache already exists from previous local run
# Script automatically uses cached results
CI=1 python3 scripts/benchmark_comparison.py
```

### Workflow 3: Detect Stale Benchmarks

```bash
# Check what's cached and if it's stale
python3 scripts/criterion_extractor.py

# Output:
# Cached Criterion.rs Benchmarks
# ======================================================================
# Found 8 benchmarks:
#   read_1k_records                          946 µs
#   read_10k_records                         9.42 ms
#   ...
```

### Workflow 4: Programmatic Access

```python
from scripts.criterion_extractor import CriterionExtractor

extractor = CriterionExtractor()

# Get single result (seconds)
rust_1k_time = extractor.get_benchmark_result("read_1k_records")

# Get all results
all_benchmarks = extractor.get_all_benchmarks()

# Check staleness
is_stale = extractor.is_stale(max_age_hours=12)

# Get summary
summary = extractor.cache_summary()
```

## Output Examples

### Criterion.rs Standalone

```bash
$ python3 scripts/criterion_extractor.py

Cached Criterion.rs Benchmarks
======================================================================

Found 8 benchmarks:

  read_1k_records                             946 µs
  read_10k_records                           9.42 ms
  read_1k_with_field_access                 946 µs
  read_10k_with_field_access                9.64 ms
  roundtrip_1k_records                       2.12 ms
  roundtrip_10k_records                      23.08 ms
  serialize_1k_to_json                       3.32 ms
  serialize_1k_to_xml                        3.99 ms
```

### Benchmark Comparison (Relevant Section)

```
Test 1: Reading 1,000 records
------

pymarc - Read records:
  Runs:    3
  Min:     13.24 ms
  Max:     14.06 ms
  Mean:    13.76 ms

mrrc - Read records:
  Runs:    3
  Min:     1.81 ms
  Max:     1.93 ms
  Mean:    1.87 ms

Python Comparison:
  pymarc: 13.76 ms
  mrrc:   1.87 ms
  Speedup: 7.3x faster

Three-Way Comparison:
  Rust (mrrc):    0.95 ms
  Python (pymrrc): 1.87 ms
  Pure Python (pymarc): 13.76 ms
  → pymrrc is 7.3x faster than pymarc
  → Rust is 14.5x faster than pymarc
  → Rust is 2.0x faster than pymrrc
```

## Performance Expectations

### Throughput (Records/Second)

| Implementation | 1k Records | 10k Records | 100k Records |
|---|---|---|---|
| Rust (mrrc) | 1,065,700 rec/s | 1,064,000 rec/s | 1,065,000 rec/s |
| Python (pymrrc) | 534,600 rec/s | 549,500 rec/s | ~537,600 rec/s |
| Pure Python (pymarc) | 72,700 rec/s | 72,600 rec/s | ~72,600 rec/s |

### Speedup

- **pymrrc vs pymarc:** 7.3-7.6x faster (consistent across all sizes)
- **Rust vs pymarc:** 14.6x faster
- **Rust vs pymrrc:** ~2.0x faster (PyO3 overhead minimal)

## Troubleshooting

### "No Criterion.rs cache found"

**Solution:** Run `cargo bench --release` to generate benchmarks.

### "Criterion.rs cache may be stale"

**Solutions:**
1. Run `cargo bench --release` to refresh
2. Or ignore warning if benchmarks haven't changed

### "100k fixture not available"

**Solution:** Run benchmark script locally (100k tests are skipped in CI).

### ImportError for criterion_extractor

**Solution:** Ensure `scripts/` is in Python path or use full path:

```bash
python3 -m scripts.criterion_extractor
```

## Future Enhancements

See tracking tickets:
- **mrrc-a6f**: Cache Criterion.rs results (DONE)
- **mrrc-syn**: Auto-detect stale benchmarks (DONE)
- **mrrc-qdq**: Skip 100k in CI (DONE)
- **mrrc-xbg**: Improved JSON structure for reports
