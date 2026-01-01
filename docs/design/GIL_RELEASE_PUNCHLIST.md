# GIL Release Implementation Punchlist

**Status:** Ready for Phase A  
**Last Updated:** January 1, 2026  
**Overall Progress:** 0% (ready to begin)  
**Critical Path:** A → B → D → E → F → G (6 weeks)  
**Optional Path:** C (deferred if speedup ≥ 2x after Phase B)

---

## Quick Reference

| Phase | Epic | Duration | Status | Ready? |
|-------|------|----------|--------|--------|
| **A** | Core Buffering | Week 1 | — | ✅ Ready |
| **B** | GIL Integration | Week 1-2 | — | Ready after A |
| **C** | Optimizations | Week 2-3 | — | Optional (deferred) |
| **D** | Writer Implementation | Week 3-4 | — | Ready after B |
| **E** | Validation | Week 4-5 | — | Ready after D |
| **F** | Benchmark Refresh | Week 5-6 | — | Ready after E |
| **G** | Documentation | Week 6-7 | — | Ready after F |

---

## Phase A: Core Buffering Infrastructure

**Epic:** mrrc-9wi.1  
**Duration:** Week 1 (20 hours)  
**Priority:** P1 (Critical Path)  
**Status:** 🟢 Ready to start  
**Plan Reference:** GIL_RELEASE_IMPLEMENTATION_PLAN.md Part 5 Phase A (lines 315-379)

### Overview
Implement `BufferedMarcReader` struct with ISO 2709 record boundary detection using SmallVec for owned record bytes.

**Key Deliverables:**
- ParseError enum for GIL-free error handling
- BufferedMarcReader with SmallVec<[u8; 4096]>
- ISO 2709 boundary detection logic
- Comprehensive unit tests

### Detailed Tasks

#### A.1: Create src-python/src/error.rs with ParseError enum
**Task:** mrrc-9wi.1.1  
**Status:** 🟢 Ready  
**Priority:** P1  
**Dependencies:** None  
**Plan Reference:** Part 2, Fix 3 (lines 244-260)

**Deliverables:**
- [ ] ParseError enum with 3 variants:
  - InvalidRecord(String)
  - RecordBoundaryError(String)
  - IoError(String)
- [ ] Display impl for ParseError
- [ ] From<ParseError> for PyErr conversion
- [ ] Module export in src-python/src/lib.rs

**Success Criteria:**
- All variants properly map to Python exception types
- No Py<T> references in ParseError (safe for use in allow_threads())
- Compiles without warnings

---

#### A.2: Create src-python/src/buffered_reader.rs with BufferedMarcReader struct
**Task:** mrrc-9wi.1.2  
**Status:** 🟢 Ready  
**Priority:** P1  
**Dependencies:** Depends on A.1  
**Plan Reference:** Part 1 (68-85), Part 2 Fix 1 (90-145), Part 5 Phase A (315-379)

**Deliverables:**
- [ ] BufferedMarcReader struct with:
  - file_wrapper: PyFileWrapper
  - buffer: SmallVec<[u8; 4096]>
  - State tracking for EOF
- [ ] Method: read_next_record_bytes(&mut self, py: Python) → PyResult<Option<Vec<u8>>>
  - Reads complete ISO 2709 MARC record
  - Returns Ok(Some(bytes)) for complete record
  - Returns Ok(None) at EOF (idempotent)
  - Returns Err(PyErr) for I/O or boundary errors
- [ ] Method: read_exact_from_file(&mut self, py: Python, n_bytes: usize) → PyResult<Vec<u8>>
  - Reads exactly n_bytes from file
  - Returns Err if fewer bytes at EOF
- [ ] Method: parse_record_length(bytes: &[u8]) → PyResult<usize>
  - Parses 5-byte ASCII record length
  - Validates digits, returns error on corruption

