# MRRC Performance Guide

This document provides comprehensive performance analysis of the MRRC library and guidance for optimizing concurrent workloads.

## Executive Summary

MRRC achieves exceptional performance through Rust implementation and GIL release during I/O operations:

- **Single-thread reading**: 549,500 records/second (18.2 ms for 10k records)
- **Multi-thread speedup**: 2.0x (2 threads), 3.74x (4 threads)
- **vs pymarc**: **7.5x faster** with same API
- **GIL released during parsing** - enables true multi-core parallelism

## Single-Thread Performance Baseline

| Metric | Value |
|--------|-------|
| Read 10k records | 18.2 ms |
| Records/second | 549,500 rec/s |
| Throughput | Consistent across all sizes |
| vs pymarc | **7.6x faster** |

## Multi-Thread Performance

The Python wrapper releases the GIL during parsing, enabling true multi-core parallelism. Performance scales with CPU cores when processing multiple files concurrently.

### Two-Thread Performance

**Setup**: 2 threads, each reading 5,000 records concurrently

| Metric | Result |
|--------|--------|
| Sequential (2 × 5k records) | 18.70 ms |
| **Parallel execution** | **9.24 ms** |
| **Speedup achieved** | **2.02x** |
| Efficiency | 101% (excellent thread locality) |

**Analysis**: True parallelism confirmed. Each thread processes independently while the other runs in parallel.

### Four-Thread Performance

**Setup**: 4 threads, each reading 2,500 records concurrently

| Metric | Result |
|--------|--------|
| Sequential (4 × 2.5k records) | 37.60 ms |
| **Parallel execution (per thread)** | **10.04 ms** |
| **Total time** | **10.04 ms** |
| **Speedup achieved** | **3.74x** |
| Efficiency | 94% (excellent scaling) |

**Analysis**: Sub-linear speedup relative to 4 cores is expected due to system scheduler overhead and memory bandwidth saturation, but 3.74x is still excellent multi-threaded performance.

### Performance by Thread Count

| Thread Count | Expected Speedup | Recommendation |
|---|---|---|
| 1 | 1.0x | Baseline (sequential) |
| 2 | 2.0x | Good parallelism |
| 4 | 3.7x | Excellent parallelism |
| 8+ | 4.5x+ | Diminishing returns |

**Recommendation**: Use CPU core count as max_workers for optimal performance.

---

## How GIL Release Works

### Three-Phase Record Parsing

Each record parse uses a three-phase GIL management pattern:

**Phase 1 (GIL Held)**
- Acquire raw record bytes from the source (file or Python object)
- Minimal work while GIL is held
- Quick transition to Phase 2

**Phase 2 (GIL Released)**
- Parse MARC record bytes into Rust structure
- This is where the actual parsing work happens
- CPU-intensive but doesn't need GIL
- Other threads can run Python code during this phase

**Phase 3 (GIL Re-acquired)**
- Convert Rust errors to Python exceptions
- Convert parsed record to Python object
- Quick return to caller

The GIL is automatically re-acquired when Phase 2 completes, so no manual lock management is needed.

### Backend Strategy: File Paths vs File Objects

**File paths** (recommended for performance):
```python
reader = MARCReader('records.mrc')  # Pure Rust I/O, zero GIL overhead
```
Uses a background Rust thread that never acquires the GIL. Optimal for multi-threaded workloads.

**File objects** (also supported):
```python
with open('records.mrc', 'rb') as f:
    reader = MARCReader(f)  # GIL released during parsing only
```
Acquires GIL to call `.read()` on the Python file object, but releases GIL during parsing.

**Bytes** (pre-loaded data):
```python
with open('records.mrc', 'rb') as f:
    data = f.read()
reader = MARCReader(data)  # GIL released during parsing
```
Fast and simple for smaller files.

---

## Usage Patterns

### Pattern 1: Multi-File Processing (Recommended)

Process multiple MARC files in parallel, each in its own thread:

```python
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader

def process_file(filename):
    """Process a single MARC file (runs in thread)."""
    record_count = 0
    reader = MARCReader(filename)  # File path for best performance
    for record in reader:
        # Process record
        record_count += 1
    return record_count

# Process 4 files in parallel on 4-core system
files = ['file1.mrc', 'file2.mrc', 'file3.mrc', 'file4.mrc']
with ThreadPoolExecutor(max_workers=4) as executor:
    futures = [executor.submit(process_file, f) for f in files]
    results = [f.result() for f in futures]
    
# Expected: 3.7x faster than sequential on 4-core system
total = sum(results)
print(f"Processed {total} records in parallel")
```

**Performance**: 3-4x speedup on 4-core system when processing 4 files.

### Pattern 2: Single-File Splitting (Advanced)

For processing a single large file with parallelism:

```python
def process_file_chunk(filename, start_record, end_record):
    """Process a chunk of records from a file."""
    count = 0
    reader = MARCReader(filename)
    for i, record in enumerate(reader):
        if i >= end_record:
            break
        if i >= start_record:
            # Process record
            count += 1
    return count

# Split 100k records into 4 chunks of 25k each
chunk_size = 25000
with ThreadPoolExecutor(max_workers=4) as executor:
    futures = [
        executor.submit(process_file_chunk, 'large_file.mrc', i*chunk_size, (i+1)*chunk_size)
        for i in range(4)
    ]
    results = [f.result() for f in futures]
```

**Performance**: Good speedup, but less efficient than multi-file due to sequential file I/O overhead.

### Pattern 3: Sequential Reading (When Parallelism Not Needed)

```python
from mrrc import MARCReader

reader = MARCReader('records.mrc')
for record in reader:
    # Process record sequentially
    title = record.title()
```

---

## Performance Tuning

### Optimal Thread Count

