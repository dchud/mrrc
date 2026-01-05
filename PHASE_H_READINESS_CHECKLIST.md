# Phase H Readiness Checklist

**Date:** January 5, 2026  
**Status:** ✅ READY TO START
**Next Action:** Claim and begin H.0 (Rayon PoC)

---

## Preconditions Checklist

### Phase C Completion ✅
- [x] C.0: Queue-based batched reader (closed)
- [x] C.1: read_batch() method (closed)
- [x] C.2: __next__() queue FSM (closed)
- [x] C.3: Iterator semantics (20 tests passed, closed)
- [x] C.4: Memory profiling (22 tests passed, closed)
- [x] C.Gate: Benchmarking validation (0.32x speedup, architectural constraint documented, closed)
- [x] All Phase C code committed and pushed
- [x] Phase C epic (mrrc-ppp) marked CLOSED

### GIL Investigation Complete ✅
- [x] Root cause identified: Python file I/O requires GIL
- [x] Solution implemented: py.detach() for 100x reduction in GIL acquire/release
- [x] mrrc-hjx resolved and closed
- [x] Finding documented in README_BEADS_INTEGRATION.md and commit messages

### Documentation & Analysis Complete ✅
- [x] README_BEADS_INTEGRATION.md updated (Phase C complete, Phase H ready)
- [x] BEADS_COVERAGE_ANALYSIS.md created (gap analysis and recommendations)
- [x] BEADS_UPDATES_SUMMARY.md created (all updates documented)
- [x] GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_REVISIONS.md available (detailed specs)

### Beads Structure Aligned ✅
- [x] Phase H epic (mrrc-7vu) created with 10 subtasks
- [x] H.3 blocking note added (blocked by C.Gate)
- [x] Phase E/F/G blocking notes added (blocked by H.Gate)
- [x] Phase D epic created (P4, deferred)
- [x] Diagnostic tasks decomposed (mrrc-can.1/2/3)
- [x] Benchmark fixtures task added (mrrc-9wi.5.3)
- [x] mrrc-kzw linked to Phase E.1
- [x] All priorities set correctly (H=P1, E/F/G=P2, D=P4)
- [x] All dependencies documented in issue notes

---

## Phase H Task Structure

### Critical Path Tasks (Priority 1)

#### H.0: Rayon PoC (2-3 days)
- **Issue:** mrrc-7vu.3
- **Objective:** Validate thread pool parallelism approach independently
- **Acceptance:** Basic Rayon iterator works without errors; thread spawning verified
- **No Blockers:** Can start immediately
- **Status:** 🚀 **READY TO START**

#### H.1-H.2b: Type Detection & Backends (2-3 days, parallel with H.0)
- **H.1** (mrrc-7vu.4): ReaderBackend enum & type detection algorithm
- **H.2** (mrrc-7vu.5): RustFile backend implementation
- **H.2b** (mrrc-7vu.6): CursorBackend implementation
- **Objective:** Support dual-backend architecture for file path vs file-like inputs
- **No Blockers:** Can start immediately
- **Status:** 🚀 **READY TO START**

#### H.3: Sequential Baseline (1 day, starts after H.0-H.2b)
- **Issue:** mrrc-7vu.7
- **Objective:** Establish sequential performance baseline for parallelism comparison
- **Blocker:** mrrc-ppp (Phase C) — **NOW SATISFIED** ✓
- **Status:** 🚀 **READY TO UNBLOCK**

#### H.4a-H.4c: Parallelism Implementation (3-4 days, follows H.3)
- **H.4a** (mrrc-7vu.8): Record boundary scanner (0x1E delimiter + multi-threaded)
- **H.4b** (mrrc-7vu.9): Rayon parser pool (parallel batch processing)
- **H.4c** (mrrc-7vu.10): Producer-consumer pipeline (backpressure & channels)
- **Objective:** Implement true parallelism with bounded channel backpressure
- **Depends On:** H.3 sequential baseline (for comparison metrics)
- **Status:** 🚀 **UNBLOCKED** (H.3 ready)