**Stream State Machine (Required Implementation):**
```
Initial → Reading → EOF (terminal)

Behavior Specifications:
1. read_next_record_bytes() on closed file → PyIOError
2. read_next_record_bytes() at EOF → Ok(None)
3. read_next_record_bytes() after EOF → Ok(None) again (idempotent)
4. Corrupted length header → PyValueError with details
5. Record header incomplete → PyIOError at offset
6. Record body incomplete → PyIOError with byte counts
7. File closed during Phase 2 → Python file.read() IOError propagates
```

**Success Criteria:**
- All 10+ unit tests pass
- Boundary detection matches src/reader.rs correctness
- SmallVec benchmark validates <5% overhead
- No panics on invalid input

---

#### A.3: Add smallvec dependency and Cargo.toml updates
**Task:** mrrc-9wi.1.3  
**Status:** 🟢 Ready  
**Priority:** P1  
**Dependencies:** Depends on A.1  
**Plan Reference:** Part 2 Fix 1 (140-145), Part 9 (1331-1338)

**Deliverables:**
- [ ] Add to Cargo.toml: `smallvec = "1.11"`
- [ ] Document in code: "We own the bytes here to safely cross GIL boundary"
- [ ] SmallVec sizing rationale in comments

**Rationale:**
- MARC records typically 100B–5KB (median ~1.5KB)
- 4KB inline buffer captures ~85-90% without allocation
- Spillover to heap for >4KB records (automatic)

**Success Criteria:**
- Cargo builds without errors
- Clippy passes with -D warnings
- Documentation clear on sizing decision

---

#### A.4: Add comprehensive unit tests for BufferedMarcReader boundary detection
**Task:** mrrc-9wi.1.4  
**Status:** 🟢 Ready  
**Priority:** P1  
**Dependencies:** Depends on A.2  
**Plan Reference:** Part 5 Phase A (359-365), Part 6 (1150-1228)

**Test Coverage (Minimum 10 test cases):**
- [ ] Complete records - reads full record correctly
- [ ] Records split across reads - partial reads handled
- [ ] Corrupted length headers - non-ASCII digits detected
- [ ] Missing terminator - record boundary violations caught
- [ ] Variable-length records - 100B, 5KB, 100KB all handled
- [ ] EOF on first byte - incomplete header detected
- [ ] EOF mid-record - incomplete body detected
- [ ] Empty files - EOF on first read returns None
- [ ] Multiple sequential records - no data mixing
- [ ] Record at file boundary - last record read completely

**Stream State Machine Tests:**
- [ ] read_next_record_bytes() on closed file → PyIOError
- [ ] read_next_record_bytes() at EOF → Ok(None)
- [ ] read_next_record_bytes() after EOF → Ok(None) (idempotent)
- [ ] Corrupted length → PyValueError with details
- [ ] Incomplete header → PyIOError with offset
- [ ] Incomplete body → PyIOError with byte counts

**Success Criteria:**
- All tests pass (no panics)
- 100% coverage of boundary logic
- Error messages include diagnostic details
- Synthetic MARC test files created

---

### Phase A Success Criteria

- ✅ All 10+ unit tests pass
- ✅ No panics on invalid input
- ✅ Boundary detection matches src/reader.rs
- ✅ SmallVec benchmark shows <5% overhead
- ✅ Code compiles without clippy warnings
- ✅ Documentation complete with GIL requirements

**Estimated Time:** 20 hours  
**Progress:** 0% (not started)

---

## Phase B: GIL Release Integration

**Epic:** mrrc-9wi.2  
**Duration:** Week 1-2 (25 hours)  
**Priority:** P1 (Critical Path)  
**Status:** 🟠 Ready after Phase A completes  
**Plan Reference:** GIL_RELEASE_IMPLEMENTATION_PLAN.md Part 5 Phase B (lines 382-412)

### Overview
Integrate BufferedMarcReader into PyMarcReader with three-phase GIL release pattern and establish baseline benchmark.

**Key Deliverables:**
- Refactor PyMarcReader to use BufferedMarcReader
- Implement three-phase pattern in __next__()
- GIL release verification test
- Baseline benchmark for Phase C deferral gate

### Detailed Tasks

