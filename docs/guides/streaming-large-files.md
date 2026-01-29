# Streaming Guide

This guide covers large file handling and memory optimization for processing MARC collections.

## Overview

MARC collections can range from hundreds to millions of records. MRRC provides several strategies for efficient processing:

| Strategy | Memory | Speed | Use Case |
|----------|--------|-------|----------|
| **Iterator (default)** | O(1) | Good | Most workloads |
| **Batch loading** | O(n) | Better | Analytics, sorting |
| **ProducerConsumerPipeline** | O(buffer) | Best | Large single files |
| **Parallel file processing** | O(workers) | Best | Multiple files |

## Iterator-Based Streaming

The default reader processes one record at a time, using constant memory regardless of file size.

### Python

```python
import mrrc

# Memory-efficient: only one record in memory at a time
for record in mrrc.read("large_file.mrc"):
    process(record)
```

### Rust

```rust
use mrrc::MarcReader;
use std::fs::File;

let file = File::open("large_file.mrc")?;
let mut reader = MarcReader::new(file);

while let Some(record) = reader.read_record()? {
    process(&record);
}
```

## When to Load All Records

Sometimes you need all records in memory:
- Sorting by field values
- Cross-record analysis
- Building indexes

### Python

```python
# Caution: loads entire file into memory
records = list(mrrc.read("file.mrc"))

# Better: use generator with filter
def filter_books(path):
    for record in mrrc.read(path):
        if record.is_book():
            yield record

books = list(filter_books("file.mrc"))
```

### Rust

```rust
use mrrc::MarcReader;
use std::fs::File;

let file = File::open("file.mrc")?;
let mut reader = MarcReader::new(file);

let records: Vec<_> = reader
    .into_iter()
    .filter_map(|r| r.ok())
    .collect();
```

## ProducerConsumerPipeline (Python)

For maximum throughput when processing a single large file, use the `ProducerConsumerPipeline`. It runs a background producer thread that reads and parses records, while the main thread processes them.

### Basic Usage

```python
from mrrc import ProducerConsumerPipeline

# Create pipeline with default config
pipeline = ProducerConsumerPipeline.from_file('large_file.mrc')

# Process records as they become available
while record := pipeline.next():
    process(record)
```

### Configuration Options

```python
from mrrc import ProducerConsumerPipeline

# Custom configuration via from_file parameters
pipeline = ProducerConsumerPipeline.from_file(
    'large_file.mrc',
    buffer_size=1024*1024,    # File I/O buffer size (default: 512 KB)
    channel_capacity=500       # Channel capacity in records (default: 1000)
)
```

### Performance Characteristics

| File Size | Iterator | Pipeline | Speedup |
|-----------|----------|----------|---------|
| 10 MB | 255k rec/s | 700k rec/s | 2.7x |
| 100 MB | 250k rec/s | 950k rec/s | 3.8x |
| 1 GB | 240k rec/s | 1M rec/s | 4.2x |

Pipeline benefits increase with file size due to reduced I/O wait time.

### Memory Considerations

```python
# Memory usage = channel_capacity * avg_record_size + buffer_size
# For 1000 records @ 2KB each + 512KB buffer = ~2.5MB

# Reduce memory for constrained environments
pipeline = ProducerConsumerPipeline.from_file(
    'large_file.mrc',
    buffer_size=64*1024,     # 64KB I/O buffer
    channel_capacity=100      # Smaller channel
)
```

## Batch Processing

For operations that benefit from batching (database inserts, bulk API calls), process records in chunks.

### Python

```python
def batch_generator(path, batch_size=1000):
    """Yield batches of records."""
    batch = []
    for record in mrrc.read(path):
        batch.append(record)
        if len(batch) >= batch_size:
            yield batch
            batch = []
    if batch:
        yield batch

# Process in batches
for batch in batch_generator("large_file.mrc"):
    insert_into_database(batch)
```

### Rust

```rust
use mrrc::MarcReader;
use std::fs::File;

fn process_batches<F>(path: &str, batch_size: usize, mut processor: F) -> mrrc::Result<()>
where
    F: FnMut(&[Record]),
{
    let file = File::open(path)?;
    let mut reader = MarcReader::new(file);
    let mut batch = Vec::with_capacity(batch_size);

    while let Some(record) = reader.read_record()? {
        batch.push(record);
        if batch.len() >= batch_size {
            processor(&batch);
            batch.clear();
        }
    }

    if !batch.is_empty() {
        processor(&batch);
    }

    Ok(())
}
```

## Parallel File Processing

When processing multiple files, parallelize at the file level.

### Python with ThreadPoolExecutor

```python
from concurrent.futures import ThreadPoolExecutor
import mrrc

def process_file(path):
    """Process a single file. Called in thread pool."""
    count = 0
    for record in mrrc.read(path):
        # Process record
        count += 1
    return count

files = ["file1.mrc", "file2.mrc", "file3.mrc", "file4.mrc"]

# Sequential: ~1x
total = sum(process_file(f) for f in files)

# Parallel: ~3-4x on 4-core system
with ThreadPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(process_file, files))
    total = sum(results)
```

