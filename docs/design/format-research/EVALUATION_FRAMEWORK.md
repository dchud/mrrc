# Binary Format Evaluation Framework

This document establishes the standardized evaluation framework and methodology for assessing binary data formats as potential MARC import/export formats. All format-specific evaluations **must** follow this framework to ensure comparable, synthesizable results.

## Overview

**Objective:** Evaluate modern binary formats for MARC data representation, measuring round-trip fidelity, performance, and ecosystem fit against the ISO 2709 baseline.

**Scope:** Each format evaluation produces a structured report following the templates in this document.

---

## 1. Schema Design Requirements

Every format evaluation must include a complete schema design addressing:

### 1.1 Required MARC Components

| Component | Description | Schema Requirement |
|-----------|-------------|-------------------|
| **Leader** | 24-character leader string | Must preserve all 24 positions |
| **Record Type** | From leader position 06 | Must be queryable |
| **Control Fields** | Tags 001-009 | No subfields, single data value |
| **Variable Fields** | Tags 010-999 | Tag + indicators + subfields |
| **Indicators** | Two indicator positions | Must preserve blank vs missing |
| **Subfields** | Code + value pairs | Variable count, ordered |
| **Encoding** | MARC-8 or UTF-8 | Must preserve original encoding |

### 1.2 Edge Cases Checklist

- [ ] Empty subfield values
- [ ] Repeating fields (multiple 650s, etc.)
- [ ] Repeating subfields within a field
- [ ] MARC-8 encoded characters (diacritics, special chars)
- [ ] UTF-8 multilingual content (CJK, Arabic, Hebrew)
- [ ] Maximum field lengths (9999 bytes)
- [ ] Control characters in data
- [ ] Blank vs missing indicators
- [ ] Records with hundreds of fields

### 1.3 Schema Notation

Provide schema in:
1. **Native format** (proto file, Avro schema, etc.)
2. **ASCII diagram** showing structure
3. **Example record** serialized with annotations

---

## 2. Test Data Sets

### 2.1 Fidelity Test Set (100 records)

Location: `tests/data/fixtures/fidelity_test_100.mrc`

Composition:
- 50 bibliographic records (varied formats: books, serials, scores, maps)
- 25 authority records (personal, corporate, subject headings)
- 15 holdings records
- 10 edge case records (encoding, maximum sizes, special characters)

**Selection criteria:**
- Must include all common record types (BKS, SER, MUS, MAP, etc.)
- Must include records with MARC-8 encoding
- Must include multilingual records (CJK, Arabic, Cyrillic)
- Must include records with 100+ fields
- Must include records with repeating fields/subfields

### 2.2 Performance Test Set (10,000 records)

Location: `tests/data/fixtures/10k_records.mrc`

Already available; used for standardized benchmark comparisons.

### 2.3 Stress Test Set (100,000 records)

Location: `tests/data/fixtures/100k_records.mrc`

For extended performance evaluation and memory profiling.

---

## 3. Round-Trip Testing Protocol

### 3.1 Test Procedure

```
1. Load original MARC record(s) from ISO 2709
2. Convert to candidate format
3. Serialize to bytes/file
4. Deserialize from bytes/file
5. Convert back to MARC
6. Compare original vs round-tripped
```

### 3.2 Validation Criteria

| Criterion | Weight | Pass Condition |
|-----------|--------|----------------|
| Leader preservation | Critical | Byte-exact match |
| Field ordering | High | Fields in same sequence |
| Tag values | Critical | All tags present and correct |
| Indicator values | Critical | Exact match including blanks |
| Subfield codes | Critical | All codes present and ordered |
| Subfield values | Critical | Byte-exact match (encoding-aware) |
| Encoding preservation | Critical | Original encoding maintained |

### 3.3 Fidelity Score

Calculate: `(records_perfect / total_records) × 100`

- **100%** = Required for recommendation
- **<100%** = Document all failures with root cause

### 3.4 Reporting Format

```markdown
## Round-Trip Fidelity Results

**Test Set:** fidelity_test_100.mrc
**Records Tested:** 100
**Perfect Round-Trips:** 100/100 (100%)

### Failures (if any)

| Record | Field | Issue | Root Cause |
|--------|-------|-------|------------|
| n/a | n/a | n/a | n/a |

### Notes

[Any format-specific observations]
```

---

## 4. Performance Benchmark Protocol

### 4.1 Test Environment

Document:
- CPU model and cores
- RAM
- Storage type (SSD/HDD)
- OS version
- Rust/Python versions
- Library versions

### 4.2 Metrics to Measure

| Metric | Unit | Description |
|--------|------|-------------|
| **Read Throughput** | records/sec | Deserialize from format to MARC objects |
| **Write Throughput** | records/sec | Serialize from MARC objects to format |
| **File Size (raw)** | bytes | Output file size without compression |
| **File Size (compressed)** | bytes | With gzip/zstd compression |
| **Compression Ratio** | % | `1 - (format_size / iso2709_size)` |
| **Peak Memory** | MB | Maximum RSS during operation |
| **Startup Cost** | ms | Time to initialize format handler |

