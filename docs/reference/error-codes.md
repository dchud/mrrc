# Error codes

Every error raised by mrrc carries a stable identifier (`Exxx`) and a
human-friendly slug. Match on the code rather than the exception class
name to keep handlers stable across enum restructures, and follow the
help URL to land here.

```python
import mrrc

try:
    list(mrrc.MARCReader.from_path("harvest.mrc"))
except mrrc.MrrcException as e:
    print(e.code, e.slug, e.help_url())
    # E201 invalid_indicator https://mrrc.dev/reference/error-codes/#E201
```

```rust
match err {
    e if e.code() == "E201" => handle_indicator_error(e),
    _ => return Err(e),
}
```

## Configuring the help URL base

By default `err.help_url()` returns a URL anchored to this page hosted on
GitHub Pages (`https://dchud.github.io/mrrc/reference/error-codes/`).
Enterprise deployments that mirror the docs internally can redirect the
help URL by setting the `MRRC_DOCS_BASE_URL` environment variable to
their docs root. Both the Rust core and the Python bindings honor it:

```bash
export MRRC_DOCS_BASE_URL="https://docs.example.com/mrrc"
# err.help_url() → "https://docs.example.com/mrrc/reference/error-codes/#E201"
```

The variable holds the docs site root; the `/reference/error-codes/#Exxx`
path is appended automatically. Trailing slashes are stripped.

## Stability

Two rules, non-negotiable:

1. **Codes never get re-purposed.** A retired check leaves its docs entry
   in place pointing to a replacement.
2. **Codes never get renumbered.** URLs that users paste into chat have
   to keep resolving.

See `CONTRIBUTING.md` for the full policy.

## Code ranges

| Range | Phase |
|---|---|
| `E0xx` | Stream / leader |
| `E1xx` | Directory / field header |
| `E2xx` | Subfield / indicator |
| `E3xx` | Encoding |
| `E4xx` | Serialization / writer |
| `Wxxx` | Warnings (pymarc parity) |

Each range reserves ~80 slots for future growth.

---

## Stream / leader (E0xx)

### E001 — `record_length_invalid` { #E001 }

The leader's record-length field (bytes 0–4) is invalid: not five ASCII
digits, or claims a length below the 24-byte minimum.

**Context:** Parse-side.
**Applies to:** Bibliographic, Authority, Holdings readers.
**Populates:** `record_index`, `byte_offset`. May also populate: `source`.

**Common causes.** Truncated download; reader fed text instead of binary;
attempt to parse a non-MARC file by accident.

**How to recover.** Verify the input is binary MARC (file usually has a
`.mrc` extension and starts with five ASCII digits). No recovery mode
salvages this — the next 24 bytes can't be trusted as a leader.

**Python class:** `mrrc.RecordLengthInvalid`.

### E002 — `leader_invalid` { #E002 }

The 24-byte leader is malformed in a way other than the record-length or
base-address fields. Two failure shapes share this code:

1. **Structural** (always fires): byte-level malformation — e.g., the
   indicator-count byte (position 10) is not an ASCII digit, or the
   reserved bytes 20–23 are not `4500`. Detected during `Leader::from_bytes`
   regardless of `validation_level`.
