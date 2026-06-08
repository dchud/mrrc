# Profiling Guide

This guide documents how to profile mrrc performance using various tools available in the Rust ecosystem.

## Prerequisites

### cargo-flamegraph

Install flamegraph for CPU profiling:

```bash
cargo install flamegraph
```

**macOS Requirements:**
- Full Xcode (not just Command Line Tools) - required for `xctrace`
- If you only have Command Line Tools, flamegraph will fail to sample

**Linux Requirements:**
- `perf` package (usually included with build tools)
- `perl` for flamegraph script processing

## CPU Profiling with Flamegraph

### Single-threaded Benchmark

Profile the `profiling_harness` benchmark (single-threaded file reading):

```bash
cargo flamegraph --bench profiling_harness -o flamegraphs/profiling_harness.svg
```

This produces an interactive SVG showing:
- Time spent in each function
- Call stack depth
- Percentage of total runtime per function

### Concurrent Benchmark

Profile the `parallel_benchmarks` benchmark (rayon-based concurrent reading):

```bash
cargo flamegraph --bench parallel_benchmarks -o flamegraphs/parallel_benchmarks.svg
```

### Detailed Profiling

Profile the `detailed_profiling` benchmark with memory and phase analysis:

```bash
cargo flamegraph --bench detailed_profiling -o flamegraphs/detailed_profiling.svg
```

### Custom Flamegraph Options

Generate flamegraph with full call stacks and all events:

```bash
cargo flamegraph --bench profiling_harness -- --verbose \
  -o flamegraphs/profiling_harness_verbose.svg
```

Frequency (sampling rate in Hz, default 99):

```bash
cargo flamegraph --bench profiling_harness -- -F 1000 \
  -o flamegraphs/profiling_harness_1khz.svg
```

## Interpreting Flamegraphs

1. **Width** = time spent in function (wider = more time)
2. **Height** = call stack depth (taller = deeper nesting)
3. **Color** = random (for visual distinction)
4. **X-axis** = execution order
5. **Y-axis** = call stack

### Common Patterns to Look For

- **Flat top** = pure computation or tight loop
- **Sawtooth pattern** = recursive calls or repeated function sequences
- **Thin slices** = low-overhead functions called many times
- **Wide base** = time spent in main execution path

## Quick Profiling Checklist

For performance optimization work:

```bash
# Build debug symbols (required for flamegraph on release builds)
# Already configured in Cargo.toml [profile.bench] section

# Profile single-threaded baseline
cargo flamegraph --bench profiling_harness -o /tmp/single.svg

# Profile concurrent implementation
cargo flamegraph --bench parallel_benchmarks -o /tmp/concurrent.svg

# Compare outputs in browser:
# - Wider functions = optimization opportunities
# - Identify bottlenecks not visible in benchmarks
```

## Integration with CI/CD

Flamegraphs can be:
- Committed to git (`flamegraphs/` directory)
- Uploaded to artifact storage
- Compared across releases for regression detection

## Related Tasks

- **mrrc-u33.2.1**: Setup flamegraph and perf profiling tools (this document)
- **mrrc-u33.1**: Review pure Rust file read performance and concurrency optimization
- **mrrc-u33.3**: Profile pure Rust mrrc concurrent performance (rayon)
- **mrrc-u33.4**: Profile Python wrapper (pymrrc) single-threaded performance
- **mrrc-u33.5**: Profile Python wrapper (pymrrc) concurrent performance

## Troubleshooting

### macOS: "tool 'xctrace' requires Xcode"
- Install full Xcode: `xcode-select --install` (installs Command Line Tools)
- Full Xcode required: Download from App Store or developer.apple.com

### Linux: "perf not found"
```bash
sudo apt-get install linux-tools  # Debian/Ubuntu
sudo yum install perf              # RHEL/Fedora
```

### Flamegraph output empty
- Ensure binary is running long enough to collect samples
- Increase bench duration or iterations
- Use `-F` flag to increase sampling frequency

## Future Enhancements

- [ ] Integrate with criterion.rs flamegraph output
- [ ] Automated flamegraph generation in CI
- [ ] Historical comparison tracking
- [ ] Multi-threaded flamegraph analysis tools
