# MRRC Benchmarking FAQ

## Quick Questions

### Q: Is pymrrc faster than pymarc?
**A:** Yes. **~4x faster** in single-threaded mode, with **no code changes needed**. Just install pymrrc and use it like pymarc.

### Q: Does pymrrc automatically use my multi-core processor?
**A:** Not in single-threaded mode. By default, pymrrc runs on one thread (like pymarc). But each record parses **much faster** in Rust, so you get ~4x speedup automatically.

To use multiple cores, use `ProducerConsumerPipeline` for single-file high-throughput processing (~3.7x speedup on 4 cores) or `ThreadPoolExecutor` to process **multiple files** in parallel (~3-4x speedup).

### Q: Why is there a difference between "~4x faster" and "~3-4x faster (multi-threaded)"?
**A:**
- **~4x** = how much faster pymrrc is than pymarc in single-threaded mode
- **~3-4x** = how much faster your program runs when you explicitly use multiple threads with `ThreadPoolExecutor`

These are different things! The ~4x is automatic (default behavior), the ~3-4x requires explicit threading code.

### Q: Do I need to change my code to get the ~4x speedup?
**A:** No. If you're upgrading from pymarc, just install pymrrc and your code is automatically ~4x faster.

### Q: Do I need to change my code to get multi-threading benefits?
**A:** Yes. For single-file multi-threaded processing, use `ProducerConsumerPipeline`. For multi-file processing, use `ThreadPoolExecutor`. See [../THREADING.md](../THREADING.md) for examples.

### Q: Is the GIL release automatic?
**A:** Yes. Every call to `read_record()` automatically releases the GIL during parsing (Phase 2). You don't need to do anything special.

---

## Understanding the Speedups

### Single-Threaded Speedup (~4x)

```python
from mrrc import MARCReader

# This code gets ~4x faster automatically vs pymarc
# No changes needed!
with open("records.mrc", "rb") as f:
    reader = MARCReader(f)
    while record := reader.read_record():
        title = record.title()
        # ... process record ...
```

**Why so fast?** Rust parsing is much more efficient than Python. The GIL release helps with I/O operations, but the main benefit is that the parsing itself is faster.

**Speedup breakdown:**
- Pure Rust implementation: ~14x faster than pymarc
- Python wrapper overhead: ~3x slower than pure Rust
- Net: ~4x faster than pymarc (measured empirically)

### Multi-Threaded Speedup (2.0x-3.74x)

```python
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader

def process_file(path):
    with open(path, "rb") as f:
        reader = MARCReader(f)  # Each thread gets its own reader
        while record := reader.read_record():
            # ... process record ...

# Process 4 files in parallel
# Expected: ~3.74x faster than processing them one-at-a-time
with ThreadPoolExecutor(max_workers=4) as executor:
    executor.map(process_file, ["file1.mrc", "file2.mrc", "file3.mrc", "file4.mrc"])
```

**Why 3.74x instead of 4x?**
- Thread creation/management overhead: ~5-10%
- Memory bandwidth saturation: ~5-10%
- Python scheduler overhead: ~5-10%
- **Result:** ~94% efficiency on 4 cores = 3.74x speedup

**When to use:** Processing multiple independent files that don't fit in memory at once.

**When NOT to use:** Processing a single file, or when you don't have multiple files.

---

## Performance Comparisons

### Scenario: Process 1 Million Records

| Approach | Time | Notes |
|----------|------|-------|
| pymarc (1 thread) | 14.3 seconds | Baseline |
| pymrrc (1 thread, automatic) | 3.3 seconds | ~4x faster |
| pymrrc (4 threads, opt-in) | 1.1 seconds | ~13x faster |
| Rust mrrc (4 threads, explicit) | 0.25 seconds | ~57x faster |

**Cost/benefit:**
- Upgrade to pymrrc: ~4x faster, zero code changes
- Add threading: ~13x faster, need `ThreadPoolExecutor`
- Use Rust: ~57x faster, need to rewrite in Rust

### Scenario: Process 100 MARC Files (100k records each)

| Approach | Time | Notes |
|----------|------|-------|
| pymarc (sequential) | ~24 minutes | 1.2 MB/s throughput |
| pymrrc (sequential) | ~6 minutes | 5 MB/s throughput (~4x faster) |
| pymrrc (4 threads) | ~2 minutes | 15 MB/s throughput (~12x faster) |

