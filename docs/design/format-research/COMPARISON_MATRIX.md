# Binary Format Comparison Matrix

**Updated:** 2026-01-14  
**Status:** In Progress (2/10 formats evaluated)  
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
| **Parquet** | TBD | TBD | TBD | TBD | TBD |
| **Avro** | TBD | TBD | TBD | TBD | TBD |
| **MessagePack** | TBD | TBD | TBD | TBD | TBD |
| **CBOR** | TBD | TBD | TBD | TBD | TBD |
| **Arrow** | TBD | TBD | TBD | TBD | TBD |

### File Size Metrics

| Format | Raw Size | vs ISO 2709 | Gzip Size | vs ISO 2709 (gzip) | Compression % |
|--------|----------|-------------|-----------|-------------------|---------------|
| **ISO 2709** | 2,645,353 B | Baseline | 85,288 B | Baseline | 96.77% |
| **Protobuf** | 7.2-8.5 MB | **+2.8-3.3x** | 1.2-1.5 MB | **+14-18x** | 84-85% |
| **FlatBuffers** | 6,703,891 B | **+2.5x** | 129,045 B | **+1.5x** | **98.08%** ✅ |
| **Parquet** | TBD | TBD | TBD | TBD | TBD |
| **Avro** | TBD | TBD | TBD | TBD | TBD |
| **MessagePack** | TBD | TBD | TBD | TBD | TBD |
| **CBOR** | TBD | TBD | TBD | TBD | TBD |
| **Arrow** | TBD | TBD | TBD | TBD | TBD |

### Memory Efficiency

| Format | Peak Memory | vs Baseline | Notes |
|--------|-------------|------------|-------|
| **Baseline ISO 2709** | ~45 MB | Baseline | Reference point |
| **Protobuf** | TBD | TBD | TBD |
| **FlatBuffers** | ~16 MB | **-64%** ✅ | Streaming model; zero-copy capable |
| **Parquet** | TBD | TBD | TBD |
| **Avro** | TBD | TBD | TBD |
| **MessagePack** | TBD | TBD | TBD |
| **CBOR** | TBD | TBD | TBD |
| **Arrow** | TBD | TBD | TBD |

---

## Integration & Ecosystem Assessment

### Rust Dependencies

| Format | Direct Deps | Transitive Deps | Maturity | Maintenance | Notes |
|--------|-------------|-----------------|----------|-------------|-------|
| **ISO 2709** | 0 | 0 | Baseline | mrrc native | Built-in; no external deps |
| **Protobuf** | 2 (prost, prost-build) | Low | ⭐⭐⭐⭐⭐ | Active | Google-backed; widely adopted |
| **FlatBuffers** | 2 (flatbuffers, serde_json) | Low | ⭐⭐⭐⭐⭐ | Active | Google-backed; production-ready |
| **Parquet** | TBD | TBD | TBD | TBD | TBD |
| **Avro** | TBD | TBD | TBD | TBD | TBD |
| **MessagePack** | TBD | TBD | TBD | TBD | TBD |
| **CBOR** | TBD | TBD | TBD | TBD | TBD |
| **Arrow** | TBD | TBD | TBD | TBD | TBD |

### Language Support

| Format | Rust | Python | Java | Go | C++ | Notes |
|--------|------|--------|------|----|----|-------|
| **ISO 2709** | ✅ mrrc | ✅ pymrrc | ⚠️ custom | ⚠️ custom | ⚠️ custom | mrrc is native Rust |
| **Protobuf** | ✅⭐⭐⭐ | ✅⭐⭐⭐ | ✅⭐⭐⭐ | ✅⭐⭐⭐ | ✅⭐⭐⭐ | Official support all languages |
| **FlatBuffers** | ✅⭐⭐⭐ | ✅⭐⭐⭐ | ✅⭐⭐⭐ | ✅⭐⭐ | ✅⭐⭐⭐ | Official support; good ecosystem |
| **Parquet** | TBD | TBD | TBD | TBD | TBD | TBD |
| **Avro** | TBD | TBD | TBD | TBD | TBD | TBD |
| **MessagePack** | TBD | TBD | TBD | TBD | TBD | TBD |
| **CBOR** | TBD | TBD | TBD | TBD | TBD | TBD |
| **Arrow** | TBD | TBD | TBD | TBD | TBD | TBD |

### Schema Evolution Capability

| Format | Score (1-5) | Forward Compat | Backward Compat | Append-Only | Notes |
|--------|-------------|----------------|-----------------|------------|-------|
| **ISO 2709** | 1 | ❌ No | ❌ No | ❌ No | Fixed binary format; no evolution |
| **Protobuf** | **5** ✅ | ✅ Yes | ✅ Yes | ✅ Yes | Bi-directional; full compatibility |
| **FlatBuffers** | 4 | ⚠️ Partial | ✅ Yes | ✅ Append-only | Append-only evolution constraint |
| **Parquet** | TBD | TBD | TBD | TBD | TBD |
| **Avro** | TBD | TBD | TBD | TBD | TBD |
| **MessagePack** | 1 | ❌ No | ❌ No | ❌ No | Untyped; no schema versioning |
| **CBOR** | 2 | ⚠️ Partial | ⚠️ Partial | ❌ No | Semantic tags provide some flexibility |
| **Arrow** | TBD | TBD | TBD | TBD | TBD |

