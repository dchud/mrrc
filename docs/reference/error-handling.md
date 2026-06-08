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
│   ├── WriterError             (mrrc)
│   └── StaleFieldError         (mrrc)
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
| Handle a field handle invalidated by removals | `StaleFieldError` — re-fetch the field from the record and retry. Raised by live field handles (see [Field handles](python-api.md#record)) after any `remove_field`/`remove_fields` call; it is a usage error, not a data error, so it carries no E-code. |

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
| `IOError` / `OSError` | `OSError` (via `PyIOError`) | I/O errors map to Python's built-in. |

#### Pymarc names mrrc deliberately omits

The following pymarc classes are intentionally absent in mrrc. Each
row gives the rationale and the mrrc-equivalent behavior a port
should rely on instead.

| pymarc class | why mrrc doesn't have it | mrrc-equivalent behavior |
|---|---|---|
| `NoFieldsFound` | An empty `Record` is a valid in-memory state in mrrc; no exception is raised. | Check `record.get_fields()` length. |
| `WriteNeedsRecord` | `MARCWriter.write_record` is type-annotated; passing a non-Record is a static-type error. | Static type check (`pyright` / `mypy`). |
| `NoActiveFile` | `MARCWriter` is context-managed; operating on a closed writer raises plain `RuntimeError`. | Use a `with` block or check writer state. |
| `BadLeaderValue` | `mrrc.Leader` validates fields at construction. | Bad values raise `ValueError`. |
| `MissingLinkedFields` | 880-linkage validation isn't part of the parser. | Validate links in caller code. |

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

### Subclass behavior reference

| If you `except` this class… | …you also catch these mrrc-specific subclasses |
|---|---|
| `RecordDirectoryInvalid` | `InvalidIndicator`, `BadSubfieldCode`, `InvalidField` |
| `EndOfRecordNotFound` | `TruncatedRecord` |
| `MrrcException` | All mrrc-specific exceptions |
| `OSError` | `PyIOError` (I/O failures) |

### `MARCReader.current_exception` / `current_chunk`

mrrc's `MARCReader` exposes pymarc-compatible `current_exception` and
`current_chunk` attributes. After each `__next__` step:

- `reader.current_chunk` holds the raw bytes of the record just read
  from the source (declared length per the leader). Set on every
  successful chunk read regardless of whether the parse step then
  succeeded or failed.
- `reader.current_exception` holds the typed `MrrcException` swallowed
  by `permissive=True`, or `None` on a clean read.

```python
reader = mrrc.MARCReader("harvest.mrc", permissive=True)
for record in reader:
    if record is None:
        log.warning(
            "skipped malformed record (%d bytes): %s",
            len(reader.current_chunk) if reader.current_chunk else 0,
            reader.current_exception,
        )
        continue
    process(record)
```

Two documented divergences from pymarc:

- **Encoding strictness.** mrrc raises `EncodingError` on invalid UTF-8
  in subfield values (swallowed via `current_exception` under
  `permissive=True`); pymarc applies lossy substitution silently. The
  iteration shape is identical (the bad record yields as `None` either
  way), so callers using `except Exception:` keep working.
- **`current_chunk` on byte-read errors.** When the underlying read
  of the next record's bytes fails before parsing begins (truncated
  stream, I/O error), `current_chunk` may be `None` even though
  `current_exception` is set. For parse failures of fully-read chunks
  (the common case), `current_chunk` carries the full record bytes as
  pymarc does.

### Known hierarchy divergences from pymarc

mrrc's exception class names match pymarc's, but two relationships in
the class tree differ. Existing `except` clauses written against a
specific class name (`except RecordDirectoryInvalid:`,
`except EndOfRecordNotFound:`, etc.) work in mrrc unchanged. The
divergences only matter for code that catches a *parent* class.

**`FatalReaderError` parentage.** In pymarc, `FatalReaderError` is the
parent of `RecordLengthInvalid`, `TruncatedRecord`, and
`EndOfRecordNotFound`; a pymarc loop can `except FatalReaderError:` to
catch any of those four. In mrrc, `FatalReaderError` is a sibling
(reserved for the specific "recovered-error cap exceeded" case under
`recovery_mode="lenient"`/`"permissive"` with `with_max_errors`).
`except FatalReaderError:` in mrrc therefore catches only the
cap-exhausted case, not the malformed-record cases. To match pymarc's
catch surface, either enumerate the four classes —

```python
except (RecordLengthInvalid, TruncatedRecord, EndOfRecordNotFound,
        FatalReaderError):
    ...
```

— or catch the mrrc base, which is broader (every typed mrrc error):

```python
except MrrcException:
    ...
```

**`PymarcException` → `MrrcException`.** The base class name differs.
`from pymarc import PymarcException` fails at import; replace with
`from mrrc import MrrcException` (or alias on import — see *Optional
symbol-level aliases* below).

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
  position from its deserializer error type, so `byte_offset` is `None`.
  Position information is available via the wrapped cause: walk
  `err.__cause__` for the original `quick_xml` error.
- **MARCJSON**. The wrapped `serde_json::Error` exposes line and column;
  `byte_offset` is `None` because translating (line, column) to a byte
  offset requires the original input bytes. Walk `err.__cause__` to
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

## Validation level vs recovery mode

Two orthogonal axes govern parsing behavior:

- **`validation_level`** — *what counts as an error*.
- **`recovery_mode`** — *what to do when one fires*.

The single rule, statable in one sentence: **`structural` is lossy
across every reader; `strict_marc` is strict across every reader — every
reader behaves the same way at each level.**

Concretely:

| | `validation_level="structural"` (default) | `validation_level="strict_marc"` |
|---|---|---|
| ISO 2709 structural errors (E001–E007, E101, E106) | fire | fire |
| Indicator byte validation (E201, byte-level) | skipped | fires |
| Per-tag MARC 21 indicator semantics (E201, e.g. 245 ind1 ∈ {0,1}) | skipped | fires |
| Subfield-code byte validation (E202) | skipped | fires |
| MARC 21 leader semantics (E002, e.g. record_status ∈ {a,c,d,n,p}) | skipped | fires |
| UTF-8 strictness (E301) | lossy decode (`U+FFFD` substitution) across bibliographic + authority + holdings | strict decode raises across all three readers |

```python
reader = mrrc.MARCReader(
    file,
    validation_level="structural",   # or "strict_marc"
    recovery_mode="strict",          # or "lenient", "permissive"
)
```

The two axes compose. `(strict_marc, lenient)` means *I want byte-level
checks AND I want to keep iterating past one bad record* — strict_marc
makes E201/E202/E301 fire, lenient absorbs them via the per-stream
recovery cap.

## Recovery modes and errors

The `RecoveryMode` setting (`Strict` / `Lenient` / `Permissive`) controls
whether a malformed record raises immediately, is salvaged with partial
data, or is skipped. The structured positional metadata is populated
identically in all three modes — the modes only differ in whether the
error is propagated, suppressed, or used to inform a salvage attempt.

### Defaults: Python `permissive`, Rust `Strict`

The Python user surface (`mrrc.MARCReader`, `mrrc.AuthorityMARCReader`,
`mrrc.HoldingsMARCReader`) defaults to `recovery_mode="permissive"` —
the same default shape as pymarc / marc4j / libmarc. A fresh
`MARCReader(file)` iterates past per-record defects rather than aborting
on the first one, so users coming from those libraries get the
expected behavior without setting any kwarg.

The Rust core (`mrrc::MarcReader`) keeps the stricter `RecoveryMode::Strict`
default. Rust callers expect explicit error handling via `Result<T, E>`
and `?` propagation; flipping the default there would convert a loud
`Err` into a quiet `record.errors` field that the caller has to
remember to inspect.

#### A gentle case for choosing `strict` when feasible

Permissive mode is the more forgiving default, but it has a real cost
worth understanding before you ship it past a prototype:

- **Unsalvageable records yield as `None`.** When the parser can't make
  even partial sense of a record's bytes, the Python wrapper hands you
  `None` rather than skipping silently. A loop written as
  `for record in reader: process(record)` will pass `None` into
  `process` unless you guard with `if record is not None:` or iterate
  via `iter_with_errors()`. Worth being deliberate about.
- **Per-record diagnostics live on `record.errors`.** A clean iteration
  in permissive mode can still be hiding malformed records — the errors
  are attached to the yielded record rather than raised. If nothing
  checks `record.errors`, defects are observable but invisible.
- **`record.errors` accumulates up to `max_errors`.** Without an
  explicit `max_errors=N` kwarg, a pathological stream can fill memory
  with diagnostic objects before anyone notices. The Rust core caps
  at `DEFAULT_MAX_ERRORS` (10 000) per parse, but the Python wrapper-
  level cap defaults to disabled (see [Capping recovered errors with
  `max_errors`](#capping-recovered-errors-with-max_errors)).

If you control the input and quality matters more than throughput,
`recovery_mode="strict"` makes defects loud: a single bad record
raises a typed exception with full positional context. Pair it with
`permissive=True` for the pymarc-shape pattern of "yield `None` for
bad records, stash the exception on `current_exception`" without
losing the precise diagnostics.

```python
# Most forgiving (default): keep going, attach defects to record.errors
reader = mrrc.MARCReader(file)

# Pymarc-shape: yield None for failed parses, stash exception
reader = mrrc.MARCReader(file, permissive=True)

# Loudest: typed exception raised on first defect
reader = mrrc.MARCReader(file, recovery_mode="strict")
```

## Inspecting per-record errors

In `lenient` and `permissive` recovery modes, errors that would have
been raised under `strict` are instead **attached to the yielded
record** as `record.errors`. The list carries one typed exception per
recovered defect, with the same positional context (record_index,
byte_offset, field_tag, etc.) as if the error had been raised directly.

```python
reader = mrrc.MARCReader(file, recovery_mode="lenient")
for record in reader:
    if record.errors:
        for err in record.errors:
            log.warning(f"[{err.code}] {err}")
    process(record)
```

In `strict` mode `record.errors` is always `[]` — the parser raises on
the first error before the record is yielded. In `lenient` and
`permissive` it carries diagnostics for every defect the parser
recovered from (subject to `max_errors` cap).

### `iter_with_errors()`

`MARCReader.iter_with_errors()` is an alternate iterator yielding
`(record, errors)` tuples instead of bare records. Equivalent to
iterating + reading `record.errors`, but more discoverable for the
"give-me-everything-defective" use case:

```python
for record, errors in reader.iter_with_errors():
    if errors:
        log.warning(f"{len(errors)} issues parsing record")
    if record:
        process(record)
```

Under `permissive=True`, records that the parser cannot salvage at all
yield as `(None, [exception])` so even unsalvageable records are
observable. Without `iter_with_errors`, those records are silently
returned as `None` and the diagnostic is lost.

```python
reader = mrrc.MARCReader(file, permissive=True)
for record, errors in reader.iter_with_errors():
    if record is None:
        log.error(f"unsalvageable: {errors[0]}")
    else:
        process(record)
```

`AuthorityMARCReader` and `HoldingsMARCReader` expose `record.errors`
the same way (the load-bearing surface). They don't carry the
`iter_with_errors` convenience method — that's a pymarc-shape ergonomic
specific to `MARCReader`. Iterate normally and check `record.errors`:

```python
for record in mrrc.AuthorityMARCReader(file, recovery_mode="lenient"):
    if record.errors:
        log.warning(...)
```

### Capping recovered errors with `max_errors`

A pathological stream in `lenient` / `permissive` mode can accumulate diagnostics without bound — every malformed record adds one or more `MrrcException` instances to `record.errors`. Pass `max_errors=N` to `MARCReader` to cap the total recovered count across the stream; once the (N+1)-th recovered error lands, the next iteration raises `FatalReaderError` (E099) instead of yielding another record.

```python
reader = mrrc.MARCReader(file, recovery_mode="lenient", max_errors=100)
try:
    for record in reader:
        process(record)
except mrrc.FatalReaderError as e:
    log.error(f"stopped after {e.errors_seen} errors (cap={e.cap})")
```

- `max_errors=None` (the default) disables the wrapper-level cap.
- `max_errors=0` also disables the cap (matches the Rust API's no-cap sentinel).
- `max_errors=N` for any `N > 0` trips on the (N+1)-th recovered error.

Observationally inert in `strict` mode: the first error raises before any recovery accumulates against the cap. `AuthorityMARCReader` and `HoldingsMARCReader` don't carry the kwarg — they inherit the Rust core's per-reader `DEFAULT_MAX_ERRORS` (10_000) directly.

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

### Notes on the shape

- Bytes fields carry their data under a `_hex` suffix key (`found_hex`,
  `bytes_near_hex`); the bare key (`found`, `bytes_near`) stays `null` so
  the dict is JSON-serializable without a custom encoder. The `_hex`
  keys appear only when bytes were captured.
- `_cause` is always a string or `null`, never nested. For the full
  exception chain pass `include_traceback=True` or walk `__cause__`.
- The emitted bytes are bounded at capture time (`found` ≤ 32 bytes,
  `bytes_near` ≤ 32 bytes from the 16+16 hex-dump window), so payloads
  don't grow unboundedly.
- `schema_version: 1` is included so callers can branch on it later if
  the shape ever changes. Pre-1.0, the shape may still evolve.

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
or for error paths that do not have buffer access at error time).

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
