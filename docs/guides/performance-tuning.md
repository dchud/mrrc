# MRRC Performance Guide

Performance analysis and optimization guidance for MRRC. For parallel processing patterns, see the [Python Concurrency Tutorial](../tutorials/python/concurrency.md). For thread safety details, see [Threading in Python](threading-python.md).

**Benchmark environment:** 2025 MacBook Air with Apple M4 chip. See [Detailed Benchmark Results](../benchmarks/results.md) for comprehensive data.

## Executive Summary

- **Single-thread**: ~300,000 records/second (~33 ms for 10k records), **~4x faster than pymarc**
- **Multi-thread**: ~3.74x speedup on 4 cores with ProducerConsumerPipeline
- **GIL released** automatically during parsing (no code changes needed)

## Performance Baselines

### Single-Thread

| Metric | Value |
|--------|-------|
| Read 10k records | ~33 ms |
| Records/second | ~300,000 rec/s |
| vs pymarc | **~4x faster** |

### Multi-Thread (4 cores)

| Pattern | Speedup | Best For |
|---------|---------|----------|
| ProducerConsumerPipeline | 3.74x | Single large file |
| ThreadPoolExecutor | 3-4x | Multiple files |
| Multiprocessing | 4-5x | CPU-heavy work |

See the [Python Concurrency Tutorial](../tutorials/python/concurrency.md) for pattern implementation details.

## Backend Strategy

### File Paths (Recommended)

```python
reader = MARCReader('records.mrc')  # Pure Rust I/O, zero GIL overhead
```

Uses background Rust thread that never acquires the GIL. Optimal for multi-threaded workloads.

### File Objects

```python
with open('records.mrc', 'rb') as f:
    reader = MARCReader(f)  # GIL released during parsing only
```

Acquires GIL for `.read()` calls, releases during parsing.

### Pre-loaded Bytes

```python
with open('records.mrc', 'rb') as f:
    data = f.read()
reader = MARCReader(data)  # GIL released during parsing
```

Fast for smaller files already in memory.

## Performance Tuning

### Optimal Thread Count

```python
import os
from concurrent.futures import ThreadPoolExecutor

optimal_workers = os.cpu_count()
with ThreadPoolExecutor(max_workers=optimal_workers) as executor:
    pass
```

### File I/O Considerations

- **Binary mode required**: Always use `open(file, 'rb')`
- **File paths preferred**: Pass filename string to MARCReader
- **Local SSD recommended**: Network filesystems may degrade performance
- **Large files (>1 GB)**: Consider splitting for better parallelism

### Memory Overhead

- **Per-reader**: ~4 KB (parsing buffer)
- **Per-record**: ~4 KB (typical)
- **Threading overhead**: < 5% memory regression

For 1 million records with 4 threads: ~4 GB peak (same as single-threaded).

## Troubleshooting

### No Speedup with Multiple Threads

**Causes:**

1. Sharing single reader across threads (each thread needs its own)
2. I/O bottleneck (network filesystem, slow disk)
3. CPU-bound processing (use multiprocessing instead)

**Diagnosis:**

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
print(f"Parallel: {par_time:.2f}s")
print(f"Speedup: {speedup:.2f}x (expected: ~3.7x)")
```

### Slow Single-Thread Performance

**Solutions:**

1. Use file paths instead of file objects
2. Check for system load or GC pressure
3. Profile with `cProfile` to identify bottlenecks

## Benchmarking

### Simple Timing Test

```python
import time
from mrrc import MARCReader

start = time.time()
count = 0
for record in MARCReader('records.mrc'):
    count += 1
elapsed = time.time() - start

print(f"Processed {count} records in {elapsed:.2f}s")
print(f"Throughput: {count / elapsed:.0f} rec/s")
```

## Comparison: pymrrc vs pymarc

| Scenario | pymrrc | pymarc |
|----------|--------|--------|
| Single-thread | ~4x faster | baseline |
| 2-thread speedup | 2.0x | 1.0x (GIL blocks) |
| 4-thread speedup | 3.74x | 1.0x (GIL blocks) |

## References

- [Python Concurrency Tutorial](../tutorials/python/concurrency.md) - Parallel processing patterns
- [Threading in Python](threading-python.md) - Thread safety and GIL behavior
- [Benchmarking Results](../benchmarks/results.md) - Detailed benchmark data
- **Rust benchmarks**: `benches/marc_benchmarks.rs`
- **Python benchmarks**: `tests/python/test_benchmark_*.py`
