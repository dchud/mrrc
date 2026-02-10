# Binary Format Evaluation Framework

This document establishes the standardized evaluation framework and methodology for assessing binary data formats as potential MARC import/export formats. All format-specific evaluations **must** follow this framework to ensure comparable, synthesizable results.

## Overview

**Objective:** Evaluate modern binary formats for MARC data representation as potential **Rust mrrc core implementations**, measuring round-trip fidelity, Rust performance, and ecosystem fit against the ISO 2709 baseline.

**Scope:** Each format evaluation focuses on **Rust implementation** with secondary consideration for Python/multi-language support:
- **Primary:** Rust implementation for mrrc core (via `cargo` dependency or internal implementation)
- **Secondary:** Python convenience wrappers (via PyO3) for applicable formats
- **Tertiary:** Broader language ecosystem (Java, Go, C++) noted but not required

Each format evaluation produces a structured report following the templates in this document.

### Encoding & Normalization Assumptions

All evaluations operate on mrrc's internal MARC data model:

- Input ISO 2709 (MARC) records MAY be in MARC-8 or UTF-8.
- **mrrc is responsible for** decoding ISO 2709 and normalizing all record content to UTF-8 `MarcRecord` objects.
- Candidate binary formats are evaluated only on their ability to faithfully represent and round-trip these **UTF-8 `MarcRecord` objects**.
- Preservation of the original ISO 2709 byte encoding (e.g., MARC-8 escape sequences) is **out of scope** for binary format evaluation — that's tested separately in mrrc's import/export layer.

All fidelity comparisons in this framework are defined on the normalized MARC data model (fields, indicators, subfields, UTF-8 strings), not on raw ISO 2709 bytes.

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
| **Text encoding** | UTF-8 strings | Must preserve all Unicode content from mrrc's normalized `MarcRecord` |

### 1.2 Edge Cases Checklist

**Data Structure & Ordering (CRITICAL):**
- [ ] **Field ordering** — 001, 245, 650, 001 sequence preserved exactly (not alphabetized or renumbered)
- [ ] **Subfield code ordering** — $d$c$a preserved as-is (not reordered to $a$c$d)
- [ ] Repeating fields (multiple 650s, etc.) — **preserves in order**
- [ ] Repeating subfields within a field (e.g., `$a $a $a`) — **preserves in order**
- [ ] Empty subfield values (distinguish `$a ""` from missing `$a`)

**Text Content:**
- [ ] UTF-8 multilingual content (CJK, Arabic, Hebrew, Cyrillic)
- [ ] Combining diacritics and special characters (NOT precomposed)
- [ ] Whitespace preservation (leading/trailing spaces in subfield values)
- [ ] Control characters in data (0x00-0x1F) — handled or rejected?

**Field Structure:**
- [ ] Control fields (001-009) vs variable fields (010+) distinction preserved
- [ ] Maximum field lengths (9999 bytes data, not including tag/indicators)
- [ ] Blank vs missing indicators (space U+0020 vs null distinction)
- [ ] Records with hundreds of fields

**MARC-Specific & Validation:**
- [ ] Repeating control fields (001 should not repeat; test preservation/error handling)
- [ ] Invalid tag values (e.g., "999", "0AB", empty) — rejected or preserved?
- [ ] Invalid subfield codes (e.g., "0", space, non-ASCII) — rejected or preserved?

### 1.3 Schema Notation

Provide schema in:
1. **Native format** (proto file, Avro schema, etc.)
2. **ASCII diagram** showing structure
3. **Example record** serialized with annotations

---

## 2. Test Data Sets

### 2.1 Fidelity Test Set (100 records)

Location: `tests/data/fixtures/fidelity_test_100.mrc`

**Status:** Currently under development. See [FIDELITY_TEST_SET.md](./FIDELITY_TEST_SET.md) for detailed composition requirements and creation checklist.

Composition:
- 50 bibliographic records (varied formats: books, serials, scores, maps)
- 25 authority records (personal, corporate, subject headings)
- 15 holdings records
- 10 edge case records (encoding, maximum sizes, special characters)

**Selection criteria:**
- Must include all common record types (BKS, SER, MUS, MAP, etc.)
- Must include multilingual records (CJK, Arabic, Cyrillic)
- Must include records with 100+ fields
- Must include records with repeating fields/subfields
- Must include synthetic worst-case records (maximum field sizes, control character boundaries)

