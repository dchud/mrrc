# Performance Optimization Proposal

**Status:** Proposed  
**Based on:** Profiling results in docs/design/profiling/  
**Related issues:** mrrc-u33 (epic), mrrc-u33.1, mrrc-u33.2, mrrc-u33.3

## Executive Summary

Based on comprehensive profiling across all implementation modes, several optimization opportunities have been identified:

1. **Rust Memory Efficiency (High ROI, Easy, 3 hours)**
   - Current: 39.4 MB heap for 10k records with 73% metadata overhead
   - Opportunity: -32% memory via SmallVec + compact tag encoding
   - Performance gain: +6%

2. **Rust Concurrency Work Distribution (High ROI, Medium effort, 4-6 hours)**
   - Current: Rayon achieves 2.52x speedup on 4 cores (63% efficiency)
   - Gap: Python ProducerConsumerPipeline achieves 3.74x (93.5% efficiency)
   - Opportunity: Batch processing + producer-consumer pattern
   - Performance gain: +10-15% on multi-core

3. **Python GIL and FFI Overhead (Medium ROI, Medium effort)**
   - Current: GIL adds ~14% overhead, FFI boundary crossing ~30% cost
   - Opportunity: Batch operations, reduce per-record FFI calls, lazy evaluation
   - Performance gain: +5-15%

## Context: Profiling Findings

### Pure Rust Single-Threaded
- **Throughput:** 1.06M rec/s (excellent)
- **Latency:** 0.94 µs/record
- **Bottleneck:** Memory allocation (73% overhead)
- **CPU:** Compute-bound, 3,340 cycles/record
- **No algorithmic bottlenecks found**

### Python Wrapper Single-Threaded
- **Throughput:** ~32k rec/s (baseline iteration)
- **Bottleneck:** FFI boundary crossing (~30%), field materialization (~22%)
- **GC Impact:** ~14% throughput loss
- **Opportunity:** Batch operations to amortize FFI cost

### Concurrency Comparison
- **Rust (rayon):** 2.52x speedup on 4 cores = 63% efficiency
- **Python (ProducerConsumerPipeline):** 3.74x speedup on 4 cores = 93.5% efficiency
- **Gap:** Better work distribution in Python, not faster parsing

## Proposed Optimizations

### Phase 1: Rust Memory Efficiency (Recommended)

**Target:** Reduce memory allocation overhead in pure Rust

**Changes:**
1. Replace `Vec<Field>` with `SmallVec<[Field; 20]>`
   - Typical records have ~20 fields
   - Eliminates heap allocation for common case
   - Expected impact: +2-3% performance, -8% memory

2. Encode tags as `u16` instead of `String`
   - Tags are always 3-digit numbers (000-999)
   - Replace 27-byte String with 2-byte u16
   - Expected impact: +1-2% performance, -24% memory

3. Encode indicators as `[u8; 2]` instead of `String`
   - Indicators are always 2 ASCII characters
   - Replace 26-byte String with 2-byte array
   - Expected impact: Minimal performance, -3% memory

**Metrics:**
- Before: 1.06M rec/s, 39.4 MB heap (10k records)
- After: 1.12M rec/s, 26.8 MB heap (10k records)
- Overall: +6% performance, -32% memory
- Backward compatibility: ✓ (internal only, no API changes)

**Implementation time:** ~3 hours

---

### Phase 2: Rust Concurrency - Producer-Consumer Pattern

**Target:** Improve work distribution and achieve Python's efficiency

**Problem:** Current rayon implementation processes records individually, leading to work starvation and context switching overhead.

**Solution:** Batch processing + producer-consumer pattern
1. Producer thread reads and buffers batches of records (e.g., 1000 at a time)
2. Consumer threads (via rayon) process batches in parallel
3. Bounded channel prevents unbounded buffering

**Expected Improvements:**
- Reduce task scheduling overhead (fewer smaller tasks)
- Better CPU cache utilization (batch processing)
- Prevent consumer starvation (predictable buffering)
- Expected: 2.52x → 3.2x+ speedup on 4 cores

**Implementation time:** 4-6 hours

**Note:** Python's ProducerConsumerPipeline already implements this pattern. Rust can benefit from similar approach.

---

### Phase 3: Python Wrapper - FFI and GIL Optimization

**Target:** Reduce FFI boundary crossing and GIL contention

