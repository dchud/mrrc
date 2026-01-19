# Binary Format Comparison Matrix

**Updated:** 2026-01-19  
**Status:** 8/10 formats complete; 7 recommended  
**Framework:** See [EVALUATION_FRAMEWORK.md](./EVALUATION_FRAMEWORK.md)

This document aggregates results from all format evaluations for side-by-side comparison.

---

## Executive Summary

| Dimension | Winner | Notes |
|-----------|--------|-------|
| **Fidelity** | Protobuf, FlatBuffers, ISO 2709, Arrow Analytics | All achieve 100% perfect round-trip |
| **Read Speed (General)** | ISO 2709 | 3.5-9x faster than alternatives |
| **Read Speed (Analytics)** | Arrow Analytics (Rust) | 1.77M rec/sec; 1.96× faster than ISO 2709 baseline |
| **Compression** | FlatBuffers, Arrow Analytics (Parquet) | FlatBuffers 98.08%; Arrow Parquet 30% smaller than ISO 2709 |
| **Memory Efficiency (API)** | FlatBuffers | 64% lower peak memory than baseline |
| **Memory Efficiency (Analytics)** | Arrow Analytics | Columnar zero-copy; sub-microsecond column access |
| **API Maturity** | Protobuf | Widest ecosystem and language support |
| **Zero-Copy** | FlatBuffers, Arrow Analytics | FlatBuffers for serialization; Arrow for in-memory analysis |
| **Schema Evolution** | Protobuf, Avro | Full bi-directional compatibility |
| **Analytics Performance** | Arrow Analytics (Rust-native) | 1.77M rec/sec; optimal for MARC discovery optimization |

---

## Comprehensive Comparison Matrix

### Performance Metrics

| Format | Fidelity | Read (rec/sec) | Write (rec/sec) | Roundtrip (rec/sec) | vs ISO 2709 Read | vs ISO 2709 Write | vs ISO 2709 Roundtrip |
|--------|----------|----------------|-----------------|--------------------|-----------------|--------------------|----------------------|
| **ISO 2709** | ✅ 100% | **903,560** | **~789,405** | **421,556** | Baseline | Baseline | Baseline |
| **Protobuf** | ✅ 100% | 100,000 | 100,000 | ~90,000 | -88.9% | -87.3% | -78.6% |
| **FlatBuffers** | ✅ 100% | 259,240 | 108,932 | 69,767 | -71.3% | -86.2% | -83.5% |
| **Parquet** | ✅ 100% | 729,002 | 767,889 | 430,000 | -19.3% | -2.7% | +2.0% |
| **Arrow** | ✅ 100% | 865,331 | 712,407 | 405,000 | -4.2% | -9.8% | -4.0% |
| **MessagePack** | ✅ 100% | 750,434 | 746,410 | 415,000 | -17.0% | -5.4% | -1.5% |
| **CBOR** | ✅ 100% | 496,186 | 615,571 | 338,000 | -45.1% | -21.9% | -19.8% |
| **Avro** | ✅ 100% | 215,887 | 338,228 | 184,000 | -76.1% | -57.1% | -56.4% |

### File Size Metrics

| Format | Raw Size | vs ISO 2709 (raw) | Gzip Size | vs ISO 2709 (gzip) | Compression Ratio |
|--------|----------|-------------------|-----------|-------------------|-------------------|
| **ISO 2709** | 2,645,353 | Baseline | 85,288 | Baseline | 96.77% |
| **Protobuf** | 7,500,000–8,500,000 | +184%–222% | 1,200,000–1,500,000 | +1,306%–1,656% | 84–85% |
| **FlatBuffers** | 6,703,891 | +153% | 129,045 | +51% | 98.08% |
| **Parquet** | 4,609,288 | +74% | ~80,000 | ~–6% | 98.3% (Snappy) |
| **Arrow** | 1,847,294 | –30% | 74,156 | –13% | 95.99% |
| **MessagePack** | 1,993,352 | –25% | 83,747 | –2% | 95.80% |
| **CBOR** | 4,800,701 | +82% | 100,090 | +17% | 97.60% |
| **Avro** | 6,291,376 | +138% | 106,314 | +25% | 98.31% |

### Memory Efficiency

