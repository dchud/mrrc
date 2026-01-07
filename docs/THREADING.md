# Threading and Concurrency with MRRC

MRRC's Python wrapper is designed for concurrent and parallel processing. This document explains GIL behavior and provides practical examples.

## GIL Behavior

### What is the GIL?

The Python Global Interpreter Lock (GIL) prevents multiple threads from executing Python bytecode simultaneously. This means:
- Pure Python code cannot benefit from multi-core systems
- I/O operations can release the GIL for parallelism

### MRRC's GIL Release Policy

MRRC **releases the GIL** during:
- ✅ **File I/O**: `read_record()`, `write_record()` - Rust code runs natively
- ✅ **Parsing**: Record parsing, field access - Rust implementation
- ✅ **Serialization**: Format conversions (JSON, XML, MARCJSON)

MRRC **holds the GIL** during:
- ⚠️ Python exception creation
- ⚠️ Type conversions between Python and Rust
- ⚠️ Python object allocation

**Implication**: I/O-bound workloads see near-linear speedup with threading. CPU-bound workloads should use `multiprocessing`.

## Usage Patterns

### Pattern 1: ProducerConsumerPipeline (Recommended for Single-File Multi-Threading)

The `ProducerConsumerPipeline` is purpose-built for high-performance multi-threaded reading from a single MARC file:

```python
from mrrc import ProducerConsumerPipeline, PipelineConfig

# Create pipeline with default config (512 KB buffer, 1000-record channel, 100-record batches)
pipeline = ProducerConsumerPipeline.from_file('large_file.mrc', PipelineConfig())

# Iterate over records - producer runs in background thread
record_count = 0
for record in pipeline.into_iter():
    # Process record
    title = record.title()
    # ... do something with record ...
    record_count += 1

print(f"Processed {record_count} records")
```

**How it works:**
- **Producer thread** reads file in 512 KB chunks, scans record boundaries, and parses in parallel with Rayon
- **Bounded channel** (1000 records) provides backpressure - producer blocks if consumer is slow
- **GIL release** - producer runs without GIL; only consumer acquisitions block on Rust code
- **Parallel parsing** - batches of 100 records parsed in parallel with Rayon (exploits all CPU cores)

**Benefits**:
- **3.74x speedup** on 4-core system for large files
- No Python threading complexity
- Automatic resource cleanup
- Memory-efficient: bounded channel prevents unbounded buffering

**Use this when:**
- Processing a single large MARC file
- Want maximum throughput from one file
- Want automatic parallelism without managing threads

**Configuration tuning** (optional):
```python
from mrrc import PipelineConfig

config = PipelineConfig(
    buffer_size=1024 * 1024,  # 1 MB chunks (larger for sequential I/O)
    channel_capacity=500,      # Smaller queue = lower memory
    batch_size=50,             # Smaller batches = lower latency
)
pipeline = ProducerConsumerPipeline.from_file('file.mrc', config)
```

### Pattern 2: Threading with Concurrent.futures (For Multi-File Processing)

For processing multiple files concurrently (or simpler single-file processing):

```python
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader

def process_marc_file(filename):
    """Process a MARC file and return record count."""
    count = 0
    with open(filename, 'rb') as f:
        reader = MARCReader(f)
        while record := reader.read_record():
            # Process record
            title = record.title()
            # ... do something with record ...
            count += 1
    return count

# Process 4 files in parallel
files = ['file1.mrc', 'file2.mrc', 'file3.mrc', 'file4.mrc']

with ThreadPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(process_marc_file, files))

total_records = sum(results)
print(f"Processed {total_records} total records")
```

**Benefits**:
- Simple API
- Automatic thread pool management
- Works well for file processing

**Performance**: ~3-4x speedup on 4-core system for I/O-bound work

### Pattern 2: Multiprocessing (For CPU-Bound Work)

If you're doing heavy processing per record, use `multiprocessing`:

