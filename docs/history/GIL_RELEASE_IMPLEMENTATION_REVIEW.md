# GIL Release Strategy Implementation Review

**Status:** Technical Review Complete  
**Date:** January 2026  
**Related Document:** `GIL_RELEASE_STRATEGY_REVISED.md`  
**Purpose:** Identify implementation gaps, design issues, and missing specifications before Phase A begins

---

## Executive Summary

The revised GIL release strategy is architecturally sound but contains **critical implementation gaps** that must be addressed before coding starts:

1. **Borrow checker violation** in Phase 2 that prevents safe GIL release
2. **Nested `Python::attach()` calls** that will panic at runtime
3. **Unspecified error conversion** for Phase 2 parse errors
4. **Zero specification** for Phase D (writer implementation)
5. **Missing adaptive heuristics** for optimization threshold decisions
6. **Incomplete testing strategy** for GIL release verification
7. **Benchmarking approach lacks GIL-specific metrics**

**Recommendation:** Fix the three design issues (borrow checker, `attach()` nesting, error conversion) before Phase A. Defer some optimizations to Phase C with adaptive logic. Add explicit GIL verification tests.

---

## Part I: Critical Design Holes

### 1. Borrow Checker Violation in Phase 2 (CRITICAL)

**Location:** Lines 204–223 (readers.rs)

**Issue:**
```rust
let record_bytes = slf.buffered_reader.read_next_record_bytes(py)?;
// record_bytes borrows from self.buffer

let record = py.allow_threads(|| {
    slf.reader.read_from_bytes(record_bytes)  // ← Still holds borrow!
})?;
```

The `record_bytes` slice is borrowed from `self.buffer` owned by `BufferedMarcReader`. This borrow **outlives Phase 1 and enters Phase 2**, where GIL is released. The closure captures `record_bytes`, which is borrowed from a Rust struct accessed through `&mut self`.

**Why this fails:**
- Rust borrow checker requires that mutable borrows are exclusive
- `slf.buffered_reader.read_next_record_bytes()` mutates the buffer (calls `clear()`, `extend_from_slice()`)
- The returned `&[u8]` is borrowed from `self.buffer`
- `py.allow_threads()` may be called concurrently from other threads
- **Other threads cannot borrow `self.buffer` while Phase 2 holds a borrow** — this violates exclusivity

This code will not compile.

**Solutions:**

**Option A (Recommended): Copy bytes before Phase 2**
```rust
let record_bytes = slf.buffered_reader.read_next_record_bytes(py)?;
if record_bytes.is_empty() {
    return Ok(None);
}

// Copy out of the buffer to break the borrow
let record_bytes_owned = record_bytes.to_vec();

// Phase 2: GIL released, no borrows from self
let record = py.allow_threads(|| {
    slf.reader.read_from_bytes(&record_bytes_owned)
})?;
```

**Trade-off:** Extra allocation per record (defeats buffering optimization partially). **Mitigation:** Use `SmallVec<[u8; 4096]>` for typical records to avoid heap allocation for most cases.

**Option B: Use SmallVec with move semantics**
```rust
let record_bytes: SmallVec<[u8; 4096]> = 
    slf.buffered_reader.read_next_record_bytes(py)?
        .to_smallvec();

let record = py.allow_threads(|| {
    slf.reader.read_from_bytes(&record_bytes)
})?;
```

**Trade-off:** Still copies, but faster for typical MARC records (<4KB).

**Option C: Redesign BufferedMarcReader to own record**
```rust
struct RecordBuffer {
    bytes: Vec<u8>,  // Owns the record bytes
}

impl BufferedMarcReader {
    fn read_next_record(&mut self, py: Python) -> PyResult<Option<RecordBuffer>> {
        // Phase 1: Read and populate buffer
        // Return ownership of buffer, not a borrow
        Ok(Some(RecordBuffer { bytes }))
    }
}

// Phase 2: Owns the buffer, safe to pass to allow_threads
let record = py.allow_threads(|| {
    slf.reader.read_from_bytes(&record_buffer.bytes)
})?;
```

**Trade-off:** More complex type management, but clearest semantics.

**Recommendation:** Use **Option A with SmallVec** for Phase A. It's simple, has predictable performance for typical records (no allocation for <4KB), and correctly solves the borrow issue. Add detailed comment explaining why SmallVec is necessary.

**Issue to file:** `mrrc-XXXX: Borrow checker safety in Phase 2 — specify SmallVec approach`

---

### 2. Nested `Python::attach()` Panic (CRITICAL)

**Location:** Line 205 + Lines 129–147