**Opportunities:**
1. Batch operations
   - Return multiple records per FFI call
   - Reduce call frequency by 10-100x
   - Expected impact: +20-30% speedup

2. Lazy field evaluation
   - Store raw field data in Record
   - Parse fields on-demand
   - Expected impact: +5-10% speedup

3. Object pooling / arena allocation
   - Pre-allocate Field objects
   - Reuse across iterations
   - Expected impact: +5-8% speedup (GC reduction)

4. Cache field lookups
   - GIL release during field access
   - Cache results to reduce FFI calls
   - Expected impact: +2-5% speedup

**Implementation time:** 6-10 hours (depending on scope)

---

### Phase 4: Advanced Optimizations (Low Priority)

**Rust Single-Threaded (Low ROI, already ~1.06M rec/s):**
- Arena allocation for subfield data
- String interning for repeated values
- SIMD vectorization for record boundary detection

**Python (Lower priority, focus on batching first):**
- Native extension module for hot paths
- Direct memory access for field parsing
- GIL-free batching via custom locks

---

## Decision Matrix

| Optimization | Effort | ROI | Risk | Priority |
|--------------|--------|-----|------|----------|
| SmallVec + Compact Tags | Low | High | Low | **1 - Implement immediately** |
| Producer-Consumer (Rust) | Medium | High | Medium | **2 - Implement after profiling complete** |
| FFI Batching (Python) | Medium | High | Medium | **3 - Implement after Rust phase 2** |
| Lazy Field Eval (Python) | Medium | Medium | Low | 4 - Consider after #3 |
| Object Pooling (Python) | Low | Medium | Low | 4 - Consider after #3 |
| Advanced Optimizations | High | Low | High | 5 - Backlog |

---

## Implementation Roadmap

### Week 1: Phase 1 (Rust Memory)
- Implement SmallVec integration
- Encode tags as u16
- Encode indicators as [u8; 2]
- Benchmark and verify +6% improvement
- Estimated: 3 hours

### Week 2: Complete Profiling
- Finish profiling of remaining modes (mrrc-u33.1, u33.3)
- Validate phase 1 improvement
- Prepare for phase 2 (producer-consumer)

### Week 3: Phase 2 (Rust Concurrency)
- Implement batching in Rust concurrent path
- Add bounded channel for producer-consumer
- Benchmark and target 3.2x+ speedup
- Estimated: 4-6 hours

### Week 4+: Phase 3 (Python Optimization)
- Implement FFI batching
- Add lazy field evaluation if beneficial
- Benchmark Python improvements

---

## Success Criteria

- [ ] Phase 1: +6% performance, -32% memory, zero API changes
- [ ] Phase 2: 2.52x → 3.2x+ speedup on 4 cores, 93%+ efficiency
- [ ] Phase 3: +20-30% speedup for batched Python operations
- [ ] All optimizations verified via profiling
- [ ] No performance regressions in other modes
- [ ] Updated benchmarks in CI

---

## Risks and Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| SmallVec increases code complexity | Low | Low | Use well-tested crate, add unit tests |
| Batching changes user-visible API | Low | High | Keep API unchanged, batch internally only |
| Concurrency introduces race conditions | Medium | High | Careful synchronization testing, add thread tests |
| Python batching reduces responsiveness | Low | Medium | Make batch size tunable, measure latency |

---

## References

- Profiling results: `docs/design/profiling/`
- Rust profiling: `docs/design/profiling/pure_rust_*_profile.md`
- Python profiling: `docs/design/profiling/pymrrc_*_profile.md`
- Related issues: mrrc-u33, mrrc-u33.1, mrrc-u33.2, mrrc-u33.3, mrrc-u33.4, mrrc-u33.5

---

## Questions & Discussion

**Q: Should we implement all phases?**  
A: Start with Phase 1 (easy win), validate results, then proceed based on impact.

**Q: Will Phase 1 break backward compatibility?**  
A: No - SmallVec is a drop-in Vec replacement, tag/indicator encoding is internal.

**Q: How much total improvement is possible?**  
A: Rust: +6% (P1) + 10-15% (P2) = +16-21% total  
   Python: +20-30% (batching) + 5-10% (lazy eval) = +25-40% total

**Q: When should we start?**  
A: Phase 1 immediately (3 hours, low risk). Phase 2 after completing all profiling (understand full picture first).
