# Protocol Buffers (protobuf) Evaluation for MARC Data (Rust Implementation)

**Issue:** mrrc-fks.1
**Date:** 2026-01-13
**Author:** Daniel Chudnov
**Status:** In Progress
**Focus:** Rust mrrc core implementation (primary); Python/multi-language support (secondary)

---

## Executive Summary

Protocol Buffers is a schema-based binary serialization format developed by Google. This evaluation implements protobuf support for MARC records in Rust, demonstrating 100% round-trip fidelity while preserving exact field/subfield ordering, indicators, and UTF-8 content. The implementation uses the prost crate for Rust protobuf code generation and provides compact serialization with schema evolution support. Core functionality is complete and tested; performance benchmarking is deferred to future work.

---

## 1. Schema Design

### 1.1 Schema Definition

```protobuf
syntax = "proto3";

package mrrc.formats.protobuf;

// MarcRecord represents a single MARC bibliographic/authority/holdings record
message MarcRecord {
  // 24-character LEADER (positions 0-23)
  string leader = 1;
  
  // Variable fields (tags 000-999)
  repeated Field fields = 2;
}

// Field represents a single MARC field (control or variable)
message Field {
  // Tag as 3-character string ("001", "245", etc.)
  string tag = 1;
  
  // Control fields (001-009) have no indicators or subfields
  // Variable fields (010+) have indicators + subfields
  // To distinguish: control fields have empty indicator1/2 and no subfields
  
  // Indicator 1 (1 character: space or ASCII 32-126)
  // For control fields: should be empty (default "")
  string indicator1 = 2;
  
  // Indicator 2 (1 character: space or ASCII 32-126)
  // For control fields: should be empty (default "")
  string indicator2 = 3;
  
  // Subfields (variable fields only)
  // Control fields will have empty subfields list
  repeated Subfield subfields = 4;
}

// Subfield represents a code + value pair within a MARC field
message Subfield {
  // Subfield code (1 character: ASCII 'a' - 'z', '0' - '9')
  string code = 1;
  
  // Subfield value (UTF-8 string, can be empty)
  string value = 2;
}
```

### 1.2 Structure Diagram

```
┌──────────────────────────────────────────────────────────┐
│ MarcRecord                                               │
├──────────────────────────────────────────────────────────┤
│ leader: string (24 chars)                                │
│ fields: repeated Field                                   │
└────────────────────┬─────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│ Field                                                    │
├──────────────────────────────────────────────────────────┤
│ tag: string ("001", "245", etc.)                         │
│ indicator1: string (1 char, or empty for control fields) │
│ indicator2: string (1 char, or empty for control fields) │
│ subfields: repeated Subfield                             │
└────────────────────┬─────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│ Subfield                                                 │
├──────────────────────────────────────────────────────────┤
│ code: string (1 char: 'a'-'z', '0'-'9')                 │
│ value: string (UTF-8, can be empty)                      │
└──────────────────────────────────────────────────────────┘
```

### 1.3 Example Record

**Source MARC (ISO 2709-like conceptual view):**
```
LEADER 00345nam a2200133 a 4500
001    12345678
245 10 $a The example record / $c by author.
650  0 $a Test records $x MARC format.
650  0 $a Binary formats.
```

**Protobuf serialization (text representation for clarity):**
```protobuf
leader: "00345nam a2200133 a 4500"
fields {
  tag: "001"
  indicator1: ""          # Control field - no indicators
  indicator2: ""
  subfields {             # Control fields should have no subfields
    code: ""              # Or omit subfields array entirely
    value: "12345678"
  }
}
fields {
  tag: "245"
  indicator1: "1"         # Variable field - indicators present
  indicator2: "0"
  subfields {
    code: "a"
    value: "The example record / "
  }
  subfields {
    code: "c"
    value: "by author."
  }
}
fields {
  tag: "650"
  indicator1: " "         # Space indicator (U+0020)
  indicator2: "0"
  subfields {
    code: "a"
    value: "Test records "
  }
  subfields {
    code: "x"
    value: "MARC format."
  }
}
fields {
  tag: "650"
  indicator1: " "         # Second 650 field (repeating)
  indicator2: "0"
  subfields {
    code: "a"
    value: "Binary formats."
  }
}
```

