# Error Handling

mrrc raises a typed exception hierarchy with structured positional metadata
on every error: where in the byte stream the problem occurred, which record
it came from, the 001 control number, the field/subfield being parsed, and
the source filename when known. The class names and parent relationships
match pymarc's exception layer, so code written against `pymarc`'s exception
classes catches the same conditions in mrrc unchanged.

## Exception hierarchy

```
Exception
├── MrrcException                      (base)
│   ├── RecordLengthInvalid
│   ├── RecordLeaderInvalid
│   ├── BaseAddressInvalid
│   ├── BaseAddressNotFound
│   ├── RecordDirectoryInvalid
│   │   ├── InvalidIndicator    (mrrc)
│   │   ├── BadSubfieldCode     (mrrc)
│   │   └── InvalidField        (mrrc)
│   ├── EndOfRecordNotFound
│   │   └── TruncatedRecord     (mrrc)
│   ├── FieldNotFound
│   ├── FatalReaderError
│   ├── EncodingError           (mrrc)
│   ├── XmlError                (mrrc)
│   ├── JsonError               (mrrc)
│   └── WriterError             (mrrc)
└── OSError
    └── PyIOError                      (Python built-in, raised on I/O failure)

class BadSubfieldCodeWarning(UserWarning)
```

Classes marked **(mrrc)** are mrrc-specific subclasses that pymarc does not
have. Each one extends the closest pymarc parent so existing
pymarc-style `except` clauses keep catching the same conditions.

## Choosing what to catch

| You want to… | Catch |
|---|---|
| Match pymarc's catch behavior exactly | The pymarc-named class (`RecordDirectoryInvalid`, `EndOfRecordNotFound`, etc.) — mrrc-specific subclasses are caught too. |
| Distinguish indicator errors from subfield errors | `InvalidIndicator` and `BadSubfieldCode` separately. |
| Catch every mrrc error, no matter the variant | `MrrcException`. |
| Catch only I/O errors | `OSError` (or its `IOError` alias). |

## Pymarc exception compatibility

This page covers exception **class names, hierarchy, and catch behavior**
only. The new positional attributes are additive: pymarc-style code that
inspects only `str(err)` keeps working without change.

Other compatibility surfaces — record APIs, reader/writer constructor
shapes, format coverage, and performance characteristics — are out of
scope for this page; consult the
[Python API reference](python-api.md) and
[Rust API reference](rust-api.md) for those.

### Exception name mapping

| pymarc class | mrrc class | Notes |
|---|---|---|
| `PymarcException` | `MrrcException` | Same role; alias if desired (see below). |
| `RecordLengthInvalid` | `RecordLengthInvalid` | Same name; gains positional attrs. |
| `RecordLeaderInvalid` | `RecordLeaderInvalid` | Same name; gains positional attrs. |
| `BaseAddressInvalid` | `BaseAddressInvalid` | Same name; gains positional attrs. |
| `BaseAddressNotFound` | `BaseAddressNotFound` | Same name; gains positional attrs. |
| `RecordDirectoryInvalid` | `RecordDirectoryInvalid` | Same name; gains positional attrs. Also catches new mrrc subclasses `InvalidIndicator`, `BadSubfieldCode`, `InvalidField`. |
| `EndOfRecordNotFound` | `EndOfRecordNotFound` | Same name; gains positional attrs. Also catches new subclass `TruncatedRecord`. |
| `FieldNotFound` | `FieldNotFound` | Same name; gains `record_control_number`, `record_index`. |
| `FatalReaderError` | `FatalReaderError` | Same name; reserved for catastrophic states. |
| `BadSubfieldCodeWarning` | `BadSubfieldCodeWarning` | Same name (UserWarning, not exception). |
| `NoFieldsFound` | *not present* | mrrc has not raised this historically; file an issue if needed. |
| `IOError` / `OSError` | `OSError` (via `PyIOError`) | I/O errors map to Python's built-in. |

### Optional symbol-level aliases

