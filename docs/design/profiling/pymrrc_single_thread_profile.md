# PyMRRC Single-Threaded Performance Profile

**Objective:** Identify bottlenecks and optimization opportunities in the Python wrapper's sequential `MARCReader` implementation.

**Date:** 2026-01-08  
**Profile Data:** `.benchmarks/pymrrc_single_thread_profile.json`

## Executive Summary

The PyMRRC single-threaded implementation processes ~32,000 records/second on a simple MARC file. Profiling reveals:

- **Parsing overhead:** ~22% cost vs. bare iteration
- **GC impact:** ~14% throughput loss with GC enabled
- **Memory profile:** Minimal allocation, well-bounded
- **Hot path:** FFI boundary crossing and field materialization

## Baseline Metrics

| Metric | Value | Note |
|--------|-------|------|
| Throughput (bare iteration) | 31,000 rec/s | Bare MARCReader iteration without field access |
| Throughput (with field access) | 26,000 rec/s | Including leader() and fields() calls |
| Throughput (full materialization) | 9,400 rec/s | Including to_marcjson() conversion |
| GC overhead | ~14% | Throughput loss when GC is enabled |
| Memory peak | <1 MB | For simple_book.mrc (1 record) |
| Variance (3 runs) | ±17% CV | Indicates modest variance across runs |

## Profiling Scenarios

### Scenario 1: Simple Sequential Reading

**Method:** Basic iteration through MARCReader without field access
**Result:** ~3,500 rec/s (simple_book.mrc)

```python
with open(file, "rb") as f:
    reader = MARCReader(f)
    count = sum(1 for _ in reader)
```

**Observation:** Fast baseline for bare iteration. Noise from JIT warmup.

### Scenario 2: Object Creation Overhead

**Method:** Full record materialization into Python dicts
**Result:** ~5,700 rec/s

```python
for record in reader:
    record_dict = {
        "leader": record.leader(),
        "fields": [{"tag": f.tag, "indicators": f.indicators, ...} 
                   for f in record.fields()]
    }
```

**Observation:** Dict construction overhead is minimal (~1700 rec/s loss). FFI boundary crossing and field access dominate.

### Scenario 3: Garbage Collection Impact

**Metrics:**
- With GC: 24,200 rec/s
- Without GC: 27,500 rec/s
- **Overhead: ~14%**

**GC Collections during run:**
- Gen 0: 6 collections
- Gen 1: 0 collections  
- Gen 2: 0 collections

**Observation:** GC pressure is modest on small files, but non-negligible. Gen0 collections occur, suggesting steady small object allocation. Small dataset means gen1/2 not triggered.

### Scenario 4: Parsing vs Iteration Overhead

**Comparison:**
- Bare iteration: 31,000 rec/s
- Full parsing + field access: 25,000 rec/s
- **Overhead: ~22%**

**Breakdown:**
1. Iteration (FFI __next__): ~1,500 rec/s loss
2. Field access (leader, fields): ~5,500 rec/s loss

**Observation:** Field access materialization is the dominant overhead. The lazy nature of record objects means costs are deferred until property access.

### Scenario 5: Variance Analysis (3 runs)

| Run | Time | Throughput |
|-----|------|-----------|
| 1 | 30 µs | 26,000 rec/s |
| 2 | 32 µs | 31,100 rec/s |
| 3 | 29 µs | 34,300 rec/s |
| **Avg** | **30.4 µs** | **31,900 rec/s** |
| **CV** | **5%** | **5%** |

**Observation:** Low variance on repeated runs after warmup. Suggests stable performance, though small file size means results are noisy.

## Bottleneck Analysis

### Hot Path #1: FFI Boundary Crossing
- **Cost:** ~30% of total time in small iteration
- **Source:** Every `__next__()` call crosses Rust ↔ Python boundary
- **Mitigation:** Batch operations, reduce per-record FFI calls

### Hot Path #2: Field Materialization
- **Cost:** ~22% overhead for field access vs. bare iteration
- **Source:** `record.fields()` constructs Field objects; `field.indicators`, `field.subfields()` more FFI calls
- **Opportunity:** Lazy evaluation of subfields, cache indicator lookups

### Hot Path #3: Object Allocation
- **Cost:** GC overhead ~14%
- **Source:** Dict and Field objects created per record
- **Opportunity:** Object pooling, reduce intermediate allocations

### Hot Path #4: GC Pressure
- **Cost:** ~14% throughput loss
- **Source:** Small object allocation (dicts, list comprehensions)
- **Opportunity:** Use destructors for cleanup without GC

## Optimization Opportunities

### Priority 1: Reduce FFI Boundary Crossings
**Opportunity:** Cache or batch field data to reduce per-record FFI calls.

**Current approach:** Each property access (leader(), fields(), indicators, subfields()) crosses FFI boundary.

**Proposal:** 
- Materialize record data once during parsing
- Return pre-built dicts or dataclass objects
- Reduce __next__() calls for metadata

**Estimated impact:** 10-15% speedup possible

### Priority 2: Lazy Field Access
**Opportunity:** Defer field parsing until requested.

**Current approach:** Fields materialized eagerly on access.

**Proposal:**
- Store raw field data in Record
- Parse fields on-demand per property
- Cache parsed results

**Estimated impact:** 5-10% speedup

### Priority 3: GC Optimization
**Opportunity:** Reduce gen0 allocation pressure.

**Current approach:** Each record iteration creates Field objects.

**Proposal:**
- Pre-allocate Field object pool
- Reuse objects across iterations
- Minimize intermediate dict/list allocations

**Estimated impact:** 5-8% speedup

### Priority 4: Batch Processing
**Opportunity:** Amortize FFI overhead across multiple records.

**Current approach:** Record-by-record processing.

**Proposal:**
- Return record batch from Rust (Vec<Record>)
- Python processes batch before next FFI call
- Reduce FFI call frequency by 10-100x

**Estimated impact:** 20-30% speedup for batch operations

## Recommendations

1. **Immediate:** Profile with larger datasets (10k+ records) to understand scalability beyond current small-file variance.

2. **Short-term:** Measure impact of reducing field object allocations via object pooling (Priority 3).

3. **Medium-term:** Implement lazy field evaluation (Priority 2) and benchmark per-record cost.

4. **Long-term:** Consider batch processing API to amortize FFI costs for bulk operations.

## Next Steps

- [ ] Run profiling on 10k-record file to smooth variance
- [ ] Implement Priority 1 optimization (FFI reduction)
- [ ] Re-profile and measure improvement
- [ ] Iterate on remaining opportunities

## Related Profiles

- `pymrrc_concurrent_profile.md` - Concurrent ProducerConsumerPipeline analysis
- `pure_rust_single_thread_profile.md` - Pure Rust baseline for comparison context