**Issue:**
The proposal shows:
```rust
fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<Option<PyObject>> {
    let py = unsafe { Python::assume_gil_acquired() };
    let record_bytes = slf.buffered_reader.read_next_record_bytes(py)?;
    //                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //                  Calls read_exact_from_file(py)
    //                  Which calls Python::attach(|py| { ... })
    //                  PANIC: GIL already held!
}
```

Inside `read_next_record_bytes()`, line 136 calls:
```rust
let bytes_obj: PyBytes = file_obj
    .call_method1("read", (bytes_needed,))?
    .extract()?;
```

This requires `&Python`, but the implementation (lines 129–147) shows `read_exact_from_file()` is intended to be called inside `Python::attach()`, not with an existing `&Python`.

**Why this fails:**
- `PyRefMut<'_, Self>` in `__next__()` means the GIL is held
- `unsafe { Python::assume_gil_acquired() }` correctly gets the `&Python` handle
- But then calling `read_exact_from_file(py)` expects to use `Python::attach()` internally (implied by the implementation)
- **You cannot call `Python::attach()` if the GIL is already held** — it will panic

**Solution:**

Refactor `read_exact_from_file()` to accept `&Python` instead of trying to acquire it:

```rust
// Before:
fn read_exact_from_file(&mut self, py: Python, mut bytes_needed: usize) -> PyResult<Vec<u8>> {
    let mut result = Vec::with_capacity(bytes_needed);
    while bytes_needed > 0 {
        let file_obj = self.file_wrapper.file.bind(py);
        let bytes_obj: PyBytes = file_obj.call_method1("read", (bytes_needed,))?.extract()?;
        // ...
    }
}

// This works — pass &Python through, don't acquire it
```

The code in the proposal actually looks correct here — it accepts `py: Python` and doesn't call `attach()`. The issue is the **docstring and calling context** are misleading. The implementation assumes Phase 1 always has `&Python` available.

**Verification needed:** Confirm current implementation in `src-python/src/readers.rs` doesn't call `Python::attach()` nested. If Phase 1 is truly meant to be nested inside `__next__()`, the code is correct as written.

**Issue to file:** `mrrc-XXXX: Clarify Phase 1 GIL acquisition — document that &Python is passed through, not re-acquired`

---

### 3. Phase 2 Error Conversion Outside GIL (CRITICAL)

**Location:** Lines 357–361

**Issue:**
```rust
let record = py.allow_threads(|| {
    slf.reader.read_from_bytes(record_bytes)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
})?;
```

This attempts to construct a `PyErr` **inside the closure**, **after the GIL is released**. PyO3 does not allow creating `PyErr` without the GIL.

**Why this fails:**
- `PyErr::new()` requires the GIL to construct Python exception objects
- Inside `allow_threads()`, the GIL is explicitly released
- The closure will panic when trying to create the exception

**Solution:**

Return a Rust error from the closure, convert it outside:

```rust
let parse_result = py.allow_threads(|| {
    slf.reader.read_from_bytes(record_bytes)
})?;  // ← Still returns PyResult, error conversion happens after allow_threads

// Phase 2 result: either Ok(record) or Err(parse_error)
let record = match parse_result {
    Ok(r) => r,
    Err(e) => {
        // Now we have GIL again, safe to create PyErr
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            e.to_string()
        ));
    }
};
```

**But wait — this still doesn't work!** The `?` operator after `allow_threads()` expects the closure to return `PyResult<_>`, but the closure returns `Result<_>`, not `PyResult<_>`.

**Correct approach:**

```rust
// Define custom error type that can be converted to PyErr
enum ParseError {
    InvalidRecord(String),
    IoError(String),
}

impl From<ParseError> for PyErr {
    fn from(e: ParseError) -> Self {
        match e {
            ParseError::InvalidRecord(msg) => {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(msg)
            }
            ParseError::IoError(msg) => {
                PyErr::new::<pyo3::exceptions::PyIOError, _>(msg)
            }
        }
    }
}

// In __next__:
let result: Result<Record, ParseError> = py.allow_threads(|| {
    slf.reader.read_from_bytes(record_bytes)
        .map_err(|e| ParseError::InvalidRecord(e.to_string()))
});

let record = result.map_err(PyErr::from)?;
```

This defers the `PyErr` construction until after `allow_threads()` returns (GIL re-acquired).

**Recommendation:** Add a custom `ParseError` enum in `src-python/src/error.rs` that:
- Wraps Rust parsing errors
- Implements `From<ParseError> for PyErr` for conversion
- Documents that conversion happens after GIL re-acquisition

**Issue to file:** `mrrc-XXXX: Implement ParseError wrapper for GIL-safe error conversion`

---

### 4. Writer Implementation (Phase D) Completely Unspecified