#### B.1: Refactor PyMarcReader to use BufferedMarcReader
**Task:** mrrc-9wi.2.1  
**Status:** 🟠 Blocked on Phase A  
**Priority:** P1  
**Dependencies:** Depends on mrrc-9wi.1 (Phase A)  
**Plan Reference:** Part 1 (68-85), Part 5 Phase B (382-392)

**Deliverables:**
- [ ] Update PyMarcReader struct to hold BufferedMarcReader
- [ ] Refactor __next__() to use new buffered reader
- [ ] Add documentation comments on GIL requirements
- [ ] Verify all existing tests still pass

**Success Criteria:**
- Code compiles without warnings
- All existing pymarc tests pass
- No behavioral changes to public API

---

#### B.2: Implement three-phase GIL release pattern in PyMarcReader.__next__()
**Task:** mrrc-9wi.2.2  
**Status:** 🟠 Blocked on Phase A  
**Priority:** P1  
**Dependencies:** Depends on mrrc-9wi.2.1  
**Plan Reference:** Part 1 (52-66), Part 2 Fix 1 (98-115), Part 2 Fix 3 (262-275)

**Deliverables:**
- [ ] Phase 1 (GIL held): Call read_next_record_bytes()
- [ ] Phase 2 (GIL released): Parse with py.allow_threads()
  - Copy to owned SmallVec
  - Call reader.read_from_bytes()
  - Handle ParseError (no PyErr creation)
- [ ] Phase 3 (GIL held): Convert to PyRecord, map ParseError → PyErr
- [ ] Add code comments explaining three-phase pattern

**Code Pattern:**
```rust
let py = unsafe { Python::assume_gil_acquired() };

// Phase 1: Read bytes (GIL held)
let record_bytes_ref = slf.buffered_reader.read_next_record_bytes(py)?;
if record_bytes_ref.is_empty() {
    return Ok(None);  // EOF
}

// CRITICAL: Copy to owned SmallVec (no borrow outlives Phase 1)
let record_bytes_owned = SmallVec::from_slice(record_bytes_ref);

// Phase 2: Parse (GIL released)
let record = py.allow_threads(|| {
    slf.reader.read_from_bytes(&record_bytes_owned)
        .map_err(|e| ParseError::InvalidRecord(format!(...)))
})?;

// Phase 3: Convert (GIL re-acquired)
PyRecord::from_rust(record, py).map(|r| Some(r.into()))
```

**Success Criteria:**
- Code compiles
- Clippy passes
- No PyErr creation in Phase 2
- SmallVec borrow pattern satisfies Rust checker

---

#### B.3: Verify Rust borrow checker accepts SmallVec pattern
**Task:** mrrc-9wi.2.3  
**Status:** 🟠 Blocked on Phase A  
**Priority:** P1  
**Dependencies:** Depends on mrrc-9wi.1.2  
**Plan Reference:** Part 2 Fix 1 (80-85, 106-115)

**Verification Checklist:**
- [ ] `cargo build --all --all-features` completes
- [ ] `cargo clippy --all --all-targets -- -D warnings` passes
- [ ] No mutable/immutable borrow warnings
- [ ] No lifetime warnings
- [ ] `cargo fmt --all -- --check` passes
- [ ] Document any surprising borrow constraints

**Success Criteria:**
- All builds pass
- All clippy checks pass
- No warnings or errors

---

#### B.4: Add GIL release verification test with threading.Event
**Task:** mrrc-9wi.2.4  
**Status:** 🟠 Blocked on Phase A  
**Priority:** P1  
**Dependencies:** Depends on mrrc-9wi.2.2  
**Plan Reference:** Part 5 Phase B (397), Part 6 (1150-1200)

**Deliverables:**
- [ ] Test that proves GIL is released during Phase 2
- [ ] Two threads: one reads records, one tries to execute Python code
- [ ] If GIL released, second thread runs immediately (fast)
- [ ] If GIL held, second thread waits (slow)

