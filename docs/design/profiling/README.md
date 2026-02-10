# Performance Profiling

This directory contains performance profiling analyses for MRRC implementations. Each profile analyzes a single implementation mode in isolation to identify bottlenecks and optimization opportunities.

**Key principle:** Profiles are within-mode analyses. We profile each implementation to understand where it spends time and what could be improved within that approach. Profiles are not comparative - we don't use profiling results to justify one mode over another.

**Status:** Refactoring in progress (mrrc-dpk) to ensure all documents align with within-mode scope. See [REFACTORING_NOTES.md](./REFACTORING_NOTES.md) for details.

## Implementation Modes

MRRC has several implementation modes, each with its own performance characteristics:

### Pure Rust
- **Single-threaded**: Direct Rust implementation, no parallelism
- **Concurrent (rayon)**: Rust with task-based parallelism via rayon

### Python Wrapper
- **Single-threaded**: Python MARCReader, sequential I/O and parsing
- **Concurrent (ProducerConsumerPipeline)**: Python with producer thread + rayon consumers

## Profile Documents

- `pure_rust_single_thread_profile.md` - Pure Rust single-threaded bottleneck analysis
- `pure_rust_concurrent_profile.md` - Pure Rust concurrent (rayon) bottleneck analysis
- `pymrrc_single_thread_profile.md` - Python wrapper sequential bottleneck analysis
- `pymrrc_concurrent_profile.md` - Python wrapper concurrent bottleneck analysis

## Profiling Methodology

Each profile includes:

1. **Throughput baseline**: Records processed per second under normal operation
2. **Hotspot identification**: Which functions/operations consume the most time?
3. **Resource analysis**: Memory, CPU, thread utilization patterns
4. **Overhead breakdown**: Quantification of major cost sources (I/O, parsing, synchronization, etc.)
5. **Bottleneck identification**: What limits performance in this mode?
6. **Optimization opportunities**: Concrete suggestions for improvement within this mode

## Tools Used

- **Rust**: Criterion.rs, Flamegraph, Perf
- **Python**: cProfile, tracemalloc, time.perf_counter, GC stats
- **General**: System profilers (perf, Instruments), custom timing instrumentation

## Running Profiles

See individual profile documents for scripts and instructions to reproduce profiling results.

## Notes for Future Work

Performance improvements should be guided by these profiles. Before optimizing:
1. Run the relevant profile to identify current bottlenecks
2. Propose improvement and estimate impact
3. Profile again to verify improvement
4. Update baseline metrics in the profile document
