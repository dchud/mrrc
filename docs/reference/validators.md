# Validators

mrrc ships four named validator types alongside its main parsing
machinery. Two of them — `IndicatorValidator` and
`RecordStructureValidator` — also run automatically inside the parser
when `validation_level="strict_marc"`; the other two are user-callable
helpers that the parser does not invoke for you.

This page documents each validator's intended use, error surface, and
relationship (if any) to the orthogonal `validation_level` axis. For
the broader axis, see [Validation level vs recovery
mode](error-handling.md#validation-level-vs-recovery-mode).

## Two roles for a validator

Validators in mrrc fill one of two roles:

- **Format-semantic** — a check that belongs to "is this a valid MARC 21
  record at all?" Examples: per-tag indicator rules from MARC 21,
  leader-byte semantics. mrrc runs these automatically at
  `validation_level="strict_marc"`. They're **also** exposed as public
  validator types, so you can call them directly on records you've
  built yourself or want to re-check.
- **Content/heuristic** — a check that inspects the *contents* of
  fields rather than the record's structural conformance. Examples:
  an ISBN checksum, a heuristic estimate of whether a record's data
  bytes match its declared encoding. mrrc never runs these
  automatically — they're opt-in.

## Format-semantic (auto-run at `strict_marc`)

### `IndicatorValidator`

Per-tag MARC 21 indicator rules — e.g., `245` first indicator must be
`0` or `1`, `100` first indicator must be `0`/`1`/`3`,
`130` first indicator is the digit count of nonfiling characters
(`0`-`9`).

When `validation_level="strict_marc"`, the parser checks both the
universal byte rule (must be ASCII digit or space) and the per-tag
semantic rule for every data field. Violations surface as
[E201 `invalid_indicator`](error-codes.md#E201) with an `expected:`
string that names the per-tag rule.

You can also call it directly:

```rust
use mrrc::IndicatorValidator;

let v = IndicatorValidator::new();
v.validate_field(&field)?;                  // by Field
v.validate_indicators("245", '0', '1')?;    // by tag + chars
```

```python
# The Python wrapper does not re-export IndicatorValidator;
# trigger per-tag checks via validation_level="strict_marc".
reader = mrrc.MARCReader(file, validation_level="strict_marc")
```

Tags without an entry in the rule table are accepted regardless of
indicator value (the table covers MARC 21's documented tags).

### `RecordStructureValidator`

MARC 21 leader-byte semantics — `record_status ∈ {a, c, d, n, p}`,
`record_type ∈ {a, c, d, e, f, g, i, j, k, m, o, p, r, t, v, z}`,
`bibliographic_level`, `encoding_level`, `cataloging_form`,
`indicator_count == 2`, `subfield_code_count == 2`, etc.

When `validation_level="strict_marc"`, the parser runs
`validate_leader` automatically after the structural leader checks
(E001/E003/E004) pass. Violations surface as
[E002 `leader_invalid`](error-codes.md#E002) — the same code as the
structural shape, distinguished by message.

You can also call it directly:

```rust
use mrrc::RecordStructureValidator;

RecordStructureValidator::validate_leader(&record.leader)?;
RecordStructureValidator::validate_record(&record)?;
RecordStructureValidator::validate_directory_structure(&record)?;
```

`validate_record` and `validate_directory_structure` are not invoked
by the parser — they're complete-record checks (e.g., "001 is
present", "directory size would fit a five-digit base address"). Use
them after building a record programmatically and before writing it.

## Content/heuristic (opt-in only)

### `IsbnValidator`

ISBN-10 and ISBN-13 checksum verification, plus an `extract_isbns`
helper for pulling identifiers out of a 020 `$a` subfield.

```rust
use mrrc::IsbnValidator;

assert!(IsbnValidator::validate_isbn10("0306406152"));
assert!(IsbnValidator::validate_isbn13("9780306406157"));
let isbns = IsbnValidator::extract_isbns("0306406152 (alk. paper)");
```

This validator inspects subfield *contents*, not record structure.
mrrc deliberately does not run it during parsing: a 020 with a bad
checksum is a data-quality issue, not a MARC-format issue. Run it
yourself when ISBN integrity matters for your pipeline.

### `EncodingValidator`

Heuristic detection of mixed encodings within a single record — e.g.,
a leader that declares UTF-8 but data fields containing MARC-8 escape
sequences, or vice versa.

```rust
use mrrc::{EncodingValidator, EncodingAnalysis};

match EncodingValidator::analyze_encoding(&record)? {
    EncodingAnalysis::Consistent(enc) => { /* OK */ }
    EncodingAnalysis::Mixed { primary, secondary, field_count } => {
        // Some fields look like a different encoding than the leader claims.
    }
    EncodingAnalysis::Undetermined => { /* not enough signal */ }
}
```

The analysis is heuristic — it counts high bytes, escape sequences,
and valid UTF-8 multibyte starts to estimate per-field encoding. mrrc
deliberately does not run it during parsing: it's not deterministic,
and `validation_level="strict_marc"` should fail the same way every
time on the same input. Run `EncodingValidator` yourself when
investigating suspect records or auditing a corpus.

E301 (`utf8_invalid`) is the *deterministic* encoding error wired into
the parser — it fires when bytes flagged for UTF-8 decoding are not
valid UTF-8. `EncodingValidator` is broader: it can flag a record
whose bytes *are* valid UTF-8 but disagree with what the leader
claims.

## See also

- [Error handling](error-handling.md) — `validation_level` vs
  `recovery_mode`, per-record diagnostics.
- [Error codes](error-codes.md) — full reference for each `Exxx`.
