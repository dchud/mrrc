# Concurrency in MRRC

This document covers concurrency and parallel processing for both the pure Rust library and the Python wrapper. MRRC provides different concurrency patterns optimized for each language.

## Quick Decision Tree

```
Do you need to process MARC records in parallel?
├─ YES
│  ├─ Are you using Rust?
│  │  └─ Use Rayon for data parallelism (work-stealing scheduler)
│  │     Example: examples/concurrent_reading.rs
│  │
│  └─ Are you using Python?
│     ├─ Single large file?
│     │  └─ Use ProducerConsumerPipeline (best: 3.74x speedup)
│     │     Recommended for maximum throughput
│     │
│     └─ Multiple files or simpler needs?
│        └─ Use ThreadPoolExecutor (good: 3-4x speedup)
│           Simpler API, easier to integrate
│
└─ NO
   └─ Sequential processing is fine
```

## Rust Concurrency (Pure Rust Library)

### Overview

Rust concurrency uses **Rayon**, a work-stealing parallelism library that makes data parallelism simple and safe. Rayon integrates seamlessly with Rust's ownership system.

**Key benefits:**
- ✅ True parallelism (no GIL)
- ✅ Zero-copy iteration
- ✅ Automatic load balancing
- ✅ Type-safe concurrency (compile-time guarantees)

### Pattern 1: Parallel Iterator (Recommended)

Use `par_iter()` for simple data parallelism:

```rust
use rayon::prelude::*;
use mrrc::MarcReader;
use std::fs::File;

fn main() -> Result<()> {
    let file = File::open("records.mrc")?;
    let mut reader = MarcReader::new(file);
    
    // Load all records into memory
    let records: Vec<_> = reader
        .collect::<Result<Vec<_>, _>>()?;
    
    // Process in parallel
    let results: Vec<_> = records
        .par_iter()
        .map(|record| {
            // CPU-intensive work
            process_record(record)
        })
        .collect();
    
    println!("Processed {} records", results.len());
    Ok(())
}

fn process_record(record: &Record) -> usize {
    // Heavy processing here
    record.fields().count()
}
```

**When to use:**
- Processing large batch of records already in memory
- Each record requires significant CPU work
- Need to extract complex information from records

**Performance:** ~2.5x speedup on 4 cores (I/O bound)

### Pattern 2: Parallel Chunks

Process records in batches:

```rust
use rayon::prelude::*;

let results: Vec<_> = records
    .par_chunks(100)  // Process 100 records at a time
    .flat_map(|chunk| {
        chunk.iter().map(|record| {
            // Process each record
            extract_data(record)
        })
    })
    .collect();
```

**Use this for:**
- Memory-intensive processing (control memory with chunk size)
- Grouping related records together
- Building aggregates per chunk

### Pattern 3: Fold-Reduce (Aggregation)

Combine results from parallel processing:

```rust
use rayon::prelude::*;
use std::collections::HashMap;

let subject_counts: HashMap<String, usize> = records
    .par_iter()
    .fold(
        || HashMap::new(),
        |mut acc, record| {
            for subject in record.subjects() {
                *acc.entry(subject.to_string()).or_insert(0) += 1;
            }
            acc
        },
    )
    .reduce(
        || HashMap::new(),
        |mut acc, other| {
            for (key, count) in other {
                *acc.entry(key).or_insert(0) += count;
            }
            acc
        },
    );
```

**Benefits:**
- No shared state (no locks)
- Each thread accumulates locally
- Final merge is fast and lock-free

**When to use:**
- Counting, summing, grouping results
- Building aggregates (dictionaries, statistics)
- Collecting unique values

### Pattern 4: Filter + Map + Reduce

Common pattern for selection and aggregation:

```rust
use rayon::prelude::*;

let result = records
    .par_iter()
    .filter(|record| record.leader.record_type == 'a')  // Select bibliographic
    .map(|record| extract_title(record))                 // Extract title
    .fold(
        || Vec::new(),
        |mut acc, title| {
            acc.push(title);
            acc
        },
    )
    .reduce(
        || Vec::new(),
        |mut acc, other| {
            acc.extend(other);
            acc
        },
    );
```

### Performance Characteristics (Rust)

| Scenario | Sequential | Parallel | Speedup |
|----------|-----------|----------|---------|
| Pure I/O (MarcReader) | Baseline | ~1.5x | Limited by disk |
| I/O + light processing | Baseline | ~2.0x | I/O-bound |
| I/O + heavy processing | Baseline | ~2.5x | Mixed workload |
| In-memory batch processing | Baseline | ~3.5x | CPU-bound |

**Notes:**
- Sequential I/O from file is inherently serial (cannot parallelize)
- Load records into memory first for best speedup
- Rayon scales well up to number of CPU cores

### Thread Pool Configuration