2. **MARC 21 semantics** (fires only at `validation_level="strict_marc"`):
   the leader parses cleanly but a position carries a value not in the
   MARC 21 allowed set — e.g., `record_status` (position 5) outside
   `{a, c, d, n, p}`, `record_type` (position 6) outside the documented
   set, `indicator_count` not equal to 2, `encoding_level` outside the
   allowed set, etc.

   The MARC 21 allowed-value sets at the per-position level **differ by
   reader type**. The bibliographic, authority, and holdings readers each
   apply their own format's spec at `strict_marc`:

   - **Bibliographic** (`MarcReader`): the union of positions defined in
     the MARC 21 Bibliographic Format leader — positions 5–11 and 17–19.
   - **Authority** (`AuthorityMarcReader`): the MARC 21 Authority Format
     allowed sets. Positions 7 (`bibliographic_level`), 8
     (`control_record_type`), and 19 (`multipart_level`) are
     **undefined** for authority records and accept any byte (including
     the MARC 21 fill character `|`). Position 5 accepts the wider set
     `{a, c, d, n, o, s, x}`, position 6 must be `z`, position 17 must
     be `n` or `o`, and position 18 carries the *punctuation policy*
     allowed set `{space, c, i, u}` instead of the bibliographic
     "cataloging form" set.
   - **Holdings** (`HoldingsMarcReader`): the MARC 21 Holdings Format
     allowed sets. Positions 7, 8, and 19 are undefined as for
     authority. Position 5 accepts `{c, d, n}`, position 6 accepts
     `{u, v, x, y}`, position 17 (encoding level) accepts
     `{1, 2, 3, 4, 5, m, u, z}`, and position 18 (item information in
     record) accepts `{space, i, n}`.

   A leader that is valid per one record type's format may be invalid
   per another's (e.g., `encoding_level='1'` is valid for bibliographic
   but invalid for authority).

**Context:** Parse-side.
**Applies to:** Bibliographic, Authority, Holdings readers.
**Populates:** `record_index`, `byte_offset`, `record_byte_offset` (= 0).
May also populate: `source`, `found`, `expected`.

**Common causes.** Records hand-crafted in a text editor and saved in the
wrong encoding; output from non-conformant exporters; pre-2000s records
using deprecated leader values.

**How to recover.** Structural shape (1) fires before any field parsing
and is not affected by `recovery_mode`. The MARC 21 semantic shape (2)
respects `recovery_mode`: in `lenient`/`permissive` the violation is
attached to the yielded record's `record.errors` list and parsing
continues; in `strict` it raises immediately. To suppress (2) entirely,
use the default `validation_level="structural"`.

**Python class:** `mrrc.RecordLeaderInvalid`.

### E003 — `base_address_invalid` { #E003 }

The leader's base-address-of-data field (bytes 12–16) is not five ASCII
digits or claims a value below 25.

**Context:** Parse-side.
**Applies to:** Bibliographic, Authority, Holdings readers.
**Populates:** `record_index`, `byte_offset`. May also populate: `source`,
`record_control_number`, `found`, `expected`.

**Common causes.** Records written by older systems that miscalculate the
directory length; corrupted bytes 12–16 from in-flight data damage.

**How to recover.** Not recoverable; the directory boundary
can't be inferred without the base address.

**Python class:** `mrrc.BaseAddressInvalid`.

### E004 — `base_address_not_found` { #E004 }

The leader claims a base address of data that exceeds the available bytes
in the input stream.

**Context:** Parse-side.
**Applies to:** Bibliographic, Authority, Holdings readers.
**Populates:** `record_index`, `byte_offset`. May also populate: `source`,
`record_control_number`.

**Common causes.** Truncated input; record header damaged so the
length/base-address pair are inconsistent.