**Test Design:**
```python
def test_gil_release_verification():
    """Prove GIL is released during Phase 2 (parsing)."""
    # Thread 1: Read records in loop
    # Thread 2: Wait for Phase 2 to start, then run Python code
    # Measure time for Thread 2 to complete
    # If GIL released, time is small; if held, time is large
```

**Success Criteria:**
- Test demonstrates GIL release (speedup test is sufficient)
- No timeout failures
- Clear evidence of parallelism

---

#### B.5: Establish baseline benchmark - current single-thread and 2-thread performance
**Task:** mrrc-9wi.2.5  
**Status:** 🟠 Blocked on Phase A  
**Priority:** P1  
**Dependencies:** Depends on mrrc-9wi.1  
**Plan Reference:** Part 5 Phase B (398-401), Part 10 (1364-1373)

**🔑 CRITICAL GATE: This baseline determines if Phase C is required**

**Baseline Measurements:**
- [ ] Single-thread: Read fixture_10k (10,000 records)
  - Time: _____ seconds
  - Rate: _____ ops/sec
  - Saved to: .benchmarks/baseline_before_gil_release.txt
  
- [ ] Two-thread: 2 threads reading 5K records each concurrently
  - Time: _____ seconds
  - Rate: _____ ops/sec
  - Speedup vs single-thread: _____ x
  - Saved to: .benchmarks/baseline_before_gil_release.txt

**Phase C Deferral Criteria (After Phase F):**
- If Phase F speedup ≥ 2.0x vs this baseline → Phase C OPTIONAL
- If Phase F speedup < 2.0x vs this baseline → Phase C REQUIRED
- Decision task: mrrc-pfw

**Success Criteria:**
- Baseline measurements completed
- Results documented with timestamps
- Measurements repeatable and stable
- File saved for later comparison

---

### Phase B Success Criteria

- ✅ Code compiles without warnings
- ✅ GIL release verification test passes (speedup ≥1.7x for 2 threads)
- ✅ All existing pymarc tests pass
- ✅ No data corruption in round-trip
- ✅ Baseline benchmark established

**Estimated Time:** 25 hours  
**Blocker for:** Phase D  
**Progress:** 0% (not started)

---

## Phase C: Performance Optimizations (OPTIONAL)

**Status:** ⏸️ DEFERRED - Will activate only if Phase F shows speedup < 2x

**Estimated Epic:** mrrc-9wi.7 (create if needed after Phase F)  
**Duration:** Week 2-3 (15 hours, if activated)  
**Priority:** P1 (if activated) or N/A (if deferred)  
**Plan Reference:** GIL_RELEASE_IMPLEMENTATION_PLAN.md Part 5 Phase C (lines 416-470)

### Overview
Optional batch reading and caching optimizations to be pursued only if Phase B doesn't achieve 2x speedup.

**Candidate Deliverables (if Phase C activated):**
- [ ] `read_batch(batch_size)` method for PyMarcReader
- [ ] PyFileWrapper method caching (optional 5-10% speedup)
- [ ] SmallVec overhead analysis
- [ ] Optional ring buffer for very large files

**Activation Decision Task:**
- **Task:** mrrc-pfw (Phase C Deferral Gate)
- **Executed after:** Phase F (mrrc-9wi.5.2)
- **Criteria:** Compare Phase F speedup vs Phase B baseline
  - Speedup ≥ 2.0x → Skip Phase C, proceed to Phase G
  - Speedup < 2.0x → Create mrrc-9wi.7 and execute Phase C

**Progress:** 0% (waiting for Phase F decision gate)

---

## Phase D: Writer Implementation

**Epic:** mrrc-9wi.3  
**Duration:** Week 3-4 (20 hours)  
**Priority:** P1 (Critical Path)  
**Status:** 🟠 Ready after Phase B completes  
**Plan Reference:** GIL_RELEASE_IMPLEMENTATION_PLAN.md Part 5 Phase D (lines 474-504)

### Overview
Apply three-phase GIL release pattern to PyMarcWriter for write-side parallelism.

**Key Deliverables:**
- PyMarcWriter with three-phase pattern
- Write-side GIL release verification test
- Round-trip read-write-read tests

