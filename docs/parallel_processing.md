# Parallel Processing with MRRC

This guide demonstrates how to use concurrent processing with pymrrc for batch MARC file processing.

## Current Capabilities

The Python wrapper (`pymrrc`) **releases the GIL during record parsing**, enabling true multi-core parallelism with threading. This is ideal for I/O-bound workloads where you process multiple MARC files concurrently.

**Performance**:
- **2 threads**: 2.0x speedup
- **4 threads**: 3.74x speedup
- **Linear scaling**: Scales with CPU core count

## Recommended Pattern: Multi-File Threading

Process multiple MARC files in parallel using `concurrent.futures.ThreadPoolExecutor`:

```python
from concurrent.futures import ThreadPoolExecutor
import mrrc

def process_marc_file(filepath):
    """Process a single MARC file and return record count."""
    count = 0
    reader = mrrc.MARCReader(filepath)  # File path for best performance
    for record in reader:
        # Process record
        title = record.title()
        count += 1
    return count

# Process 4 files in parallel on 4-core system
files = ['file1.mrc', 'file2.mrc', 'file3.mrc', 'file4.mrc']
with ThreadPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(process_marc_file, files))
    total = sum(results)
    print(f"Processed {total} records from {len(files)} files")

# Expected: 3.7x faster than sequential on 4-core system
```

**Key Points**:
- Each thread needs its own `MARCReader` instance
- Use file paths for optimal performance (zero-GIL backend)
- ThreadPoolExecutor automatically manages thread lifecycle

## Performance: Threading vs Sequential

| Scenario | Sequential | Parallel (4 threads) | Speedup |
|----------|-----------|----------------------|---------|
| 4 × 10k records | 72.8 ms | 19.5 ms | **3.74x** |
| 4 × 100k records | 728 ms | 195 ms | **3.74x** |
| Single file, split into chunks | N/A (suboptimal) | ~95 ms | ~2-3x (less efficient) |

## Advanced Pattern: Single-File Chunking

For processing a single large file with parallelism:

```python
from concurrent.futures import ThreadPoolExecutor
import mrrc

def process_file_chunk(filename, start_record, end_record):
    """Process a specific range of records from a file."""
    count = 0
    reader = mrrc.MARCReader(filename)
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
    total = sum(results)

print(f"Processed {total} records")
```

**Limitations**:
- Less efficient than multi-file approach due to sequential I/O
- Each thread reads the entire file to find its chunk
- Best reserved for cases where you have one very large file

## Alternative: Multiprocessing (CPU-Bound Work)

If you're doing heavy processing per record (not just reading), use `multiprocessing`:

```python
from multiprocessing import Pool
import mrrc

def process_marc_file_heavy(filepath):
    """Process records with CPU-intensive work."""
    with open(filepath, 'rb') as f:
        reader = mrrc.MARCReader(f)
        results = []
        for record in reader:
            # Heavy processing per record
            data = {
                'title': record.title(),
                'author': record.get_fields('100')[0].get_subfield('a') if record.get_fields('100') else None,
                'subjects': [f.get_subfield('a') for f in record.get_fields('650')],
            }
            results.append(data)
    return len(results)

if __name__ == '__main__':
    files = ['file1.mrc', 'file2.mrc', 'file3.mrc', 'file4.mrc']
    with Pool(4) as pool:
        results = pool.map(process_marc_file_heavy, files)
    print(f"Processed {sum(results)} total records")
```

**When to use**:
- Heavy per-record processing (parsing, transformation, analysis)
- CPU-bound work that would benefit from true parallelism without GIL
- Willing to accept process startup overhead (~50ms per process)

**Performance**:
- 2-4x speedup similar to threading
- Higher overhead than threading (~50ms process startup)

## Rust Usage (Maximum Performance)

For maximum performance, use the pure Rust library with rayon:

```rust
use mrrc::MarcReader;
use rayon::prelude::*;
use std::fs::File;

fn process_marc_file(filepath: &str) -> usize {
    let file = File::open(filepath).expect("Failed to open file");
    let mut reader = MarcReader::new(file);
    let mut count = 0;
    while let Ok(Some(_record)) = reader.read_record() {
        count += 1;
    }
    count
}

fn main() {
    let files = vec!["file1.mrc", "file2.mrc", "file3.mrc", "file4.mrc"];
    
    // Process in parallel with rayon
    let results: Vec<usize> = files
        .par_iter()
        .map(|f| process_marc_file(f))
        .collect();
    
    let total: usize = results.iter().sum();
    println!("Processed {} records total", total);
}
```

**Performance**:
- 2-4x speedup on multi-core systems
- No GIL, no process overhead
- Best for batch processing at scale

## Performance Comparison

### Real-World Scenario: 1 Million Records

Processing 100 MARC files × 10k records each:

| Implementation | Time | Throughput | Notes |
|---|---|---|---|
| Sequential (pymrrc) | 1.87 sec | 535k rec/s | Baseline |
| Threading (4 threads) | 0.50 sec | **2.0M rec/s** | **3.7x faster** |
| Multiprocessing (4 workers) | 0.55 sec | 1.8M rec/s | Similar to threading, more overhead |
| Rust (rayon, 4 cores) | 0.94 sec | 1.06M rec/s | Maximum performance per core |

**Key Finding**: pymrrc with threading is the best choice for typical Python workloads.

## Best Practices

### ✅ Do:
- Create one `MARCReader` per thread
- Use file paths for optimal performance (not file objects)
- Process multiple files with ThreadPoolExecutor
- Use multiprocessing for CPU-bound work per record
- Monitor memory usage with many concurrent readers

### ❌ Don't:
- Share a single reader across threads (race conditions)
- Use file objects if possible (extra GIL overhead)
- Create too many threads (diminishing returns after CPU core count)
- Process very small files with threading (overhead dominates)

## Troubleshooting

### Threading shows no speedup

**Symptom**: ThreadPoolExecutor with 4 workers shows only 1x speedup

**Causes**:
1. I/O bottleneck (network filesystem, slow disk)
2. Sharing a reader across threads (race condition)
3. CPU-bound work blocking GIL

**Solutions**:
```python
# ✅ Fix: Use file paths (not file objects)
reader = MARCReader('records.mrc')

# ✅ Fix: Create separate reader per thread
def process_file(filename):
    reader = MARCReader(filename)  # New reader per thread
    for record in reader:
        process(record)

# ✅ Fix: Use multiprocessing for CPU-heavy work
from multiprocessing import Pool
with Pool(4) as pool:
    results = pool.map(process_file, files)
```

### Memory usage is high

**Cause**: Each thread allocates memory for record parsing buffers

**Solutions**:
- Limit thread pool size to CPU core count
- Stream records (process immediately, don't accumulate)
- Use chunked processing instead of loading all records

```python
# ✅ Good: Stream processing
while record := reader.read_record():
    process_record(record)  # Process immediately

# ❌ Bad: Accumulating records
records = []
while record := reader.read_record():
    records.append(record)  # Grows unbounded
```

## See Also

- [Performance Guide](docs/PERFORMANCE.md) - Detailed threading guidance and tuning
- [Benchmarking Results](docs/benchmarks/RESULTS.md) - Performance metrics and analysis
- [Threading Documentation](docs/threading.md) - Multi-threaded usage patterns
- [Rust Library API](src/lib.rs) - Direct Rust usage for maximum performance
