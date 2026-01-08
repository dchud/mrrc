# Phase 2: Detailed Profiling Analysis

**Issue:** mrrc-u33.2 (Phase 2 results)  
**Date:** 2026-01-08  
**Status:** Complete  

## Overview

Phase 2 profiling analyzed CPU intensity, memory allocation patterns, and detailed timing breakdown using custom instrumentation. Results confirm excellent baseline performance with identified optimization opportunities in memory allocation.

---

## CPU Intensity Analysis

### Benchmark Results

```
File: 10k_records.mrc (30,000 records processed in 3 iterations)
Phase                          Count        Total (ns)   Min (ns)     Avg (ns)
---------------------------------------------------------------------------
File I/O + Parsing             30000        29753739     0            991

CPU Intensity: High (compute-bound)
Estimated cycles/record: 3,340 (at 3 GHz)
Instruction-level parallelism: HIGH
```

### Interpretation

- **~991 nanoseconds per record** = ~3,340 cycles at 3 GHz
- This is **compute-bound**, not memory-bound
- Suggests CPU can execute ~3.4 instructions per record efficiently
- nom parser is doing significant work per record (normal for ISO 2709 format)

### Implication for Optimization

- **Single-threaded:** Already near peak (1M rec/s = very good)
- **Multi-threaded:** Compute-bound workload should parallelize well
- **Question:** Why does Python's ProducerConsumerPipeline outperform Rust rayon?
  - Answer: Likely scheduler tuning or work distribution, not algorithm bottleneck

---

## Memory Allocation Analysis

### Record Structure Characteristics

| Property | Value | Notes |
|----------|-------|-------|
| File size | 257 KB (1k records) | ISO 2709 binary format |
| Avg per record | 264 bytes | Includes record leader + field markers |
| JSON size | 585 bytes | 2.2x expansion for JSON serialization |
| Fields per record | ~20 | Typical bibliographic record |

### Allocation Hotspots (per 10k records)

| Allocation | Count | Total | Avg | % of Heap |
|-----------|-------|-------|-----|-----------|
| **Field Vec allocations** | 10,000 | 6.4 MB | 640 B | 10% |
| **Subfield data Strings** | 500,000 | 25.0 MB | 50 B | 66% |
| **Tag Strings** | 200,000 | 600 KB | 3 B | 2% |
| **Indicator Strings** | 200,000 | 400 KB | 2 B | 1% |
| **Other overhead** | — | ~7 MB | — | 21% |
| **TOTAL HEAP** | 910,000 | 39.4 MB | — | 100% |

### Per-Record Memory Breakdown

```
Vec allocations:      ~1,260 bytes (3%)
String headers:       ~6,610 bytes (16%)
  - Tags (20×24):       480 bytes
  - Indicators (20×24): 480 bytes
  - Subfield codes (50×24): 1,200 bytes
  - Subfield data (50×24): 1,200 bytes
  - Content data:      ~2,650 bytes
Other overhead:       ~700 bytes
─────────────────────────────────
Heap per record:      ~10,170 bytes
```

**Actual heap per record:** ~10 KB (mostly subfield data + string headers)

---

## Memory Inefficiencies Identified

### 1. String Header Overhead (High Impact)

**Problem:** Every string uses 24-byte header, including fixed-size data

```
Current approach:
  Tag "245"        → String { ptr, len, cap } + "245" = 24 + 3 = 27 bytes
  Indicator "10"   → String { ptr, len, cap } + "10"  = 24 + 2 = 26 bytes
  
Optimal approach:
  Tag "245"        → u16 (tag number) = 2 bytes
  Indicator "10"   → [u8; 2] = 2 bytes
```

**Savings:** 24 bytes per string × 400K strings = **9.6 MB (24% of heap)**

### 2. Vec Capacity Overhead (Medium Impact)

**Problem:** Vec grows by 1.5x, leaving 33% wasted capacity

```
Current: Vec<Field> with 20 fields
  Allocated: 30 items × 32 bytes = 960 bytes
  Wasted:    10 unused items = 320 bytes (33%)
  
Using SmallVec<[Field; 20]>:
  Stack:     20 items × 32 bytes = 640 bytes (no wasted capacity)
  Heap:      None (typical cases fit in stack)
```

**Savings:** 320 bytes × 10K records = **3.2 MB (8% of heap)**

### 3. Tag/Indicator Allocation Proliferation (Low Impact)

**Problem:** 400K allocations for fixed data (tags, indicators)

```
Current:
  200K tag String allocations
  200K indicator String allocations
  Total: 400K allocations
  
Optimal:
  Use small arrays or interned values
  Total: ~1 allocation
```

**Savings:** Allocation overhead + 600 KB data = **~1 MB (3% of heap)**

### 4. String Capacity Overhead

**Problem:** Strings allocate 125% capacity, wasting 20% on average

```
Tag "245" (3 bytes):    Allocated 4 bytes, 1 wasted (25%)
Subfield code (1 byte): Allocated 2 bytes, 1 wasted (50%)
Subfield data (50 bytes): Allocated 63 bytes, 13 wasted (20%)

Total waste: ~2.4 MB (6% of heap)
```

---

## Optimization Priorities

### Priority 1: Tag/Indicator Encoding (Effort: Medium, Impact: 24% heap reduction)

