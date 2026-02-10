# Fidelity Test Set Specification

This document specifies the requirements for the 100-record fidelity test set used in format evaluation round-trip testing.

## File Location

`tests/data/fixtures/fidelity_test_100.mrc`

## Purpose

Validate that a serialization format can perfectly round-trip MARC records without any data loss or transformation artifacts.

### Encoding Note

This test set intentionally includes both MARC-8 and UTF-8 encoded ISO 2709 source records to exercise mrrc's import pipeline. During binary format evaluations:

- mrrc decodes all records and normalizes them to **UTF-8 `MarcRecord` objects** before passing to candidate formats
- Candidate binary formats are **not required to understand MARC-8** — they only see normalized UTF-8 content
- MARC-8 handling is tested separately in mrrc's import/export layer

## Composition

| Category | Count | Description |
|----------|-------|-------------|
| **Bibliographic - Books** | 20 | Monographs (BKS format) |
| **Bibliographic - Serials** | 10 | Continuing resources (SER format) |
| **Bibliographic - Music** | 5 | Scores and sound recordings (REC format) |
| **Bibliographic - Maps** | 5 | Cartographic materials (MAP format) |
| **Bibliographic - Visual** | 5 | Visual materials (VIS format) |
| **Bibliographic - Mixed** | 5 | Mixed materials (MIX format) |
| **Authority - Personal** | 10 | Personal name headings |
| **Authority - Corporate** | 8 | Corporate body headings |
| **Authority - Subject** | 7 | Subject headings |
| **Holdings** | 15 | Holdings and item records |
| **Edge Cases** | 15 | Encoding, sizes, structure, validation boundaries |
| **Total** | 105 | |

## Edge Case Requirements

The 15 edge case records MUST include (covering encoding, sizing, structure, and validation):

### Character/Encoding Edge Cases (3 records)

1. **Combining diacritics** — Record with combining diacritical marks (à, é, ñ, etc.) — sourced from MARC-8 to exercise normalization
2. **CJK characters** — Record with Chinese, Japanese, or Korean characters
3. **RTL scripts** — Record with Arabic or Hebrew text (right-to-left)

### Size Edge Cases (3 records)

4. **Maximum field length** — Field approaching 9999 byte limit
5. **Many fields** — Record with 100+ fields
6. **Many subfields** — Single field with 50+ subfields

### Structural Edge Cases (4 records)

7. **Empty subfield values** — Subfield with empty string value
8. **Repeating subfields** — Field with multiple $a subfields
9. **Control characters** — Data containing ASCII control characters
10. **Blank vs missing indicators** — Record demonstrating indicator edge cases

## Diversity Requirements

### Leader Coverage

- Multiple record status codes (n, c, d, p)
- Multiple encoding levels (blank, 1, 2, 3, 4, 5, 7, 8)
- Multiple descriptive cataloging forms

### Field Coverage

Must include examples of:

- Control fields (001-009)
- All common bibliographic fields (1XX, 2XX, 3XX, 4XX, 5XX, 6XX, 7XX, 8XX)
- Local fields (9XX)
- Holdings-specific fields (852, 853, 863, etc.)
- Authority-specific fields (1XX, 4XX, 5XX for authority)

### Source Encoding Coverage

To exercise mrrc's normalization pipeline, the source ISO 2709 files should include:

- ~20 records originally encoded in MARC-8 (mrrc normalizes these to UTF-8)
- ~50 records originally encoded in UTF-8
- Records with 066/880 encoding declarations (alternate script representations)

Note: Binary formats only see the normalized UTF-8 output from mrrc.

## Selection Criteria

Records should be selected from real-world MARC sources to ensure realistic data patterns:

- Library of Congress sample records
- OCLC WorldCat samples
- Diverse language coverage (at minimum: English, Spanish, German, French, Chinese, Arabic)

## Validation Script

A validation script should verify the test set meets all requirements:

```bash
# Verify record count
# Verify format distribution
# Verify encoding distribution
# Verify edge case presence
```

