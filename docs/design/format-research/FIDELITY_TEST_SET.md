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
| **Edge Cases** | 10 | Encoding, sizes, special characters |
| **Total** | 100 | |

## Edge Case Requirements

The 10 edge case records MUST include:

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

## Creation Status

**Status:** Not yet created

**TODO:**
1. Source diverse MARC records from public datasets
2. Select records meeting composition requirements
3. Add synthetic edge case records as needed
4. Validate against specification
5. Place at `tests/data/fixtures/fidelity_test_100.mrc`

## Related Issues

- **Framework:** mrrc-fks.8 — Defines test set requirements
- **Usage:** All mrrc-fks.1 through mrrc-fks.7, mrrc-fks.10 evaluations
