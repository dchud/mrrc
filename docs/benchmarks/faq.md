# Benchmarking FAQ

## Performance Questions

### Is pymrrc faster than pymarc?

Yes. pymrrc is ~4x faster in single-threaded mode with no code changes required.

### Does pymrrc automatically use multiple cores?

No. By default, pymrrc runs on one thread (like pymarc), but each record parses faster in Rust, providing ~4x speedup automatically.

To use multiple cores:
- Use `ProducerConsumerPipeline` for single-file high-throughput processing (~3.7x speedup on 4 cores)
- Use `ThreadPoolExecutor` to process multiple files in parallel (~3-4x speedup)

### What do the different speedup numbers mean?

- **~4x** = how much faster pymrrc is than pymarc in single-threaded mode (automatic)
- **~3-4x** = additional speedup from explicit multi-threading with `ThreadPoolExecutor` or `ProducerConsumerPipeline`

These stack: pymrrc with 4-thread processing is ~13x faster than single-threaded pymarc.

### Do I need to change my code to get the ~4x speedup?

No. If upgrading from pymarc, install pymrrc and existing code runs ~4x faster automatically.

### Do I need to change my code for multi-threading?

Yes. For single-file multi-threaded processing, use `ProducerConsumerPipeline`. For multi-file processing, use `ThreadPoolExecutor`. See the [threading guide](../guides/threading-python.md) for examples.

### Is GIL release automatic?

Yes. Every call to `read_record()` automatically releases the GIL during parsing. No special handling required.

---

## Understanding the Speedups

### Single-Threaded Speedup (~4x)

```python
from mrrc import MARCReader

# This code runs ~4x faster than pymarc automatically
with open("records.mrc", "rb") as f:
    reader = MARCReader(f)
    while record := reader.read_record():
        title = record.title()
        # ... process record ...
```

Speedup breakdown:
- Pure Rust implementation: ~14x faster than pymarc
- Python wrapper overhead: ~3x slower than pure Rust
- Net: ~4x faster than pymarc

### Multi-Threaded Speedup (2.0x-3.74x)

```python
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader

def process_file(path):
    with open(path, "rb") as f:
        reader = MARCReader(f)  # Each thread gets its own reader
        while record := reader.read_record():
            # ... process record ...

# Process 4 files in parallel (~3.74x faster than sequential)
with ThreadPoolExecutor(max_workers=4) as executor:
    executor.map(process_file, ["file1.mrc", "file2.mrc", "file3.mrc", "file4.mrc"])
```

Why 3.74x instead of 4x:
- Thread creation/management overhead: ~5-10%
- Memory bandwidth saturation: ~5-10%
- Python scheduler overhead: ~5-10%
- Result: ~94% efficiency on 4 cores = 3.74x speedup

Use multi-threading when processing multiple independent files. For single files, use `ProducerConsumerPipeline`.

---

## Performance Comparisons

### Scenario: Process 1 Million Records

| Approach | Time | Notes |
|----------|------|-------|
| pymarc (1 thread) | 14.3 seconds | Baseline |
| pymrrc (1 thread) | 3.3 seconds | ~4x faster, automatic |
| pymrrc (4 threads) | 1.1 seconds | ~13x faster, requires ThreadPoolExecutor |
| Rust mrrc (4 threads) | 0.25 seconds | ~57x faster, requires Rust |

### Scenario: Process 100 MARC Files (100k records each)

| Approach | Time | Throughput |
|----------|------|------------|
| pymarc (sequential) | ~24 minutes | 1.2 MB/s |
| pymrrc (sequential) | ~6 minutes | 5 MB/s (~4x faster) |
| pymrrc (4 threads) | ~2 minutes | 15 MB/s (~12x faster) |

---

## Upgrade Considerations

### When pymrrc provides clear benefit:

- Processing large files (100k+ records)
- MARC processing is a bottleneck in your workflow
- You want lower resource usage (pymrrc uses less CPU/memory)
- You need multi-threading for parallel file processing

### When pymrrc may not be necessary:

- Pure Python environment required (no C extensions)
- Deeply integrated custom code with pymarc internals
- Processing very small files (< 1k records)

---

## Benchmark Selection

### Which benchmark matches my use case?

- **Tests 1-2 (Raw reading):** Best case for mrrc
- **Tests 3-4 (Reading + field extraction):** Typical use case
- **Tests 5-6 (Round-trip read + write):** Format conversions
- **Tests 9-10 (Multi-threading):** Processing many files

### What does "rec/s" mean?

Records per second. Higher is better.

- Rust: ~1,000,000 rec/s = 1 million records in ~1 second
- pymrrc: ~300,000 rec/s = 1 million records in ~3.3 seconds
- pymarc: ~70,000 rec/s = 1 million records in ~14.3 seconds

---

## GIL and Threading

### What is the GIL?

Global Interpreter Lock. Python's mechanism to make the interpreter thread-safe. Only one thread can execute Python bytecode at a time. When one thread releases the GIL, other threads can run.

### Does pymrrc release the GIL?

Yes. During record parsing, the GIL is released via `py.detach()`. This happens automatically with every `read_record()` call.

### Do I need to think about the GIL?

No. In single-threaded mode, it's automatic and transparent. In multi-threaded mode, the GIL release enables concurrent parsing.

### What happens if I share a reader across threads?

Undefined behavior. The reader is not thread-safe. Create a new reader per thread.

```python
# Wrong - do not share readers across threads
reader = MARCReader(open("file.mrc", "rb"))
ThreadPoolExecutor().map(lambda _: reader.read_record(), range(4))

# Correct - each thread gets its own reader
def read_all(path):
    reader = MARCReader(open(path, "rb"))
    while record := reader.read_record():
        # ... process ...

ThreadPoolExecutor().map(read_all, ["file1.mrc", "file2.mrc", "file3.mrc", "file4.mrc"])
```

---

## Performance Tuning

### How can I make pymrrc faster?

1. **File path input:** Pass file paths directly instead of file objects
2. **Multi-threading:** Use `ThreadPoolExecutor` for multiple files, `ProducerConsumerPipeline` for single files
3. **Batch processing:** Read multiple records at once
4. **Use Rust:** Rewrite critical path in Rust if possible

### Does record size affect performance?

Minimally. Throughput stays consistent regardless of record size.

### Does Python version matter?

Only for availability. pymrrc supports Python 3.10+. Performance is similar across versions.

### Does OS matter?

Only for binary compatibility. pymrrc provides wheels for Linux, macOS, and Windows. Performance is similar across platforms.

---

## Further Reading

- [Results](results.md) - Complete benchmark data
- [Threading Guide](../guides/threading-python.md) - Threading patterns and code examples
- [Performance Tuning](../guides/performance-tuning.md) - Usage patterns and optimization
