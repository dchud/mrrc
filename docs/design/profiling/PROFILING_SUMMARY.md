# Pure Rust Single-Threaded Performance Profile Summary

**Objective:** Within-mode bottleneck analysis for pure Rust sequential implementation  
**Date:** 2026-01-08  
**Status:** Complete  

---

## Quick Links

| Document | Purpose |
|----------|---------|
| [PROFILING_PLAN.md](./PROFILING_PLAN.md) | Methodology and tools |
| [RUST_SINGLE_THREADED_PROFILING_RESULTS.md](./RUST_SINGLE_THREADED_PROFILING_RESULTS.md) | Detailed baseline results |
| [PHASE_2_DETAILED_ANALYSIS.md](./PHASE_2_DETAILED_ANALYSIS.md) | CPU intensity and memory analysis |

---

## Executive Summary

Pure Rust (mrrc) single-threaded implementation shows excellent performance with well-understood bottlenecks.

### Performance Baseline: Excellent ✓
- **Throughput:** 1.06M rec/s (0.94 µs/record latency)
- **Field access overhead:** 2-5% minimal
- **Consistency:** Linear scaling across 1k-10k record files
- **Classification:** CPU-bound, not I/O or memory-bound

### No Algorithmic Bottlenecks Found
- Parsing logic is efficient
- nom parser performs well for MARC format
- Field lookups are fast
- Record structure is sound

### Memory Efficiency: Improvable ⚠️
- **Heap usage:** 39.4 MB for 10k records
- **Allocation overhead:** 73% of heap is metadata
- **Primary issue:** String headers (24B each) for fixed-size data
- **Opportunity:** Compact encoding for tags and indicators

---

## Performance Analysis

### Phase 1: Baseline Measurements

**Benchmark Results**

| Operation | Throughput | Latency | Overhead |
|-----------|-----------|---------|----------|
| Read only (1k) | 1.06M rec/s | 0.94 µs | baseline |
| Read only (10k) | 1.06M rec/s | 0.94 µs | +0.2% |
| Parse + field access (1k) | 1.03M rec/s | 0.96 µs | +4.6% |
| Parse + field access (10k) | 1.04M rec/s | 0.96 µs | +2.6% |
| Parse + JSON serialize | 299k rec/s | 3.3 ms | +254% (expected) |
| Parse + XML serialize | 249k rec/s | 4.0 ms | +326% (expected) |

**Interpretation:**
- Read operation alone: 1.06M rec/s (excellent)
- Field access adds minimal overhead (2-5%)
- Serialization is expensive, but expected (not a parsing issue)
- Linear scaling indicates predictable performance

---

### Phase 2: CPU and Memory Analysis

#### CPU Intensity
- **Estimated cycles per record:** 3,340 (at 3 GHz)
- **Classification:** Compute-bound
- **Implication:** Should parallelize well to multiple cores

#### Memory Allocation Hotspots (per 10k records)

| Allocation Type | Count | Total | Avg | % of Heap |
|-----------------|-------|-------|-----|-----------|
| Subfield data Strings | 500k | 25 MB | 50 B | 63% |
| Field Vec | 10k | 6.4 MB | 640 B | 16% |
| Tag Strings | 200k | 600 KB | 3 B | 1.5% |
| Indicator Strings | 200k | 400 KB | 2 B | 1% |
| Other overhead | — | 7 MB | — | 18% |
| **TOTAL** | **910k** | **39.4 MB** | — | **100%** |

#### Memory Inefficiency Breakdown

**Per-Record Memory Usage**
```
Actual data content:      ~2.6 KB
String headers (24B×70):  ~1.7 KB
Vec overhead:             ~0.3 KB
Other pointers/metadata:  ~0.7 KB
─────────────────────────────────
Total per record:         ~5.3 KB (ideal)
                          ~10.2 KB (actual, with overhead)
```

**Key Issue: String Header Overhead**

Every string uses 24-byte header, including fixed-size data:
```
Current approach:
  Tag "245"        → String { ptr, len, cap } + "245" = 27 bytes
  Indicator "10"   → String { ptr, len, cap } + "10"  = 26 bytes
  
Optimal approach:
  Tag "245"        → u16 (tag number) = 2 bytes
  Indicator "10"   → [u8; 2] = 2 bytes
```

---

## Identified Bottlenecks

### Bottleneck #1: Memory Allocation Metadata (High Impact)
- **Cost:** 73% of heap is metadata, only 27% is actual data
- **Root cause:** String headers for small, fixed-size data (tags, indicators)
- **Type:** Memory efficiency, not performance-critical
- **Opportunity:** Encode tags as u16, indicators as [u8; 2]

### Bottleneck #2: Vec Allocations for Fields (Medium Impact)
- **Cost:** 16% of heap for field Vecs
- **Root cause:** Every record allocates Vec, even for typical 20-field records
- **Opportunity:** Use SmallVec<[Field; 20]> to avoid heap for common case

### Bottleneck #3: Serialization Cost (Expected, Low Priority)
- **Cost:** +254-326% overhead for JSON/XML
- **Root cause:** serde_json and quick-xml are general-purpose serializers
- **Note:** This is expected, not a parsing bottleneck
- **Resolution:** Users should avoid serializing every record if possible

---

## What Limits Performance in This Mode?

In pure Rust single-threaded:
1. **CPU throughput** - The algorithm uses 3,340 cycles per record, which is reasonable for MARC parsing
2. **Memory bandwidth** - Reading 264-byte records from file and heap
3. **Serialization** - If outputting to JSON/XML (avoidable)

What does NOT limit performance:
- I/O (buffering is adequate)
- Parsing logic (nom is efficient)
- Field access (minimal overhead)
- Allocation frequency (expected patterns)

---

## Notes on File Sizes

Results are consistent across:
- **1k records** (257 KB)
- **10k records** (2.5 MB)

Scaling is linear, suggesting:
- No pathological behavior at larger scales
- Buffer cache hits for smaller files
- Consistent algorithm performance

---

## Recommendations for Optimization

For optimization proposals based on these profiling results, see **docs/design/OPTIMIZATION_PROPOSAL.md**.

Key findings that inform optimization decisions:
- Memory overhead is fixable (73% metadata)
- CPU efficiency is excellent (no algorithmic bottlenecks)
- Concurrency gap in Rust likely comes from work distribution, not parsing
- SmallVec and compact tag encoding have low risk and high ROI

---

## Confidence Levels

| Finding | Confidence | Basis |
|---------|-----------|-------|
| Throughput is 1.06M rec/s | Very High | Criterion.rs consistent measurements |
| No algorithm bottleneck | Very High | Detailed analysis + CPU profiling |
| Memory overhead is 73% metadata | High | Allocation analysis |
| Field access overhead is 2-5% | High | Consistent measurements |

---

## References

- Profiling methodology: `docs/design/profiling/PROFILING_PLAN.md`
- Detailed results: `docs/design/profiling/RUST_SINGLE_THREADED_PROFILING_RESULTS.md`
- CPU/memory details: `docs/design/profiling/PHASE_2_DETAILED_ANALYSIS.md`
- Optimization proposals: `docs/design/OPTIMIZATION_PROPOSAL.md`
- Related issues: mrrc-u33.2, mrrc-u33.1