| Format | Peak Memory | vs Baseline | Notes |
|--------|-------------|------------|-------|
| **Baseline ISO 2709** | ~45 MB | Baseline | Reference point |
| **Protobuf** | ~45–50 MB | +0–11% | Schema serialization overhead |
| **FlatBuffers** | ~16 MB | –64% | Streaming model; zero-copy capable |
| **Parquet** | ~30–35 MB | –22–28% | Arrow columnar; efficient denormalization |
| **Arrow** | ~30–35 MB | –22–28% | In-memory columnar; efficient representation |
| **MessagePack** | ~40–45 MB | –0–11% | Streaming model; serde overhead comparable to baseline |
| **CBOR** | ~40–45 MB | –0–11% | Serde overhead; comparable to MessagePack |
| **Avro** | ~45–50 MB | +0–11% | JSON serialization adds overhead |

---

## Integration & Ecosystem Assessment

### Rust Dependencies

| Format | Direct Deps | Transitive Deps | Maturity | Maintenance | Notes |
|--------|-------------|-----------------|----------|-------------|-------|
| **ISO 2709** | 0 | 0 | ⭐⭐⭐⭐⭐ | mrrc native | Built-in; no external deps |
| **Protobuf** | 2 | Low | ⭐⭐⭐⭐⭐ | Active | Google-backed; widely adopted |
| **FlatBuffers** | 2 | Low | ⭐⭐⭐⭐⭐ | Active | Google-backed; production-ready |
| **Parquet** | 0 | 0 | ⭐⭐⭐ | mrrc custom | Arrow-based impl; no new deps |
| **Arrow** | 1 | Low | ⭐⭐⭐⭐⭐ | Active | Apache-backed; production-ready |
| **MessagePack** | 2 | Low | ⭐⭐⭐⭐⭐ | Active | Schema-less; stable ecosystem |
| **CBOR** | 2 | Low | ⭐⭐⭐⭐ | Active | RFC 7949 standard; proven |
| **Avro** | 1 | Low | ⭐⭐⭐⭐⭐ | Active | Apache-backed; schema evolution |

### Language Support

| Format | Rust | Python | Java | Go | C++ | Notes |
|--------|------|--------|------|----|----|-------|
| **ISO 2709** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐ | mrrc native; others custom |
| **Protobuf** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Official support all languages |
| **FlatBuffers** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Official support; good ecosystem |
| **Parquet** | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Arrow-based impl |
| **Arrow** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Apache-backed; excellent support |
| **MessagePack** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Universal 50+ language support |
| **CBOR** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | RFC 7949 standard; excellent support |
| **Avro** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Apache-backed; excellent ecosystem |

### Schema Evolution Capability

| Format | Score (1–5) | Forward Compat | Backward Compat | Append-Only | Notes |
|--------|-------------|---|---|---|-------|
| **ISO 2709** | 1 | ❌ | ❌ | ❌ | Fixed binary format; no evolution |
| **Protobuf** | 5 | ✅ | ✅ | ✅ | Bi-directional; full compatibility |
| **FlatBuffers** | 4 | ⚠️ | ✅ | ✅ | Append-only evolution constraint |
| **Parquet** | 2 | ⚠️ | ⚠️ | ❌ | Limited field evolution |
| **Arrow** | 4 | ✅ | ✅ | ✅ | Flexible schema; column addition/renaming |
| **MessagePack** | 2 | ⚠️ | ✅ | ✅ | Schema-less; new optional fields compatible |
| **CBOR** | 3 | ✅ | ✅ | ✅ | Semantic tags enable version metadata |
| **Avro** | 5 | ✅ | ✅ | ✅ | Best-in-class schema evolution |

---

## Error Handling & Robustness

| Format | Graceful Errors | No Panics | Silent Corruption | Notes |
|--------|-----------------|-----------|-------------------|-------|
| **Protobuf** | ✅ | ✅ | ✅ None | Comprehensive error handling |
| **FlatBuffers** | ✅ | ✅ | ✅ None | Comprehensive error handling |
| **Parquet** | ✅ | ✅ | ✅ None | Arrow serialization is robust |
| **Arrow** | ✅ | ✅ | ✅ None | Arrow validation is thorough |
| **MessagePack** | ✅ | ✅ | ✅ None | rmp-serde validates all invalid input |
| **CBOR** | ✅ | ✅ | ✅ None | ciborium validates CBOR strictly |
| **Avro** | ✅ | ✅ | ✅ None | Schema validation + field constraints |

---

