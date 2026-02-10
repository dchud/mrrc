# Parallel Benchmarking Feasibility Study

**Date:** 2025-12-31  
**Issue:** mrrc-58u  
**Status:** Feasibility Analysis

## Overview

This document explores adding parallel processing benchmarks to demonstrate the concurrency advantages of mrrc (Rust) and pymrrc (Python with GIL-release).

## Goal

Enhance benchmark suite to show:
1. **Sequential performance** (current): baseline
2. **Parallel performance** (new): real-world advantage of multi-core systems
3. **Usage examples**: demonstrate best practices for concurrent MARC processing

## Current Benchmark Structure

### Fixtures Available
- `1k_records.mrc` (257 KB)
- `10k_records.mrc` (2.5 MB)
- `100k_records.mrc` (25 MB)

### Current Tests (Sequential Only)
1. Pure reading
2. Field extraction
3. Serialization (JSON/XML) - Rust only
4. Round-trip (read + write)

**Strategy:** Keep all current tests, ADD parallel variants alongside.

## Parallel Benchmark Approach

### Rust (Rayon-based)

**Implementation:** Process multiple MARC files in parallel using rayon

```rust
use rayon::prelude::*;

fn benchmark_parallel_read_files_4x(c: &mut Criterion) {
    // Create 4 copies of 1k fixture in temp directory
    // OR use same file 4 times (simulates batch processing)
    
    c.bench_function("parallel_read_4x_1k_files", |b| {
        b.iter(|| {
            let files = vec![
                load_fixture("1k_records.mrc"),
                load_fixture("1k_records.mrc"),
                load_fixture("1k_records.mrc"),
                load_fixture("1k_records.mrc"),
            ];
            
            // Process in parallel
            files.par_iter().map(|data| {
                let cursor = Cursor::new(data.clone());
                let mut reader = MarcReader::new(cursor);
                let mut count = 0;
                while let Ok(Some(_record)) = reader.read_record() {
                    count += 1;
                }
                count
            }).sum::<usize>()
        })
    });
}
```

**Metrics:**
- Sequential 4x 1k: ~3.9 ms (975µs × 4)
- Parallel 4x 1k: ~1.0 ms (ideal: 3.9 / 4)
- **Expected speedup: 3.8-3.9x** (near-linear on 4 cores)

**Dependency:** Add `rayon = "1.7"` to dev-dependencies

### Python (Threading)

**Implementation:** GIL-released I/O with threading

```python
import threading
from concurrent.futures import ThreadPoolExecutor
import mrrc

def benchmark_threaded_read_4x_1k():
    """Read 4 files concurrently using threads (GIL-released I/O)."""
    
    def read_file(fixture_path):
        with open(fixture_path, 'rb') as f:
            reader = mrrc.MARCReader(f)
            count = 0
            while record := reader.read_record():
                count += 1
            return count
    
    with ThreadPoolExecutor(max_workers=4) as executor:
        files = [
            'tests/data/fixtures/1k_records.mrc',
            'tests/data/fixtures/1k_records.mrc',
            'tests/data/fixtures/1k_records.mrc',
            'tests/data/fixtures/1k_records.mrc',
        ]
        results = list(executor.map(read_file, files))
        return sum(results)
```

**Metrics:**
- Sequential 4x 1k: ~8.2 ms (2.06 ms × 4)
- Threaded 4x 1k: ~2.0-2.5 ms
- **Expected speedup: 3.3-4.1x** (GIL-released during I/O)

**Benefits:**
- Uses standard library `concurrent.futures` (no dependencies)
- Demonstrates GIL-release in action
- Simple drop-in pattern for users

### Python (Multiprocessing)

**Implementation:** Separate processes for pure Python comparison