### Rust with Rayon

```rust
use mrrc::MarcReader;
use rayon::prelude::*;
use std::fs::File;

fn process_files_parallel(paths: &[&str]) -> mrrc::Result<usize> {
    let counts: Vec<_> = paths
        .par_iter()
        .map(|path| {
            let file = File::open(path)?;
            let mut reader = MarcReader::new(file);
            let mut count = 0;
            while let Some(_) = reader.read_record()? {
                count += 1;
            }
            Ok(count)
        })
        .collect::<mrrc::Result<Vec<_>>>()?;

    Ok(counts.iter().sum())
}
```

## Record Boundary Scanner (Rust)

For advanced parallel processing within a single file, use the boundary scanner to find record boundaries, then parse sections in parallel.

```rust
use mrrc::boundary_scanner::RecordBoundaryScanner;
use rayon::prelude::*;
use std::fs::File;
use std::io::Read;

fn parallel_single_file(path: &str) -> mrrc::Result<Vec<Record>> {
    // Load file into memory
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    // Find record boundaries
    let scanner = RecordBoundaryScanner::new(&data);
    let boundaries: Vec<_> = scanner.collect();

    // Parse in parallel
    let records: Vec<_> = boundaries
        .par_iter()
        .filter_map(|boundary| {
            // Parse record at boundary
            // ...
            None // placeholder
        })
        .collect();

    Ok(records)
}
```

## Format Considerations for Streaming

Different formats have different streaming characteristics.

### Best for Streaming

| Format | Why |
|--------|-----|
| **ISO 2709** | Self-delimiting records, efficient scanning |
| **Protobuf** | Length-prefixed, no buffering needed |
| **MessagePack** | Self-describing, streamable |
| **FlatBuffers** | Zero-copy, memory-mapped friendly |

### Requires Buffering

| Format | Why |
|--------|-----|
| **JSON** | Must parse complete object |
| **XML** | DOM parsing common, SAX available |
| **Arrow** | Columnar format, batch-oriented |
| **Avro** | Block-based, schema at start |

### Format-Specific Tips

```python
# Protobuf streaming
for record in mrrc.ProtobufReader("large.pb"):
    process(record)

# Arrow: batch-oriented, load chunks
reader = mrrc.ArrowReader("data.arrow")
for record in reader:  # Internally batched
    process(record)
```

## Memory Profiling

Monitor memory usage to ensure streaming is working correctly.

### Python

```python
import tracemalloc

tracemalloc.start()

count = 0
for record in mrrc.read("large_file.mrc"):
    count += 1

current, peak = tracemalloc.get_traced_memory()
print(f"Processed {count} records")
print(f"Current memory: {current / 1024 / 1024:.1f} MB")
print(f"Peak memory: {peak / 1024 / 1024:.1f} MB")

tracemalloc.stop()
```

Expected output for streaming:
```
Processed 100000 records
Current memory: 0.5 MB
Peak memory: 2.1 MB
```

If peak memory grows with record count, check for:
- Accumulating records in a list
- Growing data structures
- Unclosed file handles

## Best Practices

### Do

- Use iterators for sequential processing
- Process records as you read them
- Use `ProducerConsumerPipeline` for large single files
- Parallelize at the file level for multiple files
- Set appropriate buffer sizes for your memory constraints

### Don't

- Load all records into memory unless necessary
- Create intermediate lists during filtering
- Ignore memory usage during development
- Use threading for CPU-bound work in Python (use multiprocessing instead)

## Troubleshooting

### High Memory Usage

1. Check for list accumulation:
   ```python
   # Bad: accumulates all records
   records = [r for r in mrrc.read("file.mrc")]

   # Good: process one at a time
   for record in mrrc.read("file.mrc"):
       process(record)
   ```

2. Check for closures capturing records:
   ```python
   # Bad: closure captures record
   results = []
   for record in reader:
       results.append(lambda: process(record))  # Captures all records!
   ```

### Slow Processing

1. Use file paths instead of file objects in Python:
   ```python
   # Slower: file object requires GIL for reads
   with open("file.mrc", "rb") as f:
       for record in mrrc.MARCReader(f):
           pass

   # Faster: file path uses zero-GIL Rust I/O
   for record in mrrc.read("file.mrc"):
       pass
   ```

2. Consider `ProducerConsumerPipeline` for large files.

### Pipeline Not Faster

- Ensure processing is the bottleneck (not I/O)
- Check CPU utilization during processing
- Try adjusting `buffer_size` and `channel_capacity` parameters
- For I/O-bound work, pipeline may not help

## Next Steps

- [Threading Documentation](./THREADING.md) - Thread safety and patterns
- [Performance Guide](./PERFORMANCE.md) - Detailed optimization
- [Format Selection Guide](./FORMAT_SELECTION_GUIDE.md) - Choose optimal formats