**Validation requirement:** Before using in evaluations, must pass validation script verifying composition, encoding coverage, and edge case presence.

Note: The test set includes MARC-8 encoded source records to exercise mrrc's normalization pipeline; binary formats only see the resulting UTF-8 `MarcRecord` objects.

### 2.2 Performance Test Set (10,000 records)

Location: `tests/data/fixtures/10k_records.mrc`

Already available; used for standardized benchmark comparisons.

### 2.3 Stress Test Set (100,000 records)

Location: `tests/data/fixtures/100k_records.mrc`

For extended performance evaluation and memory profiling.

---

## 3. Round-Trip Testing Protocol

### 3.1 Test Procedure

The round-trip test validates that candidate formats can preserve `MarcRecord` data with perfect fidelity:

```
Step 1: Load ISO 2709 → MarcRecord (mrrc's import layer)
        ↓
Step 2: MarcRecord → Candidate Format (serialize)
        ↓
Step 3: Candidate Format → MarcRecord (deserialize)
        ↓
Step 4: Compare Step 1 MarcRecord vs Step 3 MarcRecord (field-by-field)
        ↓
Result: PASS if identical, FAIL if any mismatch
```

**Important:** Comparison is performed on **normalized `MarcRecord` structures** (leader, fields, indicators, subfields, UTF-8 strings). We do NOT compare original ISO 2709 bytes or MARC-8 escape sequences — those are handled by mrrc's import layer, not the candidate format.

### 3.2 Validation Criteria

| Criterion | Weight | Pass Condition |
|-----------|--------|----------------|
| Leader preservation | Critical | Positions 0-3, 12-15 may recalculate; all others match exactly |
| Field ordering | **Critical** | **Fields in exact input sequence (not reordered or sorted)** |
| Tag values | Critical | All tags present, matching baseline exactly (e.g., "001", "245") |
| Indicator values | Critical | Exact match including blank (space char) vs missing distinction |
| Subfield code ordering | **Critical** | **All codes present and in exact input order (not reordered)** |
| Subfield values | Critical | Exact UTF-8 string match, preserving empty strings vs null distinction |

#### 3.2.1 Correctness Specification

**MUST Match Exactly:**
- **Leader (24 chars):** Positions 0-3 (logical record length) and 12-15 (base address) MAY be recalculated, but all OTHER positions (5-11, 17-23) MUST match exactly
- **Tag values:** String representation exactly (e.g., "001", "245") with numeric value preserved
- **Field ordering:** Fields in exact input sequence (001, 245, 650, 001 → NOT reordered alphabetically or numerically)
- **Indicator values:** Character values, where space (U+0020) is distinct from null/missing
- **Subfield codes:** Character values in correct order (e.g., $d$c$a → NOT reordered to $a$c$d)
- **Subfield values:** Exact UTF-8 byte-for-byte match, preserving empty strings (`""`) as distinct from missing values

**MUST NOT Collapse or Transform:**
- Whitespace within strings must be preserved exactly (no trim, no collapse, leading/trailing spaces matter)
- Empty subfield values (`$a ""`) are distinct from absent subfields (no `$a`)
- Combining diacritical marks must preserve their UTF-8 encoding (do NOT normalize to precomposed forms)
- Field and subfield sequence must be preserved exactly (no reordering, no sorting, no deduplication)

**MUST NOT Compare:**
- Original ISO 2709 byte encoding (MARC-8 vs UTF-8) — that's mrrc's responsibility
- Internal serialization order (e.g., protobuf field order vs JSON property order)
- Memory representation or object IDs

**Failure Threshold:**
- Any mismatch in the "MUST Match" criteria → **FAIL** for that record
- Any crash/panic on invalid input → **FAIL** entire evaluation
- **100% perfect round-trip rate is MANDATORY for recommendation**

### 3.3 Fidelity Score

Calculate: `(records_perfect / total_records) × 100`

- **100%** = Required for recommendation
- **<100%** = Document all failures with root cause

### 3.4 Failure Analysis Template

For any failed round-trip, provide:

| Record ID | Field/Tag | Criterion | Expected | Actual | Root Cause |
|-----------|-----------|-----------|----------|--------|------------|
| n/a | n/a | n/a | n/a | n/a | n/a |

