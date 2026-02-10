# Binary Format Evaluation Plan — Revisions Summary

**Date:** 2026-01-13  
**Scope:** Correctness & Rigor Improvements

---

## Overview

Revised the complete binary format evaluation plan to strengthen **correctness validation** and **failure handling** before performance benchmarking. All changes prioritize fidelity and robustness over premature optimization.

---

## Key Changes

### 1. EVALUATION_FRAMEWORK.md

#### ✅ Explicit Correctness Specification (Section 3.2.1)

**Added:**
- Clear equality semantics for round-trip testing
- Byte-for-byte UTF-8 comparison requirements
- Indicator distinction (space vs null/missing)
- Leader position handling rules:
  - Positions 0-4, 12-16: MAY be recalculated
  - All other positions: MUST match exactly
- What is NOT compared (original byte encoding, serialization details)
- Failure threshold: 100% perfect round-trip required

**Prevents:** Ambiguous pass/fail criteria, sneaky data loss via recalculation

---

#### ✅ Failure Analysis Template (Section 3.4)

**Added:**
- Detailed failure investigation checklist
- Root cause analysis for each failure:
  - Encoding normalization issues?
  - Indicator handling?
  - Subfield ordering?
  - Empty string vs null?
  - Leader position recalculation?
  - Type conversion errors?
  - Data truncation?

**Prevents:** Superficial "test failed" reports without diagnostic detail

---

#### ✅ Failure Modes Testing (New Section 4)

**Added before performance benchmarking:**
- 7 error handling scenarios (truncated records, invalid tags, oversized fields, etc.)
- Expected behavior: graceful error with clear message (no panics)
- Reporting template for all error cases
- Overall assessment of format robustness

| Scenario | Test Input | Expected |
|----------|-----------|----------|
| Truncated record | Incomplete serialized record | Graceful error |
| Invalid tag | "99A" or empty | Validation error |
| Oversized field | >9999 bytes | Error or preserved exactly |
| Invalid indicator | Non-ASCII | Validation error |
| Null subfield value | Null pointer | Consistent handling |
| Malformed UTF-8 | Invalid bytes | Clear error message |
| Missing leader | No 24-char leader | Validation error |

**Prevents:** Discovering format panics mid-evaluation, unreliable deserialization

---

#### ✅ ISO 2709 Baseline Establishment (Section 5.3)

**Added requirement:**
- Measure and document ISO 2709 baseline BEFORE evaluating other formats
- Store permanently in `BASELINE_ISO2709.md`
- Use for all subsequent comparative analysis

**Prevents:** Retroactive baseline adjustments, unfair format comparisons

---

#### ✅ Synthetic Worst-Case Records Requirement (Section 2.1)

**Added to test set composition:**
- Must include synthetic records covering size, character, and structural boundaries
- Examples:
  - Field exactly 9998 bytes (1 byte under limit)
  - Record with 255+ subfields in single field
  - Record with 500+ fields total
  - Unicode boundary characters
  - 10+ consecutive combining diacritics
  - Mixed LTR+RTL+LTR text

**Prevents:** Formats passing on "normal" data but failing on edge cases

---

### 2. FIDELITY_TEST_SET.md

#### ✅ Clarified Status & Synchronization

**Changed:**
- Status now clearly states "In progress / Not yet created"
- Cross-references back to EVALUATION_FRAMEWORK.md for requirements
- Links to validation script

**Prevents:** Confusion about whether test set exists

---

#### ✅ Synthetic Worst-Case Records Specification (New section)

**Added explicit requirements:**

**Size Boundaries:**
- Minimum field (single char)
- Maximum field (9998 bytes)
- Many subfields (255+)
- Many fields (500+)

**Character Boundaries:**
- Unicode 0xFFFF boundary
- Combining mark sequences (10+)
- Mixed directionality (LTR+RTL+LTR)
- Control characters (0x00-0x1F)

**Structural Boundaries:**
- Duplicate fields (invalid but test preservation)
- Empty vs filled indicators
- Null bytes in strings

**Prevents:** Test set too simple, missing corner cases that break formats

---

#### ✅ Validation Script Stub

**Added:**
```bash
scripts/validate_fidelity_test.sh
```

