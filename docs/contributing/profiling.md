# Profiling Guide

This guide covers local CPU profiling of mrrc with `cargo flamegraph`. It is
the "where is the time going?" companion to
[Benchmarking](benchmarking.md), which covers "how fast is it?" measurement
with criterion, pytest-benchmark, and CodSpeed.

## Where profiling fits in the perf workflow

The benchmark layers tell you *that* something is slow or regressed:

- **criterion** (`cargo bench`) gives local walltime numbers per scenario.
- **CodSpeed** runs the single-threaded criterion benches in CI under
  simulation mode and flags PR-vs-main drift. Simulation mode serializes
  threads, so rayon speedup is only measurable locally — profile and bench
  `parallel_benchmarks` on your own machine.

A flamegraph tells you *why*: which functions dominate the hot path, and
where allocations or copies hide. Profile locally when a bench number moves
and the cause isn't obvious, or before starting optimization work to find
the real bottleneck.

## Prerequisites

### cargo-flamegraph

Install flamegraph for CPU profiling:

```bash
cargo install flamegraph
```

**macOS requirements:**

- Full Xcode (not just Command Line Tools) — `cargo flamegraph` samples via
  `xctrace`, which ships only with the full Xcode install from the App Store
  or developer.apple.com.

**Linux requirements:**

- `perf` (usually packaged as `linux-tools` or `perf`)
- `perl` for the flamegraph script processing

### Debug symbols

`Cargo.toml` sets `debug = true` in `[profile.bench]`, so bench builds keep
the debuginfo flamegraph needs for readable stacks. No extra flags required.

### Fixtures

The benches read `tests/data/fixtures/` (1k / 5k / 10k record files, checked
in). See [Benchmarking](benchmarking.md) for regenerating the larger
gitignored fixtures.

## CPU profiling with flamegraph

All bench targets are criterion harnesses (`harness = false`). When invoked
through `cargo flamegraph`, pass `--bench` after `--` so the criterion
binary runs in benchmark mode, and `--profile-time <secs>` so it loops the
benchmark for a fixed duration instead of spending samples on criterion's
statistical analysis. A trailing filter narrows to one scenario.

### Single-threaded read path

`marc_benchmarks` covers the core ISO 2709 read, field-access, and
JSON/XML serialization paths:

```bash
cargo flamegraph --bench marc_benchmarks -o flamegraphs/read_1k.svg \
  -- --bench --profile-time 10 read_1k_records
```

Other useful scenario filters: `read_10k_records`, `read_1k_records_from_path`,
`read_1k_with_field_access`, `serialize_1k_to_json`, `serialize_1k_to_xml`,
`roundtrip_1k_records`.

### Concurrent read path

`parallel_benchmarks` covers rayon-based concurrent reading against
sequential baselines:

```bash
cargo flamegraph --bench parallel_benchmarks -o flamegraphs/parallel_4x.svg \
  -- --bench --profile-time 10 parallel_4x_1k_records
```

### Format conversion paths

`format_benchmarks` covers CSV, MARC-in-JSON, MODS, Dublin Core, and
BIBFRAME serialization plus the parser-pool path:

```bash
cargo flamegraph --bench format_benchmarks -o flamegraphs/formats.svg \
  -- --bench --profile-time 10
```

### Error-handling overhead

`error_handling_benchmarks` compares strict and lenient parsing of clean
versus malformed input:

```bash
cargo flamegraph --bench error_handling_benchmarks -o flamegraphs/errors.svg \
  -- --bench --profile-time 10
```

### Sampling frequency

The default sampling rate is 99 Hz. For short hot paths, raise it with
flamegraph's `--freq` flag (before the `--`):

```bash
cargo flamegraph --freq 1000 --bench marc_benchmarks \
  -o flamegraphs/read_1k_1khz.svg -- --bench --profile-time 10 read_1k_records
```

## Interpreting flamegraphs

1. **Width** = time spent in function (wider = more time)
2. **Height** = call stack depth (taller = deeper nesting)
3. **Color** = random (for visual distinction)
4. **X-axis** = alphabetical merge order, not time
5. **Y-axis** = call stack

### Common patterns to look for

- **Flat top** = pure computation or tight loop
- **Sawtooth pattern** = recursive calls or repeated function sequences
- **Thin slices** = low-overhead functions called many times
- **Wide base** = time spent in main execution path

## Optimization checklist

Hard-won rules from past mrrc profiling work:

1. **Measure before optimizing.** Architectural changes that aren't on the
   critical path produce zero improvement; a flamegraph confirms the change
   targets where time actually goes.
2. **Allocations on the hot path are the usual culprit.** The biggest past
   wins came from eliminating per-record `String` and `Vec` allocations in
   frequently-called parse functions, not from clever restructuring.
3. **Find *where* allocations happen, not just how many.** The call stack
   matters: one allocation in a per-record loop outweighs many in setup code.
4. **Re-bench after every change.** Small incremental optimizations with
   direct criterion measurement beat large speculative refactors. Compare
   `cargo bench` output before and after each commit.

## Troubleshooting

### macOS: "tool 'xctrace' requires Xcode"

Command Line Tools alone are not enough — install the full Xcode from the
App Store or developer.apple.com, then point the tools at it:

```bash
sudo xcode-select -s /Applications/Xcode.app
```

### Linux: "perf not found"

```bash
sudo apt-get install linux-tools-common linux-tools-generic  # Debian/Ubuntu
sudo dnf install perf                                        # Fedora/RHEL
```

### Flamegraph output empty or sparse

- Ensure the run is long enough to collect samples — raise `--profile-time`
- Increase sampling frequency with `--freq`
- Confirm the scenario filter matches a bench name (no match = instant exit)