**Failure Investigation Checklist:**
- [ ] **Field ordering changed** (001, 245, 650 reordered alphabetically or numerically)?
- [ ] **Subfield code order changed** ($d$c$a reordered to $a$c$d)?
- [ ] Encoding issue (UTF-8 normalization, combining diacritics)?
- [ ] Indicator handling (space vs null)?
- [ ] Subfield presence missing (wrong count, missing codes)?
- [ ] Empty string handling (empty `$a ""` vs missing `$a`)?
- [ ] Whitespace trimmed (leading/trailing spaces lost)?
- [ ] Leader position recalculation (only 0-3, 12-15 expected to vary)?
- [ ] Type conversion error (string, integer, character)?
- [ ] Data truncation (field size limit)?

### 3.5 Reporting Format

```markdown
## Round-Trip Fidelity Results

**Test Set:** fidelity_test_100.mrc
**Records Tested:** 100
**Perfect Round-Trips:** XX/100 (XX%)
**Test Date:** YYYY-MM-DD

### Failures (if any)

[Use failure analysis template above]

### Notes

[Any format-specific observations about data preservation]
```

---

## 4. Failure Modes Testing

**Before** running performance benchmarks, each format must be tested for robustness against invalid or edge-case input:

### 4.1 Error Handling Protocol

| Scenario | Test Input | Expected Behavior |
|----------|-----------|-------------------|
| **Truncated record** | Incomplete serialized record | Graceful error with clear message, no panic |
| **Invalid tag** | Tag value "99A" or empty string | Rejected with validation error |
| **Oversized field** | Field >9999 bytes | Either preserved exactly OR rejected with clear error |
| **Invalid indicator** | Non-ASCII character as indicator | Rejected with validation error |
| **Null subfield value** | Subfield with null pointer | Handled consistently (error or serialize as empty) |
| **Malformed UTF-8** | Invalid UTF-8 byte sequence in text | Rejected with clear error message |
| **Missing leader** | Record without 24-char leader | Rejected with validation error |

### 4.2 Reporting Format

Document in evaluation report:

```markdown
## 3. Failure Modes

| Scenario | Result | Notes |
|----------|--------|-------|
| Truncated record | ✓ Graceful error | "Unexpected end of data" |
| Invalid tag | ✓ Validation error | Rejected at deserialization |
| Oversized field | ✓ Error | Rejected (limit enforced) |
| ... | ... | ... |

**Overall Assessment:** [Format handles errors gracefully / Format panics on invalid input / etc.]
```

---

## 5. Performance Benchmark Protocol

### 5.1 Test Environment

Document:
- CPU model and cores
- RAM
- Storage type (SSD/HDD)
- OS version
- **Rust version** (primary) and rustc optimization level (-O for release)
- Format library version (Rust crate)
- Python version and libraries (if evaluating multi-language support, secondary)

### 5.2 Metrics to Measure

| Metric | Unit | Description |
|--------|------|-------------|
| **Read Throughput** | records/sec | Deserialize from format to `MarcRecord` objects |
| **Write Throughput** | records/sec | Serialize from `MarcRecord` objects to format |
| **File Size (raw)** | bytes | Output file size without compression |
| **File Size (gzip)** | bytes | Compressed with `gzip -9` |
| **Compression Ratio** | % | `1 - (format_size / iso2709_size)` |
| **Peak Memory** | MB | Maximum RSS during operation |

Benchmarks should run single-threaded to ensure comparable results across formats.

### 5.3 Baseline Measurement (ISO 2709)

Before evaluating other formats, establish the ISO 2709 baseline:

```
Test Date: YYYY-MM-DD
Environment: [CPU, RAM, OS, Rust version]

Metric | Value
-------|-------
Read (rec/sec) | X
Write (rec/sec) | Y
File Size (10k records) | Z bytes
Gzip Compressed | W bytes
Peak Memory | V MB
```

Store this baseline result permanently in `BASELINE_ISO2709.md` for all subsequent format comparisons.

### 5.4 Benchmark Procedure (Rust Primary)

**Rust benchmarks using Criterion.rs:**