```python
from multiprocessing import Pool
from mrrc import MARCReader
import json

def process_batch(filename, output_file):
    """Process records from one file and write results."""
    results = []
    with open(filename, 'rb') as f:
        reader = MARCReader(f)
        while record := reader.read_record():
            # Heavy processing
            data = {
                'title': record.title(),
                'author': record.get_fields('100')[0].get_subfield('a') if record.get_fields('100') else None,
                'subjects': [f.get_subfield('a') for f in record.get_fields('650')],
                'year': record.get_fields('008')[0].data[7:11] if record.get_fields('008') else None,
            }
            results.append(data)
    
    # Write output
    with open(output_file, 'w') as f:
        json.dump(results, f)
    
    return len(results)

if __name__ == '__main__':
    files = [
        ('file1.mrc', 'results1.json'),
        ('file2.mrc', 'results2.json'),
        ('file3.mrc', 'results3.json'),
        ('file4.mrc', 'results4.json'),
    ]
    
    with Pool(4) as pool:
        results = [pool.apply_async(process_batch, f) for f in files]
        total = sum(r.get() for r in results)
    
    print(f"Processed {total} records")
```

**Benefits**:
- True parallelism (no GIL contention)
- Best for CPU-intensive work

**Drawback**: Higher overhead (process creation, IPC)

### Pattern 3: Async/Await (Not Yet Supported)

MRRC does not yet support async/await. Use threading for I/O-bound work instead.

## Performance Characteristics

### Throughput by Concurrency Method

```
Method                              Scalability    Overhead    Use Case
─────────────────────────────────────────────────────────────────────────
Sequential (single-threaded)        N/A (1x)       -           Baseline
ProducerConsumerPipeline            3.74x          Low         Single large file
Threading with ThreadPoolExecutor   3-4x           Low         Multiple files
Multiprocessing                     4-5x           High        CPU-heavy processing
```

### Expected Speedups

With 4-core system:

| Workload | Sequential | Concurrency Method | Speedup | Notes |
|----------|-----------|-------------------|---------|-------|
| ProducerConsumerPipeline (280 MB MARC) | 750 ms | Pipeline | 3.74x | Rayon parallel parsing |
| Multi-file with ThreadPoolExecutor | 644 ms | Threaded | 4.0x | 4 × 10k records, separate files |
| Reading + extracting | 594 ms | Threaded | 3.8x | Mixed I/O and processing |
| Reading + JSON conversion | 800+ ms | Threaded | 3.2x | Serialization overhead |

**Note**: `ProducerConsumerPipeline` achieves better scaling than manual threading by avoiding Python thread management overhead and leveraging Rayon's work-stealing for parallel record parsing.

## Thread Safety

### Is MRRC Thread-Safe?

**Yes, but with caveats:**

✅ **Thread-safe:**
- Creating multiple `MARCReader` instances
- Each thread reading from different files
- Reading from different offsets in the same file (with `seek()`)

❌ **Not thread-safe:**
- Sharing a single `MARCReader` between threads
- Concurrent modifications to the same `Record` object
- Concurrent writes to the same output file

### Best Practice: Thread Confinement

Each thread should have its own reader:

```python
# ✅ GOOD: Each thread has its own reader
def process_file(filename):
    reader = MARCReader(open(filename, 'rb'))  # Reader per thread
    while record := reader.read_record():
        # process record
```

```python
# ❌ BAD: Sharing reader across threads
reader = MARCReader(open(file, 'rb'))
def worker():
    while record := reader.read_record():  # Race condition!
        # process record
```

## Memory Usage with Threading

### GIL and Memory

The GIL doesn't prevent memory leaks or races in extension modules. However:

- **Per-thread overhead**: ~8 MB (OS-dependent)
- **MRRC record objects**: Shared heap, reference counted
- **GC**: CPython GC still works normally across threads

### Preventing Memory Leaks