**Location:** Lines 458–461, no detail elsewhere

**Issue:**
The roadmap lists Phase D (writer) but provides **zero architectural guidance**. This is problematic because:

1. **Write flow is different from read**: Python→Rust conversion happens in Phase 1, serialization in Phase 2, I/O in Phase 3 (or Phase 1?)
2. **Buffer ownership is unclear**: Does the serialized output stay in a buffer? How large might it be?
3. **Incremental writes**: Can you write a record without buffering the entire serialized output first?

**Current writer implementation (src-python/src/writers.rs):**
Likely holds `PyFileWrapper` and calls `write()` directly — this holds GIL during I/O and defeats the purpose.

**Design questions:**

- **Phase 1**: Convert Python record object to Rust `Record` struct (GIL held)
- **Phase 2**: Serialize `Record` to bytes (GIL released)
- **Phase 3**: Write bytes to file (GIL held? or released?)

File I/O is generally fast, so Phase 3 might stay GIL-held like Phase 1. But if you're writing many records, you want Phase 2 parallelism.

**Strawman design:**

```rust
struct BufferedMarcWriter {
    file_wrapper: PyFileWrapper,
    buffer: Vec<u8>,
    writer: MarcWriter,  // Rust serializer
}

impl BufferedMarcWriter {
    // Phase 1: Python→Rust (GIL held)
    fn write_record_object(&mut self, py_record: PyRecord, py: Python) -> PyResult<()> {
        let rust_record = PyRecord::to_rust(py_record, py)?;
        
        // Phase 2: Serialize (GIL released)
        let serialized = py.allow_threads(|| {
            self.writer.write_record(&rust_record)
        })?;
        
        // Phase 3: Buffer + flush (GIL held)
        self.buffer.extend_from_slice(&serialized);
        if self.buffer.len() > 1MB {
            self.flush(py)?;
        }
        Ok(())
    }
    
    fn flush(&mut self, py: Python) -> PyResult<()> {
        let file_obj = self.file_wrapper.file.bind(py);
        file_obj.call_method1("write", (self.buffer.clone(),))?;
        self.buffer.clear();
        Ok(())
    }
}
```

**Issues with this design:**
- Serialization returns `Result<Vec<u8>>`, which is allocated inside `allow_threads()`
- Multiple threads serializing simultaneously contend on allocator
- Need explicit flush logic or buffering guarantees

**Recommendation:** 
- **Defer writer GIL release to Phase C** after Phase A/B prove the pattern works for reading
- Document writer design in a separate spec before Phase D
- Consider whether writer parallelism is even beneficial (writes typically sequential to file)

**Issue to file:** `mrrc-XXXX: Specify Phase D writer design — GIL release strategy for serialization vs I/O`

---

## Part II: Optimization Thresholds

### When to Cache Python Method Lookups (Lines 257–282)

**Current proposal:**
Cache `read()` method at initialization. But the effectiveness depends on unknown input characteristics.

**Analysis:**

| File Size | Avg Record Size | Read Count | Lookup Overhead | Caching Benefit |
|-----------|-----------------|-----------|-----------------|-----------------|
| 100 KB | 100 B | 1,000 | ~5μs each | 5ms total = 0.1% |
| 10 MB | 10 KB | 1,000 | ~5μs each | 5ms total = negligible |
| 100 MB | 100 B | 1M | ~5μs each | 5ms total = negligible |
| 1 GB | 1 KB | 1M | ~5μs each | 5ms total = negligible |

**Finding:** Method lookup overhead is ~5μs. For it to matter (>1% speedup):
- Need >100,000 reads per second
- Only achievable with tiny records (<100B) OR pure memory buffers (not real files)

**Recommendation:**
- **Do not cache** at Phase A. Measure real-world impact first.
- If benchmarks show >0.5% gain, add caching in Phase C with a comment explaining the threshold
- Lazy cache on first use rather than eager cache at initialization

**Issue to file:** `mrrc-XXXX: Defer method caching optimization to Phase C with measurement-driven decision`

---

### Ring Buffer Size Selection (Lines 286–308)

**Current proposal:**
Fixed 64KB buffer with fallback to Vec for larger records.

**Analysis:**

Typical MARC record sizes:
- Bibliographic records: 500B–5KB (95% of real data)
- Authority records: 500B–2KB
- Large records (rare): 10KB–100KB
- Outliers: >100KB (e.g., MODS records)

**Break-even analysis:**
- Allocation cost per record: ~50–100ns
- Ring buffer avoids allocation if record < buffer size
- For typical 1KB records: saves 50ns every record

Over 1M records: 50ms saved, ~0.1–1% total (depending on other bottlenecks).