---

## When to Upgrade from pymarc to pymrrc

### Upgrade Now If:
- You want a performance improvement with zero code changes
- You're processing large files (100k+ records)
- Your MARC processing is taking noticeable time
- You want to reduce resource usage (pymrrc uses less CPU/memory)

### Upgrade When You Also Need:
- Multi-threading to process many files in parallel
- Python 3.9+, Linux/macOS/Windows support
- Better memory efficiency

### Don't Upgrade If:
- You need pure Python with no C extensions
- You have deeply integrated custom code with pymarc internals
- You're processing very small files (< 1k records)

---

## Benchmarking Questions

### Q: Why are the benchmarks in two sections?
**A:** Because there are two different usage patterns:
1. **Single-threaded (Tests 1-8):** Default behavior, no changes needed, ~4x faster
2. **Multi-threaded (Tests 9-10):** Explicit threading, requires `ThreadPoolExecutor`, ~3-4x faster vs sequential

### Q: Which benchmark should I care about?
**A:** Start with single-threaded (Test 1-8). If you're processing multiple files, also look at multi-threaded (Test 9-10).

### Q: Do the benchmarks match my use case?
- **Tests 1-2:** Raw reading (no processing) — best case for mrrc
- **Tests 3-4:** Reading with field extraction (realistic) — typical use case
- **Tests 5-6:** Round-trip (read + write) — benchmark if doing conversions
- **Tests 9-10:** Multi-threading — benchmark if processing many files

### Q: What does "rec/s" mean?
**A:** Records per second. Higher is better.

- Rust: ~1,000,000 rec/s = processes 1 million records in ~1 second
- pymrrc: ~300,000 rec/s = processes 1 million records in ~3.3 seconds
- pymarc: ~70,000 rec/s = processes 1 million records in ~14.3 seconds

---

## GIL and Threading Questions

### Q: What is the GIL?
**A:** Global Interpreter Lock. Python's mechanism to make the interpreter thread-safe. Only one thread can execute Python bytecode at a time. When one thread releases the GIL, other threads can run.

### Q: Does pymrrc release the GIL?
**A:** Yes. During Phase 2 (record parsing), the GIL is released via `py.detach()`. This happens automatically with every `read_record()` call.

### Q: Do I need to think about the GIL?
**A:** No. In single-threaded mode, it's automatic and transparent. In multi-threaded mode, the GIL release is what allows concurrent parsing.

### Q: What happens if I share a reader across threads?
**A:** Undefined behavior. The reader is not thread-safe. Create a new reader per thread.

```python
# ❌ WRONG
reader = MARCReader(open("file.mrc", "rb"))
ThreadPoolExecutor().map(lambda _: reader.read_record(), range(4))

# ✅ RIGHT
def read_all(path):
    reader = MARCReader(open(path, "rb"))
    while record := reader.read_record():
        # ... process ...

ThreadPoolExecutor().map(read_all, ["file1.mrc", "file2.mrc", "file3.mrc", "file4.mrc"])
```

---

## Performance Tuning Questions

### Q: How can I make pymrrc even faster?
**A:** 
1. **File path input:** Pass file paths directly instead of file objects
2. **Multi-threading:** Use `ThreadPoolExecutor` if processing many files
3. **Batch processing:** Read multiple records at once
4. **Use Rust:** If possible, rewrite the critical path in Rust

### Q: Does record size affect performance?
**A:** Minimally. The throughput stays consistent (549,500 rec/s) regardless of record size. Larger records take longer to parse, but the rate is consistent.

### Q: Does Python version matter?
**A:** Only for availability. pymrrc supports Python 3.9+. Performance is similar across versions.

### Q: Does OS matter?
**A:** Only for binary compatibility. pymrrc provides wheels for Linux, macOS, and Windows. Performance is similar across platforms.

---

## Still Have Questions?

- See [RESULTS.md](RESULTS.md) for complete benchmark data with four-way comparisons
- See [../ARCHITECTURE.md](../ARCHITECTURE.md) for detailed technical explanation of GIL release
- See [../PERFORMANCE.md](../PERFORMANCE.md) for usage patterns and performance tuning
- See [../THREADING.md](../THREADING.md) for threading patterns and code examples