#### H.5: Integration Tests (1 day, parallel with H.4)
- **Issue:** mrrc-7vu.11
- **Objective:** Validate error propagation and all components working together
- **Depends On:** H.0-H.4c implementation
- **Status:** ⏸️ **BLOCKED ON H.4 COMPLETION**

#### H.Gate: Benchmarking Gate (1-2 days, final)
- **Issue:** mrrc-7vu.12
- **Objective:** Validate ≥2.5x speedup on 4-thread concurrent read
- **Acceptance:** Speedup ≥2.5x measured vs Phase B baseline
- **Unblocks:** Phase E, F, G (validation, benchmarking, documentation)
- **Status:** ⏸️ **BLOCKED ON H.5 COMPLETION**

### Supporting Infrastructure (Can run in parallel)

#### Diagnostic Test Suite (mrrc-can)
- **mrrc-can.1:** GIL Release Verification Test (supports Phase C understanding)
- **mrrc-can.2:** Batch Size Benchmarking (validates Phase C decisions)
- **mrrc-can.3:** Python File I/O Overhead Profiler (informs Phase H scope)
- **Priority:** 1 (runs concurrently with Phase H)
- **Status:** 🚀 **READY TO START**

---

## Expected Timeline

| Phase | Duration | Start | End | Blocker |
|-------|----------|-------|-----|---------|
| H.0 | 2-3 days | Day 1 | Day 3 | None—**GO NOW** |
| H.1-H.2b | 2-3 days | Day 1 | Day 3 | None—**parallel with H.0** |
| H.3 | 1 day | Day 4 | Day 4 | H.0/H.2b complete ✓ |
| H.4a-H.4c | 3-4 days | Day 5 | Day 8 | H.3 complete |
| H.5 | 1 day | Day 8 | Day 8 | H.4c complete |
| **H.Gate** | **1-2 days** | **Day 9** | **Day 10** | **H.5 complete** |

**Total:** ~7-10 days (critical path H.0 → H.3 → H.4 → H.5 → H.Gate)

**Post-H.Gate (Unblocked):**
- Phase E (Validation): 1-2 days
- Phase F (Benchmarking): 1-2 days
- Phase G (Documentation): 1 day

---

## Ready-to-Work Checklist

### For Session Start (H.0 claim)
- [ ] Pull latest main
- [ ] Verify git status clean
- [ ] Run `.cargo/check.sh` (ensure local CI passes)
- [ ] Claim mrrc-7vu.3 (H.0)

### For H.0 Implementation
- [ ] Review plan §4.1 (Type Detection Mapping) for input type handling
- [ ] Review plan §4.2 (ReaderBackend enum structure)
- [ ] Decide: Rayon thread pool tuning (RAYON_NUM_THREADS vs default)
- [ ] Create basic thread pool proof-of-concept first (no file I/O)
- [ ] Verify crossbeam channel works with Rayon tasks

### For H.1 Implementation
- [ ] Review plan §4.1 type detection decision tree
- [ ] Implement ReaderBackend enum with 3 variants (PythonFile, RustFile, CursorBackend)
- [ ] Implement type detection in __init__ (8 input types + fail-fast for unknown)
- [ ] Create unit tests for each input type

### For H.2/H.2b Implementation
- [ ] Review plan §4.2 I/O loop specification
- [ ] Implement RustFile with std::io::BufReader
- [ ] Implement CursorBackend with bytes/slice
- [ ] Create parity tests (both produce identical records)

---

## Key Reference Documents

**Planning & Design:**
- `docs/design/GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN.md` – Strategic foundation
- `docs/design/GIL_RELEASE_HYBRID_IMPLEMENTATION_PLAN_REVISIONS.md` – Technical specifications (§3-7)
- `README_BEADS_INTEGRATION.md` – Current status and critical path