Adjust Rayon's thread pool (optional):

```rust
use rayon::ThreadPoolBuilder;

// Use fewer threads
rayon::ThreadPoolBuilder::new()
    .num_threads(2)
    .build_global()
    .unwrap();

// Now rayon uses 2 threads for all parallel operations
```

### GIL? (Not applicable in Rust)

Rust has no GIL. True parallel execution on all cores.

## Python Concurrency (With GIL)

### Overview

Python's Global Interpreter Lock (GIL) prevents multiple threads from executing Python bytecode simultaneously. However, MRRC's Rust implementation releases the GIL during record parsing and I/O, enabling true parallelism for I/O-bound workloads.

**Key points:**
- ✅ GIL released during record parsing (automatic)
- ✅ Multiple threads can read/parse records concurrently
- ✅ Near-linear speedup for I/O-bound work
- ❌ Python code paths still held (minor)

### Pattern 1: ProducerConsumerPipeline (Best for Single Large File)

Purpose-built for maximum throughput from a single file:

```python
from mrrc import ProducerConsumerPipeline, PipelineConfig

# Create with default config
pipeline = ProducerConsumerPipeline.from_file(
    'large_file.mrc',
    PipelineConfig()
)

# Producer runs in background, consumer in foreground
record_count = 0
for record in pipeline.into_iter():
    # Process record
    title = record.title()
    record_count += 1

print(f"Processed {record_count} records")
```

**How it works:**
1. **Producer thread** reads file in 512 KB chunks, identifies record boundaries
2. **Bounded channel** (1000-record buffer) provides backpressure
3. **Rayon parallel parsing** processes batches of 100 records using all CPU cores
4. **Consumer thread** gets parsed records from channel
5. **GIL release** during producer reading and rayon parsing

**Performance:** **3.74x speedup** on 4-core system

**Configuration:**
```python
from mrrc import PipelineConfig

config = PipelineConfig(
    buffer_size=1024 * 1024,      # 1 MB chunks (larger = more I/O efficiency)
    channel_capacity=500,          # Queue size (smaller = lower memory)
    batch_size=50,                 # Parsing batch (smaller = lower latency)
)
pipeline = ProducerConsumerPipeline.from_file('file.mrc', config)
```

**When to use:**
- Processing single large MARC file (100+ MB)
- Want maximum throughput
- Simple one-file scenarios
- Don't care about managing threads explicitly

### Pattern 2: ThreadPoolExecutor (Good for Multiple Files)

Simple threading for concurrent file processing:

```python
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader

def process_file(filename: str) -> int:
    """Process one file (runs in thread)."""
    count = 0
    with open(filename, 'rb') as f:
        reader = MARCReader(f)
        for record in reader:
            # Process record
            count += 1
    return count

# Process 4 files in parallel
files = ['file1.mrc', 'file2.mrc', 'file3.mrc', 'file4.mrc']

with ThreadPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(process_file, files))

total = sum(results)
print(f"Processed {total} records")
```

**Key points:**
- Each thread must have its own `MARCReader` instance
- GIL released during `read_record()` calls
- Works well for multiple files or I/O patterns

**Performance:** **3-4x speedup** on 4-core system