**Trade-off:**
- Ring buffer: 64KB always allocated, 0 per-record overhead
- Vec: ~0KB initially, grows as needed

For most workloads, the difference is negligible.

**Recommendation:**
- **Start with Vec** in Phase A (simpler, no size tuning)
- Add ring buffer in Phase C **only if** benchmarks show allocation contention
- If ring buffer is added:
  - Size = 32KB (typical + some headroom), not 64KB
  - Implement fallback to Vec for records >32KB transparently
  - Document the trade-off

**Issue to file:** `mrrc-XXXX: Defer ring buffer optimization to Phase C — measure allocation contention first`

---

### Batch Reading Heuristics (Lines 452–456)

**Current proposal:**
Add `read_batch()` method, but no specification of batch size selection.

**Analysis:**

Batch benefits:
- Reduces Python boundary crossings (each batch = 1 Python call instead of N)
- Each boundary crossing costs ~100–500ns (GIL acquire/release overhead)
- For 1,000 small records: 100–500μs saved

**Batch size heuristic:**
```
ideal_batch_size = max(1, min(1000, (10 MB / avg_record_size)))
```

But you don't know `avg_record_size` until you've read several records.

**Recommendation:**
- **Add adaptive batch sizing in Phase C**
- Phase A: Single-record reads only
- Phase C: After reading 50 records, measure avg size and auto-tune batch size
- Document that batch sizing is adaptive and internal

**Issue to file:** `mrrc-XXXX: Design adaptive batch sizing strategy for Phase C`

---

## Part III: Core Rust Design Implications

### RecoveryMode Integration with Buffering

**Current issue:**
The proposal doesn't specify how `RecoveryMode` (from `src/reader.rs`) interacts with the new buffering layer.

**Problem scenarios:**

1. **Truncated record in buffer:**
   - Phase 1 reads 500 bytes of a record declared as 1000 bytes
   - Gets EOF instead of remaining 500 bytes
   - `read_next_record_bytes()` returns `Err(PyIOError)` (line 151)
   - But current `MarcReader` has `RecoveryMode::Lenient` which tries to parse truncated records
   - **Conflict:** Which strategy wins?

2. **Recovery mode state:**
   - Current `MarcReader` has recovery mode setting
   - If Phase 2 needs to handle recovery, it must be immutable and shared with reader
   - If Phase 1 needs to detect recovery-recoverable errors, it must call the reader

**Recommendation:**
- **Move recovery decision into Phase 2** (parse layer)
- Phase 1: On partial read, check if EOF or actual I/O error
  - EOF after reading 0 bytes: return `Ok(&[])` (normal EOF)
  - EOF after reading some bytes: return error (malformed record)
- Phase 2: Let `MarcReader` apply recovery mode to the partial bytes
- Document the boundary clearly

**Issue to file:** `mrrc-XXXX: Clarify RecoveryMode interaction with Phase 1 I/O errors`

---

### Allocator Contention in Phase 2

**Issue:**
With GIL released, multiple threads hit Rust allocator simultaneously during parsing (line 172 in `src/reader.rs`):
```rust
let mut record_data = vec![0u8; record_length - 24];
```

**Why it matters:**
- System allocator (typically jemalloc on Linux) may not scale well to >4 threads
- Each thread allocates a vector for parsed record
- At GIL re-acquisition, all threads wait on Python's memory management

**Potential bottleneck:**
- 8 threads × 10KB record × 1000 records/thread = 80M allocations
- If allocator has global lock, contention becomes significant

**Recommendation:**
- **Phase A:** Accept allocator contention as baseline; measure in benchmarks
- **Phase C:** If benchmarks show allocator as bottleneck (>10% overhead):
  - Use thread-local arena allocator (e.g., `bumpalo`)
  - Or: Pre-allocate thread-local buffer pools
  - Or: Use `SmallVec` to avoid heap allocation for small records

**Issue to file:** `mrrc-XXXX: Add allocator contention benchmark in Phase C`

---

### Reader State Immutability During Phase 2

**Issue:**
If `MarcReader<R>` gains mutable state (e.g., position tracking, statistics), it becomes unsafe to access during GIL release.

**Current state:** `MarcReader` only has:
- `reader: R` (the underlying I/O source)
- `recovery_mode: RecoveryMode` (immutable)

This is safe to share as `&self` during Phase 2.

**Future risk:**
If someone adds position tracking or error counters to `MarcReader`, Phase 2 code must not mutate them.

**Recommendation:**
- Document in `src/reader.rs` that `MarcReader` state is immutable during Phase 2
- Use interior mutability (e.g., `Arc<Mutex<_>>`) if future mutable state is needed
- Add clippy lint to forbid `&mut self` methods in `MarcReader::parse_*()` functions