**How to recover.** See [E005](#E005) for the related truncation case.

**Python class:** `mrrc.BaseAddressNotFound`.

### E005 — `truncated_record` { #E005 }

The reader hit EOF before reading the number of bytes the leader claims
the record should contain.

**Context:** Parse-side.
**Applies to:** Bibliographic, Authority, Holdings readers.
**Populates:** `record_index`, `byte_offset`, `record_byte_offset`,
`expected_length`, `actual_length`. May also populate: `source`,
`record_control_number`.

**Common causes.** Network read truncated by connection drop; partially-
written file from a crashed exporter; deliberate fuzzing.

**How to recover.** `recovery_mode="lenient"` salvages whatever fields
parsed cleanly before the truncation point. `recovery_mode="strict"`
raises this error immediately.

**Python class:** `mrrc.TruncatedRecord` (subclass of
`mrrc.EndOfRecordNotFound`).

### E006 — `end_of_record_not_found` { #E006 }

The end-of-record byte (`0x1D`) was not found at the position the leader
implied.

**Context:** Parse-side.
**Applies to:** Bibliographic, Authority, Holdings readers.
**Populates:** `record_index`, `byte_offset`, `record_byte_offset`. May
also populate: `source`, `record_control_number`.

**Common causes.** Concatenated records where one was truncated mid-
stream; corrupted bytes near the end of a record; encoder bugs.

**How to recover.** `recovery_mode="lenient"` accepts the partial record
and continues reading from the next leader.

**Python class:** `mrrc.EndOfRecordNotFound`. Also catches
[E005](#E005) (`TruncatedRecord` is a subclass).

### E007 — `io_error` { #E007 }

An I/O error occurred reading from the underlying source.

**Context:** Parse-side (or anywhere I/O can fail).
**Applies to:** All readers.
**Populates:** `cause` (the underlying `std::io::Error`). May also
populate: `record_index`, `byte_offset`, `source`.

**Common causes.** File permissions; broken pipe; network read failure;
disk error.

**How to recover.** Inspect `e.__cause__` for the underlying I/O kind.
Non-recoverable in general; the caller decides whether to retry.

**Python class:** raised as Python's built-in `OSError` (via `IOError`)
rather than a typed mrrc class — matches pymarc behavior. Catch `OSError`
to handle alongside other I/O errors.

### E099 — `fatal_reader_error` { #E099 }

A fatal condition halted the reader. Currently raised when the
per-stream recovered-error cap is exceeded in `RecoveryMode::Lenient`
or `Permissive` — see [`MarcReader::with_max_errors`][with_max_errors].
The code is reserved for future fatal-reader scenarios as well.

**Context:** Parse-side, lenient/permissive only. Strict mode aborts
on the first error, so this code never fires there.
**Applies to:** `MarcReader`, `AuthorityMarcReader` (the holdings
reader exposes the builder for parity but has no active recovery
sites today).
**Populates:** `cap` (the configured limit) and `errors_seen` (count
at the moment of the trip). May also populate: `record_index`,
`source`.

**Common causes.** Feeding a pathological stream (mostly-malformed
records) through a lenient/permissive reader. Without the cap the
accumulated per-record diagnostics could exhaust memory.

**How to recover.** If the cap is a false positive for the input,
raise it (`reader.with_max_errors(n)` with a larger `n`, or `0` to
disable). If it reflects the actual state of the input, investigate
the source — large counts of recovered errors usually indicate
upstream corruption.

After this error is raised the reader is exhausted; subsequent
`read_record()` calls return `Ok(None)`.

[with_max_errors]: https://docs.rs/mrrc/latest/mrrc/struct.MarcReader.html#method.with_max_errors

---

## Directory / field header (E1xx)

### E101 — `directory_invalid` { #E101 }

A directory entry (12 bytes: 3-byte tag + 4-byte length + 5-byte start
position) is structurally invalid: bad tag bytes, non-numeric length or
start, or claimed field bytes extending past the data area.

**Context:** Parse-side.
**Applies to:** Bibliographic, Authority, Holdings readers.
**Populates:** `record_index`, `byte_offset`, `record_byte_offset`. May
also populate: `field_tag` (when the bad entry's tag was decodable),
`record_control_number`, `source`, `found`, `expected`.

**Common causes.** Encoder bugs; corrupted bytes; legacy records with
non-standard tag formats.

**How to recover.** `recovery_mode="lenient"` skips the bad entry and
continues parsing the rest of the directory.

**Python class:** `mrrc.RecordDirectoryInvalid`. Also catches
[E106](#E106), [E201](#E201), [E202](#E202) (subclasses).

### E105 — `field_not_found` { #E105 }

A requested field was not present in the parsed record. This is an
**accessor error**, not a parse error — it surfaces when code calls e.g.
`record.get_field("245")` and the record doesn't contain that tag.

**Context:** Accessor (post-parse).
**Applies to:** All record types.
**Populates:** `field_tag`. May also populate: `record_control_number`,
`record_index`. **Never populates:** `byte_offset` (not a parse error).

**Common causes.** Calling a `get_field` on records that don't have the
tag; programming error or assumption about input shape.

**How to recover.** Use `try/except` or check `field in record` first.

**Python class:** `mrrc.FieldNotFound`.

### E106 — `invalid_field` { #E106 }

A data field is structurally invalid in a way not covered by the more
specific [E201](#E201) / [E202](#E202) subclasses (e.g., field bytes too
short for indicators, field declared length exceeds available bytes).

**Context:** Parse-side.
**Applies to:** Bibliographic, Authority, Holdings readers.
**Populates:** `record_index`, `byte_offset`, `record_byte_offset`,
`field_tag`. May also populate: `record_control_number`, `source`. The
`message` attribute carries a human-readable description of the problem.

**Common causes.** Encoder dropped subfields; declared field length
inconsistent with actual data.

**How to recover.** `recovery_mode="lenient"` skips the bad field and
continues with the rest.

**Python class:** `mrrc.InvalidField` (subclass of
`mrrc.RecordDirectoryInvalid`).

---

## Subfield / indicator (E2xx)

### E201 — `invalid_indicator` { #E201 }

A variable-data field's indicator byte is not a valid value for the given
tag. Two failure shapes are reported under this code at
`validation_level="strict_marc"`:

1. **Byte-level**: the indicator byte is not an ASCII digit (`0`-`9`) or
   space. `expected` is `"ASCII digit (0-9) or space"`.
2. **Per-tag MARC 21 semantics**: the byte passes (1) but violates the
   per-tag rule for the field. For example, the first indicator of `245`
   (Title statement) is restricted to `0` or `1` per MARC 21; a `9` is
   byte-valid but tag-invalid. `expected` describes the tag-specific rule
   (e.g., `"'0' or '1'"`, `"digit 0-9"`).

The two shapes share an error code because they share a position
(`field_tag`, `indicator_position`) and a remedy (fix the indicator); they
differ only in the `expected` string.

**Context:** Parse-side.
**Applies to:** Bibliographic, Authority, Holdings readers — fired
uniformly when `validation_level="strict_marc"`. At the default
`validation_level="structural"` indicator bytes are accepted as-is and
this code does not fire.
**Populates:** `record_index`, `byte_offset`, `record_byte_offset`,
`field_tag`, `indicator_position`, `found`, `expected`. May also populate:
`record_control_number`, `source`.

**Common causes.** Source systems emitting local-use indicators;
records round-tripped through non-conformant ILSes; sloppy cataloging
from pre-2000s records.

**How to recover.** Use `validation_level="structural"` (default) to
accept the bytes silently, or `recovery_mode="lenient"` to keep iterating
past the offending record under `strict_marc`.

**Python class:** `mrrc.InvalidIndicator` (subclass of
`mrrc.RecordDirectoryInvalid`).

### E202 — `bad_subfield_code` { #E202 }

A subfield code byte (immediately following a `0x1F` delimiter) is not a
printable ASCII character.

**Context:** Parse-side.
**Applies to:** Bibliographic, Authority, Holdings readers — fired
uniformly when `validation_level="strict_marc"`. At the default
`validation_level="structural"` subfield-code bytes are accepted as-is
and this code does not fire.
**Populates:** `record_index`, `byte_offset`, `record_byte_offset`,
`field_tag`, `subfield_code` (the offending byte). May also populate:
`record_control_number`, `source`.

**Common causes.** Bytes corrupted near a subfield boundary; encoder
emitting non-ASCII codes for local-use subfields.

**How to recover.** Use `validation_level="structural"` (default) to
accept the bytes silently, or `recovery_mode="lenient"` to keep iterating
past the offending record under `strict_marc`.

**Python class:** `mrrc.BadSubfieldCode` (subclass of
`mrrc.RecordDirectoryInvalid`).

---

## Encoding (E3xx)

### E301 — `utf8_invalid` { #E301 }

A subfield value or control field contains bytes that are not valid UTF-8.

**Context:** Parse-side (or wherever a string conversion runs).
**Applies to:** Bibliographic, authority, and holdings readers — fired
uniformly when `validation_level="strict_marc"` and a value contains
bytes that aren't valid UTF-8. At the default `validation_level="structural"`
all three readers fall back to lossy decoding (`U+FFFD` substitution)
and don't surface this code.
**Populates:** `record_index`. May also populate: `field_tag`,
`byte_offset`, `source`, `record_control_number`. The `message` attribute
carries the underlying `std::str::Utf8Error` description.

**Common causes.** Records cataloged in MARC-8 encoding without correct
character-coding leader byte; legacy records with embedded byte sequences
that valid in MARC-8 but not in UTF-8.

**How to recover.** Convert input to UTF-8 before parsing, or set
`validation_level="structural"` if you can tolerate `U+FFFD` substitutions
and don't need byte-perfect fidelity.

**Python class:** `mrrc.EncodingError`.

---

## Serialization / writer (E4xx)

### E401 — `marcxml_invalid` { #E401 }

A MARCXML document failed to parse.

**Context:** Parse-side (XML parser layer).
**Applies to:** `mrrc.marcxml_to_record` / `marcxml_to_records`.
**Populates:** `cause` (the underlying `quick_xml` error). May also
populate: `record_index`, `byte_offset` (when the parser exposes a
position), `source`. The `message` attribute carries the parser's
diagnostic.

**Common causes.** Malformed XML (unclosed tags, invalid characters);
namespace-prefix mismatch; non-MARCXML XML where MARCXML was expected.

**How to recover.** Inspect `e.__cause__` for the parser's specific
error. The bytes can't be re-parsed without correction.

**Python class:** `mrrc.XmlError`.

### E402 — `marcjson_invalid` { #E402 }

A MARCJSON document failed to parse.

**Context:** Parse-side (JSON parser layer).
**Applies to:** `mrrc.marcjson_to_record`, `mrrc.json_to_record`.
**Populates:** `cause` (the underlying `serde_json::Error` with `line()`
and `column()` available). May also populate: `record_index`,
`byte_offset`, `source`.

**Common causes.** Truncated JSON; mixed text encodings; non-MARCJSON
JSON where MARCJSON was expected.

**How to recover.** Inspect `e.__cause__.line` and `.column` for the
position; re-encode the input or fix upstream.

**Python class:** `mrrc.JsonError`.

### E404 — `record_too_large_for_iso2709` { #E404 }

The writer attempted to serialize a record whose total length or base-
address-of-data exceeds the ISO 2709 5-digit limit (99999 bytes for
length, same for base address).

**Context:** Writer-side.
**Applies to:** `MARCWriter`, `AuthorityMarcWriter`, `HoldingsMarcWriter`.
**Populates:** `record_index`, `record_control_number`. The `message`
attribute names which limit was exceeded with the actual byte count.
**Never populates:** `byte_offset` (this fires before any bytes are written).

**Common causes.** Records with very large fields (full-text content in
505 or 520); aggregations of records with many repeated fields.

**How to recover.** Split the record into smaller units; use a different
serialization format (MARCXML or MARCJSON) that doesn't have the 5-digit
length limit.

**Python class:** `mrrc.WriterError`.

---

## Warnings (Wxxx)

### W001 — `bad_subfield_code_warning` { #W001 }

A subfield code is unusual but the field is otherwise valid (pymarc
compatibility — pymarc raises this as a `UserWarning`).

**Context:** Warning during parsing; does not abort the parse.
**Python class:** `mrrc.BadSubfieldCodeWarning` (a `UserWarning`, not an
exception).
