# MRRC Performance Guide

Performance analysis and optimization guidance for MRRC. For parallel processing patterns, see the [Python Concurrency Tutorial](../tutorials/python/concurrency.md). For thread safety details, see [Threading in Python](threading-python.md).

## Executive Summary

- **Single-thread**: record parsing runs in Rust; early benchmarking suggested
  at least a 4x speedup over pymarc (these benchmarks need to be updated and
  reconsidered — see [Benchmark Results](../benchmarks/results.md))
- **Multi-thread**: GIL release during parsing lets speedup scale with core
  count
- **GIL released** automatically during parsing (no code changes needed)

## Threading Patterns

| Pattern | Best For |
|---------|----------|
| ProducerConsumerPipeline | Single large file |
| ThreadPoolExecutor | Multiple files |
| Multiprocessing | CPU-heavy per-record work |

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

**Filter in Rust with the Query DSL.** When you only need a subset of a
record's fields, the [Query DSL](query-dsl.md) lets you express the match
(by indicator, tag range, subfield presence, or regex) so the filtering
runs in Rust and only the matching fields cross into Python — instead of
materializing every field as a Python object and filtering in a loop.

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
print(f"Speedup: {speedup:.2f}x")
# For CPU-bound parsing, expect speedup approaching the core count,
# sub-linear due to thread management and memory bandwidth.
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

pymrrc parses each record in Rust and releases the GIL while doing so, so it
is faster single-threaded and its threading speedup scales with cores; pymarc
parses in Python under the GIL, so threads provide no parsing parallelism.
Early benchmarking suggested at least a 4x single-threaded speedup; these
benchmarks need to be updated and reconsidered. Use the timing pattern above
to measure on your own workload.

## References

- [Python Concurrency Tutorial](../tutorials/python/concurrency.md) - Parallel processing patterns
- [Threading in Python](threading-python.md) - Thread safety and GIL behavior
- [Benchmarking Results](../benchmarks/results.md) - Detailed benchmark data
- **Rust benchmarks**: `benches/marc_benchmarks.rs`
- **Python benchmarks**: `tests/python/test_benchmark_*.py`