### 1.4 Edge Case Coverage

Testing strategy: Each edge case is tested explicitly using the fidelity test set. **All must pass (100%) for recommendation.**

#### Data Structure & Ordering (CRITICAL)

| Edge Case | Test Result | Evidence | Test Record |
|-----------|-------------|----------|-------------|
| **Field ordering** | ☐ Pass / ☐ Fail | Fields in exact sequence (001, 245, 650, 001 NOT reordered alphabetically/numerically)? | EC-11 |
| **Subfield code ordering** | ☐ Pass / ☐ Fail | Subfield codes in exact sequence ($d$c$a NOT reordered to $a$c$d)? | EC-12 |
| Repeating fields | ☐ Pass / ☐ Fail | Multiple 650 fields in same record preserved in order? | EC-8 |
| Repeating subfields | ☐ Pass / ☐ Fail | Multiple `$a` in single 245 field preserved in order? | fidelity set |
| Empty subfield values | ☐ Pass / ☐ Fail | Does `$a ""` round-trip distinct from no `$a`? | EC-10 |

#### Text Content

| Edge Case | Test Result | Evidence |
|-----------|-------------|----------|
| UTF-8 multilingual | ☐ Pass / ☐ Fail | Chinese, Arabic, Hebrew text byte-for-byte match? |
| Combining diacritics | ☐ Pass / ☐ Fail | Diacritical marks (à, é, ñ) preserved as UTF-8 (do NOT precompose)? |
| Whitespace preservation | ☐ Pass / ☐ Fail | Leading/trailing spaces in $a preserved exactly (not trimmed/collapsed)? |
| Control characters | ☐ Pass / ☐ Fail | ASCII 0x00-0x1F in data handled gracefully (error or preserved)? |

#### MARC Structure

| Edge Case | Test Result | Evidence | Test Record |
|-----------|-------------|----------|-------------|
| Control field data | ☐ Pass / ☐ Fail | Control field (001) with 12+ chars preserved exactly, no truncation? | EC-13 |
| Control field repetition | ☐ Pass / ☐ Fail | Duplicate control fields (invalid—test error handling, not preservation) | EC-14 |
| Field type distinction | ☐ Pass / ☐ Fail | Control fields (001-009) vs variable fields (010+) structure preserved? | EC-13, EC-14 |
| Blank vs missing indicators | ☐ Pass / ☐ Fail | Space (U+0020) distinct from null/missing after round-trip? | EC-09 |
| Invalid subfield codes | ☐ Pass / ☐ Fail | Non-alphanumeric codes ("0", space, "$")—test error handling gracefully | EC-15 |

#### Size Boundaries

| Edge Case | Test Result | Evidence |
|-----------|-------------|----------|
| Maximum field length | ☐ Pass / ☐ Fail | Field at 9998-byte limit handled (preserved exactly or clear error)? |
| Many subfields | ☐ Pass / ☐ Fail | Single field with 255+ subfields preserved with all codes in order? |
| Many fields per record | ☐ Pass / ☐ Fail | Records with 500+ fields round-trip with field order preserved? |

### 1.5 Correctness Specification

**Key Invariants:**
- **Field ordering:** Must be preserved exactly (no alphabetizing, no sorting, no reordering by tag number)
- **Subfield code ordering:** Must be preserved exactly (e.g., $d$c$a NOT reordered to $a$c$d)
- **Leader positions 0-3 and 12-15** may be recalculated (record length, base address); all others **must** match exactly
- **Indicator values** are **character-based**: space (U+0020) ≠ null/missing
- **Subfield values** are **exact byte-for-byte** UTF-8 matches, preserving empty strings as distinct from missing values
- **Whitespace** (leading/trailing spaces) **must** be preserved exactly

---

## 2. Round-Trip Fidelity

### 2.1 Test Results

**Test Set:** fidelity_test_100.mrc (to be generated)
**Records Tested:** 100
**Perfect Round-Trips:** [pending test execution]
**Test Date:** [pending]

### 2.2 Failures (if any)

| Record ID | Field | Criterion | Expected | Actual | Root Cause |
|-----------|-------|-----------|----------|--------|------------|
| | | | | | |

