# Binary Format Research for MARC Data

This directory contains research, evaluation, and documentation for the **mrrc-fks** epic: evaluating binary data formats for MARC representation and import/export.

## Purpose

Assess modern binary serialization formats as alternatives or complements to existing MARC formats (ISO 2709, JSON, XML, CSV), focusing on:

- **Round-trip fidelity** — 100% data preservation required
- **Performance** — Read/write throughput, file size, memory efficiency
- **Ecosystem fit** — Dependencies, language support, schema evolution

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

## Quick Start

1. **Read the framework** — [EVALUATION_FRAMEWORK.md](./EVALUATION_FRAMEWORK.md)
2. **Copy the template** — [TEMPLATE_evaluation.md](./TEMPLATE_evaluation.md)
3. **Follow the protocol** — Ensure results are comparable across formats

## Related Issues

- **Epic:** mrrc-fks — Evaluate binary data formats for MARC
- **Framework:** mrrc-fks.8 — Define evaluation framework and methodology
- **Synthesis:** mrrc-fks.9 — Aggregate into comparison matrix

## Test Data

| File | Records | Purpose |
|------|---------|---------|
| `tests/data/fixtures/fidelity_test_100.mrc` | 100 | Round-trip validation |
| `tests/data/fixtures/10k_records.mrc` | 10,000 | Performance benchmarks |
| `tests/data/fixtures/100k_records.mrc` | 100,000 | Stress testing |