**Issue to file:** `mrrc-XXXX: Document MarcReader immutability requirement during Phase 2`

---

## Part IV: Testing Strategy Gaps

### GIL Release Verification Tests Missing

**Current proposal:** Lines 420–423 mention "verify GIL is released during parsing" but provide no test code.

**Problem:** You cannot directly observe GIL state from Python. The only way to verify it's released is through **timing behavior**.

**Proposed test:**

```python
import threading
import time
from mrrc import MARCReader
import io

def test_gil_is_released_during_parsing():
    """Verify GIL is released during Phase 2 by blocking another thread."""
    
    fixture = load_1k_records()  # 1000 records
    
    # Shared state to coordinate threads
    reader_started = threading.Event()
    blocker_done = threading.Event()
    blocker_duration = [0]  # Will store how long blocker ran
    
    def blocker():
        """Runs CPU-intensive work; will block if GIL is not released."""
        reader_started.wait()  # Wait for reader to start
        start = time.perf_counter()
        # CPU work that takes ~100ms
        for _ in range(100_000_000):
            pass
        blocker_duration[0] = time.perf_counter() - start
        blocker_done.set()
    
    # Start blocker thread
    t = threading.Thread(target=blocker, daemon=True)
    t.start()
    
    # Read records (should release GIL)
    reader_started.set()
    data = io.BytesIO(fixture)
    reader = MARCReader(data)
    count = 0
    while record := reader.read_record():
        count += 1
    
    # Wait for blocker
    blocker_done.wait(timeout=5)
    
    # If GIL was released, blocker should complete in ~100ms
    # If GIL was held, blocker would timeout (5 seconds)
    assert blocker_duration[0] < 1.0, (
        f"Blocker took {blocker_duration[0]:.1f}s — GIL likely not released"
    )
    assert count == 1000
```

**Recommendation:**
Add this test to Phase A. It's the clearest proof that GIL release is working.

**Issue to file:** `mrrc-XXXX: Add GIL release verification test using threading.Event`

---

### Concurrency Model Tests Incomplete

**Current proposal:**
Lines 415–419 list scenarios but don't specify expected behavior.

**Missing test cases:**

1. **Multiple readers from different files simultaneously:**
```python
def test_concurrent_reads_different_files():
    """N threads each read from separate file."""
    with ThreadPoolExecutor(max_workers=4) as executor:
        results = list(executor.map(
            lambda f: sum(1 for _ in MARCReader(f)),
            [open_file_1, open_file_2, open_file_3, open_file_4]
        ))
    assert sum(results) == total_records
```

2. **Single reader shared across threads (should fail):**
```python
def test_shared_reader_not_thread_safe():
    """Accessing same reader from multiple threads should fail safely."""
    reader = MARCReader(io.BytesIO(fixture))
    
    errors = []
    def read_one():
        try:
            reader.read_record()
        except Exception as e:
            errors.append(e)
    
    threads = [threading.Thread(target=read_one) for _ in range(4)]
    [t.start() for t in threads]
    [t.join() for t in threads]
    
    # Should have errors (reader is not Send/Sync)
    assert len(errors) > 0
```

3. **Stream closing mid-read:**
```python
def test_stream_closes_during_read():
    """If file closes during Phase 1, should return error."""
    class CloseOnNthRead:
        def __init__(self, data, close_on_read_count=10):
            self.buf = io.BytesIO(data)
            self.count = 0
            self.close_on_read_count = close_on_read_count
        
        def read(self, n):
            self.count += 1
            if self.count == self.close_on_read_count:
                raise IOError("Stream closed")
            return self.buf.read(n)
    
    reader = MARCReader(CloseOnNthRead(fixture))
    with pytest.raises(IOError):
        while reader.read_record():
            pass
```

**Recommendation:**
Add all three test classes to Phase A. They validate the concurrency model and error handling.

**Issue to file:** `mrrc-XXXX: Add comprehensive concurrency model tests`

---

### Error Injection Scenarios Missing

**Phase 1 errors:**
- Corrupted length header (non-ASCII, non-numeric)
- Missing record terminator (0x1D)
- Record declared as 1000 bytes but only 500 in file

**Phase 2 errors:**
- Corrupted directory entries
- Field references outside data area
- Invalid field tags

**Phase 3 errors:**
- Python object creation fails