### Detailed Tasks

#### D.1: Implement PyMarcWriter with three-phase write pattern
**Task:** mrrc-9wi.3.1  
**Status:** 🟠 Blocked on Phase B  
**Priority:** P1  
**Dependencies:** Depends on mrrc-9wi.2 (Phase B)  
**Plan Reference:** Part 1 (52-66), Part 5 Phase D (474-504), Part 2 Fix 3 (268-283)

**Write-Side Pattern (differs from reader):**
- Phase 1 (GIL held): Extract PyRecord fields into Rust MarcRecord
- Phase 2 (GIL released): Serialize MarcRecord to bytes
- Phase 3 (GIL held): Write bytes to Python file object via .write()

**Deliverables:**
- [ ] Implement PyMarcWriter.write(record, py) method
- [ ] Buffer management for writes
- [ ] Error handling (SerializationError or reuse ParseError)
- [ ] Flush/close semantics

**Success Criteria:**
- Code compiles without warnings
- All existing write tests pass
- Round-trip tests pass (next task)

---

#### D.2: Add write-side GIL release verification test
**Task:** mrrc-9wi.3.2  
**Status:** 🟠 Blocked on Phase B  
**Priority:** P1  
**Dependencies:** Depends on mrrc-9wi.3.1  
**Plan Reference:** Part 5 Phase D (492), Part 6 (1193-1200)

**Deliverables:**
- [ ] Test proving GIL release during write-side serialization
- [ ] Two threads writing different files concurrently
- [ ] Measure speedup: 1.8x+ for 2 threads

**Success Criteria:**
- Write speedup ≥ 1.8x for 2 threads
- Output files identical to sequential writes
- Write performance matches read performance

---

#### D.3: Implement round-trip tests: read-write-read verification
**Task:** mrrc-9wi.3.3  
**Status:** 🟠 Blocked on Phase B  
**Priority:** P1  
**Dependencies:** Depends on mrrc-9wi.3.1  
**Plan Reference:** Part 6 (1222-1228), Part 8 (1306)

**Deliverables:**
- [ ] Load original file (fixture_1k.mrc)
- [ ] Read all records into list
- [ ] Write to new file
- [ ] Read new file back
- [ ] Verify byte-for-byte identical

**Test Coverage:**
- [ ] Empty files (0 records)
- [ ] Single record
- [ ] Large records (>100KB)
- [ ] Special characters, non-Latin scripts
- [ ] Variable-length fields
- [ ] Control fields vs data fields

**Success Criteria:**
- All records survive round-trip
- Field order preserved
- Encoding preserved
- No data loss

---

### Phase D Success Criteria

- ✅ Write-side GIL release verified (speedup ≥1.8x)
- ✅ Round-trip tests pass
- ✅ Write performance matches read performance
- ✅ All existing write tests pass

**Estimated Time:** 20 hours  
**Blocker for:** Phase E  
**Progress:** 0% (not started)

---

## Phase E: Comprehensive Validation and Testing

**Epic:** mrrc-9wi.4  
**Duration:** Week 4-5 (15 hours)  
**Priority:** P2 (Support Phase)  
**Status:** 🟠 Ready after Phase D completes  
**Plan Reference:** GIL_RELEASE_IMPLEMENTATION_PLAN.md Part 6 (1150-1278)

### Overview
Comprehensive testing of concurrency edge cases and regression against pymarc compatibility.

### Detailed Tasks

#### E.1: Concurrency testing: threading contention, EOF, file close semantics
**Task:** mrrc-9wi.4.1  
**Status:** 🟠 Blocked on Phase D  
**Priority:** P2  
**Dependencies:** Depends on mrrc-9wi.3 (Phase D)  
**Plan Reference:** Part 6 (1280-1302)

**Test Coverage:**
- [ ] Threading contention (1, 2, 4, 8 threads)
- [ ] EOF behavior (StopIteration consistent)
- [ ] File close semantics (IOError on closed file)
- [ ] Concurrent readers on separate files
- [ ] Single reader multiple threads (should fail safely)

