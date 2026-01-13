# ISO 2709 Baseline Performance (Rust mrrc)

Baseline performance measurements for ISO 2709 (MARC binary format) using the Rust mrrc library.
This baseline is frozen and used as reference for all subsequent format evaluations.

## Test Dataset

- **File:** `tests/data/fixtures/10k_records.mrc`
- **Records:** 10,000
- **Size:** 2.52 MB (uncompressed)

## System Environment

| Property | Value |
|----------|-------|
| **OS** | Darwin (macOS) 24.6.0 |
| **CPU** | Apple M4 (10-core) |
| **RAM** | 24.0 GB |
| **Architecture** | arm64 |
| **Storage** | SSD |
| **Rust version** | (see output from `rustc --version`) |
| **mrrc version** | Commit: 82efc114 |
| **Optimization** | `cargo build --release` (default -O optimizations) |

## Performance Metrics

| Metric | Value |
|--------|-------|
| **Read throughput** | 73808 records/sec |
| **Write throughput** | 146891 records/sec |
| **File size (raw)** | 2.52 MB |
| **File size (gzipped)** | 83.29 KB |
| **Compression ratio** | 96.8% |
| **Gzip time** | 0.01s |

## Measurement Methodology

**Rust mrrc measurements:**
- Read throughput: `cargo bench --bench marc_benchmarks -- read_10k --baseline iso2709`
- Write throughput: `cargo bench --bench marc_benchmarks -- write_10k --baseline iso2709`
- Single-threaded measurements (no parallelization)
- Release build with default Rust optimizations
- Averaged over multiple runs to account for variance

**Compression measurement:**
- Raw file: `wc -c tests/data/fixtures/10k_records.mrc`
- Gzipped: `gzip -9 < tests/data/fixtures/10k_records.mrc | wc -c`

## Interpretation

- **Read throughput:** Records deserialized per second (mrrc → internal representation)
- **Write throughput:** Records serialized per second (internal representation → ISO 2709)
- **Compression ratio:** Higher percentage = more compressible (better for storage)
- All other format evaluations will be measured using the same environment and compared against these metrics

## Date & Reference

- **Measured:** 2026-01-13
- **Commit:** 82efc114 (binary format evaluation infrastructure committed)
- **Script:** `scripts/measure_iso2709_baseline.py` (Python pymarc used for initial baseline)

---

## Important Notes

This baseline is **FROZEN** and used as the permanent reference for all subsequent format evaluations.

**Constraints:**
- Retroactive adjustments to this baseline are NOT permitted (prevents cherry-picking results)
- All format evaluations MUST measure against this baseline using the same environment
- If environment changes significantly (major Rust version, CPU), create a NEW baseline document rather than modifying this one
- Format recommendations are only valid when compared against THIS baseline