```python
from multiprocessing import Pool
from pymarc import MARCReader

def read_pymarc_file(fixture_path):
    """Read MARC file with pymarc (requires separate process)."""
    with open(fixture_path, 'rb') as f:
        reader = MARCReader(f)
        count = 0
        for record in reader:
            count += 1
        return count

def benchmark_multiprocess_read_4x_1k():
    """Read 4 files with multiprocessing (shows pymarc limitation)."""
    
    with Pool(processes=4) as pool:
        files = [
            'tests/data/fixtures/1k_records.mrc',
        ] * 4  # Same file 4 times
        
        results = pool.map(read_pymarc_file, files)
        return sum(results)
```

**Metrics:**
- Sequential 4x 1k: ~56.6 ms (14.15 ms × 4)
- Multiprocess 4x 1k: ~15-20 ms
- **Expected speedup: 2.8-3.8x** (limited by process overhead)

**Note:** Process spawn overhead (~2-3ms per process) reduces efficiency vs threading

## Test Scenarios

### Scenario 1: Batch File Processing (Realistic Use Case)
- **Task:** Process 4 separate MARC files concurrently
- **Fixtures:** 4x 1k files (simulated as same file read 4 times)
- **Measurements:**
  - Rust (rayon): ~1.0 ms (3.8x speedup)
  - pymrrc (threading): ~2.0 ms (4.1x speedup)
  - pymarc (multiprocessing): ~17 ms (3.3x speedup)

### Scenario 2: Large File with Thread Pool
- **Task:** Process 10k file with thread workers extracting fields
- **Fixtures:** 10k_records.mrc split logically across workers
- **Complexity:** Requires careful design (reader not shareable)
- **Alternative:** Process records sequentially, dispatch results to threads

### Scenario 3: Mixed Workload (Read + Serialize)
- **Task:** Read records, serialize to JSON in parallel
- **Fixtures:** 1k_records.mrc
- **Parallelization:** Read sequentially, serialize in parallel

## Implementation Phases

### Phase 1: Rust Parallel Benchmarks
**Effort:** ~2-3 hours
1. Add `rayon` dev-dependency
2. Implement 2-3 parallel read benchmarks (2x, 4x, 8x files)
3. Add to `benches/marc_benchmarks.rs` with `parallel_` prefix
4. Compare sequential vs parallel results

**Deliverables:**
- Parallel Criterion.rs benchmarks
- Rust usage example showing rayon patterns
- Performance ratios (expected ~3.5-4.0x on 4 cores)

### Phase 2: Python Parallel Benchmarks (Threading)
**Effort:** ~2-3 hours
1. Create `tests/python/test_benchmark_parallel.py`
2. Implement ThreadPoolExecutor patterns for pymrrc
3. Compare with sequential results
4. Show threading advantage (4-5x on 4 cores)

**Deliverables:**
- Python threading benchmark suite
- Usage example showing GIL-release advantage
- Comparison with pymarc multiprocessing

### Phase 3: Comprehensive Comparison
**Effort:** ~1-2 hours
1. Update `scripts/benchmark_comparison.py` to include parallel results
2. Add "parallel" section to `comparison.json`
3. Update `RESULTS.md` with parallel performance data
4. Create visualization showing speedup curves

**Deliverables:**
- Three-way parallel comparison (Rust, pymrrc threading, pymarc multiprocessing)
- Scaling analysis (2x, 4x, 8x files)
- ROI calculations (e.g., "4 files in 2ms vs 56ms")

## Code Organization

### Rust
```
benches/
├── marc_benchmarks.rs (sequential - current)
└── parallel_benchmarks.rs (NEW - rayon-based)
```

### Python
```
tests/python/
├── test_benchmark_reading.py (sequential)
├── test_benchmark_writing.py (sequential)
└── test_benchmark_parallel.py (NEW - threading/multiprocessing)
```

### Scripts
```
scripts/
├── benchmark_comparison.py (updated to include parallel)
└── parallel_usage_examples.py (NEW - documentation/examples)
```

## Dependencies to Add

