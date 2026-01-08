# Pure Rust Performance Optimization Proposal

**Issue:** mrrc-u33.1  
**Status:** Ready for Implementation  
**Date:** 2026-01-08  

## Executive Summary

Comprehensive profiling of pure Rust (mrrc) single-threaded and memory characteristics reveals:

1. **Single-threaded performance is excellent** (~1M rec/s, no bottlenecks)
2. **Memory allocation is the key optimization opportunity** (41% reduction possible)
3. **Concurrency gap likely due to work distribution**, not baseline performance
4. **Recommended:** Implement 2 quick wins (SmallVec, compact tags) → +6% performance, -32% memory

---

## Problem Statement

The pure Rust concurrent implementation (rayon) achieves 2.52x speedup on 4 cores, while Python's ProducerConsumerPipeline achieves 3.74x—a 48% performance gap. Analysis suggests this is not a single-threaded bottleneck but rather a work-distribution/concurrency issue.

### Current Performance Metrics

| Metric | Value | Status |
|--------|-------|--------|
| Single-threaded read throughput | ~1.06M rec/s | ✓ Excellent |
| Per-record latency | ~0.94 µs | ✓ Excellent |
| 4-core speedup (rayon) | 2.52x | ⚠️ Below potential |
| 4-core speedup (Python) | 3.74x | ✓ Reference |
| Memory per 10k records | 39.4 MB | ⚠️ Improvable |

---

## Root Cause Analysis

### Single-Threaded Bottlenecks (NONE FOUND)

**Profiling Results:**
- No cache thrashing detected (CPU-bound, not memory-bound)
- Field access overhead minimal (2-5%)
- Parsing overhead excellent (1 µs/record)
- No obvious hot functions

**Conclusion:** Single-threaded implementation is optimal for algorithm and data structure.

### Identified Optimization Opportunities

#### 1. Memory Allocation Inefficiency (41% reduction possible)

**Root causes:**
- String headers (24 bytes each) for fixed-size data (tags, indicators)
- Vec capacity overhead (1.5x growth factor leaves 33% unused)
- Per-String allocations for high-cardinality data

**Impact metrics:**
- Allocation hotspots: 910K allocations for 10K records
- Memory per record: 10 KB (subfield data only ~2.6 KB)
- Overhead: 73% of heap is allocation metadata

**Quick wins:**
- SmallVec for field array: 8% reduction, very low effort
- Compact tag/indicator encoding: 24% reduction, medium effort

#### 2. Concurrency Work Distribution (Estimated 15-20% improvement possible)

**Root cause:** Rayon's default task granularity may not be optimal for MARC parsing

**Evidence:**
- Python ProducerConsumerPipeline: 3.74x on 4 cores (93.5% efficiency)
- Rayon implementation: 2.52x on 4 cores (63% efficiency)
- Gap: 30.5% efficiency difference suggests work-stealing imbalance

**Hypothesis:** 
- Producer-consumer decouples I/O from parsing, allowing better buffering
- Rayon's per-record task spawning may have overhead
- Work distribution may be unbalanced (some threads idle)

**Potential fixes:**
- Batch processing: Process N records per task (not 1)
- Custom channels: Pre-buffer records for workers
- Rayon pool tuning: Adjust thread pool size and task granularity

---

## Recommended Optimization Strategy

### Phase 1: Quick Wins (Effort: 3 hours, Impact: +6% perf, -32% memory)

#### 1a. SmallVec for Field Array

**Change:** `Vec<Field>` → `SmallVec<[Field; 20]>`

**Rationale:**
- Typical records have ~20 fields (fits on stack)
- Eliminates heap allocation for common case
- Zero-cost abstraction (drop-in replacement)

**Expected improvement:**
- Memory: -3.2 MB (8% for 10k records)
- Performance: +2-3% (better cache locality)

**Implementation:**
```rust
// src/record.rs
use smallvec::SmallVec;

pub struct MarcRecord {
    fields: SmallVec<[Field; 20]>,  // Stack-backed for typical records
    // ...
}
```

**Effort:** 15 minutes (cargo add smallvec, replace Vec)

#### 1b. Compact Tag/Indicator Encoding

**Change:** String tags/indicators → u16 tags + [u8; 2] indicators

**Rationale:**
- Tags are always 3-digit numbers (001-999)
- Indicators are always 2 characters (ASCII)
- Fix the encoding, eliminate String overhead

**Expected improvement:**
- Memory: -9.6 MB (24% for 10k records)
- Performance: +1-2% (fewer allocations)

**Implementation:**
```rust
// src/tag.rs
#[repr(transparent)]
pub struct Tag(u16);

impl Tag {
    pub fn new(s: &str) -> Result<Self, ParseError> {
        let num = s.parse::<u16>()?;
        if num > 999 { return Err(ParseError); }
        Ok(Tag(num))
    }
    
    pub fn as_str(&self) -> String {
        format!("{:03}", self.0)
    }
}

// In Field:
pub tag: Tag,                    // 2 bytes vs 27
pub indicators: [u8; 2],        // 2 bytes vs 26
```

**Effort:** 1 hour (parsing changes, API updates)

### Phase 2: Concurrency Optimization (Effort: 4-6 hours, Impact: +10-15% perf)

#### 2a. Batch Processing for Rayon

**Change:** Process records in batches, not individual records

**Rationale:**
- Reduces task scheduling overhead
- Better work distribution (larger chunks per worker)
- Similar to Python's producer-consumer buffering

**Implementation sketch:**
```rust
pub fn read_parallel_batched<P: AsRef<Path>>(path: P) -> Vec<MarcRecord> {
    let reader = MarcReader::new(BufReader::new(File::open(path)?));
    
    reader
        .by_batch(100)  // Process 100 records per task
        .par_bridge()
        .flat_map(|batch| batch)
        .collect()
}
```