Based on benchmarking:

```python
import os
from concurrent.futures import ThreadPoolExecutor

optimal_workers = os.cpu_count()  # Use all cores
with ThreadPoolExecutor(max_workers=optimal_workers) as executor:
    # Process files
    pass
```

### File I/O Considerations

- **Binary mode required**: Always use `open(file, 'rb')` for MARC files
- **File paths preferred**: Pass filename string to MARCReader for best performance
- **Network filesystems**: Performance may degrade significantly
- **Local SSD**: Recommended for optimal performance
- **File splitting**: Consider splitting large files (>1 GB) for better parallelism

### Memory Overhead

- **Per-reader**: ~4 KB (parsing buffer)
- **Per-record in memory**: ~4 KB (typical)
- **Memory regression vs single-threaded**: < 5%

For processing 1 million records with 4 threads:
- Single-threaded: ~4 GB peak memory
- 4 threads: ~4 GB + (4 × 4 KB) = ~4 GB peak

No additional memory accumulation from threading.

---

## Comparison to Other Approaches

### Threading Efficiency: pymrrc vs Pure Python

| Scenario | pymrrc | pymarc |
|---|---|---|
| 2-thread speedup | 2.0x | 1.0x (GIL blocks) |
| 4-thread speedup | 3.74x | 1.0x (GIL blocks) |
| Single-thread vs pymarc | 7.6x faster | baseline |

pymrrc enables true parallelism through GIL release. pymarc cannot benefit from threading.

### Rust vs Python Performance

| Operation | Rust (mrrc) | Python (pymrrc) | Speedup |
|---|---|---|---|
| Read 1k records | 0.94 ms | 1.87 ms | Rust 2.0x |
| Read 10k records | 9.40 ms | 18.2 ms | Rust 2.0x |
| Multi-threaded (4 cores) | 3.2x speedup | 3.74x speedup | Python matches Rust |

Pure Rust is 2x faster single-threaded, but multi-threaded pymrrc achieves comparable throughput per core.

---

## Troubleshooting

### No Speedup with Multiple Threads

**Symptom**: ThreadPoolExecutor with 4 workers shows only 1x speedup

**Causes**:
1. **Wrong pattern**: Sharing single reader across threads
   - Solution: Create separate reader per thread
2. **I/O bottleneck**: Network filesystem or slow disk
   - Solution: Use local SSD, profile with `cProfile`
3. **CPU-bound work**: Heavy processing per record
   - Solution: Use `multiprocessing` instead of `threading`

**Diagnosis**:
```python
import time
from concurrent.futures import ThreadPoolExecutor

def process_file(filename):
    count = 0
    for record in MARCReader(filename):
        count += 1
    return count

files = ['file1.mrc', 'file2.mrc', 'file3.mrc', 'file4.mrc']

# Sequential baseline
start = time.time()
sequential = sum(process_file(f) for f in files)
seq_time = time.time() - start

# Parallel execution
start = time.time()
with ThreadPoolExecutor(max_workers=4) as executor:
    parallel = sum(executor.map(process_file, files))
par_time = time.time() - start

speedup = seq_time / par_time
print(f"Sequential: {seq_time:.2f}s")
print(f"Parallel: {par_time:.2f}s")
print(f"Speedup: {speedup:.2f}x (expected: ~3.7x)")
```

### Slow Single-Thread Performance

**Causes**:
1. Python file object overhead (use file paths)
2. Large records (>4 KB) causing allocations
3. System load or garbage collection pressure

**Solutions**:
```python
# Faster: File path (zero GIL I/O backend)
reader = MARCReader('records.mrc')

# Also fast: Pre-loaded bytes
with open('records.mrc', 'rb') as f:
    data = f.read()
reader = MARCReader(data)

# Slower: Python file object (GIL acquired for .read())
with open('records.mrc', 'rb') as f:
    reader = MARCReader(f)
```

---

## Benchmarking Your Application

### Simple Timing Test

```python
import time
from mrrc import MARCReader

start = time.time()
count = 0
reader = MARCReader('records.mrc')
for record in reader:
    count += 1
elapsed = time.time() - start

throughput = count / elapsed
print(f"Processed {count} records in {elapsed:.2f}s")
print(f"Throughput: {throughput:.0f} rec/s")
```

### Multi-Thread Speedup Test

```python
import time
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader

def process_file(filename):
    count = 0
    for record in MARCReader(filename):
        count += 1
    return count

files = ['file1.mrc', 'file2.mrc', 'file3.mrc', 'file4.mrc']

# Sequential baseline
start = time.time()
sequential = sum(process_file(f) for f in files)
seq_time = time.time() - start

# Parallel execution
start = time.time()
with ThreadPoolExecutor(max_workers=4) as executor:
    parallel = sum(executor.map(process_file, files))
par_time = time.time() - start

speedup = seq_time / par_time
print(f"Sequential: {seq_time:.2f}s")
print(f"Parallel (4 threads): {par_time:.2f}s")
print(f"Speedup: {speedup:.2f}x")
```

---

## Key Findings

1. **GIL Release Works**: 2.0x speedup on 2 threads validates GIL release implementation
2. **Scales to 4 cores**: 3.74x speedup on 4 threads shows continued multi-core benefit
3. **Backend matters**: File paths use zero-GIL I/O, file objects still acquire GIL for `.read()`
4. **7.5x faster than pymarc**: Massive performance advantage even on single thread

---

## References

- **Benchmarking results**: `docs/benchmarks/RESULTS.md`
- **Threading guide**: `docs/threading.md`
- **Parallel processing**: `docs/parallel_processing.md`
- **Rust benchmarks**: `benches/marc_benchmarks.rs`
- **Python benchmarks**: `tests/python/test_benchmark_*.py`