**When to use:**
- Processing multiple files concurrently
- Simpler API preferred
- Existing ThreadPoolExecutor code
- Limited file size (doesn't need producer-consumer)

### Pattern 3: Queue-Based Producer-Consumer (Advanced)

Manual control over producer-consumer pipeline:

```python
from queue import Queue
from threading import Thread
from mrrc import MARCReader, MARCWriter

def produce(input_file, queue):
    """Read records and put on queue."""
    with open(input_file, 'rb') as f:
        reader = MARCReader(f)
        for record in reader:
            queue.put(record)
    queue.put(None)  # Sentinel

def consume(output_file, queue):
    """Get records and write."""
    with open(output_file, 'wb') as f:
        writer = MARCWriter(f)
        while True:
            record = queue.get()
            if record is None:
                break
            writer.write_record(record)

# Run producer and consumer concurrently
queue = Queue(maxsize=100)
producer = Thread(target=produce, args=('input.mrc', queue))
consumer = Thread(target=consume, args=('output.mrc', queue))

producer.start()
consumer.start()

producer.join()
consumer.join()
```

**When to use:**
- Need fine-grained control
- Custom processing between reading and writing
- Multiple producers or consumers
- Advanced patterns

### Pattern 4: Multiprocessing (For CPU-Bound Work)

Use separate processes for heavy per-record processing:

```python
from multiprocessing import Pool
from mrrc import MARCReader

def process_batch(filename: str) -> list:
    """Heavy processing per record (runs in separate process)."""
    results = []
    with open(filename, 'rb') as f:
        reader = MARCReader(f)
        for record in reader:
            # Heavy CPU work here (GIL doesn't apply)
            result = expensive_nlp(record.title())
            results.append(result)
    return results

if __name__ == '__main__':
    files = ['file1.mrc', 'file2.mrc', 'file3.mrc', 'file4.mrc']
    
    with Pool(4) as pool:
        results = pool.map(process_batch, files)
    
    total_results = sum(len(r) for r in results)
    print(f"Processed {total_results} results")
```

**Tradeoffs:**
- ✅ True parallelism for CPU-bound work
- ✅ No GIL contention
- ❌ Higher overhead (process creation, IPC)
- ❌ Cannot share in-memory data easily

**When to use:**
- Heavy per-record processing (NLP, ML inference, etc.)
- File processing is secondary
- Can tolerate process overhead

### GIL Release Details

MRRC releases the GIL during:
- ✅ `read_record()` - file I/O and parsing
- ✅ `write_record()` - serialization and write
- ✅ Format conversions (`.to_json()`, `.to_xml()`, etc.)
- ✅ Rayon parallel parsing (in ProducerConsumerPipeline)

MRRC holds the GIL during:
- ⚠️ Python object allocation
- ⚠️ Exception creation
- ⚠️ Type conversions

**Implication:** I/O-bound workloads see near-linear speedup with threading.

### Performance Comparison (Python)

| Method | Single File | Multiple Files | Speedup | Use Case |
|--------|-------------|-----------------|---------|----------|
| Sequential | Baseline | Baseline | 1x | Simple scripts |
| ProducerConsumerPipeline | 750ms | N/A | 3.74x | Single large file |
| ThreadPoolExecutor | 594ms | 644ms | 3.8x / 4.0x | Multiple files |
| Multiprocessing | ~1000ms | ~1000ms | 4-5x | CPU-heavy work |

**Notes:**
- ProducerConsumerPipeline is specialized (single file only)
- ThreadPoolExecutor works for any file pattern
- Multiprocessing has overhead but no GIL

## Comparison: Rust vs Python

| Aspect | Rust | Python |
|--------|------|--------|
| **Speedup (4 cores)** | 2.5x (I/O-limited) | 3.74x (specialized) |
| **Ease of use** | Explicit (rayon API) | Simple (ThreadPoolExecutor) |
| **GIL** | No GIL | GIL released during parsing |
| **Memory** | Lower | Higher (Python objects) |
| **Best for** | Batch processing | Web apps, scripts |
| **API** | Iterators, closures | Familiar threading |

## Common Patterns

### Load Balance Check

Monitor Rayon's load distribution:

```rust
use rayon::prelude::*;

let results: Vec<_> = items
    .par_iter()
    .map(|item| {
        let thread_id = rayon::current_thread_index();
        println!("Processing on thread {}", thread_id);
        process(item)
    })
    .collect();
```

### Dynamic Work Adjustment

Adjust parallelism based on workload:

```rust
// For small datasets, use sequential
if records.len() < 100 {
    return records.iter().map(process).collect();
}

// For larger datasets, parallelize
records.par_iter().map(process).collect()
```

### Batch Processing with Error Handling

Handle errors in parallel operations:

```rust
let results: Vec<Result<_, _>> = records
    .par_iter()
    .map(|record| process_record(record))
    .collect();

let success_count = results.iter().filter(|r| r.is_ok()).count();
let error_count = results.iter().filter(|r| r.is_err()).count();

println!("Success: {}, Errors: {}", success_count, error_count);
```

## Thread Safety

### Rust

Rust's type system ensures thread safety at compile time. No explicit synchronization needed for `par_iter()` operations.

### Python

**Safe:**
- Each thread has own `MARCReader` instance ✅
- Each thread has own `MARCWriter` instance ✅
- Reading different files in parallel ✅
- ProducerConsumerPipeline (internally synchronized) ✅

**Unsafe:**
- Sharing single `MARCReader` across threads ❌
- Sharing single `MARCWriter` across threads ❌
- Concurrent modifications to same record ❌

## Debugging

### Rust (Rayon)

```rust
// Limit to single thread for debugging
std::env::set_var("RAYON_NUM_THREADS", "1");
```

### Python (Threading)

```python
import logging
import threading

logging.basicConfig(
    format='%(threadName)-10s %(levelname)-8s %(message)s'
)

logger = logging.getLogger()
logger.setLevel(logging.DEBUG)

def worker(filename):
    logger.info(f"Processing {filename}")
    # ... work ...
    logger.info(f"Done with {filename}")
```

## References

- [Rayon Documentation](https://docs.rs/rayon/)
- [Python Threading](https://docs.python.org/3/library/threading.html)
- [Python GIL](https://docs.python.org/3/glossary.html#term-GIL)
- [PyO3 Threading](https://pyo3.rs/latest/advanced/)
