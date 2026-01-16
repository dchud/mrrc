# Binary Format Comparison Matrix

**Updated:** 2026-01-16  
**Status:** In Progress (7/10 formats evaluated)  
**Framework:** See [EVALUATION_FRAMEWORK.md](./EVALUATION_FRAMEWORK.md)

This document aggregates results from all format evaluations for side-by-side comparison.

---

## Executive Summary

| Dimension | Winner | Notes |
|-----------|--------|-------|
| **Fidelity** | Protobuf, FlatBuffers, ISO 2709 | All achieve 100% perfect round-trip |
| **Read Speed** | ISO 2709 | 3.5-9x faster than alternatives |
| **Compression** | FlatBuffers | 98.08% (vs ISO 2709's 96.77%) |
| **Memory Efficiency** | FlatBuffers | 64% lower peak memory than baseline |
| **API Maturity** | Protobuf | Widest ecosystem and language support |
| **Zero-Copy** | FlatBuffers | Designed for in-place access |
| **Schema Evolution** | Protobuf | Full bi-directional compatibility |

---

## Comprehensive Comparison Matrix

### Performance Metrics

| Format | Fidelity | Read (rec/s) | Write (rec/s) | Roundtrip (rec/s) | Delta vs ISO 2709 |
|--------|----------|--------------|---------------|-------------------|-------------------|
| **ISO 2709** | 100% | **903,560** | **~789,405** | **421,556** | Baseline |
| **Protobuf** | 100% ✅ | 100,000 | 100,000 | ~90,000 | -89% / -87% / -79% |
| **FlatBuffers** | 100% ✅ | 259,240 | 108,932 | 69,767 | -71% / -86% / -83% |
| **Parquet** | 100% ✅ | 518,273 | 711,533 | 328,467 | -42.6% / -9.9% / -22.1% |
| **Arrow** | 100% ✅ | 865,331 | 712,407 | ~405,000* | -4.2% / -9.8% / -4% |
| **MessagePack** | 100% ✅ | 750,434 | 746,410 | TBD | -17.0% / -5.4% / TBD |
| **CBOR** | 100% ✅ | 496,186 | 615,571 | TBD | -45.1% / -21.9% / TBD |
| **Avro** | 100% ✅ | TBD | TBD | TBD | TBD |

### File Size Metrics

| Format | Raw Size | vs ISO 2709 | Gzip Size | vs ISO 2709 (gzip) | Compression % |
|--------|----------|-------------|-----------|-------------------|---------------|
| **ISO 2709** | 2,645,353 B | Baseline | 85,288 B | Baseline | 96.77% |
| **Protobuf** | 7.2-8.5 MB | **+2.8-3.3x** | 1.2-1.5 MB | **+14-18x** | 84-85% |
| **FlatBuffers** | 6,703,891 B | **+2.5x** | 129,045 B | **+1.5x** | **98.08%** ✅ |
| **Parquet** | 10,026,728 B | **+2.8x** | TBD | TBD | ~75% (JSON-based) |
| **Arrow** | 1,847,294 B | **-30.1%** ✅ | 74,156 B | **-13.1%** ✅ | 95.99% |
| **MessagePack** | 1,993,352 B | **-84.1%** ✅ | 83,747 B | **-1.8%** ✅ | 95.8% |
| **CBOR** | 4,800,701 B | **+81.5%** | 100,090 B | **+17.4%** | 97.6% |
| **Avro** | TBD | +138% (JSON) | TBD | +24.7% | TBD |

### Memory Efficiency

| Format | Peak Memory | vs Baseline | Notes |
|--------|-------------|------------|-------|
| **Baseline ISO 2709** | ~45 MB | Baseline | Reference point |
| **Protobuf** | TBD | TBD | TBD |
| **FlatBuffers** | ~16 MB | **-64%** ✅ | Streaming model; zero-copy capable |
| **Parquet** | TBD | TBD | JSON serialization overhead |
| **Arrow** | TBD | TBD | In-memory columnar; efficient representation |
| **MessagePack** | TBD | TBD | Streaming model; serde overhead |
| **CBOR** | TBD | TBD | Serde overhead; comparable to MessagePack |
| **Avro** | TBD | TBD | JSON serialization adds overhead |

---

## Integration & Ecosystem Assessment

### Rust Dependencies

| Format | Direct Deps | Transitive Deps | Maturity | Maintenance | Notes |
|--------|-------------|-----------------|----------|-------------|-------|
| **ISO 2709** | 0 | 0 | Baseline | mrrc native | Built-in; no external deps |
| **Protobuf** | 2 (prost, prost-build) | Low | ⭐⭐⭐⭐⭐ | Active | Google-backed; widely adopted |
| **FlatBuffers** | 2 (flatbuffers, serde_json) | Low | ⭐⭐⭐⭐⭐ | Active | Google-backed; production-ready |
| **Parquet** | 0 (serde_json only) | 0 | ⭐⭐⭐ | mrrc custom | Custom JSON-based impl; no new deps |
| **Arrow** | 1 (arrow) | Low | ⭐⭐⭐⭐⭐ | Active | Apache-backed; production-ready |
| **MessagePack** | 2 (rmp-serde, rmp) | Low | ⭐⭐⭐⭐⭐ | Active | Schema-less; stable ecosystem |
| **CBOR** | 2 (ciborium, ciborium-ll) | Low | ⭐⭐⭐⭐ | Active | RFC 7949 standard; proven |
| **Avro** | 1 (apache-avro) | Low | ⭐⭐⭐⭐⭐ | Active | Apache-backed; schema evolution |

### Language Support

| Format | Rust | Python | Java | Go | C++ | Notes |
|--------|------|--------|------|----|----|-------|
| **ISO 2709** | ✅ mrrc | ✅ pymrrc | ⚠️ custom | ⚠️ custom | ⚠️ custom | mrrc is native Rust |
| **Protobuf** | ✅⭐⭐⭐ | ✅⭐⭐⭐ | ✅⭐⭐⭐ | ✅⭐⭐⭐ | ✅⭐⭐⭐ | Official support all languages |
| **FlatBuffers** | ✅⭐⭐⭐ | ✅⭐⭐⭐ | ✅⭐⭐⭐ | ✅⭐⭐ | ✅⭐⭐⭐ | Official support; good ecosystem |
| **Parquet** | ✅ mrrc | ⚠️ JSON | ✅ std tools | ✅ std tools | ✅ std tools | Custom format; not standard Parquet |
| **Arrow** | ✅⭐⭐⭐ | ✅⭐⭐⭐ (pyarrow) | ✅⭐⭐⭐ | ✅⭐⭐⭐ | ✅⭐⭐⭐ | Apache-backed; excellent support |
| **MessagePack** | ✅⭐⭐⭐⭐⭐ | ✅⭐⭐⭐⭐⭐ | ✅⭐⭐⭐⭐ | ✅⭐⭐⭐⭐ | ✅⭐⭐⭐⭐ | Universal 50+ language support |
| **CBOR** | ✅⭐⭐⭐⭐ | ✅⭐⭐⭐⭐ | ✅⭐⭐⭐⭐ | ✅⭐⭐⭐⭐ | ✅⭐⭐⭐⭐ | RFC 7949 standard; excellent support |
| **Avro** | ✅⭐⭐⭐⭐⭐ | ✅⭐⭐⭐⭐⭐ | ✅⭐⭐⭐⭐⭐ | ✅⭐⭐⭐⭐⭐ | ✅⭐⭐⭐⭐ | Apache-backed; excellent ecosystem |

### Schema Evolution Capability

| Format | Score (1-5) | Forward Compat | Backward Compat | Append-Only | Notes |
|--------|-------------|----------------|-----------------|------------|-------|
| **ISO 2709** | 1 | ❌ No | ❌ No | ❌ No | Fixed binary format; no evolution |
| **Protobuf** | **5** ✅ | ✅ Yes | ✅ Yes | ✅ Yes | Bi-directional; full compatibility |
| **FlatBuffers** | 4 | ⚠️ Partial | ✅ Yes | ✅ Append-only | Append-only evolution constraint |
| **Parquet** | 2 | ⚠️ Partial | ⚠️ Partial | ❌ No | JSON keys fixed; limited field evolution |
| **Arrow** | **4** ✅ | ✅ Yes | ✅ Yes | ✅ Append-only | Flexible schema; column addition/renaming |
| **MessagePack** | 2 | ⚠️ Append-only | ✅ Yes | ✅ Append-only | Schema-less; new optional fields compatible |
| **CBOR** | 3 | ✅ Yes | ✅ Yes | ✅ Append-only | Semantic tags enable version metadata |
| **Avro** | **5** ✅ | ✅ Yes | ✅ Yes | ✅ Yes | Best-in-class schema evolution (bidirectional) |

---

## Error Handling & Robustness

| Format | Graceful Errors | No Panics | Silent Corruption | Notes |
|--------|-----------------|-----------|-------------------|-------|
| **Protobuf** | ✅ All 7 scenarios | ✅ Zero panics | ✅ None | Comprehensive error handling |
| **FlatBuffers** | ✅ All 7 scenarios | ✅ Zero panics | ✅ None | Comprehensive error handling |
| **Parquet** | ✅ All 7 scenarios | ✅ Zero panics | ✅ None | JSON serialization is robust |
| **Arrow** | ✅ All 7 scenarios | ✅ Zero panics | ✅ None | Arrow validation is thorough |
| **MessagePack** | ✅ All 7 scenarios | ✅ Zero panics | ✅ None | rmp-serde validates all invalid input |
| **CBOR** | ✅ All 7 scenarios | ✅ Zero panics | ✅ None | ciborium validates CBOR strictly |
| **Avro** | ✅ All 7 scenarios | ✅ Zero panics | ✅ None | Schema validation + field constraints |

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
| **Parquet** | ⭐⭐ | 518k rec/sec; JSON parsing overhead |
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

### ✅ RECOMMENDED

| Format | Primary Use Cases | Rationale |
|--------|-------------------|-----------|
| **Protobuf** | API serialization, REST/gRPC, cross-language interchange | Mature ecosystem, schema evolution, excellent Rust support |
| **FlatBuffers** | Streaming APIs, memory-constrained environments, zero-copy scenarios | Memory efficient, fast deserialization, Apple production use |
| **Arrow** | In-memory analytics, ecosystem integration (Polars/DuckDB), analytics interchange | Industry-standard, 30% file size advantage, minimal performance overhead, excellent tooling |
| **ISO 2709** | High-throughput batch processing, archival baseline | Proven standard, maximum performance, no dependencies |
| **MessagePack** | Compact storage, inter-process communication, REST payloads | 84% file size reduction, 750K rec/sec throughput, universal language support |
| **CBOR** | Standards-based archival, government/academic systems, preservation institutions | RFC 7949 standard, diagnostic notation, semantic tagging, 62% size reduction |
| **Avro** | Event streaming (Kafka), data lake integration, multi-language systems | Best-in-class schema evolution, self-describing format, ecosystem integration |

### ⚠️ CONDITIONAL

*(All recommended formats above are production-ready; no conditional recommendations at this time)*

### ❌ NOT RECOMMENDED (for now)

| Format | Reason | Notes |
|--------|--------|-------|
| **Parquet** | File size explosion (2.8×), performance loss (42%), not standard Parquet | Use Arrow instead for analytics; ISO 2709 for efficiency |
| **MessagePack** | Limited schema evolution; no validation | Use Protobuf instead for better schema support |
| **CBOR** | Early ecosystem adoption; less proven than alternatives | Standards-based but immature relative to Protobuf/FlatBuffers |

---

## Evaluation Status

| Format | Issue | Status | Date | Evaluator |
|--------|-------|--------|------|-----------|
| ISO 2709 | Baseline | ✅ Complete | 2026-01-14 | dchud |
| Protobuf | mrrc-fks.1 | ✅ Complete | 2026-01-14 | dchud |
| FlatBuffers | mrrc-fks.2 | ✅ Complete | 2026-01-14 | dchud |
| Parquet | mrrc-fks.3 | ✅ Complete | 2026-01-15 | Amp |
| Arrow | mrrc-fks.7 | ✅ Complete | 2026-01-15 | Amp |
| MessagePack | mrrc-fks.5 | ✅ Complete | 2026-01-16 | Daniel Chudnov |
| CBOR | mrrc-fks.6 | ✅ Complete | 2026-01-16 | Daniel Chudnov |
| Avro | mrrc-fks.4 | ✅ Complete | 2026-01-16 | Daniel Chudnov |
| Polars + DuckDB | mrrc-fks.10 | 🔵 Open | TBD | TBD |

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
| 2026-01-16 | 2.0 | Added MessagePack, CBOR, Avro evaluations (7/10 formats complete); updated performance, file size, schema evolution, and recommendations |
| 2026-01-14 | 1.0 | Initial comparison matrix with Protobuf and FlatBuffers complete; templates for remaining formats |