## Use Case Fit Scoring (1-5 scale)

### Binary Interchange (API/gRPC)

| Format | Score | Notes |
|--------|-------|-------|
| **Protobuf** | ⭐⭐⭐⭐⭐ | Native gRPC support; schema contracts; cross-language |
| **FlatBuffers** | ⭐⭐⭐⭐ | Fast deserialization; zero-copy potential; good for REST |
| **MessagePack** | ⭐⭐⭐ | Simple; compact; limited schema validation |
| ISO 2709 | ⭐⭐ | Legacy standard; not designed for modern APIs |

### High-Performance Batch Processing

| Format | Score | Notes |
|--------|-------|-------|
| **ISO 2709** | ⭐⭐⭐⭐⭐ | 900k+ rec/sec; highly optimized |
| **Arrow** | ⭐⭐⭐⭐ | 865k rec/sec; negligible overhead vs ISO 2709 |
| **MessagePack** | ⭐⭐⭐ | Compact; reasonable throughput |
| **Parquet (Arrow)** | ⭐⭐⭐ | 729k rec/sec; columnar but denormalized design |
| **Protobuf** | ⭐⭐ | 100k rec/sec; acceptable but not ideal |
| **FlatBuffers** | ⭐⭐ | 259k rec/sec; memory efficient but slow |

### Analytics & Big Data (Spark/Hadoop)

| Format | Score | Notes |
|--------|-------|-------|
| **Parquet** | ⭐⭐⭐⭐⭐ | Purpose-built for columnar analytics |
| **Arrow** | ⭐⭐⭐⭐⭐ | In-memory columnar; zero-copy; GPU support |
| **Avro** | ⭐⭐⭐ | Hadoop ecosystem integration; schema registry |
| All binary row formats | ⭐ | Not suitable for columnar analytics |

### Memory-Constrained Environments

| Format | Score | Notes |
|--------|-------|-------|
| **FlatBuffers** | ⭐⭐⭐⭐⭐ | 64% lower memory; zero-copy; perfect for mobile |
| **Arrow** | ⭐⭐⭐⭐ | In-memory columnar; efficient representation |
| **MessagePack** | ⭐⭐⭐⭐ | Very compact serialization |
| **Protobuf** | ⭐⭐⭐ | Reasonable; not specifically optimized |
| ISO 2709 | ⭐⭐ | No built-in memory optimization |

### Long-Term Archival (10+ years)

| Format | Score | Notes |
|--------|-------|-------|
| **ISO 2709** | ⭐⭐⭐⭐ | Proven 50+ year track record; stable standard |
| **Protobuf** | ⭐⭐⭐⭐ | Schema versioning; forward/backward compatible |
| **Avro** | ⭐⭐⭐⭐ | Self-describing; schema registry support |
| **Arrow** | ⭐⭐⭐ | Ecosystem growing; 30% file size advantage valuable |
| **FlatBuffers** | ⭐⭐⭐ | Append-only evolution; less flexible |
| **Parquet** | ⭐⭐ | File size overhead (3.8×) not justified |
| **CBOR** | ⭐⭐⭐ | Standards-based; less mature than others |

---

## Analytics Tier: Arrow (Rust-Native) as Specialized MARC Workload

### Purpose & Architecture

Arrow Analytics is **NOT a replacement for general binary formats**, but a **specialized columnar analytics tier** for MARC data. It complements ISO 2709 by providing:

- **Column-oriented analysis** (field frequency, subject distribution, cataloging metrics)
- **SQL-based discovery optimization** via Arrow IPC format integration with DuckDB, Polars
- **Zero-copy in-memory operations** for bulk analytical queries
- **Parquet archival** for long-term analytical snapshots

### Performance & Fidelity

| Metric | Value | Notes |
|--------|-------|-------|
| **Throughput (Rust)** | 1.77M rec/sec | Deserialization speed (5.7 ms for 10k records); **1.96× faster than ISO 2709 baseline** |
| **Round-Trip Fidelity** | 100% | All 100 test records achieve perfect preservation (field/subfield ordering, indicators, UTF-8 content) |
| **Column Access Latency** | Sub-microsecond | Per-column iteration via Arrow native arrays (zero-copy) |
| **In-Memory Size (10k records)** | ~5.8 MB | Columnar normalization to long format (62,667 rows); overhead vs 2.6 MB ISO 2709 |
| **Parquet Compression** | 1.8 MB | **30% smaller than ISO 2709** (2.6 MB raw); recommended for analytical archives |
| **Subject Analysis (650 fields)** | 45 ms | SQL GROUP BY across 10k records via DuckDB |
| **Authority Reconciliation** | 18 ms | Efficient joins on 700/710/711 name fields |
| **Date Range Filtering (008)** | 22 ms | Field-level extraction and filtering |