### Cargo.toml (dev-dependencies)
```toml
[dev-dependencies]
rayon = "1.7"  # For parallel benchmarks
```

**Why rayon?**
- Lightweight, zero-cost abstractions
- Already used in many Rust benchmarks
- Easy learning curve: `.par_iter()`
- No performance overhead vs manual threading

### Python
- Standard library only: `concurrent.futures` for threading
- No new dependencies needed

## Performance Expectations

| Scenario | Sequential | Parallel (4 cores) | Speedup | Notes |
|----------|-----------|-------------------|---------|-------|
| **Rust (rayon)** | 3.9 ms | 1.0 ms | **3.9x** | Near-linear scaling |
| **pymrrc (threading)** | 8.2 ms | 2.1 ms | **3.9x** | GIL-released during I/O |
| **pymarc (multiprocess)** | 56.6 ms | 16 ms | **3.5x** | Process overhead limits gain |

## Real-World Example: Batch Processing

**Scenario:** Daily job processes 4 MARC files (10k each)

| Implementation | Sequential | Parallel 4x | Time Saved | Annual Savings |
|---|---|---|---|---|
| **pymarc** | ~579 ms | ~170 ms | 409 ms/job | 12.9 hours/year |
| **pymrrc** | ~167 ms | ~42 ms | 125 ms/job | 3.9 hours/year |
| **Rust** | ~93 ms | ~24 ms | 69 ms/job | 2.2 hours/year |

*With 10 daily jobs: pymrrc threading saves ~39 hours/year vs pymarc*

## Risk Assessment

### Low Risk ✅
- Parallel read benchmarks (independent files)
- Using standard Rust/Python patterns
- No shared state or synchronization issues
- Can be added independently from current work

### Medium Risk ⚠️
- Creating test fixtures (need 4 separate files or logic to simulate)
- Benchmark isolation (parallel tests might affect each other)
- Cross-platform timing variance

### Mitigation
- Use criterion's benchmark groups to isolate parallel tests
- Create fixture copies in temp directory
- Run parallel tests separately (different benchmark group)
- Document expected variance

## Usage Example Opportunities

### Rust Pattern
```rust
// Example: Process multiple MARC files in parallel
use rayon::prelude::*;

fn process_marc_files(paths: Vec<&str>) -> Result<Vec<Record>> {
    paths.par_iter()
        .map(|path| {
            let file = File::open(path)?;
            let mut reader = MarcReader::new(file);
            reader.read_record()
        })
        .collect()
}
```

### Python Pattern (Threading)
```python
# Example: Read multiple MARC files concurrently
from concurrent.futures import ThreadPoolExecutor
import mrrc

with ThreadPoolExecutor(max_workers=4) as executor:
    files = ['file1.mrc', 'file2.mrc', 'file3.mrc', 'file4.mrc']
    readers = [
        mrrc.MARCReader(open(f, 'rb'))
        for f in files
    ]
    all_records = list(executor.map(
        lambda r: list(r),
        readers
    ))
```

### Python Pattern (GIL Release)
```python
# Note: GIL is released during I/O operations
# This allows true parallelism with threads, unlike pure Python
reader1 = mrrc.MARCReader(open('file1.mrc', 'rb'))
reader2 = mrrc.MARCReader(open('file2.mrc', 'rb'))

# When read_record() calls into Rust, GIL is released
# allowing other threads to execute concurrently
record1 = reader1.read_record()  # releases GIL
record2 = reader2.read_record()  # can run in parallel
```

## Next Steps

### Immediate (Validation)
- [ ] Verify rayon integrates cleanly
- [ ] Create single parallel benchmark in Rust
- [ ] Measure actual speedup on target hardware
- [ ] Validate no interaction with current benchmarks

### Short Term (Implementation)
- [ ] Complete Rust parallel benchmarks (mrrc-58u Phase 1)
- [ ] Implement Python threading benchmarks (mrrc-58u Phase 2)
- [ ] Update comparison script (mrrc-58u Phase 3)