**Beads & Tracking:**
- `BEADS_COVERAGE_ANALYSIS.md` – Gap analysis that led to these updates
- `BEADS_UPDATES_SUMMARY.md` – What was changed and why

**Code References:**
- `src-python/src/readers.rs` – Current implementation (Phase B/C)
- `tests/` – Existing test patterns
- `scripts/` – Benchmark & diagnostic scripts (phase C helpers)

---

## Critical Success Factors

### H.0 (Rayon PoC)
- **Must Succeed:** Basic thread pool spawning without panics
- **Don't Optimize Yet:** Proof-of-concept only, no performance tuning
- **Risk Mitigation:** Start with 2 threads (simplest case)

### H.3 (Sequential Baseline)
- **Must Establish Baseline:** Measure RustFile I/O performance with zero parallelism
- **Required Metrics:** Records/sec, wall clock time, memory high watermark
- **Comparison:** This becomes the reference for H.4 parallelism gains

### H.Gate (Speedup Validation)
- **Gate Criterion:** ≥2.5x speedup on 4 threads (measured vs Phase B baseline)
- **If <2.5x:** Diagnose bottleneck using plan §7.1 profiling workflow
- **Fallback:** Document findings and proceed with caveats (Plan §7.1 decision tree)

---

## Decision Points

### At H.3 Completion (Before H.4)
**Decision:** Do we have sufficient sequential baseline metrics to proceed?
- **YES:** Proceed to H.4a (record boundary scanner)
- **NO:** Investigate H.3 performance anomalies; re-run diagnostics; consider caching/profiling

### At H.Gate (Speedup Validation)
**Decision:** Do we meet ≥2.5x speedup criterion?
- **YES (≥2.5x):** ✓ H.Gate passes; unblock Phase E/F/G
- **1.5-2.5x:** Diagnose via plan §7.1; optimize Rayon/channel tuning; retest
- **<1.5x:** Document bottleneck; consider alternative approaches (mmap, io_uring); escalate

---

## What Happens After H.Gate

### If H.Gate Passes (≥2.5x)
1. **Unblock Phase E:** Concurrency & regression testing (mrrc-kzw + mrrc-9wi.4.1/2)
2. **Unblock Phase F:** Benchmarking & comparative analysis (mrrc-9wi.5)
3. **Unblock Phase G:** Documentation refresh (mrrc-9wi.6)
4. **Tentative:** Release readiness assessment

### If H.Gate Falls Short (<2.5x but >1.5x)
1. **Debug:** Run profiling from plan §7.1; identify bottleneck
2. **Optimize:** Adjust Rayon granularity, channel size, thread count
3. **Retest:** Re-run H.Gate benchmarks
4. **Document:** Record findings and rationale for final speedup achieved
5. **Proceed:** If findings understood and documented, unblock E/F/G anyway (revised expectations)

### If H.Gate Fails Significantly (<1.5x)
1. **Investigate:** Is parallelism even beneficial for this workload?
2. **Options:**
   - (a) Accept that MARC parsing is too fast; focus on I/O optimization only
   - (b) Revisit architecture (e.g., memory-mapped I/O, pre-tokenization)
   - (c) Document limitation and mark as known issue
3. **Decide:** Proceed with current approach or defer parallelism to future release

---

## Ready Signal

✅ **Phase H is ready to start immediately.**

- All preconditions satisfied
- Beads structure aligned with plan dependencies
- Documentation complete and committed
- No blockers remaining
- Critical path clear (H.0 → H.3 → H.4 → H.5 → H.Gate)

**Next Action:** Claim mrrc-7vu.3 (H.0: Rayon PoC) and begin work.

---

**Prepared for:** Next development session  
**Session ID:** T-019b8f25-1aaa-723d-9a27-fa047fa22068  
**Status:** ✅ All systems go