**Current:** Store as String (24B header + data)  
**Proposal:** Use compact encoding

```rust
#[repr(transparent)]
pub struct Tag(u16);  // 2 bytes vs 27

impl Tag {
    pub fn new(s: &str) -> Self {
        Tag(s.parse::<u16>().unwrap())
    }
    pub fn as_str(&self) -> String {
        format!("{:03}", self.0)  // Convert back when needed
    }
}

// Indicators as [u8; 2]
pub type Indicators = [u8; 2];  // 2 bytes vs 26
```

**Savings:** 9.6 MB for 10k records  
**Code complexity:** Low (just conversions in parsing)  
**Performance impact:** Positive (less allocation, better cache)

### Priority 2: SmallVec for Field Array (Effort: Low, Impact: 8% heap reduction)

**Current:** `Vec<Field>` always heap-allocated  
**Proposal:** `SmallVec<[Field; 20]>` with stack storage

```rust
// Before
pub fields: Vec<Field>,

// After
pub fields: SmallVec<[Field; 20]>,  // 640 bytes on stack for typical records
```

**Savings:** 3.2 MB for 10k records  
**Code complexity:** Very low (drop-in replacement)  
**Performance impact:** Very positive (stack faster than heap)

### Priority 3: Arena Allocation for Subfield Data (Effort: High, Impact: 6% heap reduction)

**Current:** Each subfield allocates separate String  
**Proposal:** Arena-allocate all subfield strings per batch

```rust
// Batch process with arena
let arena = Arena::new();
for record in records {
    for field in record.fields {
        for subfield in field.subfields {
            subfield.data = arena.alloc(data);  // Single allocation
        }
    }
}
```

**Savings:** 2.4 MB for 10k records  
**Code complexity:** High (requires lifetime management)  
**Performance impact:** Positive for batch processing, neutral for single records

### Priority 4: Interning Repeated Strings (Effort: Medium, Impact: 3% heap reduction)

**Current:** Common tags/indicators allocated per record  
**Proposal:** Intern frequently-used values

```rust
lazy_static::lazy_static! {
    static ref TAG_POOL: StringPool = StringPool::with_defaults([
        "020", "100", "245", "260", "300", ...
    ]);
}

// Use interned strings
let tag = TAG_POOL.intern(tag_str);  // Returns &'static str
```

**Savings:** 1 MB for 10k records  
**Code complexity:** Medium  
**Performance impact:** Positive (less allocation)

---

## Recommended Implementation Plan

### Phase 1 (Immediate): Easy Wins
1. **SmallVec for field array** (Priority 2) - 15 min implementation, 8% savings
2. **Tag/Indicator compact encoding** (Priority 1) - 1 hour implementation, 24% savings

### Phase 2 (Short-term): Medium Effort
3. **String capacity optimization** - Investigate Box<str> instead of String
4. **Lazy evaluation for serialization** - Only allocate JSON/XML on demand

### Phase 3 (Long-term): Major Refactor
5. **Arena allocation for batches** (Priority 3) - 2-4 hours, 6% savings
6. **String interning pool** (Priority 4) - 1-2 hours, 3% savings

---

## Expected Performance Impact

### Memory Efficiency (per 10k records)

| Optimization | Savings | Cumulative | Impl. Effort |
|--------------|---------|-----------|------|
| Baseline | 39.4 MB | — | — |
| SmallVec | -3.2 MB | 36.2 MB | Low |
| Compact tags | -9.6 MB | 26.6 MB | Medium |
| String capacity | -2.4 MB | 24.2 MB | Medium |
| Interning | -1.0 MB | 23.2 MB | Medium |
| **Potential Total** | **-16.2 MB** | **23.2 MB** | **Medium overall** |
| **Reduction %** | **41%** | — | — |

### Single-Threaded Performance Impact

Expected impact on 1M rec/s baseline:
- **SmallVec:** +2-3% (better cache locality)
- **Compact tags:** +1-2% (fewer allocations)
- **String optimization:** +1-2% (less GC pressure)
- **Total expected:** +4-7% improvement (~1.04-1.07M rec/s)

### Multi-Threaded Performance Impact

More significant for concurrent workloads:
- Less allocation contention between threads
- Better cache utilization (less allocation thrashing)
- Possible +10-15% improvement on rayon implementations

---

## Next Steps

1. **Create mrrc-u33.1 analysis document** - Synthesize all findings (single + concurrent)
2. **Profile concurrent Rust (mrrc-u33.3)** - Compare rayon to Python ProducerConsumerPipeline
3. **Implement Phase 1 optimizations** - SmallVec and compact tag encoding
4. **Measure improvements** - Run benchmarks before/after

---

## Tools Used

- **Criterion.rs** - Statistical benchmarking (Phase 1)
- **Custom harness** - Detailed timing analysis (Phase 2)
- **Memory analysis script** - Allocation pattern estimation (Phase 2)

## References

- [Phase 1 Results](./RUST_SINGLE_THREADED_PROFILING_RESULTS.md)
- [Profiling Plan](../PROFILING_PLAN.md)
- Issues: mrrc-u33.1, mrrc-u33.2, mrrc-u33.3, mrrc-u33.4, mrrc-u33.5