For projects swapping `pymarc` imports to `mrrc` and wanting
`from pymarc import RecordLeaderInvalid`-style imports to keep working:

```python
from mrrc import MrrcException as PymarcException
from mrrc import (
    RecordLengthInvalid,
    RecordLeaderInvalid,
    BaseAddressInvalid,
    BaseAddressNotFound,
    RecordDirectoryInvalid,
    EndOfRecordNotFound,
    FieldNotFound,
    FatalReaderError,
    BadSubfieldCodeWarning,
)
```

The catch hierarchy behaves the same as in pymarc. Code outside the
exception layer (record manipulation, reader/writer APIs, format I/O)
may still need changes; consult the [Python API reference](python-api.md).

### What you gain on the exception layer

Three patterns, in order of effort:

**Same `except`, more context.** Existing pymarc-style code keeps working.
The same `except` clause now also gets structured attributes:

```python
try:
    for record in mrrc.MARCReader(open("harvest.mrc", "rb")):
        ...
except mrrc.RecordDirectoryInvalid as e:
    log.warning(
        "directory error in record %d (001=%s, field %s) at byte 0x%X",
        e.record_index, e.record_control_number, e.field_tag, e.byte_offset,
    )
```

**Opt-in granularity.** mrrc-aware code can catch the new subclasses
directly to make decisions on the specific error kind:

```python
try:
    ...
except mrrc.InvalidIndicator as e:
    log.warning(
        "Bad indicator at field %s ind%d in record %d",
        e.field_tag, e.indicator_position, e.record_index,
    )
except mrrc.BadSubfieldCode as e:
    log.warning("Bad subfield code 0x%02X at field %s", e.subfield_code, e.field_tag)
```

**Diagnostic dump.** The `detailed()` method produces a multi-line
diagnostic suitable for logs:

```python
try:
    ...
except mrrc.MrrcException as e:
    log.error(e.detailed())
```

```text
InvalidIndicator at record 847, field 245
  source:          harvest.mrc
  001:             ocm01234567
  indicator 1:     found b':', expected digit or space
  byte offset:     0x1C31 (7217) in stream
  record-relative: byte 42
```

### Out of scope for this guide

This page does **not** claim compatibility for:

- Reader/writer constructor signatures or behavior differences.
- `Record`, `Field`, `Subfield` API differences.
- Format-coverage differences (MARCXML/MARCJSON edge cases, character
  encoding handling, etc.).
- Performance and memory behavior.

For those, see the linked reference pages above.

### Subclass behavior reference

| If you `except` this class… | …you also catch these mrrc-specific subclasses |
|---|---|
| `RecordDirectoryInvalid` | `InvalidIndicator`, `BadSubfieldCode`, `InvalidField` |
| `EndOfRecordNotFound` | `TruncatedRecord` |
| `MrrcException` | All mrrc-specific exceptions |
| `OSError` | `PyIOError` (I/O failures) |

### `MARCReader.current_exception` / `current_chunk`

pymarc exposes `MARCReader.current_exception` and `MARCReader.current_chunk`
attributes that callers can inspect after a recovered error. mrrc does not
currently expose these reader-side attributes; the per-error positional
metadata (record index, byte offset, source) typically replaces the
patterns those attributes enabled. File an issue if a workflow specifically
needs them.

## Per-variant field reference

Each exception class accepts the following keyword arguments at construction
time (all optional). Attributes of the same name are populated by the parser
when the information is available; absent values stay `None`.