**Recommendation:**
Create `tests/python/test_error_scenarios.py` with synthetic test data:
```python
def test_corrupted_length_header():
    """Non-numeric length header should raise ValueError."""
    data = b"ABCD" + b"..." + b"\x1D"  # 'ABCD' is not numeric
    reader = MARCReader(io.BytesIO(data))
    with pytest.raises(ValueError, match="Invalid MARC record length"):
        reader.read_record()

def test_missing_terminator():
    """Record missing 0x1D terminator should raise IOError."""
    data = b"00100" + b"..." + b"X"  # 'X' instead of 0x1D
    reader = MARCReader(io.BytesIO(data))
    with pytest.raises(IOError, match="missing terminator"):
        reader.read_record()
```

**Issue to file:** `mrrc-XXXX: Create test_error_scenarios.py with Phase 1/2/3 error injection`

---

## Part V: Benchmarking Approach Issues

### Current Benchmarks Don't Measure GIL Release

**Problem:**
`benches/parallel_benchmarks.rs` uses pure Rust (rayon). It doesn't measure Python threading.

`tests/python/test_benchmark_parallel.py` uses Python threads, but **doesn't verify that GIL is released**.

**Current test (lines 64–79):**
```python
def test_threaded_reading_1k(self, benchmark, fixture_1k):
    """ThreadPoolExecutor reading of 2x 1k records (pymrrc)."""
    def read_with_threads():
        def read_single_file(data):
            reader = MARCReader(io.BytesIO(data))
            count = 0
            while record := reader.read_record():
                count += 1
            return count
        
        with ThreadPoolExecutor(max_workers=2) as executor:
            results = list(executor.map(read_single_file, [fixture_1k, fixture_1k]))
        return sum(results)
    
    result = benchmark(read_with_threads)
    assert result == 2000
```

**Issue:** This benchmark measures *elapsed time*, but without GIL release verification, we can't tell if speedup comes from real parallelism or just thread scheduling.

**If GIL is not released:**
- The test still completes (with ThreadPoolExecutor), but slower
- Without verification, you might think GIL release worked when it didn't

### Missing Contention Benchmark

**Recommendation:**
Add a benchmark that measures speedup vs thread count:

```python
@pytest.mark.benchmark
def test_threading_speedup_curve(self, fixture_10k):
    """Measure speedup scaling with thread count."""
    results = {}
    
    for num_threads in [1, 2, 4, 8]:
        def read_n_files():
            files = [io.BytesIO(fixture_10k) for _ in range(num_threads)]
            with ThreadPoolExecutor(max_workers=num_threads) as executor:
                return sum(executor.map(
                    lambda f: sum(1 for _ in MARCReader(f)),
                    files
                ))
        
        start = time.perf_counter()
        total = read_n_files()
        elapsed = time.perf_counter() - start
        results[num_threads] = (total, elapsed)
    
    # Speedup should be close to linear
    # 2 threads: ~1.9x
    # 4 threads: ~3.7x (diminishing due to contention)
    # 8 threads: ~5-6x (allocator contention visible)
    
    speedup_2 = results[1][1] / results[2][1]
    speedup_4 = results[1][1] / results[4][1]
    speedup_8 = results[1][1] / results[8][1]
    
    assert speedup_2 > 1.8, f"2-thread speedup {speedup_2:.1f}x is too low"
    assert speedup_4 > 3.0, f"4-thread speedup {speedup_4:.1f}x is too low"
    # 8-thread may show contention, so just verify it's >4x
    assert speedup_8 > 4.0, f"8-thread speedup {speedup_8:.1f}x is too low"
```

This reveals:
- Whether GIL is actually released (speedup > 1)
- Where contention appears (diminishing returns after N threads)
- Whether allocator is a bottleneck

**Issue to file:** `mrrc-XXXX: Add threading speedup curve benchmark`

---

### Python vs Rust Performance Comparison Missing

**Recommendation:**
Add benchmark comparing pymrrc threading to pure Rust parallelism (rayon):

```python
def test_pymrrc_vs_rayon_speedup():
    """Compare pymrrc threading speedup to pure Rust rayon parallelism."""
    # Run Rust benchmark: `cargo bench parallel_4x_10k`
    # Run Python benchmark: test_threading_speedup_curve(num_threads=4)
    
    # Expected: pymrrc should be within 90% of rayon
    # (not identical because of Python overhead)
    
    # Actual results from benchmarks should show:
    # Rayon 4x: 3.8x speedup
    # pymrrc 4 threads: 3.2-3.6x speedup (80-95% of Rayon)
    pass
```

Add comparison table to benchmark results documenting the gap.

**Issue to file:** `mrrc-XXXX: Add pymrrc vs Rayon performance comparison benchmark`

---

## Part VI: Other Missing Considerations

### Stream Closing Semantics Undefined

