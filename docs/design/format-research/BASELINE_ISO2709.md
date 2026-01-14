# ISO 2709 Baseline Performance (Rust mrrc)

Baseline performance measurements for ISO 2709 (MARC binary format) using the **Rust mrrc library** (primary).
This baseline is frozen and used as the permanent reference for all subsequent format evaluations.

## Test Dataset

- **File:** `tests/data/fixtures/10k_records.mrc`
- **Records:** 10,000 MARC bibliographic records
- **File size (raw):** 2,645,353 bytes (2.52 MB)
- **File size (gzip -9):** 90,364 bytes (88 KB)

## System Environment

| Property | Value |
|----------|-------|
| **OS** | Darwin (macOS) 14.6.0 |
| **CPU** | Apple M4 (10-core: 2P + 8E) |
| **RAM** | 24.0 GB |
| **Architecture** | arm64 |
| **Storage** | SSD (Apple) |
| **Rust version** | 1.92.0 (ded5c06cf 2025-12-08, Homebrew) |
| **Cargo version** | 1.92.0 (Homebrew) |
| **Build type** | `cargo build --release` |
| **Rust optimization** | Default (-C opt-level=3) |
| **mrrc commit** | 82efc114 (binary format evaluation infrastructure) |

## Rust Baseline Performance Metrics (PRIMARY)

All evaluations compare against these **Rust mrrc** benchmarks using **Criterion.rs**:

| Metric | Value | Notes |
|--------|-------|-------|
| **Read throughput (ISO 2709)** | **903,560 rec/sec** | 10k records in 11.06 ms |
| **Roundtrip throughput** | **421,556 rec/sec** | Read + write combined (23.72 ms) |
| **File size (raw)** | **2,645,353 bytes** | Original ISO 2709 |
| **File size (gzip -9)** | **85,288 bytes** | Compressed with gzip -9 |
| **Compression ratio** | **96.77%** | (1 - 85288/2645353) × 100% |

### Derivation of Write Throughput

The roundtrip benchmark (read + write) achieved **421,556 rec/sec** for 10,000 records in 23.72 ms.
Since read throughput is ~903K rec/sec, this implies:
- Read: ~11.06 ms for 10k records
- Write: ~12.66 ms for 10k records
- **Estimated write throughput: ~789,405 rec/sec** (conservative)

## Measurement Methodology (Rust Primary)

### Read Benchmark
```bash
cargo bench --bench marc_benchmarks -- read_10k_records
```
- **Test:** Deserialize ISO 2709 bytes → MarcRecord objects
- **Procedure:** MarcReader processes fixture, returns record count
- **Samples:** Multiple iterations with warmup
- **Result:** 11.0576 ms average per 10k records

### Roundtrip Benchmark
```bash
cargo bench --bench marc_benchmarks -- roundtrip_10k_records
```
- **Test:** Read ISO 2709 → MarcRecord objects → Write back to ISO 2709
- **Procedure:** MarcReader deserializes, MarcWriter re-serializes
- **Samples:** Multiple iterations with warmup
- **Result:** 23.7196 ms average per 10k records

### Compression Measurement
```bash
# Raw size
wc -c tests/data/fixtures/10k_records.mrc

# Gzip compression
gzip -9 < tests/data/fixtures/10k_records.mrc | wc -c
```
- **Raw:** 2,645,353 bytes
- **Gzipped:** 85,288 bytes
- **Ratio:** 96.77% compression (97% less space after gzip)

## Reference: Python Wrapper Baseline (SECONDARY)

Python mrrc wrapper (used for early infrastructure validation only):
- Read: 88,455 rec/sec
- Write: 189,386 rec/sec
- Compression: 96.77%

**Note:** Python metrics are for reference and infrastructure validation only.
**All format evaluations must compare Rust mrrc performance against the Rust baseline above.**

## Interpretation of Metrics

### Read Throughput (903,560 rec/sec)
- **Definition:** Records deserialized from ISO 2709 bytes per second
- **Measurement:** MarcReader.read_record() on streaming input
- **Significance:** Maximum speed mrrc can parse binary MARC data
- **Comparison basis:** All candidate formats evaluated on same 10k_records.mrc file

### Write Throughput (~789,405 rec/sec estimated)
- **Definition:** MarcRecord objects serialized to ISO 2709 bytes per second
- **Measurement:** MarcWriter.write_record() for each record
- **Significance:** Maximum speed mrrc can generate binary MARC output
- **Comparison basis:** Candidate formats measured on internal MarcRecord representation

### Compression Ratio (96.77%)
- **Definition:** Percentage reduction in size after gzip -9
- **Formula:** `(1 - gzip_size / raw_size) × 100%`
- **Interpretation:** ISO 2709 is highly compressible; formats achieving >95% are roughly equivalent
- **Significance:** Most binary MARC formats compress similarly due to repetitive structure

## Freeze Declaration

This baseline is **FROZEN** and represents the permanent reference point.

### Constraints on Future Changes

**PROHIBITED:**
- ❌ Retroactive adjustments to these metrics
- ❌ Re-running benchmarks with different compiler flags and updating this document
- ❌ "Optimizing" the baseline upward to improve format comparisons
- ❌ Changing test dataset after baseline is established

**REQUIRED for Environment Changes:**
- ✅ If Rust version changes significantly (e.g., 1.92 → 2.0), create a **NEW** baseline document (BASELINE_ISO2709_RUST_2_0.md)
- ✅ If CPU/system changes (e.g., different hardware), create a **NEW** baseline document with environment prefix
- ✅ If test data changes (10k_records.mrc modified), create a **NEW** baseline document with version suffix

### Why Frozen?

1. **Prevents cherry-picking:** Guarantees all formats evaluated against same reference
2. **Prevents regression creep:** Ensures baseline isn't "improved" to make poor formats look better
3. **Enables reproducibility:** Future evaluations can reference this exact baseline
4. **Supports recommendations:** All verdicts are tied to this specific environment and metrics

## Date & Version Control

- **Baseline updated:** 2026-01-14
- **Rust version:** 1.92.0 (2025-12-08)
- **Criterion.rs version:** As specified in Cargo.lock (see `cargo tree`)
- **Benchmark script:** `benches/marc_benchmarks.rs`

## References

- **Benchmark code:** [benches/marc_benchmarks.rs](../../../benches/marc_benchmarks.rs)
- **Test dataset:** [tests/data/fixtures/10k_records.mrc](../../../tests/data/fixtures/10k_records.mrc)
- **Evaluation framework:** [EVALUATION_FRAMEWORK.md](./EVALUATION_FRAMEWORK.md)
- **Evaluation template:** [TEMPLATE_evaluation.md](./TEMPLATE_evaluation.md)

---

## Appendix: How to Reproduce This Baseline

To verify these metrics on your own system:

```bash
# 1. Check Rust version
rustc --version
cargo --version

# 2. Run read benchmark
cargo bench --bench marc_benchmarks -- read_10k_records

# 3. Run roundtrip benchmark
cargo bench --bench marc_benchmarks -- roundtrip_10k_records

# 4. Measure file sizes
wc -c tests/data/fixtures/10k_records.mrc
gzip -9 < tests/data/fixtures/10k_records.mrc | wc -c

# 5. Calculate compression ratio
# compression_ratio = (1 - gzipped_size / raw_size) * 100
```

**Note:** Results may vary by ±5% due to system load and CPU thermal throttling.
On Apple Silicon (M1/M2/M3/M4), consistent performance is expected within these bounds.