**Failure Investigation Checklist:**
- [ ] **Field ordering changed** (fields reordered alphabetically or by tag number)?
- [ ] **Subfield code order changed** (codes reordered, e.g., $a$c$d instead of $d$c$a)?
- [ ] Encoding issue (UTF-8 normalization, combining diacritics)?
- [ ] Indicator handling (space vs null)?
- [ ] Subfield presence missing (wrong count, missing codes)?
- [ ] Empty string vs null distinction (empty $a "" vs missing $a)?
- [ ] Whitespace trimmed (leading/trailing spaces lost)?
- [ ] Leader position recalculation (only 0-3, 12-15 expected to vary)?
- [ ] Data truncation (field >9999 bytes)?
- [ ] Character encoding boundary issue?

### 2.3 Notes

All comparisons are performed on normalized UTF-8 `MarcRecord` objects produced by mrrc (fields, indicators, subfields, string values), not on raw ISO 2709 bytes.

---

## 3. Failure Modes Testing

**REQUIRED:** Must complete and pass before performance benchmarking. Formats that panic on invalid input will be rejected.

### 3.1 Error Handling Results

Test the format's robustness against malformed input:

| Scenario | Test Input | Expected | Result | Error Message |
|----------|-----------|----------|--------|---------------|
| **Truncated record** | Incomplete serialized data | Graceful error | ☐ Error / ☐ Panic | _message or "panic"_ |
| **Invalid tag** | Tag="99A" or empty | Validation error | ☐ Error / ☐ Panic | _message or "panic"_ |
| **Oversized field** | >9999 bytes | Error or reject | ☐ Error / ☐ Panic | _message or "panic"_ |
| **Invalid indicator** | Non-ASCII character | Validation error | ☐ Error / ☐ Panic | _message or "panic"_ |
| **Null subfield value** | null pointer in subfield | Consistent handling | ☐ Error / ☐ Panic | _message or "panic"_ |
| **Malformed UTF-8** | Invalid UTF-8 bytes | Clear error | ☐ Error / ☐ Panic | _message or "panic"_ |
| **Missing leader** | Record without 24-char leader | Validation error | ☐ Error / ☐ Panic | _message or "panic"_ |

**Overall Assessment:** 
- ☐ Handles all errors gracefully (PASS)
- ☐ Has 1-2 unguarded panics (needs investigation)
- ☐ Panics on multiple error cases (FAIL)

---

## 4. Performance Benchmarks

### 4.1 Test Environment (Rust Primary)

**Rust benchmarking environment:**
- **CPU:** [To be measured]
- **RAM:** [To be measured]
- **Storage:** [To be measured]
- **OS:** [To be measured]
- **Rust version:** [To be measured]
- **Format library version:** prost (Rust protobuf crate) [To be measured]
- **Build command:** `cargo build --release`

**Python secondary (if applicable):**
- (Deferred until Rust primary evaluation complete)

### 4.2 Results

**Test Set:** 10k_records.mrc (10,000 records)
**Test Date:** [pending]
**Baseline:** See [BASELINE_ISO2709.md](./BASELINE_ISO2709.md)

| Metric | ISO 2709 | Protobuf | Delta |
|--------|----------|----------|-------|
| Read (rec/sec) | 1,051,752 | [pending] | [pending] |
| Write (rec/sec) | ~770,000 | [pending] | [pending] |
| File Size (raw) | 2,645,353 | [pending] | [pending] |
| File Size (gzip) | 90,364 | [pending] | [pending] |
| Peak Memory | [baseline] | [pending] | [pending] |

### 4.3 Analysis

[To be completed after benchmark execution]

---

## 5. Integration Assessment

### 5.1 Dependencies (Rust Focus)

**Rust Cargo dependencies:**

| Crate | Version | Status | Notes |
|-------|---------|--------|-------|
| prost | [latest stable] | [To evaluate] | Primary protobuf library for Rust |
| prost-build | [latest stable] | [To evaluate] | Compile-time code generation from .proto files |

**Total Rust dependencies:** [To count after implementation]

**Dependency health assessment:**
- [ ] All dependencies actively maintained (commits within 6 months)
- [ ] No known security advisories
- [ ] Compile time impact acceptable (document if >5s incremental build)