---

## Error Handling & Robustness

| Format | Graceful Errors | No Panics | Silent Corruption | Notes |
|--------|-----------------|-----------|-------------------|-------|
| **Protobuf** | ✅ All 7 scenarios | ✅ Zero panics | ✅ None | Comprehensive error handling |
| **FlatBuffers** | ✅ All 7 scenarios | ✅ Zero panics | ✅ None | Comprehensive error handling |
| **Parquet** | TBD | TBD | TBD | TBD |
| **Avro** | TBD | TBD | TBD | TBD |
| **MessagePack** | TBD | TBD | TBD | TBD |
| **CBOR** | TBD | TBD | TBD | TBD |
| **Arrow** | TBD | TBD | TBD | TBD |

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
| **MessagePack** | ⭐⭐⭐ | Compact; reasonable throughput |
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
| **MessagePack** | ⭐⭐⭐⭐ | Very compact serialization |
| **Protobuf** | ⭐⭐⭐ | Reasonable; not specifically optimized |
| ISO 2709 | ⭐⭐ | No built-in memory optimization |

### Long-Term Archival (10+ years)

| Format | Score | Notes |
|--------|-------|-------|
| **Protobuf** | ⭐⭐⭐⭐ | Schema versioning; forward/backward compatible |
| **Avro** | ⭐⭐⭐⭐ | Self-describing; schema registry support |
| **ISO 2709** | ⭐⭐⭐⭐ | Proven 50+ year track record; stable standard |
| **FlatBuffers** | ⭐⭐⭐ | Append-only evolution; less flexible |
| **CBOR** | ⭐⭐⭐ | Standards-based; less mature than others |

---

## Implementation Complexity

| Format | LOC (Rust) | Dev Time | Maintenance | Build Impact |
|--------|-----------|----------|-------------|--------------|
| **Protobuf** | 415 | 2-3 hrs | Low (code-gen) | +5.88s full build |
| **FlatBuffers** | 214 | 1-2 hrs | Low (minimal schema) | <5s full build |
| **Parquet** | TBD | TBD | TBD | TBD |
| **Avro** | TBD | TBD | TBD | TBD |
| **MessagePack** | TBD | TBD | TBD | TBD |
| **CBOR** | TBD | TBD | TBD | TBD |
| **Arrow** | TBD | TBD | TBD | TBD |

---

## Summary Recommendations

### ✅ RECOMMENDED

| Format | Primary Use Cases | Rationale |
|--------|-------------------|-----------|
| **Protobuf** | API serialization, REST/gRPC, cross-language interchange | Mature ecosystem, schema evolution, excellent Rust support |
| **FlatBuffers** | Streaming APIs, memory-constrained environments, zero-copy scenarios | Memory efficient, fast deserialization, Apple production use |
| **ISO 2709** | High-throughput batch processing, archival baseline | Proven standard, maximum performance, no dependencies |

### ⚠️ CONDITIONAL

| Format | Conditions | Trade-offs |
|--------|-----------|-----------|
| **Parquet** | Big data analytics with Spark/Hadoop | Not suitable for record-by-record API access |
| **Arrow** | In-memory analytics, multi-language processing | Requires columnar data model; columnar ↔ record conversion overhead |
| **Avro** | Event streaming (Kafka), data lake architectures | Kafka ecosystem; not optimized for single-record performance |

### ❌ NOT RECOMMENDED (for now)

| Format | Reason | Notes |
|--------|--------|-------|
| **MessagePack** | Limited schema evolution; no validation | Use Protobuf instead for better schema support |
| **CBOR** | Early ecosystem adoption; less proven than alternatives | Standards-based but immature relative to Protobuf/FlatBuffers |

---

## Evaluation Status

| Format | Issue | Status | Date | Evaluator |
|--------|-------|--------|------|-----------|
| ISO 2709 | Baseline | ✅ Complete | 2026-01-14 | dchud |
| Protobuf | mrrc-fks.1 | ✅ Complete | 2026-01-14 | dchud |
| FlatBuffers | mrrc-fks.2 | ✅ Complete | 2026-01-14 | dchud |
| Parquet | mrrc-fks.3 | 🔵 Open | TBD | TBD |
| Avro | mrrc-fks.4 | 🔵 Open | TBD | TBD |
| MessagePack | mrrc-fks.5 | 🔵 Open | TBD | TBD |
| CBOR | mrrc-fks.6 | 🔵 Open | TBD | TBD |
| Arrow | mrrc-fks.7 | 🔵 Open | TBD | TBD |
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
| 2026-01-14 | 1.0 | Initial comparison matrix with Protobuf and FlatBuffers complete; templates for remaining formats |