**Success Criteria:**
- All concurrency tests pass
- No panics on edge cases
- Graceful error handling

---

#### E.2: Regression testing: existing pymarc compatibility tests
**Task:** mrrc-9wi.4.2  
**Status:** 🟠 Blocked on Phase D  
**Priority:** P2  
**Dependencies:** Depends on mrrc-9wi.3 (Phase D)  
**Plan Reference:** Part 6 (1304-1312)

**Coverage:**
- [ ] Data type conversions
- [ ] Field access and iteration
- [ ] Record iteration
- [ ] Leader/directory/fields parsing
- [ ] Encoding handling
- [ ] Special characters and non-Latin scripts

**Success Criteria:**
- All existing tests pass without modification
- No behavioral changes
- Data integrity guaranteed

---

### Phase E Success Criteria

- ✅ All concurrency tests pass
- ✅ All regression tests pass
- ✅ No panics on edge cases
- ✅ Implementation stable and robust

**Estimated Time:** 15 hours  
**Blocker for:** Phase F  
**Progress:** 0% (not started)

---

## Phase F: Benchmark Refresh and Performance Analysis

**Epic:** mrrc-9wi.5  
**Duration:** Week 5-6 (16 hours)  
**Priority:** P2 (Support Phase)  
**Status:** 🟠 Ready after Phase E completes  
**Plan Reference:** GIL_RELEASE_IMPLEMENTATION_PLAN.md Part 5 Phase C (lines 416-470)

### Overview
Measure threading speedup and compare pymrrc efficiency to pure Rust baseline. **Results gate Phase C activation decision.**

### Detailed Tasks

#### F.1: Measure threading speedup curve (1, 2, 4, 8 threads) - post implementation
**Task:** mrrc-9wi.5.1  
**Status:** 🟠 Blocked on Phase E  
**Priority:** P2  
**Dependencies:** Depends on mrrc-9wi.4 (Phase E)  
**Plan Reference:** Part 5 Phase C (472-480)

**Measurements:**
- [ ] Single-thread: fixture_10k
  - Current: _____ ops/sec
  - Post-change: _____ ops/sec
  
- [ ] 2 threads: 2 × 5K records
  - Time: _____ seconds
  - Speedup vs single-thread: _____ x (target: ≥2.0x)
  
- [ ] 4 threads: 4 × 2.5K records
  - Time: _____ seconds
  - Speedup: _____ x (target: ≥3.0x)
  
- [ ] 8 threads: 8 × 1.25K records
  - Time: _____ seconds
  - Speedup: _____ x

**Deliverables:**
- [ ] Speedup curve plot (1, 2, 4, 8 threads)
- [ ] Results saved to .benchmarks/post_phase_f_results.txt
- [ ] Comparison vs Phase B baseline

**Success Criteria:**
- 2-thread speedup ≥ 2.0x (target: 2.0x)
- 4+ thread speedup ≥ 3.0x (target: 3.0x+)
- Curve is monotonic (no inversions)

---

#### F.2: Compare pymrrc threading efficiency to pure Rust baseline (rayon)
**Task:** mrrc-9wi.5.2  
**Status:** 🟠 Blocked on Phase E  
**Priority:** P2  
**Dependencies:** Depends on mrrc-9wi.5.1  
**Plan Reference:** Part 5 Phase C (481-488)

**Comparison:**
- [ ] Run Rust bench: `cargo bench parallel_4x_1m`
  - Rayon 4-thread result: _____ ops/sec
  
- [ ] Run Python bench: 4 threads on fixture_1m
  - pymrrc 4-thread result: _____ ops/sec
  
- [ ] Calculate efficiency: (pymrrc / Rayon) × 100%
  - Target: ≥ 90% of Rust efficiency
  - Result: _____ %

**Deliverables:**
- [ ] Benchmark comparison table
- [ ] Efficiency analysis
- [ ] Saved to .benchmarks/pymrrc_vs_rust_efficiency.txt

**Success Criteria:**
- pymrrc efficiency ≥ 90% of pure Rust
- Speedup curve is reasonable given hardware

