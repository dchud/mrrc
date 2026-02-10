# Threading and Thread Safety in MRRC

This document covers thread safety guarantees, GIL behavior, and safe usage patterns for MRRC. For parallel processing patterns and code examples, see [CONCURRENCY.md](CONCURRENCY.md).

## GIL Behavior

### What is the GIL?

The Python Global Interpreter Lock (GIL) prevents multiple threads from executing Python bytecode simultaneously. This means:
- Pure Python code cannot benefit from multi-core systems
- I/O operations can release the GIL for parallelism

### MRRC's GIL Release Policy

MRRC **releases the GIL** during:
- **File I/O**: `read_record()`, `write_record()` - Rust code runs natively
- **Parsing**: Record parsing, field access - Rust implementation
- **Serialization**: Format conversions (JSON, XML, MARCJSON)

MRRC **holds the GIL** during:
- Python exception creation
- Type conversions between Python and Rust
- Python object allocation

**Implication**: I/O-bound workloads see near-linear speedup with threading. CPU-bound workloads should use `multiprocessing`.

### Three-Phase GIL Management

Each record parse uses a three-phase pattern:

1. **Phase 1 (GIL Held)**: Acquire raw bytes from source - minimal work
2. **Phase 2 (GIL Released)**: Parse MARC bytes in Rust - CPU-intensive, other threads can run
3. **Phase 3 (GIL Re-acquired)**: Convert to Python objects, handle errors

## Thread Safety

### Is MRRC Thread-Safe?

**Yes, but with caveats:**

**Thread-safe:**
- Creating multiple `MARCReader` instances
- Each thread reading from different files
- Reading from different offsets in the same file (with `seek()`)
- Using `ProducerConsumerPipeline` (internally synchronized)

**Not thread-safe:**
- Sharing a single `MARCReader` between threads
- Concurrent modifications to the same `Record` object
- Concurrent writes to the same output file

### Best Practice: Thread Confinement

Each thread should have its own reader:

```python
# GOOD: Each thread has its own reader
def process_file(filename):
    reader = MARCReader(open(filename, 'rb'))  # Reader per thread
    while record := reader.read_record():
        # process record
```

```python
# BAD: Sharing reader across threads
reader = MARCReader(open(file, 'rb'))
def worker():
    while record := reader.read_record():  # Race condition!
        # process record
```

## Memory Usage with Threading

### Per-Thread Overhead

- **Per-thread**: ~8 MB (OS-dependent)
- **Per-reader**: ~4 KB (parsing buffer)
- **Per-record in memory**: ~4 KB (typical)
- **Memory regression vs single-threaded**: < 5%

### Preventing Memory Leaks

```python
# GOOD: Close resources explicitly
def process_file(filename):
    with open(filename, 'rb') as f:
        reader = MARCReader(f)
        while record := reader.read_record():
            process(record)
    # File and reader are closed

# BAD: Implicit cleanup
def process_file(filename):
    reader = MARCReader(open(filename, 'rb'))
    while record := reader.read_record():
        process(record)
    # File only closed at garbage collection
```

## Common Gotchas

### String Operations Re-acquire GIL

Be careful with `str` objects in tight loops:

```python
# Not ideal: Frequent Python operations
while record := reader.read_record():
    title = record.title()  # GIL released
    msg = f"Processing: {title}"  # GIL re-acquired
    print(msg)  # GIL re-acquired

# Better: Minimize Python work per iteration
while record := reader.read_record():
    title = record.title()
    do_heavy_processing(title)  # Batch Python work
```

### Callback Complexity

Avoid callback-heavy patterns:

```python
# Complex callback chain
executor.submit(lambda: process_file('f1.mrc', callback1))

# Simpler approach
results = executor.map(process_file, ['f1.mrc', 'f2.mrc'])
```

## Debugging Concurrent Programs

### Enable Thread Logging

```python
import logging

logging.basicConfig(
    format='%(threadName)-10s %(levelname)-8s %(message)s',
    level=logging.DEBUG
)

logger = logging.getLogger()

def process_file(filename):
    logger.info(f"Processing {filename}")
    # ... process ...
    logger.info(f"Done with {filename}")
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

## Limitations and Future Work

### Current Limitations

- No async/await support
- No background thread safety for record modifications
- No distributed processing built-in

### Planned Improvements

- `async` reader/writer support
- Convenience functions for ProducerConsumerPipeline configuration
- Memory-mapped file support

## References

- [CONCURRENCY.md](CONCURRENCY.md) - Parallel processing patterns and code examples
- [PERFORMANCE.md](PERFORMANCE.md) - Performance tuning and benchmarks
- [Python GIL Documentation](https://docs.python.org/3/glossary.html#term-GIL)
- [Concurrent.futures Documentation](https://docs.python.org/3/library/concurrent.futures.html)
- [PyO3 Threading Guide](https://pyo3.rs/latest/advanced/index.html)