### Use Cases & Scoring

| Use Case | Score | When to Use |
|----------|-------|------------|
| **Subject/authority frequency analysis** | ⭐⭐⭐⭐⭐ | SQL GROUP BY queries on repeating fields; fast aggregation |
| **Discovery index optimization** | ⭐⭐⭐⭐⭐ | Field presence/absence analysis, cataloging workflows |
| **Bulk field transformation** | ⭐⭐⭐ | Polars good for column-level ops; less ideal for complex record restructuring |
| **Interactive single-record access** | ⭐ | 80 ms latency unacceptable for APIs; use ISO 2709 cache instead |
| **Streaming large files** | ⭐ | Requires full materialization; ISO 2709 streaming more efficient |

### Integration Approach

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| **Rust Core** | arrow-rs crate (v57.0) | Native Arrow implementation; zero unsafe code; 1.77M rec/sec |
| **Persistence** | Parquet format | Long-term analytical archives; 30% compression vs ISO 2709 |
| **Interoperability** | Arrow IPC format | Readable by DuckDB, Polars, DataFusion (user responsibility for external queries) |
| **SQL Analytics** | DuckDB (optional) | External tool; read Arrow IPC format for SQL-based MARC analysis |
| **Python Analytics** | Polars + PyArrow (optional) | Via PyO3 FFI; data scientists can perform additional analysis on Arrow IPC exports |

### Key Differences from General Formats

| Aspect | General Arrow | Analytics Arrow |
|--------|---------------|-----------------|
| **Purpose** | API serialization, ecosystem interchange | MARC-specific column analysis |
| **Schema** | Flexible application schema | Normalized long format (record_id, field_tag, indicators, subfield_code, subfield_value) |
| **Serialization** | Standard Arrow IPC/columnar | Rust-native builders; 100% MARC fidelity preservation |
| **Deployment** | External tools manage data flow | Integrated into mrrc library; no external dependencies |
| **Performance Target** | Balanced read/write | Optimized for analytical throughput (1.77M rec/sec) |
| **Persistence** | Flexible (Parquet, Arrow IPC) | Parquet for archival; Arrow IPC for external tool integration |

### Recommendation Context

✅ **RECOMMENDED** for organizations doing heavy MARC analytics (discovery optimization, cataloging metrics, authority reconciliation).

⚠️ **NOT RECOMMENDED** as primary MARC import/export format. Continue using ISO 2709 for general purposes.

**Implementation tier:** Medium priority (after core MARC format support is stable). Add as opt-in feature in mrrc library (Phase 2+).

---

## Implementation Complexity

| Format | LOC (Rust) | Dev Time | Maintenance | Build Impact |
|--------|-----------|----------|-------------|--------------|
| **FlatBuffers** | 214 | 1-2 hrs | Low (minimal schema) | <5s full build |
| **Protobuf** | 415 | 2-3 hrs | Low (code-gen) | +5.88s full build |
| **Parquet** | 290 | ~2 hrs | Low (JSON serialization) | Negligible (<1s) |
| **Arrow** | 410 | ~3 hrs | Low (Arrow library) | +2-3s full build |
| **MessagePack** | ~150 | 1-2 days | Very Low (serde handles) | +1s incremental |
| **CBOR** | ~150 | 1-2 days | Very Low (ciborium handles) | +1s incremental |
| **Avro** | ~300 | 2-3 hrs | Low (schema management) | +2-3s incremental |

---

## Summary Recommendations

### ✅ RECOMMENDED (General-Purpose Formats)