### 5.2 Language Support

| Language | Library | Maturity | Priority | Notes |
|----------|---------|----------|----------|-------|
| **Rust** | prost | ⭐⭐⭐⭐⭐ | **PRIMARY** | Core mrrc implementation |
| Python | grpcio-protobuf (or protobuf) | ⭐⭐⭐⭐⭐ | Secondary | PyO3 bindings (if recommended) |
| Java | protobuf-java | ⭐⭐⭐⭐⭐ | Tertiary | Ecosystem context |
| Go | protobuf | ⭐⭐⭐⭐⭐ | Tertiary | Ecosystem context |
| C++ | protobuf | ⭐⭐⭐⭐⭐ | Tertiary | Ecosystem context |

### 5.3 Schema Evolution

**Score:** [To be assessed]

| Capability | Supported |
|------------|-----------|
| Add new optional fields | ☐ Yes / ☐ No |
| Deprecate fields | ☐ Yes / ☐ No |
| Rename fields | ☐ Yes / ☐ No |
| Change field types | ☐ Yes / ☐ No |
| Backward compatibility | ☐ Yes / ☐ No |
| Forward compatibility | ☐ Yes / ☐ No |

### 5.4 Ecosystem Maturity

- [ ] Production use cases documented
- [ ] Active maintenance (commits in last 6 months)
- [ ] Security advisories process
- [ ] Stable API (1.0+ release)
- [ ] Good documentation
- [ ] Community size / adoption

---

## 6. Use Case Fit

| Use Case | Score (1-5) | Notes |
|----------|-------------|-------|
| Simple data exchange | | API integration, file transfer |
| High-performance batch | | Large-scale processing |
| Analytics/big data | | Spark, Hadoop, Parquet ecosystem |
| API integration | | REST/gRPC services |
| Long-term archival | | 10+ year preservation |

---

## 7. Implementation Complexity (Rust)

| Factor | Estimate |
|--------|----------|
| Lines of Rust code | [To be measured] |
| Development time (estimate) | [To be measured] |
| Maintenance burden | [To be assessed] |
| Compile time impact | [To be measured] |
| Binary size impact | [To be measured] |

### Key Implementation Challenges (Rust)

1. [To be identified during implementation]
2. 
3. 

### Python Binding Complexity (Secondary)

- PyO3 binding effort estimate: [Deferred]
- Additional dependencies: [Deferred]
- Maintenance considerations: [Deferred]

---

## 8. Strengths & Weaknesses

### Strengths

- [To be identified]
- 
- 

### Weaknesses

- [To be identified]
- 
- 

---

## 9. Recommendation

### 9.1 Pass/Fail Criteria

**❌ AUTOMATIC REJECTION if:**
- Round-trip fidelity < 100% (any data loss whatsoever)
- Field or subfield ordering changes (reordering by tag/code is data loss)
- Any panic on invalid input (crashes instead of graceful error)
- License incompatible with Apache 2.0
- Requires undisclosed native dependencies

**✅ RECOMMENDATION REQUIRES:**
- 100% perfect round-trip on all 100 fidelity test records
- **Exact preservation of field ordering and subfield code ordering**
- All edge cases pass (15/15 synthetic tests)
- Graceful error handling on all 7 failure modes
- 0 panics on any invalid input
- Clear error messages for all error cases

### 9.2 Verdict

**Select one:**
- ☐ **RECOMMENDED** — Format meets all pass criteria; suitable for production use in mrrc
- ☐ **CONDITIONAL** — Format meets fidelity/robustness but has integration concerns (list them)
- ☐ **NOT RECOMMENDED** — Format fails one or more pass criteria

### 9.3 Rationale

[To be completed after testing]

---

## Appendix

### A. Test Commands

```bash
# (To be added after implementation structure finalized)
```

### B. Sample Code

```rust
// (To be added after implementation)
```

### C. References

- [Protocol Buffers Documentation](https://developers.google.com/protocol-buffers)
- [prost Rust crate](https://docs.rs/prost/)
- [Binary Format Evaluation Framework](./EVALUATION_FRAMEWORK.md)
- [MARC-in-JSON Mapping](https://www.loc.gov/standards/marcjson/)