**Effort:** 2-3 hours (batch iterator, par_bridge setup)

#### 2b. ProducerConsumer Pattern for Parallel Reading

**Change:** Implement bounded-channel producer-consumer pattern

**Rationale:**
- Producer thread reads ahead (I/O-bound)
- Consumer threads parse (CPU-bound)
- Decoupling allows better scheduling

**Implementation sketch:**
```rust
pub fn read_parallel_producer_consumer<P: AsRef<Path>>(
    path: P,
    batch_size: usize,
    num_workers: usize,
) -> Vec<MarcRecord> {
    let (tx, rx) = crossbeam_channel::bounded(batch_size * 2);
    
    let producer = std::thread::spawn(move || {
        // Read and batch records into channel
        for batch in reader.by_batch(batch_size) {
            tx.send(batch).ok();
        }
    });
    
    let results: Vec<_> = rx
        .into_iter()
        .par_bridge()  // Rayon processes in parallel
        .flat_map(|batch| batch)
        .collect();
    
    producer.join().ok();
    results
}
```

**Effort:** 3-4 hours (channel setup, thread management, testing)

### Phase 3: Advanced Optimizations (Effort: 6-8 hours, Impact: +2-5% perf)

#### 3a. Arena Allocation for Subfield Data

**Benefit:** Reduce string allocation overhead for subfield data  
**Effort:** High (lifetime management)  
**Impact:** +2-3% performance, -6% memory

#### 3b. String Interning Pool

**Benefit:** Deduplicate frequently-used tag/indicator strings  
**Effort:** Medium (lazy_static setup)  
**Impact:** +1% performance, -3% memory

---

## Implementation Timeline

### Week 1: Quick Wins
- [ ] Implement SmallVec (15 min)
- [ ] Implement compact tags (1 hour)
- [ ] Test and benchmark (+3-6% improvement)
- [ ] Update documentation

### Week 2: Concurrency
- [ ] Implement batch processing for rayon (2-3 hours)
- [ ] Benchmark against Python baseline
- [ ] Profile with detailed instrumentation
- [ ] Fine-tune batch size and worker count

### Week 3: Optional Advanced
- [ ] Arena allocation (if time permits)
- [ ] Final performance report

---

## Success Criteria

### Performance Targets

| Metric | Current | Target | Priority |
|--------|---------|--------|----------|
| Single-threaded throughput | 1.06M | 1.12M | High |
| 4-core speedup | 2.52x | 3.2x+ | High |
| Memory per 10k records | 39.4 MB | 23.2 MB | Medium |
| GC pressure (allocations/sec) | 91k | 50k | Low |

### Implementation Quality

- [ ] All changes backward compatible
- [ ] Benchmark suite shows improvements
- [ ] Memory profiling confirms 30%+ reduction
- [ ] Python wrapper still achieves parity
- [ ] Documentation updated

---

## Risk Assessment

### Low Risk
- SmallVec: Widely used, well-tested, zero-cost abstraction
- Compact tags: Isolated change, easy to revert

### Medium Risk
- Batch processing: Requires careful synchronization testing
- Producer-consumer: Thread safety requires careful review

### Mitigation
- Add comprehensive benchmarks before/after
- Run full test suite after each change
- Use CI for continuous verification
- Keep changes modular and reversible

---

## References

### Profiling Reports
- [Phase 1: Single-threaded baseline](./profiling/RUST_SINGLE_THREADED_PROFILING_RESULTS.md)
- [Phase 2: Memory analysis](./profiling/PHASE_2_DETAILED_ANALYSIS.md)
- [Profiling methodology](./profiling/PROFILING_PLAN.md)

### Benchmark Data
- docs/benchmarks/RESULTS.md
- docs/PERFORMANCE.md

### Related Issues
- **mrrc-u33.2**: Complete ✓ (single-threaded profiling)
- **mrrc-u33.3**: Pending (concurrent Rust profiling)
- **mrrc-u33.4-5**: Pending (Python wrapper profiling)

---

## Next Steps

1. **Approval:** Review and approve optimization proposal
2. **Implementation:** Create mrrc-u33.2.4 for Phase 1 implementation
3. **Testing:** Run benchmarks and confirm improvements
4. **Concurrent work:** Start mrrc-u33.3 (concurrent profiling) in parallel
5. **Phase 2:** Implement concurrency optimizations based on mrrc-u33.3 findings

---

## Appendix: Detailed Findings

### A. CPU Intensity

- **Compute-bound workload:** High (3,340 cycles/record at 3 GHz)
- **Cache efficiency:** Good (L1/L2 cache pressure within normal range)
- **Instruction-level parallelism:** HIGH
- **Implication:** Should scale well to multi-core with work distribution fixes

### B. Memory Inefficiencies

**Top 3 opportunities:**
1. String headers (24B each) for fixed-size data: 24% reduction
2. Vec capacity overhead (1.5x growth): 8% reduction
3. String capacity buffering (125% allocated): 6% reduction

### C. Concurrency Analysis

**Python ProducerConsumerPipeline advantages:**
1. I/O and parsing are decoupled
2. Producer can prefetch while consumers work
3. Buffering prevents consumer starvation
4. Simpler synchronization (bounded channel)

**Pure Rust rayon disadvantages:**
1. Per-record task spawning (high overhead)
2. No prefetching/buffering between I/O and parsing
3. Work-stealing adds complexity
4. May have load imbalance issues

**Solution:** Adopt buffering + batching approach in pure Rust