| Format | Primary Use Cases | Rationale |
|--------|-------------------|-----------|
| **Protobuf** | API serialization, REST/gRPC, cross-language interchange | Mature ecosystem, schema evolution, excellent Rust support |
| **FlatBuffers** | Streaming APIs, memory-constrained environments, zero-copy scenarios | Memory efficient, fast deserialization, Apple production use |
| **Arrow (Columnar)** | In-memory analytics, ecosystem integration (Polars/DuckDB), analytics interchange | Industry-standard, 30% file size advantage, minimal performance overhead, excellent tooling |
| **ISO 2709** | High-throughput batch processing, archival baseline | Proven standard, maximum performance, no dependencies |
| **MessagePack** | Compact storage, inter-process communication, REST payloads | 25% file size reduction, 750K rec/sec throughput, universal language support |
| **CBOR** | Standards-based archival, government/academic systems, preservation institutions | RFC 7949 standard, diagnostic notation, semantic tagging, 45% size reduction after gzip |
| **Avro** | Event streaming (Kafka), data lake integration, multi-language systems | Best-in-class schema evolution, self-describing format, ecosystem integration |

### ✅ RECOMMENDED (Specialized Analytics Tier)

| Format | Primary Use Cases | Rationale |
|--------|-------------------|-----------|
| **Arrow Analytics (Rust-native)** | MARC discovery analytics, subject/authority frequency analysis, cataloging workflow optimization, bulk field filtering | 1.77M rec/sec (1.96× faster than ISO 2709), 100% round-trip fidelity, zero-copy sub-microsecond column access, Parquet archival with 30% compression; NOT a general-purpose format; complements ISO 2709 |

### ⚠️ CONDITIONAL

*(All recommended formats above are production-ready; no conditional recommendations at this time)*

### ⚠️ CONDITIONAL (Special Use Cases Only)

| Format | Use Case | Condition |
|--------|----------|-----------|
| **Parquet (Arrow columnar)** | Analytics on very large collections (100M+ records) | Only recommended when columnar benefits (selective column access) justify 74% size overhead |

---

## Evaluation Status

| Format | Issue | Status | Date | Fidelity | Recommended |
|--------|-------|--------|------|----------|-------------|
| ISO 2709 | Baseline | ✅ Complete | 2026-01-14 | 100% | ✅ Yes |
| Protobuf | mrrc-fks.1 | ✅ Complete | 2026-01-14 | 100% | ✅ Yes |
| FlatBuffers | mrrc-fks.2 | ✅ Complete | 2026-01-14 | 100% | ✅ Yes |
| Parquet | mrrc-ks7 | ✅ Complete | 2026-01-17 | 100% | ❌ No |
| Arrow (Columnar) | mrrc-fks.7 | ✅ Complete | 2026-01-15 | 100% | ✅ Yes |
| MessagePack | mrrc-fks.5 | ✅ Complete | 2026-01-16 | 100% | ✅ Yes |
| CBOR | mrrc-fks.6 | ✅ Complete | 2026-01-16 | 100% | ✅ Yes |
| Avro | mrrc-fks.4 | ✅ Complete | 2026-01-16 | 100% | ⚠️ Conditional |
| Arrow Analytics (Rust-native) | mrrc-fks.10 | ✅ Complete | 2026-01-17 | 100% | ✅ Yes* |

---

## How to Update This Matrix

When completing a new format evaluation (e.g., mrrc-fks.3 for Parquet):

1. **Extract performance data** from EVALUATION_PARQUET.md section 4
2. **Fill in metrics tables** above (Read, Write, File Size, Memory)
3. **Update integration assessment** (Dependencies, Language Support, Schema Evolution)
4. **Update use case fit scoring** based on evaluation findings
5. **Add to evaluation status** with issue ID and completion date
6. **Update summary recommendations** if needed

**Template for new row:**
```markdown
| **[Format]** | 100% ✅ | [read] | [write] | [roundtrip] | [delta %] |
```

---

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-01-19 | 2.2 | Added new "Analytics Tier: Arrow (Rust-Native)" section; integrated findings from EVALUATION_POLARS_ARROW_DUCKDB.md; clarified distinction between general-purpose formats and specialized analytics tier; split recommendations into two tables (general vs analytics); marked Arrow Analytics as separate entry in evaluation status |
| 2026-01-19 | 2.1 | Filled in all performance numbers from evaluation docs; use consistent stars (ratings) vs checkmarks (pass/fail); removed mixed notation; improved readability |
| 2026-01-16 | 2.0 | Added MessagePack, CBOR, Avro evaluations (7/10 formats complete); updated performance, file size, schema evolution, and recommendations |
| 2026-01-14 | 1.0 | Initial comparison matrix with Protobuf and FlatBuffers complete; templates for remaining formats |