Verifies:
- Record count by type
- Encoding distribution
- Edge case presence
- Field type coverage
- Leader position diversity
- Size boundary presence
- Report: PASS or list missing requirements

**Prevents:** Silently using incomplete test set

---

### 3. TEMPLATE_evaluation.md

#### ✅ Enhanced Edge Case Checklist (Section 1.4)

**Improved:**
- Added specific test examples for each edge case
- Clarified what "supported" means (e.g., `$a ""` vs no `$a` distinction)
- Example: "Multiple 650 fields in same record?"

**Prevents:** Evaluators claiming vague "support" without verification

---

#### ✅ Correctness Specification in Template (Section 1.5)

**Added:**
- Key invariants directly in template
- Leader position rules (0-4, 12-16 OK to recalculate; others exact)
- Indicator semantics (character-based, space ≠ null)
- Subfield matching (exact byte-for-byte UTF-8)

**Prevents:** Evaluators using different standards across formats

---

#### ✅ Failure Modes Testing Section (New Section 3)

**Added to template:**
- Error handling results table
- 7 scenarios with pass/fail and error message capture
- Overall robustness assessment
- **Must complete before performance benchmarking**

**Prevents:** Benchmarking unstable formats

---

#### ✅ ISO 2709 Baseline Reference (Section 4.2)

**Added:**
- Link to `BASELINE_ISO2709.md`
- Explicit requirement to document baseline before format testing

**Prevents:** Formats evaluated in vacuum without reference

---

#### ✅ Failure Investigation Checklist (Section 2.2)

**Added:**
- Checkbox list of common failure causes
- Helps evaluators diagnose root causes systematically

**Prevents:** Vague "round-trip failed" notes

---

#### ✅ Mandatory Recommendation Criteria (Section 9)

**Added strict requirements:**
- **Fidelity:** 100% perfect round-trip rate is **mandatory**
- **Error Handling:** Must handle all failure modes gracefully
- Rationale must reference specific fidelity results and failure mode handling

**Prevents:** Recommending formats with known data loss

---

## Testing & Verification

All revisions have been applied and are consistent across documents:

✅ EVALUATION_FRAMEWORK.md — Core methodology with explicit correctness semantics  
✅ FIDELITY_TEST_SET.md — Test set definition with synthetic worst-cases  
✅ TEMPLATE_evaluation.md — Evaluation template enforcing standards

---

## Next Steps

1. **Generate fidelity_test_100.mrc** with validation script
   - Issue: Create this test file with required composition
   
2. **Establish ISO 2709 baseline**
   - Create `BASELINE_ISO2709.md` with measured performance
   
3. **Begin format evaluations**
   - All new evaluations use updated template and framework
   - Failure modes testing required before performance benchmarks

---

## Impact on Evaluations

| Evaluation Phase | Change | Impact |
|-----------------|--------|--------|
| Schema design | More detailed checklist | Clearer requirements |
| Fidelity testing | Explicit equality semantics | Unambiguous pass/fail |
| Failure modes | NEW required section | Catch robustness issues early |
| Performance | Baseline required | Fair comparisons |
| Recommendation | Stricter criteria | Only truly viable formats recommended |

---

## Correctness Philosophy

**Three-layer validation:**

1. **Fidelity:** Can the format preserve MARC data exactly (100% round-trip)?
2. **Robustness:** Does it handle invalid input gracefully (no panics)?
3. **Performance:** Only then measure throughput and size (if 1+2 pass).

This prevents recommending fast formats that lose data or crash.

---

## Clarity Improvements (2026-01-13, v1.3)

After v1.2 (correctness), made additional revisions for clarity and usability:

### README.md
- **Clarified purpose:** Explicitly mention three evaluation layers (fidelity → robustness → performance)
- **Better quick start:** Added numbered steps with explicit cross-references to other docs
- **Test data status:** Clearly marked fidelity_test_100.mrc as TODO, added blocker note

### EVALUATION_FRAMEWORK.md
- **Improved round-trip diagram:** Changed from numbered list to visual flow diagram with "Step 1 → Step 2 → Result"
- **Clearer correctness spec:** Reorganized into three categories (MUST Match, MUST NOT Collapse, MUST NOT Compare) with concrete examples
- **Better integration guidance:** Replaced vague score ranges with clear guidance on what to evaluate (list dependencies, check licenses, etc.)