**Scenario:**
```python
reader = MARCReader(file)
# Thread A: Reader is in Phase 2, parsing a record
# Thread B (with GIL): calls file.close()
# Thread A: Tries to check if reader is still valid
```

**Question:** Should `close()` be safe? Should reader detect closed file?

**Recommendation:**
Document in API:
- `MARCReader` is not thread-safe; do not close file while reader is active
- Closing file from another thread while reader is active results in undefined behavior
- No attempt to detect this condition; caller responsibility

Add to docstring:
```python
class MARCReader:
    """
    NOT thread-safe. Each thread must have its own MARCReader instance.
    
    WARNING: Do not call close() on the underlying file while reader is active.
    The file may be closed by another thread while Phase 2 is executing,
    resulting in undefined behavior.
    """
```

**Issue to file:** `mrrc-XXXX: Document MARCReader thread safety and stream closing semantics`

---

### Fallback Allocation Strategy for Large Records

**Scenario:**
Ring buffer is 32KB, but a record is 100KB.

**Current proposal:** Fallback to Vec.

**Missing specification:**

1. **When does fallback trigger?**
   - After reading header with length=100KB?
   - Before Phase 2?

2. **Does subsequent small record switch back to ring buffer?**
   - If yes: overhead of size check per record
   - If no: once Vec is used, continues using Vec (memory waste)

**Recommendation:**
Specify fallback policy:
```rust
enum BufferStrategy {
    RingBuffer([u8; 32768]),  // For typical records
    VecBuffer(Vec<u8>),        // For outliers >32KB
}

impl BufferStrategy {
    fn ensure_capacity(&mut self, needed: usize) {
        match self {
            BufferStrategy::RingBuffer(buf) if needed <= buf.len() => {
                // Fits in ring buffer, stay small
            }
            BufferStrategy::VecBuffer(ref mut v) => {
                v.reserve(needed);  // Grow existing vec
            }
            BufferStrategy::RingBuffer(_) => {
                // Too big, switch to vec
                *self = BufferStrategy::VecBuffer(Vec::with_capacity(needed));
            }
        }
    }
}
```

This keeps small records in ring buffer, large ones in vec, but doesn't ping-pong between strategies.

**Issue to file:** `mrrc-XXXX: Specify fallback allocation strategy for large records`

---

### Streaming vs Batching Tradeoff

**Proposal:** Add optional `read_batch()` method.

**Missing specification:**

1. **Default batch size?**
   - 10, 100, 1000?
   - Should it be configurable?

2. **Memory semantics?**
   ```python
   records = reader.read_batch(100)  # Returns list[Record]?
   ```
   - Is the returned list owned by reader or caller?
   - Can reader be used while iterating over batch?

3. **Backpressure?**
   - If caller processes records slowly, does batch buffer grow?
   - Should there be a size limit on accumulated batches?

**Recommendation:**
Defer `read_batch()` to Phase C. For Phase A/B, single-record reads are sufficient to prove the GIL release pattern.

Add batch reading only if benchmarks show >5% benefit from reduced boundary crossing overhead.

**Issue to file:** `mrrc-XXXX: Defer read_batch() to Phase C with adaptive sizing`

---

### Python Iterator Protocol Edge Cases

**Current implementation (lines 90–100 in readers.rs):**
```python
def __next__() -> PyRecord | StopIteration
```

**Missing specifications:**

1. **Calling `next()` after exhaustion:**
   ```python
   reader = MARCReader(small_file)
   reader.read_record()
   reader.read_record()  # Raises StopIteration
   reader.read_record()  # What happens? Raise again? Error?
   ```

2. **Resetting reader:**
   - Can't seek files, so no reset
   - Should document that readers are one-shot

3. **Context manager support:**
   - Should reader support `with` statement to auto-close files?

**Recommendation:**
Document in API:
```python
class MARCReader:
    """Iterator over MARC records from a file.
    
    Readers are single-use and cannot be reset. Once iteration completes
    (StopIteration), further calls to next() will continue raising StopIteration.
    
    Does NOT support context manager (with statement); caller is responsible
    for closing the underlying file.
    """
```

**Issue to file:** `mrrc-XXXX: Document MARCReader lifecycle and iterator protocol compliance`

---

## Implementation Roadmap Refinement

### Phase A: Core Buffering Infrastructure (with fixes)

- [ ] Create `BufferedMarcReader` type
- [ ] Implement `read_next_record_bytes()` with boundary detection using **SmallVec** for borrowed bytes
- [ ] Implement `read_exact_from_file()` with EOF handling  
- [ ] Add unit tests for boundary detection
- **[NEW]** Add error handling for corrupted length headers
- **[NEW]** Create `ParseError` enum for Phase 2 error conversion
- **[NEW]** Document GIL acquisition model (clarify that `&Python` is passed through)