---

### 🔑 CRITICAL GATE: Phase C Deferral Decision

**Decision Task:** mrrc-pfw (Phase C Deferral Gate)  
**Executes after:** F.2 completes

**Decision Criteria:**
- Compare 2-thread speedup (from F.1) vs baseline (from B.5)
- If speedup ≥ 2.0x → Phase C SKIPPED
- If speedup < 2.0x → Phase C REQUIRED

**Outcome Actions:**
- ✅ Speedup ≥ 2.0x: Proceed to Phase G immediately
- ❌ Speedup < 2.0x: Create mrrc-9wi.7 (Phase C epic) and execute before Phase G

---

### Phase F Success Criteria

- ✅ Speedup curve measured (1, 2, 4, 8 threads)
- ✅ pymrrc efficiency ≥ 90% of Rust baseline
- ✅ All results documented
- ✅ Phase C decision made and documented

**Estimated Time:** 16 hours  
**Blocker for:** Phase G (pending gate decision)  
**Progress:** 0% (not started)

---

## Phase G: Documentation Refresh

**Epic:** mrrc-9wi.6  
**Duration:** Week 6-7 (20 hours)  
**Priority:** P2 (Final Phase)  
**Status:** 🟠 Ready after Phase F gate decision  
**Plan Reference:** GIL_RELEASE_IMPLEMENTATION_PLAN.md Part 5 Phase G

### Overview
Update all documentation to reflect GIL release feature and performance improvements.

**Blocker:** Phase C decision gate (mrrc-pfw) - proceed only if Phase C skipped or completed

### Detailed Tasks

#### G.1: Update README and API docs with threading performance section
**Task:** mrrc-9wi.6.1  
**Status:** 🟠 Blocked on Phase F gate  
**Priority:** P2  
**Dependencies:** Depends on mrrc-9wi.6 (Phase G)

**Deliverables:**
- [ ] README.md: Add threading performance section
  - Feature overview
  - Expected speedups (2x for 2 threads, 3x+ for 4 threads)
  - Example usage
  
- [ ] API docs: Add GIL release notes
  - Note that reads/writes can be parallelized
  - Thread safety guarantees
  - Usage patterns

**Success Criteria:**
- Clear explanation of threading benefits
- Performance expectations documented
- Examples are correct

---

#### G.2: Create PERFORMANCE.md with detailed benchmark results and analysis
**Task:** mrrc-9wi.6.2  
**Status:** 🟠 Blocked on Phase F gate  
**Priority:** P2  
**Dependencies:** Depends on mrrc-9wi.6 (Phase G)

**Deliverables:**
- [ ] PERFORMANCE.md documenting:
  - Baseline (single-thread) performance
  - Threading speedup curve
  - pymrrc vs Rust efficiency comparison
  - SmallVec allocation overhead analysis
  - GIL release timing measurements
  
- [ ] Include all Phase F benchmark results
- [ ] Performance tuning recommendations

**Success Criteria:**
- Results clearly presented
- Analysis actionable
- Baseline documented for future reference

---

#### G.3: Update ARCHITECTURE.md and CHANGELOG.md
**Task:** mrrc-9wi.6.3  
**Status:** 🟠 Blocked on Phase F gate  
**Priority:** P2  
**Dependencies:** Depends on mrrc-9wi.6 (Phase G)

**Deliverables:**
- [ ] ARCHITECTURE.md:
  - Three-phase pattern explanation
  - BufferedMarcReader design
  - SmallVec strategy rationale
  - ParseError enum approach
  
- [ ] CHANGELOG.md:
  - Feature: GIL release for I/O operations
  - Performance: 2-3x threading speedup
  - Internal changes (not breaking)

**Success Criteria:**
- Architecture clearly explained
- Design rationale documented
- CHANGELOG follows format

---

#### G.4: Create example code: concurrent_reading.py and concurrent_writing.py
**Task:** mrrc-9wi.6.4  
**Status:** 🟠 Blocked on Phase F gate  
**Priority:** P2  
**Dependencies:** Depends on mrrc-9wi.6 (Phase G)