```python
# ✅ GOOD: Close resources explicitly
def process_file(filename):
    with open(filename, 'rb') as f:
        reader = MARCReader(f)
        while record := reader.read_record():
            process(record)
    # File and reader are closed

# ❌ BAD: Implicit cleanup
def process_file(filename):
    reader = MARCReader(open(filename, 'rb'))
    while record := reader.read_record():
        process(record)
    # File only closed at garbage collection
```

## Common Patterns and Gotchas

### Pattern: Producer-Consumer with Queue

Process records from one file while writing to another:

```python
from queue import Queue
from threading import Thread
from mrrc import MARCReader, MARCWriter

def produce(input_file, queue):
    """Read records and put on queue."""
    with open(input_file, 'rb') as f:
        reader = MARCReader(f)
        while record := reader.read_record():
            queue.put(record)
    queue.put(None)  # Sentinel

def consume(output_file, queue):
    """Get records from queue and write."""
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

### Gotcha: String Concatenation in Threads

Be careful with `str` objects in tight loops (GIL is re-acquired):

```python
# ❌ Not ideal: Frequent Python operations
while record := reader.read_record():
    title = record.title()  # This releases GIL
    msg = f"Processing: {title}"  # GIL re-acquired
    print(msg)  # GIL re-acquired

# ✅ Better: Minimize Python work
while record := reader.read_record():
    title = record.title()
    # Batch processing or I/O
    do_heavy_processing(title)
```

### Gotcha: Callback Hell

Avoid callback-heavy patterns with threads:

```python
# ❌ Complex callback chain
executor.submit(lambda: process_file('f1.mrc', callback1))

# ✅ Simpler approach
results = executor.map(process_file, ['f1.mrc', 'f2.mrc'])
```

## Debugging Concurrent Programs

### Enable Thread Logging

```python
import logging
import threading

logging.basicConfig(
    format='%(threadName)-10s %(levelname)-8s %(message)s',
    level=logging.DEBUG
)

logger = logging.getLogger()

def process_file(filename):
    logger.info(f"Processing {filename}")
    # ... process ...
    logger.info(f"Done with {filename}")

# Now log messages include thread name
```

### Detect Deadlocks with Timeout

```python
from concurrent.futures import ThreadPoolExecutor
import signal

def timeout_handler(signum, frame):
    raise TimeoutError("Benchmark timed out")

signal.signal(signal.SIGALRM, timeout_handler)
signal.alarm(30)  # 30-second timeout

try:
    with ThreadPoolExecutor(max_workers=4) as executor:
        results = executor.map(process_file, files, timeout=5)
finally:
    signal.alarm(0)  # Cancel timeout
```

## Performance Tuning

### Thread Pool Size

```python
import os

optimal_workers = min(
    4,  # CPU cores
    len(files),  # Number of files
    32  # Max pool size
)

with ThreadPoolExecutor(max_workers=optimal_workers) as executor:
    results = executor.map(process_file, files)
```

### Batch Size Optimization

For producer-consumer patterns:

```python
# Tune queue size based on record and memory usage
BATCH_SIZE = 1000  # Adjust based on memory
queue = Queue(maxsize=BATCH_SIZE // 10)
```

## Limitations and Future Work

### Current Limitations

1. ❌ No async/await support
2. ❌ No background thread safety for record modifications
3. ❌ No distributed processing built-in

### Current Capabilities

- ✅ `ProducerConsumerPipeline` for single-file multi-threaded reading (3.74x speedup)
- ✅ `ThreadPoolExecutor` support for multi-file concurrent processing
- ✅ GIL release during I/O and parsing
- ✅ Rayon-based parallel record parsing

### Planned Improvements

- [ ] `async` reader/writer support
- [ ] Convenience functions for ProducerConsumerPipeline configuration
- [ ] Memory-mapped file support

## References

- [Python GIL Documentation](https://docs.python.org/3/glossary.html#term-GIL)
- [Concurrent.futures Documentation](https://docs.python.org/3/library/concurrent.futures.html)
- [Threading Best Practices](https://docs.python.org/3/library/threading.html)
- [PyO3 Threading Guide](https://pyo3.rs/latest/advanced/index.html)