### 4.3 Benchmark Procedure

```bash
# Warm-up: 3 iterations (discarded)
# Measured: 10 iterations (averaged)

# Read benchmark
hyperfine --warmup 3 --runs 10 'cargo run --release read_format input.{fmt}'

# Write benchmark  
hyperfine --warmup 3 --runs 10 'cargo run --release write_format input.mrc output.{fmt}'
```

### 4.4 Baseline Comparison

All metrics compared against ISO 2709 baseline:

| Operation | ISO 2709 Baseline | Format Result | Delta |
|-----------|-------------------|---------------|-------|
| Read | X rec/sec | Y rec/sec | +/-Z% |
| Write | X rec/sec | Y rec/sec | +/-Z% |
| Size | X bytes | Y bytes | +/-Z% |

### 4.5 Reporting Table

```markdown
## Performance Results

**Test Set:** 10k_records.mrc (10,000 records)
**Test Date:** YYYY-MM-DD
**Environment:** [specs]

| Metric | ISO 2709 | [Format] | Delta |
|--------|----------|----------|-------|
| Read (rec/sec) | 150,000 | TBD | TBD |
| Write (rec/sec) | 120,000 | TBD | TBD |
| File Size | 12.5 MB | TBD | TBD |
| Compressed Size | 4.2 MB | TBD | TBD |
| Peak Memory | 45 MB | TBD | TBD |
```

---

## 5. Integration Assessment Criteria

### 5.1 Dependency Analysis

| Factor | Score Range | Description |
|--------|-------------|-------------|
| **External deps** | Count | Number of required dependencies |
| **Dep health** | 1-5 | Maintenance status, security track record |
| **Build complexity** | 1-5 | 1=simple cargo add, 5=complex toolchain |

### 5.2 Language Support Matrix

| Language | Library | Maturity | Notes |
|----------|---------|----------|-------|
| Rust | crate_name | ⭐⭐⭐⭐⭐ | |
| Python | package_name | ⭐⭐⭐⭐ | |
| Java | package_name | ⭐⭐⭐ | |
| Go | package_name | ⭐⭐ | |
| C++ | library_name | ⭐⭐⭐ | |

### 5.3 Schema Evolution Score

| Capability | Score |
|------------|-------|
| No schema evolution | 1 |
| Append-only evolution | 2 |
| Backward compatible | 3 |
| Forward compatible | 4 |
| Full bi-directional | 5 |

### 5.4 Ecosystem Maturity

- [ ] Production use cases documented
- [ ] Active maintenance (commits in last 6 months)
- [ ] Security advisories process
- [ ] Stable API (1.0+ release)
- [ ] Good documentation
- [ ] Community size / adoption

---

## 6. Use Case Fit Scoring

Rate each format 1-5 for each use case:

| Use Case | Score | Notes |
|----------|-------|-------|
| **Simple data exchange** | | API integration, file transfer |
| **High-performance batch** | | Large-scale processing |
| **Analytics/big data** | | Spark, Hadoop integration |
| **IoT/embedded** | | Resource-constrained environments |
| **Real-time streaming** | | Event-driven pipelines |
| **API integration** | | REST/gRPC services |
| **Long-term archival** | | 10+ year preservation |

---

## 7. Evaluation Report Template

Each format evaluation produces a report following this structure:

```markdown
# [Format Name] Evaluation for MARC Data

**Issue:** mrrc-fks.N
**Date:** YYYY-MM-DD
**Author:** [name]
**Status:** Draft | Complete

## Executive Summary

[2-3 sentences: Is this format viable? Key findings?]

## 1. Schema Design

### 1.1 Schema Definition
[Native schema file]

### 1.2 Structure Diagram
[ASCII diagram]

### 1.3 Example Record
[Annotated example]

### 1.4 Edge Case Coverage
[Checklist from section 1.2]

## 2. Round-Trip Fidelity

[Results per section 3]

## 3. Performance Benchmarks

[Results per section 4]

## 4. Integration Assessment

[Analysis per section 5]

## 5. Use Case Fit

[Scoring per section 6]

## 6. Implementation Complexity

- Lines of code estimate
- Development time estimate
- Maintenance burden assessment

## 7. Strengths & Weaknesses

### Strengths
- Bullet points

### Weaknesses
- Bullet points

## 8. Recommendation

**Verdict:** Recommended | Conditional | Not Recommended

[Rationale paragraph]

## Appendix

### A. Test Commands
### B. Sample Code
### C. References
```

---

## 8. Comparison Matrix Template

After all evaluations complete, aggregate into:

| Format | Fidelity | Read (rec/s) | Write (rec/s) | Size | Deps | Evolution | Overall |
|--------|----------|--------------|---------------|------|------|-----------|---------|
| ISO 2709 | 100% | 150k | 120k | 1.0x | 0 | 1 | Baseline |
| Protobuf | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| FlatBuffers | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| ... | ... | ... | ... | ... | ... | ... | ... |

---

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-01-12 | 1.0 | Initial framework definition |