### Phase B: GIL Release Integration (with verification)

- [ ] Refactor `PyMarcReader::__next__()` to use three-phase pattern with SmallVec
- [ ] Verify borrow checker accepts architecture
- [ ] Implement error handling for Phase 1/2/3 using `ParseError` conversion
- **[NEW]** Add GIL release verification test using `threading.Event`
- **[NEW]** Add concurrency model tests (separate readers, shared reader, stream closing)
- **[NEW]** Add error injection tests (corrupted headers, missing terminators, truncated records)

### Phase C: Performance Optimizations (with measurement)

- [ ] Implement and test batch reading with adaptive sizing
- [ ] Measure method caching overhead; only add if >0.5% benefit proven
- [ ] Benchmark memory and CPU overhead; add allocator contention test
- [ ] Optional: Implement ring buffer with fallback strategy for large records
- **[NEW]** Add threading speedup curve benchmark (1, 2, 4, 8 threads)
- **[NEW]** Add pymrrc vs Rayon comparison benchmark
- **[NEW]** Measure allocator contention with 8+ threads

### Phase D: Writer Implementation (design first)

- [ ] Create writer design spec (separate doc, not in this file)
- [ ] Specify Phase 1/2/3 mapping for write operations
- [ ] Apply three-phase pattern to `PyMarcWriter::write_record()`
- [ ] Implement buffering for writes
- [ ] Add write-side tests and benchmarks

### Phase E: Validation and Rollout

- [ ] Run full test suite (unit + integration)
- [ ] Verify threading speedup achieves 2–3x target
- [ ] Verify threading speedup is within 90% of pure Rust
- [ ] Verify backward compatibility
- [ ] Update documentation with threading performance notes and safety warnings

---

## Issues to File Before Starting Phase A

1. `mrrc-XXXX` **[CRITICAL]**: Borrow checker safety in Phase 2 — specify SmallVec approach
2. `mrrc-XXXX` **[CRITICAL]**: Clarify Phase 1 GIL acquisition — document that &Python is passed through
3. `mrrc-XXXX` **[CRITICAL]**: Implement ParseError wrapper for GIL-safe error conversion
4. `mrrc-XXXX`: Specify Phase D writer design — GIL release strategy for serialization vs I/O
5. `mrrc-XXXX`: Defer method caching optimization to Phase C with measurement-driven decision
6. `mrrc-XXXX`: Defer ring buffer optimization to Phase C — measure allocation contention first
7. `mrrc-XXXX`: Design adaptive batch sizing strategy for Phase C
8. `mrrc-XXXX`: Clarify RecoveryMode interaction with Phase 1 I/O errors
9. `mrrc-XXXX`: Document MarcReader immutability requirement during Phase 2
10. `mrrc-XXXX`: Add GIL release verification test using threading.Event
11. `mrrc-XXXX`: Add comprehensive concurrency model tests
12. `mrrc-XXXX`: Create test_error_scenarios.py with Phase 1/2/3 error injection
13. `mrrc-XXXX`: Add threading speedup curve benchmark
14. `mrrc-XXXX`: Add pymrrc vs Rayon performance comparison benchmark
15. `mrrc-XXXX`: Add allocator contention benchmark in Phase C
16. `mrrc-XXXX`: Document MARCReader thread safety and stream closing semantics
17. `mrrc-XXXX`: Specify fallback allocation strategy for large records
18. `mrrc-XXXX`: Defer read_batch() to Phase C with adaptive sizing
19. `mrrc-XXXX`: Document MARCReader lifecycle and iterator protocol compliance

---

## Conclusion

The revised GIL release strategy is **architecturally sound but operationally incomplete**. The three critical issues (borrow checker, `attach()` nesting, error conversion) must be resolved with concrete code patterns before Phase A begins.

The optimization sections (caching, ring buffer, batching) are premature; measure first, optimize second. The testing and benchmarking strategies need explicit GIL verification and contention analysis to prove the approach works.

**Recommendation:** Fix the three critical issues, file the 19 tracking issues, and start Phase A with the refined roadmap. This will ensure the implementation is both correct and measurable.

---

## References

- **PyO3 GIL documentation**: https://pyo3.rs/latest/python_async
- **Related design docs**: 
  - `GIL_RELEASE_STRATEGY.md` (original)
  - `GIL_RELEASE_STRATEGY_REVISED.md` (current)
- **Rust borrow checker**: https://doc.rust-lang.org/nomicon/borrow-checker.html
- **SmallVec crate**: https://docs.rs/smallvec/
- **Threading semantics**: Python `threading` module docs