```bash
# Build release binary with optimizations
cargo build --release

# Run Criterion benchmarks
cargo bench --bench format_benchmarks -- [format_name] --baseline iso2709

# For manual timing:
# Warm-up: 3 iterations (discarded)
# Measured: 10 iterations (averaged)
hyperfine --warmup 3 --runs 10 'target/release/mrrc_bench_read --format {fmt}'
```

**Python secondary (if applicable):**
- Only after Rust primary evaluation is complete
- Document with lower priority than Rust metrics

### 5.5 Baseline Comparison

All metrics compared against ISO 2709 baseline:

| Operation | ISO 2709 Baseline | Format Result | Delta |
|-----------|-------------------|---------------|-------|
| Read | X rec/sec | Y rec/sec | +/-Z% |
| Write | X rec/sec | Y rec/sec | +/-Z% |
| Size | X bytes | Y bytes | +/-Z% |

### 5.6 Reporting Table

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

## 6. Integration Assessment Criteria

### 6.1 Dependency Analysis (Rust Focus)

Evaluate the cost of integrating the format library into mrrc's Rust implementation:

| Factor | Guidance |
|--------|----------|
| **Rust external deps** | Count Cargo crate dependencies (direct + transitive). Fewer is better. |
| **Dep health** | Rate each Rust dependency: actively maintained? Security advisories? Recent commits? |
| **Build complexity** | Simple `cargo add` (score 1) vs complex build scripts/native compilation (score 5) |
| **License compatibility** | Must be compatible with Apache 2.0 (mrrc's license) |
| **Compile time impact** | Measure incremental build time; Rust compile time matters for CI/iteration speed |

### 6.2 Language Support Matrix

**Priority order for mrrc:**

| Language | Library | Maturity | Priority | Notes |
|----------|---------|----------|----------|-------|
| **Rust** | crate_name | ⭐⭐⭐⭐⭐ | **PRIMARY** | Must have mature Rust implementation |
| Python | package_name | ⭐⭐⭐⭐ | Secondary | For PyO3 bindings (if format recommended) |
| Java | package_name | ⭐⭐⭐ | Tertiary | Ecosystem context only |
| Go | package_name | ⭐⭐ | Tertiary | Ecosystem context only |
| C++ | library_name | ⭐⭐⭐ | Tertiary | Ecosystem context only |

### 6.3 Schema Evolution Score

| Capability | Score |
|------------|-------|
| No schema evolution | 1 |
| Append-only evolution | 2 |
| Backward compatible | 3 |
| Forward compatible | 4 |
| Full bi-directional | 5 |

### 6.4 Ecosystem Maturity

- [ ] Production use cases documented
- [ ] Active maintenance (commits in last 6 months)
- [ ] Security advisories process
- [ ] Stable API (1.0+ release)
- [ ] Good documentation
- [ ] Community size / adoption

---

## 7. Use Case Fit Scoring

Rate each format 1-5 for each use case:

| Use Case | Score | Notes |
|----------|-------|-------|
| **Simple data exchange** | | API integration, file transfer |
| **High-performance batch** | | Large-scale processing |
| **Analytics/big data** | | Spark, Hadoop, Parquet ecosystem |
| **API integration** | | REST/gRPC services |
| **Long-term archival** | | 10+ year preservation |

---

## 8. Evaluation Report Template

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

## 9. Comparison Matrix Template

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
| 2026-01-12 | 1.1 | Clarify encoding assumptions: mrrc normalizes to UTF-8, formats don't handle MARC-8; remove startup cost metric; simplify use cases |
| 2026-01-13 | 1.2 | **Correctness improvements:** Add explicit equality semantics; add failure modes testing; establish ISO 2709 baseline requirement; add synthetic worst-case records to test set; clarify leader position handling; add failure investigation checklist |
| 2026-01-13 | 1.5 | **Ordering emphasis:** Elevated field & subfield ordering to CRITICAL in validation table; reorganized edge cases checklist to highlight ordering; enhanced failure checklist with field/subfield reordering detection; clarified correctness spec that field/subfield sequence must be preserved exactly |
| 2026-01-13 | 2.0 | **Rust-first focus:** Reframed evaluations to prioritize Rust implementation for mrrc core; updated benchmark procedure to use Rust (Criterion.rs); added Rust dependency analysis emphasis; clarified language support matrix with priority tiers (Rust primary, Python secondary, others tertiary); updated environment documentation to emphasize Rust version and optimization level |