| Field | Type | Meaning |
|---|---|---|
| `record_index` | `int \| None` | 1-based position of the record in the input stream. |
| `record_control_number` | `str \| None` | Value of the 001 control field for the record being parsed. `None` for errors raised before 001 is decoded (invalid leader, invalid directory, pre-001 truncation). |
| `field_tag` | `str \| None` | Tag of the field being parsed (e.g., `"245"`). |
| `indicator_position` | `int \| None` | Indicator position (`0` or `1`), populated for `InvalidIndicator`. |
| `subfield_code` | `int \| None` | Offending subfield code byte, populated for `BadSubfieldCode`. |
| `found` | `bytes \| None` | The bad bytes that triggered the error, capped at 32 bytes. |
| `expected` | `str \| None` | Human-readable description of what was expected. |
| `byte_offset` | `int \| None` | Absolute byte offset within the input stream. |
| `record_byte_offset` | `int \| None` | Byte offset within the current record. |
| `source` | `str \| None` | Filename or stream identifier, populated when the reader was constructed via `from_path`. |
| `bytes_near` | `bytes \| None` | Up to 32 bytes around the error offset, for hex-dump rendering. `None` when the parser did not have access to a buffer at error time. |
| `bytes_near_offset` | `int \| None` | Absolute stream offset of the first byte of `bytes_near`. |

Subclass-specific extras:

- `InvalidField`, `EncodingError`, `XmlError`, `JsonError`, `WriterError` add
  a `message: str | None` field carrying a human-readable description of the
  problem.
- `TruncatedRecord` adds `expected_length` and `actual_length` (both
  `int | None`) describing how far short the record was of its declared
  length.

### Always-present vs may-be-present per variant

The parser populates `record_index` and `byte_offset` on every parse-path
error; `record_control_number` whenever 001 is already decoded;
`source` whenever the reader was constructed via `with_source()` or
`from_path()`. Other fields are populated when applicable to the variant
(e.g., `indicator_position` only on `InvalidIndicator`).

`FieldNotFound` is an accessor error rather than a parse error; it carries
`field_tag`, `record_control_number`, and `record_index` but not byte
offsets.

## Position semantics by format

`byte_offset` and `record_byte_offset` mean different things depending on the
input format:

- **ISO 2709** (binary MARC). `byte_offset` is the absolute byte position in
  the input stream; `record_byte_offset` is relative to the start of the
  current record. This is the primary case.
- **MARCXML**. The underlying `quick_xml` parser does not expose a byte
  position from its deserializer error type, so `byte_offset` is currently
  `None`. Position information is available via the wrapped cause: walk
  `err.__cause__` for the original `quick_xml` error.
- **MARCJSON**. The wrapped `serde_json::Error` exposes line and column;
  `byte_offset` is currently `None` because translating (line, column) to a
  byte offset requires the original input bytes. Walk `err.__cause__` to
  read `cause.line` and `cause.column`.

When a format's underlying parser does not expose usable position
information, the field stays `None` rather than being fabricated.

## Source filename plumbing

The `source` attribute on errors is populated when the reader was told its
input identity. There are two ways to set it:

```python
# 1. Builder method: any reader, any input source.
reader = mrrc.MARCReader(file_obj).with_source("harvest.mrc")

# 2. Convenience constructor: opens a file and sets source from the path.
reader = mrrc.MARCReader.from_path("harvest.mrc")
```

When neither is used (e.g., reading from `BytesIO`), `source` stays `None`
on emitted errors.

The same `with_source` / `from_path` pattern is available on
`AuthorityMARCReader` and `HoldingsMARCReader`.

## Recovery modes and errors

The `RecoveryMode` setting (`Strict` / `Lenient` / `Permissive`) controls
whether a malformed record raises immediately, is salvaged with partial
data, or is skipped. The structured positional metadata is populated
identically in all three modes — the modes only differ in whether the
error is propagated, suppressed, or used to inform a salvage attempt.

## Structured serialization (`to_dict` / `to_json`)

Every `MrrcException` exposes `to_dict()` and `to_json()` for emitting the
error into structured logging platforms (ELK, Datadog, Splunk,
JSON-line pipelines) without writing an adapter. The Rust side offers a
matching `MarcError::to_json_value()` / `to_json()` that produces the same
schema.

```python
try:
    ...
except mrrc.MrrcException as e:
    log.error(json.dumps({**e.to_dict(), "app": "ingest"}))
```

Sample output:

