# ISO 2709 Baseline Performance

Baseline performance measurements for ISO 2709 (MARC binary format) on the mrrc library.

## Test Dataset

- **File:** `tests/data/fixtures/10k_records.mrc`
- **Records:** 10,000
- **Size:** 2.52 MB (uncompressed)

## System Environment

| Property | Value |
|----------|-------|
| **OS** | Darwin 24.6.0 |
| **CPU** | Apple M4 |
| **Cores** | 10 |
| **RAM** | 24.0 GB |
| **Architecture** | arm64 |
| **Storage** | Unknown |
| **Python** | 3.12.8 |

## Performance Metrics

| Metric | Value |
|--------|-------|
| **Read throughput** | 73808 records/sec |
| **Write throughput** | 146891 records/sec |
| **File size (raw)** | 2.52 MB |
| **File size (gzipped)** | 83.29 KB |
| **Compression ratio** | 96.8% |
| **Gzip time** | 0.01s |

## Interpretation

- **Read throughput:** Lower is slower; higher is faster
- **Write throughput:** Lower is slower; higher is faster
- **Compression ratio:** Higher percentage = more compressible (better for storage)
- All other format evaluations will be compared against these metrics

## Date & Commit

- **Measured:** 2026-01-13 12:20:45
- **Commit:** 141a0f1

---

This baseline is frozen and used as the reference for all subsequent format evaluations.
Retroactive adjustments to this baseline are not permitted (to prevent cherry-picking results).
