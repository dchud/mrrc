# Pure Rust Single-Threaded Profiling Plan

**Issue:** mrrc-u33.2  
**Objective:** Comprehensive profiling of pure Rust (mrrc) single-threaded file reading performance to identify bottlenecks for optimization.

## Background

Current baseline performance (from criterion.rs):
- **Read 1k records:** ~0.94ms (1,062,995 rec/s)
- **Read 10k records:** ~9.39ms (1,064,711 rec/s)

This profiling aims to identify bottlenecks within the pure Rust single-threaded implementation to understand where performance is limited and what optimization opportunities exist in this mode.

## Profiling Targets

### 1. Raw File I/O (syscall overhead)
- Baseline: Current buffered read strategy
- Question: How much time is spent in read operations vs. processing?
- Method: perf syscall tracing, strace

### 2. Record Boundary Detection (leader identification)
- Current: Scan for `0x1d` (field terminator) to find record boundaries
- Question: Is byte-scanning the bottleneck? Can vectorization help?
- Method: flamegraph to see time in parsing loop, cachegrind for branch prediction

### 3. MARC Record Parsing (field extraction)
- Current: nom parser for variable fields
- Question: Is nom overhead significant? Are there hot loops?
- Method: flamegraph, perf instruction-level profiling

### 4. Memory Allocation Patterns
- Current: Vec allocations for records and fields
- Question: Are we allocating too often? Are sizes predictable?
- Method: heaptrack, cachegrind

## Tools and Methods

| Tool | Purpose | Command |
|------|---------|---------|
| **Criterion.rs** | Baseline measurements (already in use) | `cargo bench --release` |
| **flamegraph** | Wall-clock profiling, identify hot functions | `cargo flamegraph --bench marc_benchmarks` |
| **perf** | CPU profiling, cache behavior | `perf record / perf report` |
| **cachegrind** | Cache efficiency, memory patterns | `valgrind --tool=cachegrind` |
| **heaptrack** | Memory allocation hotspots | `heaptrack_app` |

## Execution Plan

### Phase 1: Baseline & Hot Function Identification (15 min)
1. Run criterion benchmarks with default 10k test set
2. Generate flamegraph for 10k record read
3. Identify top 3 time-consuming functions

### Phase 2: Detailed Analysis (30 min)
1. **For top function:** Run cachegrind to understand cache behavior
2. **For syscalls:** Run perf with syscall tracing
3. **For memory:** Run heaptrack to find allocation patterns

### Phase 3: Bottleneck Hypothesis (10 min)
- Synthesize findings
- Generate hypothesis about root cause(s)
- List potential optimization targets

## Success Criteria

✓ Generate flamegraph showing function breakdown  
✓ Identify top 3 bottleneck functions by time spent  
✓ Quantify cache miss rate for hot functions  
✓ Document allocation patterns (count, sizes, frequency)  
✓ Produce written analysis with findings and hypotheses  
✓ Create actionable recommendations for mrrc-u33.1  

## Deliverables

All outputs to be stored in `docs/design/profiling/`:

1. **RUST_SINGLE_THREADED_PROFILING_RESULTS.md**
   - Flamegraph analysis (images + interpretation)
   - Cache statistics (L1, L2, L3 hit rates)
   - Syscall breakdown
   - Memory allocation report
   - Summary table of findings

2. **Flamegraph images**
   - `read_10k_flamegraph.svg` (full 10k record read)
   - `read_1k_flamegraph.svg` (quick profile)

3. **Perf output** (raw data)
   - `perf_syscalls.txt`
   - `perf_report.txt`

4. **Heaptrack output** (raw data)
   - `heaptrack.data` or summary report

5. **Cachegrind output** (raw data)
   - Top functions by cache misses

## Notes

- All benchmarks use `--release` mode (opt-level=3)
- Test fixture: `10k_records.mrc` (standard, ~2.5MB)
- Flamegraph uses sampling at 99Hz frequency (default)
- Cachegrind simulates modern Intel CPU cache behavior
- Heaptrack captures every allocation (may slow execution)

## Next Steps (After Profiling)

Results feed into bottleneck analysis and optimization proposals (see **docs/design/OPTIMIZATION_PROPOSAL.md**).

Key questions this profiling answers:
1. Is I/O the bottleneck or parsing?
2. Can we reduce allocations?
3. Are there cache-friendly optimizations?
4. What limits performance in this single-threaded mode?