```python
>>> err.to_dict()
{
  "schema_version": 1,
  "class": "InvalidIndicator",
  "code": "E201",
  "slug": "invalid_indicator",
  "severity": "error",
  "help_url": "https://dchud.github.io/mrrc/reference/error-codes/#E201",
  "record_index": 847,
  "record_control_number": "ocm01234567",
  "field_tag": "245",
  "indicator_position": 0,
  "found": None,
  "found_hex": "3a",
  "expected": "digit or space",
  "byte_offset": 7217,
  "record_byte_offset": 42,
  "source": "harvest.mrc",
  "bytes_near": None,
  "bytes_near_hex": "323032336e79752020202020202020203a3030203020656e6720641e323435",
  "bytes_near_offset": 7201,
  "_cause": None
}
```

### Schema contract (v1)

The `schema_version: 1` key is the stability anchor. Downstream consumers
can rely on:

- **Fixed-position keys always present**: `schema_version`, `class`, `code`,
  `slug`, `severity`, `help_url`, every positional field, and `_cause`
  appear in every dict. Values may be `null` but keys are never missing.
- **Bytes fields hex-encoded with a `_hex` suffix**: `found` (always `null`
  in the dict), `found_hex` (present only when bytes were captured), and
  similarly `bytes_near` / `bytes_near_hex`. This keeps the dict
  JSON-serializable without a custom encoder.
- **Bounded payload size**: `found` is capped at 32 bytes at capture time
  and `bytes_near` at 32 bytes (16 before + 16 after the error offset), so
  the full dict stays well under typical log-platform ingestion limits.
- **`_cause` is flat**: always a string or `null`, never a nested dict.
  Consumers who need the full exception chain pass
  `include_traceback=True` or walk `__cause__` themselves.

Any change to the dict shape (adding, removing, or re-purposing a key)
must bump `schema_version` and the crate's minor version (pre-1.0) or
major version (post-1.0).

### `include_traceback`

`to_dict(include_traceback=True)` adds a `traceback` key with formatted
traceback lines (only present when the exception was actually raised).
`to_json(include_traceback=True)` forwards the flag to `to_dict`.

## Hex dump in `detailed()`

When the parser captures a byte window around the error offset, the
exception's `detailed()` output appends a 32-byte hex + ASCII dump with a
caret pointing at the offending byte:

```text
InvalidIndicator at record 847, field 245
  source:          harvest.mrc
  001:             ocm01234567
  indicator 0:     found b':', expected digit or space
  byte offset:     0x1C31 (7217) in stream
  record-relative: byte 42

bytes near offset 0x1C31:
    0x1C21:  32 30 32 33 6e 79 75 20  20 20 20 20 20 20 20 20 |2023nyu         |
    0x1C31:  3a 30 00 30 20 30 20 65  6e 67 20 64 1e 32 34 35 |:0.0 0 eng d.245|
             ^^ offending byte
```

The window is up to 16 bytes before + 16 bytes after the error offset,
clamped at buffer boundaries. Non-printable bytes render as `.` in the
ASCII sidecar. The window layout is fixed at 16 bytes per row with an
8-byte gap for readability; the format is byte-for-byte identical in
Rust (`MarcError::detailed()`) and Python (`MrrcException.detailed()`).

The `bytes_near` attribute on the exception is `None` when the parser
did not have access to a buffer at the point the error was raised
(e.g., for wrapping variants like `IoError` / `XmlError` / `JsonError`,
or for error paths that do not yet plumb the buffer through).

## Pickle round-trip

Exception instances round-trip through `pickle` with all positional
attributes preserved (subclass extras like `expected_length`/`message`
included). For security, `__setstate__` whitelists incoming attribute names
against the per-class allowed set; a maliciously-crafted pickle that tries
to set arbitrary attributes (including method names) will raise `TypeError`
rather than silently shadowing methods on the instance.

This is a defense-in-depth measure only. As with any pickle-based
deserialization, do not unpickle data from untrusted sources — the
unpickling step itself is the relevant attack surface.
