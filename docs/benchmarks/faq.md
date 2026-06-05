# Benchmarking FAQ

## Performance Questions

### Is pymrrc faster than pymarc?

Early benchmarking suggested at least a 4x single-threaded speedup over
pymarc; these benchmarks need to be updated and reconsidered. The mechanism
behind the speedup is structural: each record is parsed by compiled Rust code
instead of interpreted Python. To measure the difference on your own workload,
see [Results](results.md).

### Does pymrrc automatically use multiple cores?

No. By default, pymrrc runs on one thread (like pymarc), but each record
parses faster because parsing happens in Rust.

To use multiple cores:

- Use `ProducerConsumerPipeline` for single-file processing
- Use `ThreadPoolExecutor` to process multiple files in parallel

### Do I need to change my code to get the single-threaded speedup?

No. If upgrading from pymarc, install pymrrc and existing code benefits
automatically.

### Do I need to change my code for multi-threading?

Yes. For single-file multi-threaded processing, use
`ProducerConsumerPipeline`. For multi-file processing, use
`ThreadPoolExecutor`. See the [threading guide](../guides/threading-python.md)
for examples.

### Is GIL release automatic?

Yes. Every call to `read_record()` automatically releases the GIL during
parsing. No special handling required.

---

## Understanding the Speedups

### Single-Threaded

```python
from mrrc import MARCReader

# Parsing runs in Rust; no per-record Python interpretation.
# Pass a path (not a file object) so I/O also runs in Rust, GIL-free.
reader = MARCReader("records.mrc")
while record := reader.read_record():
    title = record.title
    # ... process record ...
```

The Python wrapper adds overhead relative to pure Rust (each record crosses
the Rust/Python boundary), but record parsing itself runs at Rust speed.
File objects also work, but the GIL is held for their `.read()` calls —
prefer paths.

### Multi-Threaded

```python
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader

def process_file(path):
    # Each thread gets its own reader; path input keeps I/O in Rust,
    # so threads never contend on the GIL for reads.
    reader = MARCReader(path)
    while record := reader.read_record():
        ...  # process record

# Process 4 files in parallel
with ThreadPoolExecutor(max_workers=4) as executor:
    executor.map(process_file, ["file1.mrc", "file2.mrc", "file3.mrc", "file4.mrc"])
```

Because the GIL is released during parsing, threads parse concurrently and
speedup scales with core count. Efficiency is sub-linear: thread management,
memory bandwidth, and the Python code between records all take their share.
Measure with your own files to find the practical ceiling on your hardware.

Use multi-threading when processing multiple independent files. For single
files, use `ProducerConsumerPipeline`.

---

## Upgrade Considerations

### When pymrrc provides clear benefit:

- Processing large files (100k+ records)
- MARC processing is a bottleneck in your workflow
- You need multi-threading for parallel file processing

### When pymrrc may not be necessary:

- Pure Python environment required (no C extensions)
- Deeply integrated custom code with pymarc internals
- Processing very small files (< 1k records)

---

## Benchmark Selection

### Which benchmark matches my use case?

- **Raw reading** (`test_benchmark_reading.py`): best case for mrrc
- **Reading + field extraction** (`test_benchmark_reading.py`): typical use
- **Round-trip read + write** (`test_benchmark_writing.py`): format conversion
- **Parallel processing** (`test_benchmark_parallel.py`,
  `test_benchmark_pipeline_parallel.py`): processing many files or one large
  file with threads (local runs only — see [Results](results.md))

### What does "rec/s" mean?

Records per second. Higher is better. Throughput depends on hardware, record
size, and what your code does with each record — measure on your own data.

---

## GIL and Threading

### What is the GIL?

Global Interpreter Lock. Python's mechanism to make the interpreter
thread-safe. Only one thread can execute Python bytecode at a time. When one
thread releases the GIL, other threads can run.

### Does pymrrc release the GIL?

Yes. During record parsing, the GIL is released via `py.detach()`. This
happens automatically with every `read_record()` call.

### Do I need to think about the GIL?

No. In single-threaded mode, it's automatic and transparent. In
multi-threaded mode, the GIL release enables concurrent parsing.

### What happens if I share a reader across threads?

Undefined behavior. The reader is not thread-safe. Create a new reader per
thread.

```python
# Wrong - do not share readers across threads
reader = MARCReader("file.mrc")
ThreadPoolExecutor().map(lambda _: reader.read_record(), range(4))

# Correct - each thread gets its own reader
def read_all(path):
    reader = MARCReader(path)
    while record := reader.read_record():
        # ... process ...

ThreadPoolExecutor().map(read_all, ["file1.mrc", "file2.mrc", "file3.mrc", "file4.mrc"])
```

---

## Performance Tuning

### How can I make pymrrc faster?

1. **File path input:** Pass file paths directly instead of file objects
2. **Multi-threading:** Use `ThreadPoolExecutor` for multiple files,
   `ProducerConsumerPipeline` for single files
3. **Batch processing:** Read multiple records at once
4. **Use Rust:** Rewrite critical path in Rust if possible

### Does record size affect performance?

Minimally. Throughput stays consistent regardless of record size.

### Does Python version matter?

Only for availability. pymrrc supports Python 3.10+. Performance is similar
across versions.

### Does OS matter?

Only for binary compatibility. pymrrc provides wheels for Linux, macOS, and
Windows. Performance is similar across platforms.

---

## Further Reading

- [Results](results.md) - Measurement infrastructure and benchmarking
  procedure
- [Threading Guide](../guides/threading-python.md) - Threading patterns and
  code examples
- [Performance Tuning](../guides/performance-tuning.md) - Usage patterns and
  optimization