**Deliverables:**
- [ ] examples/concurrent_reading.py
  - ThreadPoolExecutor with multiple files
  - Shows 2x+ speedup vs sequential
  - Realistic workload
  
- [ ] examples/concurrent_writing.py
  - ThreadPoolExecutor writing multiple files
  - Shows write-side parallelism
  - Realistic workload

**Success Criteria:**
- Examples are runnable
- Clearly demonstrate speedup
- Include performance measurement

---

### Phase G Success Criteria

- ✅ README updated with threading section
- ✅ PERFORMANCE.md created with all results
- ✅ ARCHITECTURE.md updated with design explanation
- ✅ CHANGELOG.md documents feature
- ✅ Example code demonstrating concurrent usage

**Estimated Time:** 20 hours  
**Progress:** 0% (not started)

---

## Summary Table

| Phase | Epic | Duration | Status | Blocker | Notes |
|-------|------|----------|--------|---------|-------|
| A | mrrc-9wi.1 | 1 wk | 🟢 Ready | — | 4 tasks, core infrastructure |
| B | mrrc-9wi.2 | 1-2 wk | 🟠 After A | A | 5 tasks, GIL release, **baseline gate** |
| C | mrrc-9wi.7 | 1 wk | ⏸️ Optional | Phase F | Only if speedup < 2x (activate or skip) |
| D | mrrc-9wi.3 | 1 wk | 🟠 After B | B | 3 tasks, writer implementation |
| E | mrrc-9wi.4 | 1 wk | 🟠 After D | D | 2 tasks, validation and testing |
| F | mrrc-9wi.5 | 1 wk | 🟠 After E | E | 2 tasks, **Phase C deferral gate** |
| G | mrrc-9wi.6 | 1-2 wk | 🟠 After F | F gate | 4 tasks, documentation |

**Total Duration:** 5–7 weeks (with Phase C optional)  
**Critical Path:** A → B → D → E → F → G (6 weeks minimum)

---

## Key Decision Gates

### Gate 1: Baseline Benchmark (Task B.5)
**When:** End of Phase B  
**Action:** Document single-thread and 2-thread baseline performance  
**Output:** `.benchmarks/baseline_before_gil_release.txt`  
**Next:** Proceed to Phase D

### Gate 2: Phase C Deferral (Task mrrc-pfw)
**When:** End of Phase F  
**Action:** Compare Phase F speedup vs Phase B baseline
**Criteria:**
- If speedup ≥ 2.0x: **Phase C SKIPPED**, proceed to Phase G
- If speedup < 2.0x: **Create mrrc-9wi.7** and execute Phase C before Phase G
**Output:** `.benchmarks/PHASE_C_DECISION.txt`  
**Next:** Phase G (or Phase C if activated)

### Gate 3: Quality Check (Phase D completion)
**When:** End of Phase D  
**Action:** Verify round-trip tests pass, write speedup ≥1.8x  
**Criteria:** All D tasks complete successfully  
**Next:** Proceed to Phase E

---

## Progress Tracking Template

Use this template to track progress. Update weekly:

```markdown
## Week [N] Progress Update

**Dates:** [Start] - [End]  
**Phase Focus:** [A/B/C/D/E/F/G]

### Completed Tasks
- [ ] Task ID: [Short description]
- [ ] Task ID: [Short description]

### In Progress
- [ ] Task ID: [Short description]

### Blockers
- [Blocker description]

### Notes
- [Any notes or context]

**Next Week Plan:**
- [Next priority tasks]
```

---

## References

- **Implementation Plan:** GIL_RELEASE_IMPLEMENTATION_PLAN.md
- **Work Review:** .beads/GIL_RELEASE_WORK_REVIEW.md
- **Beads Tickets:** `bd list --json | jq '.[] | select(.id | startswith("mrrc-9wi"))'`
- **Design Docs:** docs/design/GIL_RELEASE_*.md

---

**Last Updated:** January 1, 2026  
**Next Review:** After Phase A completion
