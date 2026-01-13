# Binary Format Research for MARC Data

This directory contains research, evaluation, and documentation for the **mrrc-fks** epic: evaluating binary data formats for MARC representation and import/export.

## Purpose

Assess modern binary serialization formats as alternatives or complements to existing MARC formats (ISO 2709, JSON, XML, CSV), focusing on three key evaluation dimensions:

- **Round-trip fidelity** — 100% data preservation **including exact field and subfield ordering** (evaluated first)
- **Robustness** — Graceful error handling, no panics (evaluated before performance)
- **Performance** — Read/write throughput, file size, memory efficiency (evaluated last, only if fidelity+robustness pass)
- **Ecosystem fit** — Dependencies, language support, schema evolution

**Critical constraint:** Formats that reorder fields by tag number or reorder subfield codes are automatically rejected (reordering = data loss for semantics-preserving applications).

## Directory Structure

```
format-research/
├── README.md                    # This file
├── EVALUATION_FRAMEWORK.md      # Standardized methodology (mrrc-fks.8)
├── TEMPLATE_evaluation.md       # Blank template for format reports
├── COMPARISON_MATRIX.md         # Aggregated results (mrrc-fks.9)
│
├── protobuf/                    # Protocol Buffers (mrrc-fks.1)
│   ├── evaluation.md
│   └── marc.proto
│
├── flatbuffers/                 # FlatBuffers (mrrc-fks.2)
│   ├── evaluation.md
│   └── marc.fbs
│
├── parquet/                     # Apache Parquet (mrrc-fks.3)
│   └── evaluation.md
│
├── avro/                        # Apache Avro (mrrc-fks.4)
│   ├── evaluation.md
│   └── marc.avsc
│
├── messagepack/                 # MessagePack (mrrc-fks.5)
│   └── evaluation.md
│
├── cbor/                        # CBOR RFC 7049 (mrrc-fks.6)
│   └── evaluation.md
│
├── arrow/                       # Apache Arrow (mrrc-fks.7)
│   └── evaluation.md
│
└── polars-duckdb/               # Polars + Arrow + DuckDB (mrrc-fks.10)
    └── evaluation.md
```

## Formats Under Evaluation

| Format | Issue | Status | Primary Use Case |
|--------|-------|--------|------------------|
| Protocol Buffers | mrrc-fks.1 | Pending | API integration, compact serialization |
| FlatBuffers | mrrc-fks.2 | Pending | Zero-copy access, embedded systems |
| Apache Parquet | mrrc-fks.3 | Pending | Analytics, columnar storage |
| Apache Avro | mrrc-fks.4 | Pending | Schema evolution, streaming |
| MessagePack | mrrc-fks.5 | Pending | Simple exchange, cache serialization |
| CBOR | mrrc-fks.6 | Pending | Standards compliance, IoT |
| Apache Arrow | mrrc-fks.7 | Pending | In-memory analytics, interchange |
| Polars + DuckDB | mrrc-fks.10 | Pending | SQL analytics, DataFrames |

## Quick Start for Evaluators

1. **Read the framework** — [EVALUATION_FRAMEWORK.md](./EVALUATION_FRAMEWORK.md) for methodology and correctness rules
2. **Understand the test data** — [FIDELITY_TEST_SET.md](./FIDELITY_TEST_SET.md) describes what you'll be testing
3. **Copy the template** — [TEMPLATE_evaluation.md](./TEMPLATE_evaluation.md) is your evaluation report structure
4. **Follow the three-layer evaluation:**
   - Layer 1: Schema design + round-trip fidelity testing
   - Layer 2: Failure modes testing (error handling)
   - Layer 3: Performance benchmarks (only if layers 1+2 pass)

## Related Issues

- **Epic:** mrrc-fks — Evaluate binary data formats for MARC
- **Framework:** mrrc-fks.8 — Define evaluation framework and methodology
- **Synthesis:** mrrc-fks.9 — Aggregate into comparison matrix

## Test Data

| File | Records | Purpose | Status |
|------|---------|---------|--------|
| `tests/data/fixtures/fidelity_test_100.mrc` | 100 | Round-trip validation (required for all evaluations) | **TODO:** Create per [FIDELITY_TEST_SET.md](./FIDELITY_TEST_SET.md) |
| `tests/data/fixtures/10k_records.mrc` | 10,000 | Performance benchmarks | Available |
| `tests/data/fixtures/100k_records.mrc` | 100,000 | Stress testing | Available |

**Note:** The fidelity test set must be created and validated before any format evaluations can begin.
