# Concurrency Patterns in MRRC

This document covers parallel processing patterns for both Rust and Python. For thread safety details and GIL behavior, see [THREADING.md](THREADING.md). For performance tuning, see [PERFORMANCE.md](PERFORMANCE.md).

## Quick Decision Tree

```
Do you need to process MARC records in parallel?
├─ YES
│  ├─ Are you using Rust?
│  │  └─ Use Rayon for data parallelism (work-stealing scheduler)
│  │
│  └─ Are you using Python?
│     ├─ Single large file?
│     │  └─ Use ProducerConsumerPipeline (best: 3.74x speedup)
│     │
│     └─ Multiple files or simpler needs?
│        └─ Use ThreadPoolExecutor (good: 3-4x speedup)
│
└─ NO
   └─ Sequential processing is fine
```

## Rust Concurrency

Rust uses **Rayon**, a work-stealing parallelism library with compile-time safety guarantees.

### Pattern 1: Parallel Iterator (Recommended)

```rust
use rayon::prelude::*;
use mrrc::MarcReader;
use std::fs::File;

fn main() -> Result<()> {
    let file = File::open("records.mrc")?;
    let mut reader = MarcReader::new(file);

    // Load records into memory
    let records: Vec<_> = reader.collect::<Result<Vec<_>, _>>()?;

    // Process in parallel
    let results: Vec<_> = records
        .par_iter()
        .map(|record| process_record(record))
        .collect();

    println!("Processed {} records", results.len());
    Ok(())
}
```

**When to use:** Batch processing records already in memory with CPU-intensive work.

### Pattern 2: Parallel Chunks

```rust
use rayon::prelude::*;

let results: Vec<_> = records
    .par_chunks(100)  // Process 100 records at a time
    .flat_map(|chunk| {
        chunk.iter().map(|record| extract_data(record))
    })
    .collect();
```

**When to use:** Memory-intensive processing, grouping related records.

### Pattern 3: Fold-Reduce (Aggregation)

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

**When to use:** Counting, summing, grouping, collecting unique values.

### Rust Thread Pool Configuration

```rust
use rayon::ThreadPoolBuilder;

rayon::ThreadPoolBuilder::new()
    .num_threads(2)
    .build_global()
    .unwrap();
```

## Python Concurrency

Python's GIL limits parallelism, but MRRC releases the GIL during parsing and I/O, enabling true parallelism for I/O-bound workloads.

### Pattern 1: ProducerConsumerPipeline (Single Large File)

Purpose-built for maximum throughput from a single file:

```python
from mrrc import ProducerConsumerPipeline

pipeline = ProducerConsumerPipeline.from_file('large_file.mrc')

record_count = 0
for record in pipeline:
    title = record.title()
    record_count += 1

print(f"Processed {record_count} records")
```

**How it works:**
1. Producer thread reads file in 512 KB chunks
2. Bounded channel (1000 records) provides backpressure
3. Rayon parses batches of 100 records in parallel
4. GIL released during producer reading and parsing

**Performance:** 3.74x speedup on 4-core system

**Configuration:**
```python
pipeline = ProducerConsumerPipeline.from_file(
    'file.mrc',
    buffer_size=1024 * 1024,  # 1 MB chunks (default: 512 KB)
    channel_capacity=500       # Queue size (default: 1000)
)
```

### Pattern 2: ThreadPoolExecutor (Multiple Files)

```python
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader

def process_file(filename: str) -> int:
    count = 0
    with open(filename, 'rb') as f:
        reader = MARCReader(f)
        for record in reader:
            count += 1
    return count

files = ['file1.mrc', 'file2.mrc', 'file3.mrc', 'file4.mrc']

with ThreadPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(process_file, files))

total = sum(results)
print(f"Processed {total} records")
```

**Performance:** 3-4x speedup on 4-core system

### Pattern 3: Queue-Based Producer-Consumer

Manual control over producer-consumer pipeline:

```python
from queue import Queue
from threading import Thread
from mrrc import MARCReader, MARCWriter

def produce(input_file, queue):
    with open(input_file, 'rb') as f:
        reader = MARCReader(f)
        for record in reader:
            queue.put(record)
    queue.put(None)  # Sentinel

def consume(output_file, queue):
    with open(output_file, 'wb') as f:
        writer = MARCWriter(f)
        while True:
            record = queue.get()
            if record is None:
                break
            writer.write_record(record)

queue = Queue(maxsize=100)
producer = Thread(target=produce, args=('input.mrc', queue))
consumer = Thread(target=consume, args=('output.mrc', queue))

producer.start()
consumer.start()
producer.join()
consumer.join()
```

**When to use:** Fine-grained control, custom processing, multiple producers/consumers.

### Pattern 4: Multiprocessing (CPU-Bound Work)

```python
from multiprocessing import Pool
from mrrc import MARCReader

def process_batch(filename: str) -> list:
    results = []
    with open(filename, 'rb') as f:
        reader = MARCReader(f)
        for record in reader:
            result = expensive_nlp(record.title())
            results.append(result)
    return results

if __name__ == '__main__':
    files = ['file1.mrc', 'file2.mrc', 'file3.mrc', 'file4.mrc']

    with Pool(4) as pool:
        results = pool.map(process_batch, files)

    print(f"Processed {sum(len(r) for r in results)} results")
```

**When to use:** Heavy per-record CPU work (NLP, ML inference) where threading isn't enough.

## Pattern Comparison

| Pattern | Best For | Speedup (4 cores) | Complexity |
|---------|----------|-------------------|------------|
| ProducerConsumerPipeline | Single large file | 3.74x | Simple |
| ThreadPoolExecutor | Multiple files | 3-4x | Simple |
| Queue producer-consumer | Custom pipelines | 3-4x | Medium |
| Multiprocessing | CPU-heavy work | 4-5x | Medium |

## Rust vs Python

| Aspect | Rust | Python |
|--------|------|--------|
| **Speedup (4 cores)** | 2.5x (I/O-limited) | 3.74x (ProducerConsumerPipeline) |
| **GIL** | No GIL | Released during parsing |
| **Memory** | Lower | Higher (Python objects) |
| **Best for** | Batch processing | Web apps, scripts |

## Debugging

### Rust (Rayon)

```rust
// Limit to single thread for debugging
std::env::set_var("RAYON_NUM_THREADS", "1");
```

### Python

```python
import logging

logging.basicConfig(
    format='%(threadName)-10s %(levelname)-8s %(message)s'
)
logger = logging.getLogger()
logger.setLevel(logging.DEBUG)
```

## References

- [THREADING.md](THREADING.md) - Thread safety and GIL behavior
- [PERFORMANCE.md](PERFORMANCE.md) - Performance tuning and benchmarks
- [Rayon Documentation](https://docs.rs/rayon/)
- [Python Threading](https://docs.python.org/3/library/threading.html)