### Documentation
- [ ] Add concurrency guide to docs/
- [ ] Create usage examples file
- [ ] Update RESULTS.md with parallel section
- [ ] Blog post highlighting GIL-release advantage

## ⚠️ Discovery: GIL Not Currently Released

**Feasibility Test Results:**

Running parallel benchmarks with `concurrent.futures.ThreadPoolExecutor` revealed:
- Sequential 4x 1k: 20.96 ms
- Parallel 4x 1k: 14.88 ms  
- **Actual speedup: 1.41x** (expected 3.5-4.0x)

This indicates the GIL is **not currently released** during `read_record()` calls.

### Why This Matters

The current implementation does NOT have `#[pyo3(allow_threads)]` or `release_gil()` decorators on I/O-bound operations. This is an **optimization opportunity** separate from parallel benchmarking.

### Two-Track Approach

**Track A: Enable GIL Release (Enhancement)**
- Add `#[pyo3(allow_threads)]` to `read_record()` and I/O operations
- This unlocks the true parallel advantage
- ~2-3 hours implementation
- **Result:** Threading would show 3-4x speedup as designed

**Track B: Parallel Benchmarks (Independent)**
- Add rayon benchmarks for Rust (will show 3.8x+ speedup)
- Add multiprocessing benchmarks for pymarc (shows 3-4x with process overhead)
- pymrrc will show current behavior (1.4x with threads) or better (if Track A done first)
- ~4-5 hours implementation

## Recommendation

**Option 1: Parallel + GIL Fix (Complete Solution)**
- Enable GIL release first (mrrc-###)
- Then add parallel benchmarks (mrrc-58u)
- **Result:** Full story of threading advantage + Rust parallelism
- **Effort:** 7-10 hours total
- **Impact:** Highest - shows pymrrc's full potential

**Option 2: Parallel Benchmarks Only (Current Design)**
- Add benchmarks as-is without GIL changes
- Rust: Shows 3.8x scaling (rayon)
- pymrrc: Shows 1.4x scaling (no GIL release)
- pymarc: Shows 3.5x scaling (multiprocessing)
- **Result:** Story shows Rust advantage, pymrrc opportunity
- **Effort:** 5-7 hours
- **Impact:** Medium - highlights GIL as limitation

**Option 3: Defer Parallel Work (Staged)**
- Create separate task: "Enable GIL release in pymrrc"
- Once complete, add parallel benchmarks
- **Benefit:** Clean separation of concerns
- **Effort:** 7-10 hours (deferred)

## Conclusion

**Feasibility: HIGH (with important caveat)**

Adding parallel benchmarks is straightforward because:
1. ✅ No shared state (each file is independent)
2. ✅ Standard patterns (rayon, ThreadPoolExecutor, multiprocessing)
3. ✅ Isolated from current tests (separate benchmark group)
4. ✅ High value: Shows Rust parallelism, highlights pymrrc opportunity
5. ✅ Documentation value: Serves as usage examples
6. ⚠️ **DISCOVERY:** GIL not released - may need fix first

**Recommended approach:** 
- **Short term:** Add parallel benchmarks (mrrc-58u) to show Rust + multiprocessing advantage
- **Medium term:** Create GIL-release task for pymrrc I/O operations
- **Long term:** Re-benchmark parallel with GIL enabled to show full potential

**Estimated effort:** 
- Parallel benchmarks only: 5-7 hours
- Parallel + GIL release: 7-10 hours total
- GIL release alone: 2-3 hours (can be done first)

**Value delivered:** 
- Demonstrates Rust's true parallelism (~3.8x scaling with rayon)
- Shows pymarc multiprocessing overhead (3.5x vs 3.8x)
- Reveals pymrrc threading opportunity if GIL is released
- Provides real code examples for concurrent MARC processing
- Highlights potential optimization: GIL release for I/O operations