## Synthetic Worst-Case Records

The 10 edge case records MUST include synthetic records deliberately testing format boundaries. These are not real-world examples but engineering tests designed to break unprepared implementations.

### Size Boundaries (test field limit handling)

| Record | Size Test | Purpose |
|--------|-----------|---------|
| EC-01 | Single-char subfield | Minimum: $a "x" in 245 field |
| EC-02 | 9998-byte field (1 byte under MARC limit) | Maximum: Field data of exactly 9998 bytes |
| EC-03 | 255+ subfields in single field | Many: Single 999 field with 255 subfields |
| EC-04 | 500+ fields in record | Many: Record with 500 fields (unusual but valid) |
| EC-04b | Whitespace: leading/trailing spaces in $a | Preservation: $a "  text  " not collapsed or trimmed |

### Character/Encoding Boundaries (test text handling)

| Record | Character Test | Purpose |
|--------|----------------|---------|
| EC-05 | Unicode 0xFFFD + combining marks | Character boundary: Invalid Unicode + valid combining |
| EC-06 | 10+ consecutive combining marks | Diacritics: "e\u0301\u0301\u0301..." (NOT precomposed é) |
| EC-07 | Arabic + English + Hebrew in single 650 field | Mixed script: RTL + LTR + RTL in same subfield |

### Structural Boundaries (test MARC structure)

| Record | Structure Test | Purpose |
|--------|---|---------|
| EC-08 | Multiple 245 fields (semantically invalid) | Order: Verify all fields preserved in exact input order, not deduplicated or reordered |
| EC-09 | All blank indicators vs mixed blank/filled | Indicator: Space (U+0020) vs null in all positions |
| EC-10 | Empty string in 650$a (not repeating, genuinely empty) | Empty value: $a "" distinct from missing $a |
| EC-11 | Field reordering (001, 650, 245, 001 sequence) | Order: Test that field order is preserved exactly, **NOT reordered alphabetically/numerically** |
| EC-12 | Subfield code ordering ($d$c$a, not reordered to $a$c$d) | Subfield order: Test that subfield sequence is preserved exactly, **NOT reordered to $a$c$d** |
| EC-13 | Control field (001) with exactly 12 chars | Control field: Test that 001 data is preserved exactly, no truncation |
| EC-14 | Repeating control field (multiple 001 fields—invalid) | Validation: Test graceful error handling on invalid duplicate control field (should error, not crash) |
| EC-15 | Invalid subfield codes ("0", space, "$") in data | Validation: Test graceful error handling on non-alphanumeric subfield codes (should error, not crash) |

## Creation & Validation

### Creation Status

**Status:** In progress / Not yet created (select one)

**TODO:**
1. [ ] Source ~70 diverse MARC records from:
   - Library of Congress sample records
   - OCLC WorldCat samples
   - Diverse language coverage (English, Spanish, German, French, Chinese, Arabic)
2. [ ] Create 10 synthetic edge case records with checklist coverage
3. [ ] Assemble into ISO 2709 file with composition as specified
4. [ ] Run validation script (see below)
5. [ ] Place at `tests/data/fixtures/fidelity_test_100.mrc`

### Validation Script

Create script `scripts/validate_fidelity_test.sh` to verify:

```bash
#!/bin/bash
# Input: fidelity_test_100.mrc
# Output: report with pass/fail for each requirement

# Count records by type
# Verify encoding distribution (MARC-8 vs UTF-8)
# Verify edge case presence (combining marks, CJK, RTL, etc.)
# Verify field type coverage (001-009, 1XX, 2XX, etc.)
# Verify leader position diversity
# Verify max field size is present
# Verify max subfield count is present
# Report: PASS or list missing requirements
```

## Related Issues

- **Framework:** mrrc-fks.8 — Defines test set requirements and correctness semantics
- **Usage:** All mrrc-fks.1 through mrrc-fks.7, mrrc-fks.10 evaluations