### TEMPLATE_evaluation.md
- **Edge case scoring:** Changed from vague "Yes/No" to actual test results (Pass/Fail) with evidence column
- **Failure modes table:** Added Test Input + Expected + Error Message columns; removed ambiguity about what passes
- **Recommendation structure:** Added 9.1 Pass/Fail Criteria section showing automatic rejections upfront before verdict
- **Rationale guidance:** Gave evaluators a template for what to discuss in rationale (fidelity, robustness, performance, ecosystem, use cases)

### FIDELITY_TEST_SET.md
- **Synthetic records table:** Converted bullet lists to structured tables with record IDs (EC-01 through EC-10), making it concrete which tests are required
- **Purpose for each test:** Added "Purpose" column explaining why each test exists (e.g., "test Unicode boundary handling")

### General language improvements across all files
- Changed passive to active voice ("must include" → "MUST include")
- Added MUST/MUST NOT/MUST NOT terminology for non-negotiable requirements
- Improved Unicode/diacritics examples with explicit guidance ("do NOT precompose")
- Clarified leader position numbers (positions 0-3 not 0-4, 12-15 not 12-16 for recalculation)
- Added more concrete examples (e.g., "Tag="99A"" instead of "invalid tag")

---

## Edge Case Coverage Expansion (2026-01-13, v1.4)

After v1.3 (clarity), reviewed edge case coverage and added critical gaps:

### EVALUATION_FRAMEWORK.md
- **Reorganized checklist:** Grouped by category (Data Structure, Text Content, Field Structure, MARC-Specific)
- **Field ordering:** Added explicit test for field sequence preservation (001, 650, 245, 001 must not reorder)
- **Subfield ordering:** Added explicit test for subfield code sequence within fields ($d$c$a must not reorder to $a$c$d)
- **Whitespace preservation:** Added test for leading/trailing spaces (NOT trimmed/collapsed)
- **Control field validation:** Added tests for control field (001-009) vs variable field distinction
- **Invalid tag/code handling:** Added tests for "999", non-ASCII subfield codes, empty tags

### FIDELITY_TEST_SET.md
- **Expanded edge cases:** Increased from 10 to 15 synthetic tests (EC-01 through EC-15)
- **New field ordering test (EC-11):** 001, 650, 245, 001 sequence to ensure no reordering
- **New subfield ordering test (EC-12):** $d$c$a sequence to ensure codes preserve order
- **New control field tests (EC-13, EC-14):** Control field data preservation and duplicate detection
- **New validation test (EC-15):** Invalid subfield codes ("0", space, "$") for graceful handling
- **Added whitespace test (EC-04b):** Leading/trailing spaces in subfield values
- **Updated total:** Test set now 105 records (90 real + 15 edge cases)

### TEMPLATE_evaluation.md
- **Reorganized into 4 categories:** Data Structure & Ordering, Text Content, MARC Structure, Size Boundaries
- **Explicit ordering tests:** Field order and subfield code order (major data loss vectors)
- **Control field distinction:** Test that control fields (001-009) vs variable fields (010+) structure is preserved
- **Invalid code handling:** Test for graceful errors on non-alphanumeric subfield codes
- **Mandatory pass rate:** Updated from "9/9" to "15/15" edge cases for recommendation
- **Added emphasis:** "All edge cases must pass (15/15) for recommendation"

### Why These Gaps Matter

| Gap | Data Loss Risk | Severity |
|-----|----------------|----------|
| Field reordering | Tags reordered alphabetically; loses semantic meaning | Critical |
| Subfield reordering | Subfield codes reordered; scrambles field data | Critical |
| Whitespace trimming | Leading/trailing spaces lost silently | High |
| Control field vs variable field | Type distinction lost; deserialization breaks | High |
| Invalid code handling | Crashes instead of graceful error | Medium |

---

## Ordering Emphasis Improvements (2026-01-13, v1.5)

After v1.4, elevated field and subfield ordering to **CRITICAL severity** and made auto-rejection explicit:

### EVALUATION_FRAMEWORK.md
- **Reorganized validation table:** Field ordering and subfield code ordering now marked **Critical** (upgraded from High/unspecified)
- **Reorganized edge cases checklist:** Moved field/subfield ordering to top under "CRITICAL" section header
- **Expanded correctness spec:** Added explicit examples ("001, 245, 650, 001 → NOT reordered") to Section 3.2.1
- **Enhanced failure checklist:** Added field/subfield reordering as first two checklist items for quick diagnosis
- **MUST NOT rules:** Added "Field and subfield sequence must be preserved exactly" as non-negotiable constraint

### README.md
- **Clarified purpose:** Added **"including exact field and subfield ordering"** to round-trip fidelity definition
- **Added constraint statement:** New explicit paragraph: "Formats that reorder fields by tag number or reorder subfield codes are automatically rejected"

### TEMPLATE_evaluation.md
- **Edge case table:** Marked field & subfield ordering tests as CRITICAL in headers and added test record references (EC-11, EC-12)
- **Correctness spec (1.5):** Added field/subfield ordering as first two key invariants with explicit "NOT" examples
- **Failure checklist:** Moved ordering checks to top two positions for visibility
- **Pass/fail criteria (9.1):** Added explicit auto-rejection condition: "Field or subfield ordering changes"
- **Recommendation criteria:** Made ordering preservation a mandatory requirement separate from fidelity percentage

### FIDELITY_TEST_SET.md
- **EC-11 clarification:** Changed purpose to emphasize "NOT reordered alphabetically/numerically" with bold emphasis
- **EC-12 clarification:** Changed purpose to emphasize "NOT reordered to $a$c$d" with bold emphasis
- **EC-14 & EC-15:** Clarified these test **error handling gracefully** on invalid input (not preservation of invalid data)
- **EC-08:** Refocused as ordering test (verify fields preserved in exact order despite semantic invalidity)

### Why Ordering Is CRITICAL

| Aspect | Impact | Consequence |
|--------|--------|-------------|
| Field reordering | Tags 001, 245, 650 → 001, 245, 650 reordered | Semantic meaning lost; catalog records corrupted |
| Subfield reordering | $d (date) $c (place) $a (author) → $a $c $d | Author-date-place becomes author-author-author; comprehension fails |
| Collation ordering | Tags sorted 001→999 | Cross-references and structural dependencies broken |
| Deduplication | Multiple 245 → single 245 | Semantic plural information lost silently |

Formats that reorder without warning are worse than formats that fail loudly (crash/error) because silent data corruption is undetectable.

---

## Summary of All Revisions (v1.0 → v1.5)

| Layer | v1.0 | v1.1 | v1.2 (Correctness) | v1.3 (Clarity) | v1.4 (Edge Cases) | v1.5 (Ordering Emphasis) |
|-------|------|------|------------------|-----------------|------------------|---------------------------|
| **Evaluation order** | Vague | Clear | Explicit (3 layers) | Emphasized | Same | Same |
| **Fidelity rules** | Basic | Same | Detailed spec | Reorganized | Same | **Elevated ordering to CRITICAL** |
| **Error handling** | Not tested | Not tested | NEW section | Improved | Same | Same |
| **Baseline** | Reference | Same | Explicit requirement | Blocker status | Same | Same |
| **Synthetic tests** | Not specified | Not specified | 10 tests (EC-01–EC-10) | Structured tables | 15 tests (EC-01–EC-15) | Clarified intent & auto-rejection |
| **Field ordering** | Uncovered | Uncovered | Not explicit | Not explicit | NEW test (EC-11) | **CRITICAL severity** |
| **Subfield ordering** | Uncovered | Uncovered | Not explicit | Not explicit | NEW test (EC-12) | **CRITICAL severity** |
| **Whitespace handling** | Not tested | Not tested | Not explicit | Not explicit | NEW test | Same |
| **Control field handling** | Not tested | Not tested | Not explicit | Not explicit | NEW tests | Clarified error handling |
| **Pass criteria** | Implicit | Same | Explicit 100% | Obvious | **15/15 required** | **Added ordering auto-rejection** |
| **Evaluator guidance** | Minimal | Limited | Good | Comprehensive | Comprehensive + ordering | **Ordering emphasized throughout** |

