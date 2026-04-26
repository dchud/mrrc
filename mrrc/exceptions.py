"""Exception hierarchy for mrrc.

The pymarc-named classes (``RecordLengthInvalid``, ``RecordLeaderInvalid``,
``BaseAddressInvalid``, ``BaseAddressNotFound``, ``RecordDirectoryInvalid``,
``EndOfRecordNotFound``, ``FieldNotFound``, ``FatalReaderError``) preserve
pymarc's names and parent relationships. mrrc-specific subclasses
(``InvalidIndicator``, ``BadSubfieldCode``, ``InvalidField``,
``TruncatedRecord``, ``EncodingError``, ``XmlError``, ``JsonError``,
``WriterError``) extend the closest pymarc parent so existing
``except RecordDirectoryInvalid:`` style catches still trigger for the new
subclasses, while mrrc-aware code can opt into the more specific subclass.

All exception classes accept positional context as keyword arguments:
``record_index``, ``record_control_number``, ``field_tag``,
``indicator_position``, ``subfield_code``, ``found``, ``expected``,
``byte_offset``, ``record_byte_offset``, ``source``. Every kwarg is
optional so bare-constructor compatibility is preserved (e.g.,
``raise RecordLeaderInvalid()`` still works).

Pickle round-trip preserves all positional attributes. ``__setstate__``
whitelists incoming attribute names against the per-class allowed set as a
defense-in-depth measure — unpickling untrusted data remains the relevant
attack surface regardless.
"""

from typing import Optional


_POSITIONAL_FIELDS = (
    "record_index",
    "record_control_number",
    "field_tag",
    "indicator_position",
    "subfield_code",
    "found",
    "expected",
    "byte_offset",
    "record_byte_offset",
    "source",
    "bytes_near",
    "bytes_near_offset",
)


# Hex-dump window layout: rendered as 16 bytes per row with an ASCII sidecar.
# Mirrors the Rust `render_hex_dump` in `src/error.rs`; any change must be
# applied in both places so cross-language output stays byte-for-byte equal.
_HEX_DUMP_ROW_WIDTH = 16


# Base URL for the docs site. Used by help_url() which appends
# `/reference/error-codes/#Exxx`.
DOCS_BASE_URL = "https://dchud.github.io/mrrc"


def _render_hex_dump(
    window: bytes,
    window_start_offset: int,
    byte_offset: Optional[int],
) -> str:
    """Render a byte window as a hex + ASCII dump with an optional caret.

    Matches the Rust ``render_hex_dump`` in ``src/error.rs`` byte-for-byte.
    See ``docs/reference/error-handling.md`` for the format contract.
    """
    anchor = byte_offset if byte_offset is not None else window_start_offset
    lines: list[str] = [f"bytes near offset 0x{anchor:X}:"]
    row_width = _HEX_DUMP_ROW_WIDTH
    caret_line: Optional[str] = None
    for row_idx in range(0, len(window), row_width):
        chunk = window[row_idx:row_idx + row_width]
        row_start = window_start_offset + row_idx
        hex_parts: list[str] = []
        for i, b in enumerate(chunk):
            if i == 8:
                hex_parts.append(" ")
            hex_parts.append(f"{b:02x} ")
        for i in range(len(chunk), row_width):
            if i == 8:
                hex_parts.append(" ")
            hex_parts.append("   ")
        ascii_parts = "".join(
            chr(b) if 0x20 <= b <= 0x7E else "." for b in chunk
        )
        ascii_parts = ascii_parts + " " * (row_width - len(chunk))
        row = f"    0x{row_start:04X}:  {''.join(hex_parts)}|{ascii_parts}|"
        lines.append(row)
        if (
            byte_offset is not None
            and row_start <= byte_offset < row_start + len(chunk)
        ):
            col = byte_offset - row_start
            # Prefix: 4 spaces + "0x####:  " = 13 chars; 3 chars per byte for
            # `col` bytes; one extra space after 8 bytes.
            caret_col = 13 + col * 3 + (1 if col >= 8 else 0)
            caret_line = " " * caret_col + "^^ offending byte"
            lines.append(caret_line)
    return "\n".join(lines)


class _MrrcExceptionBase:
    """Mixin providing positional context attributes, formatting, and pickle
    support for every mrrc exception class.

    Kept separate from the actual base ``MrrcException`` so subclasses can
    override ``_body_text`` independently of the inheritance chain to
    Exception itself.
    """

    # Stable error-code identifiers. Every leaf exception class overrides
    # ``code`` and ``slug`` with the canonical values matching the Rust
    # MarcError variant. The base values "" / "" act as sentinels and
    # never reach end users — MrrcException itself isn't raised directly.
    code: str = ""
    slug: str = ""

    # Class-level attribute annotations so mypy/pyright see the typed
    # positional context fields populated by __init__ via setattr.
    record_index: Optional[int]
    record_control_number: Optional[str]
    field_tag: Optional[str]
    indicator_position: Optional[int]
    subfield_code: Optional[int]
    found: Optional[bytes]
    expected: Optional[str]
    byte_offset: Optional[int]
    record_byte_offset: Optional[int]
    source: Optional[str]
    bytes_near: Optional[bytes]
    bytes_near_offset: Optional[int]

    def __init__(self, *args, **kwargs) -> None:
        for field in _POSITIONAL_FIELDS:
            setattr(self, field, kwargs.pop(field, None))
        if kwargs:
            unexpected = ", ".join(sorted(kwargs))
            raise TypeError(
                f"{type(self).__name__}() got unexpected keyword argument(s): {unexpected}"
            )
        # Fall back to the formatted positional summary as the Exception
        # message when no positional args were supplied; preserves
        # `str(err)` rendering even for bare-constructor instances.
        super().__init__(*(args or (self._format(),)))

    # --- pickle support -------------------------------------------------
    # By default, pickle for Exception subclasses round-trips only the args
    # tuple, dropping instance __dict__. Override __reduce__ so kwargs
    # survive a pickle round-trip.

    # Per-subclass extra attributes that should be preserved across pickle
    # round-trips in addition to _POSITIONAL_FIELDS. Subclasses with extra
    # __init__ kwargs (e.g., InvalidField.message) override this.
    _pickle_extra_fields: tuple = ()

    def __reduce__(self):
        return (self.__class__, (), self._pickle_state())

    def __setstate__(self, state) -> None:
        # Whitelist state keys against the per-class allowed set. Without
        # this, a maliciously-crafted pickle could setattr arbitrary names
        # (including __dict__ or method names) and shadow class methods on
        # the instance — pickle deserialization itself is the RCE primitive,
        # but blind setattr amplifies the blast radius unnecessarily.
        allowed = set(_POSITIONAL_FIELDS) | set(self._pickle_extra_fields)
        unexpected = set(state) - allowed
        if unexpected:
            raise TypeError(
                f"Refusing to set unexpected attributes during pickle restore: "
                f"{', '.join(sorted(unexpected))}"
            )
        for k, v in state.items():
            setattr(self, k, v)

    def _pickle_state(self) -> dict:
        state = {f: getattr(self, f) for f in _POSITIONAL_FIELDS}
        for f in self._pickle_extra_fields:
            state[f] = getattr(self, f, None)
        return state

    # --- rendering ------------------------------------------------------
    # _format is the actionable one-liner shown by str(err); detailed() is
    # the multi-line diagnostic. Output mirrors the Rust Display and
    # MarcError::detailed() shapes byte-for-byte where possible.

    def _format(self) -> str:
        parts = []
        if self.record_index is not None:
            parts.append(f"record {self.record_index}")
        if self.record_control_number:
            parts.append(f"001 '{self.record_control_number}'")
        if self.field_tag:
            parts.append(f"field {self.field_tag}")
        if self.indicator_position is not None:
            parts.append(f"ind{self.indicator_position}")
        header = (
            f"[{' · '.join(parts)}] " if parts else f"{type(self).__name__}: "
        )
        body = self._body_text()
        offset = ""
        if self.byte_offset is not None:
            offset = f"  (byte 0x{self.byte_offset:X} / {self.byte_offset})"
        return f"{header}{body}{offset}"

    def _body_text(self) -> str:
        # Subclasses override to provide variant-specific text; fallback is
        # the class name (humanized).
        return type(self).__name__

    def detailed(self) -> str:
        """Return a multi-line diagnostic with all populated positional fields visible.

        Output mirrors the Rust `MarcError::detailed()` format: header line
        followed by zero or more `  label: value` lines, with labels padded
        to the width of the widest label so columns align consistently.
        """
        # Header
        ctx_parts: list[str] = []
        if self.record_index is not None:
            ctx_parts.append(f"record {self.record_index}")
        if self.field_tag is not None:
            ctx_parts.append(f"field {self.field_tag}")
        if ctx_parts:
            header = f"{type(self).__name__} at {', '.join(ctx_parts)}"
        else:
            header = type(self).__name__

        # Detail rows: list of (label, value) pairs in display order.
        rows: list[tuple[str, str]] = []
        if self.source:
            rows.append(("source:", self.source))
        if self.record_control_number:
            rows.append(("001:", self.record_control_number))
        if self.indicator_position is not None and self.expected is not None:
            found_repr = repr(self.found) if self.found is not None else "?"
            rows.append(
                (
                    f"indicator {self.indicator_position}:",
                    f"found {found_repr}, expected {self.expected}",
                )
            )
        if self.subfield_code is not None:
            rows.append(
                ("subfield:", f"invalid code byte 0x{self.subfield_code:02X}")
            )
        # Subclasses with extra context attributes (TruncatedRecord) hook in
        # via _extra_detail_rows so detailed() doesn't need a per-class
        # override of the whole method.
        rows.extend(self._extra_detail_rows())
        if self.byte_offset is not None:
            rows.append(
                (
                    "byte offset:",
                    f"0x{self.byte_offset:X} ({self.byte_offset}) in stream",
                )
            )
        if self.record_byte_offset is not None:
            rows.append(("record-relative:", f"byte {self.record_byte_offset}"))

        if not rows:
            header_out = header
        else:
            label_width = max(len(label) for label, _ in rows)
            body = "\n".join(
                f"  {label}{' ' * (label_width - len(label) + 1)}{value}"
                for label, value in rows
            )
            header_out = f"{header}\n{body}"

        if self.bytes_near is not None and self.bytes_near_offset is not None:
            dump = _render_hex_dump(
                self.bytes_near,
                self.bytes_near_offset,
                self.byte_offset,
            )
            return f"{header_out}\n\n{dump}"
        return header_out

    def _extra_detail_rows(self) -> list[tuple[str, str]]:
        """Hook for subclasses to add detail rows for their extra
        attributes. Default returns no extra rows.
        """
        return []

    def __repr__(self) -> str:
        kwargs = ", ".join(
            f"{f}={getattr(self, f)!r}"
            for f in _POSITIONAL_FIELDS
            if getattr(self, f) is not None
        )
        return f"{type(self).__name__}({kwargs})"

    @classmethod
    def help_url(cls) -> str:
        """Return the canonical docs URL for this exception class's error
        code, pointing at the ``#Exxx`` anchor on the error-codes reference
        page.
        """
        return f"{DOCS_BASE_URL}/reference/error-codes/#{cls.code}"

    # --- structured serialization --------------------------------------
    # to_dict / to_json emit a JSON-ready dict suitable for structured
    # logging pipelines (ELK, Datadog, Splunk). Bytes fields are
    # hex-encoded under a `_hex`-suffixed key; the bare key stays None so
    # the dict is JSON-serializable without a custom encoder. `_cause` is
    # a flat string or None (never nested). SCHEMA_VERSION is included so
    # consumers can branch on it if the shape changes later — pre-1.0 the
    # shape may still evolve.

    SCHEMA_VERSION: int = 1

    # Per-subclass extra fields to include in to_dict() output beyond the
    # base _POSITIONAL_FIELDS. Subclasses with extras like `message` or
    # `expected_length` declare them here.
    _diagnostic_extra_fields: tuple = ()

    def to_dict(self, *, include_traceback: bool = False) -> dict:
        """Render this exception as a JSON-ready dict.

        Bytes fields are hex-encoded under a ``_hex``-suffixed key (e.g.,
        ``found_hex``) so the result is JSON-serializable without a
        custom encoder.

        ``include_traceback=True`` adds a ``traceback`` key with the
        formatted traceback lines (only present when ``self.__traceback__``
        is set, i.e., the exception was actually raised).
        """
        result: dict = {
            "schema_version": self.SCHEMA_VERSION,
            "class": type(self).__name__,
            "code": getattr(self, "code", None) or None,
            "slug": getattr(self, "slug", None) or None,
            "severity": getattr(self, "severity", "error"),
            "help_url": (
                self.help_url()
                if getattr(self, "code", None)
                else None
            ),
        }
        for field in _POSITIONAL_FIELDS:
            value = getattr(self, field, None)
            if isinstance(value, bytes):
                result[f"{field}_hex"] = value.hex()
                result[field] = None
            else:
                result[field] = value
        for field in self._diagnostic_extra_fields:
            value = getattr(self, field, None)
            if isinstance(value, bytes):
                result[f"{field}_hex"] = value.hex()
                result[field] = None
            else:
                result[field] = value
        cause = self.__cause__ or self.__context__
        result["_cause"] = str(cause) if cause is not None else None
        if include_traceback and self.__traceback__:
            import traceback

            result["traceback"] = traceback.format_exception(
                type(self), self, self.__traceback__
            )
        return result

    def to_json(self, **kwargs) -> str:
        """JSON-serialize this exception via :meth:`to_dict`. Any kwargs are
        forwarded to ``json.dumps`` (e.g., ``indent=2`` for pretty-print).
        """
        import json

        # Pull include_traceback out of kwargs so it's passed to to_dict
        # rather than json.dumps (which would reject it).
        include_traceback = kwargs.pop("include_traceback", False)
        return json.dumps(self.to_dict(include_traceback=include_traceback), **kwargs)


class MrrcException(_MrrcExceptionBase, Exception):
    """Base exception for all mrrc errors."""


class RecordLengthInvalid(MrrcException):
    """The leader's record-length field is invalid (non-numeric, too small, etc.)."""

    code = "E001"
    slug = "record_length_invalid"

    def _body_text(self) -> str:
        if self.found is not None and self.expected is not None:
            return f"invalid record length {self.found!r} — expected {self.expected}"
        return "invalid record length"


class RecordLeaderInvalid(MrrcException):
    """The 24-byte record leader is malformed."""

    code = "E002"
    slug = "leader_invalid"
    _pickle_extra_fields = ("message",)
    _diagnostic_extra_fields = ("message",)
    message: Optional[str]

    def __init__(self, *args, message=None, **kwargs) -> None:
        self.message = message
        super().__init__(*args, **kwargs)

    def _body_text(self) -> str:
        if self.message:
            return f"invalid leader: {self.message}"
        return "invalid leader"


class BaseAddressInvalid(MrrcException):
    """The leader's base-address-of-data field is invalid."""

    code = "E003"
    slug = "base_address_invalid"

    def _body_text(self) -> str:
        if self.found is not None and self.expected is not None:
            return f"invalid base address {self.found!r} — expected {self.expected}"
        return "invalid base address"


class BaseAddressNotFound(MrrcException):
    """The leader claims a base address of data that does not exist in the stream."""

    code = "E004"
    slug = "base_address_not_found"

    def _body_text(self) -> str:
        return "base address not found"


class RecordDirectoryInvalid(MrrcException):
    """A directory entry is structurally invalid (bad tag, length, or start position).

    Catches mrrc-specific subclasses ``InvalidIndicator``, ``BadSubfieldCode``,
    and ``InvalidField`` as well — pymarc-style ``except`` clauses keep
    working unchanged.
    """

    code = "E101"
    slug = "directory_invalid"

    def _body_text(self) -> str:
        if self.found is not None and self.expected is not None:
            return f"invalid directory entry {self.found!r} — expected {self.expected}"
        return "invalid directory entry"


class EndOfRecordNotFound(MrrcException):
    """The end-of-record marker was not found where expected.

    Catches mrrc-specific subclass ``TruncatedRecord`` as well.
    """

    code = "E006"
    slug = "end_of_record_not_found"

    def _body_text(self) -> str:
        return "end-of-record marker not found"


class FieldNotFound(MrrcException):
    """A requested field was not present in the record (accessor error)."""

    code = "E105"
    slug = "field_not_found"

    def _body_text(self) -> str:
        return f"field {self.field_tag} not found" if self.field_tag else "field not found"


class FatalReaderError(MrrcException):
    """Unrecoverable error during record reading — the reader is halted.

    Currently raised when the per-stream recovered-error cap is exceeded
    in ``RecoveryMode.Lenient`` / ``Permissive`` (see
    :py:meth:`mrrc.MARCReader`'s ``max_errors`` parameter). The class is
    also reserved for future catastrophic reader states.

    When the cap has been exceeded, ``cap`` and ``errors_seen`` carry
    the configured limit and the count at the moment of the trip. After
    this exception is raised the reader is exhausted; subsequent
    iteration returns nothing.
    """

    code = "E099"
    slug = "fatal_reader_error"
    _pickle_extra_fields = ("cap", "errors_seen")
    _diagnostic_extra_fields = ("cap", "errors_seen")
    cap: Optional[int]
    errors_seen: Optional[int]

    def __init__(self, *args, cap=None, errors_seen=None, **kwargs) -> None:
        self.cap = cap
        self.errors_seen = errors_seen
        super().__init__(*args, **kwargs)

    def _body_text(self) -> str:
        if self.cap is not None and self.errors_seen is not None:
            return (
                f"fatal reader error: recovered-error cap exceeded "
                f"({self.errors_seen} errors, cap {self.cap})"
            )
        return "fatal reader error"


# --- mrrc-specific subclasses (extend the pymarc-named parents) -----------


class InvalidIndicator(RecordDirectoryInvalid):
    """An indicator byte was invalid for its position."""

    code = "E201"
    slug = "invalid_indicator"

    def _body_text(self) -> str:
        if self.found is not None and self.expected is not None:
            return f"invalid {self.found!r} — expected {self.expected}"
        return "invalid indicator"


class BadSubfieldCode(RecordDirectoryInvalid):
    """A subfield code byte was not a printable ASCII character."""

    code = "E202"
    slug = "bad_subfield_code"

    def _body_text(self) -> str:
        if self.subfield_code is not None:
            return f"invalid subfield code 0x{self.subfield_code:02X}"
        return "invalid subfield code"


class InvalidField(RecordDirectoryInvalid):
    """A data field is structurally invalid in some way not covered by the more specific subclasses."""

    code = "E106"
    slug = "invalid_field"
    _pickle_extra_fields = ("message",)
    _diagnostic_extra_fields = ("message",)
    message: Optional[str]

    def __init__(self, *args, message=None, **kwargs) -> None:
        self.message = message
        super().__init__(*args, **kwargs)

    def _body_text(self) -> str:
        if self.message:
            return f"invalid field: {self.message}"
        return "invalid field"


class TruncatedRecord(EndOfRecordNotFound):
    """The record was truncated mid-stream."""

    code = "E005"
    slug = "truncated_record"
    _pickle_extra_fields = ("expected_length", "actual_length")
    _diagnostic_extra_fields = ("expected_length", "actual_length")
    expected_length: Optional[int]
    actual_length: Optional[int]

    def __init__(self, *args, expected_length=None, actual_length=None, **kwargs) -> None:
        self.expected_length = expected_length
        self.actual_length = actual_length
        super().__init__(*args, **kwargs)

    def _body_text(self) -> str:
        if self.expected_length is not None and self.actual_length is not None:
            return (
                f"truncated record: expected {self.expected_length} bytes, "
                f"found {self.actual_length}"
            )
        return "truncated record"

    def _extra_detail_rows(self) -> list[tuple[str, str]]:
        if self.expected_length is not None and self.actual_length is not None:
            return [
                (
                    "length:",
                    f"expected {self.expected_length} bytes, found {self.actual_length}",
                )
            ]
        return []


class EncodingError(MrrcException):
    """A character encoding conversion failed."""

    code = "E301"
    slug = "utf8_invalid"
    _pickle_extra_fields = ("message",)
    _diagnostic_extra_fields = ("message",)
    message: Optional[str]

    def __init__(self, *args, message=None, **kwargs) -> None:
        self.message = message
        super().__init__(*args, **kwargs)

    def _body_text(self) -> str:
        if self.message:
            return f"encoding error: {self.message}"
        return "encoding error"


class XmlError(MrrcException):
    """An error occurred during MARCXML parsing."""

    code = "E401"
    slug = "marcxml_invalid"
    _pickle_extra_fields = ("message",)
    _diagnostic_extra_fields = ("message",)
    message: Optional[str]

    def __init__(self, *args, message=None, **kwargs) -> None:
        self.message = message
        super().__init__(*args, **kwargs)

    def _body_text(self) -> str:
        if self.message:
            return f"XML parse error: {self.message}"
        return "XML parse error"


class JsonError(MrrcException):
    """An error occurred during MARCJSON parsing."""

    code = "E402"
    slug = "marcjson_invalid"
    _pickle_extra_fields = ("message",)
    _diagnostic_extra_fields = ("message",)
    message: Optional[str]

    def __init__(self, *args, message=None, **kwargs) -> None:
        self.message = message
        super().__init__(*args, **kwargs)

    def _body_text(self) -> str:
        if self.message:
            return f"JSON parse error: {self.message}"
        return "JSON parse error"


class WriterError(MrrcException):
    """An error occurred while writing a MARC record."""

    code = "E404"
    slug = "record_too_large_for_iso2709"
    _pickle_extra_fields = ("message",)
    _diagnostic_extra_fields = ("message",)
    message: Optional[str]

    def __init__(self, *args, message=None, **kwargs) -> None:
        self.message = message
        super().__init__(*args, **kwargs)

    def _body_text(self) -> str:
        if self.message:
            return f"writer error: {self.message}"
        return "writer error"


class BadSubfieldCodeWarning(UserWarning):
    """Warning for invalid subfield codes (pymarc compatibility)."""

    code = "W001"
    slug = "bad_subfield_code_warning"


__all__ = [
    "BadSubfieldCode",
    "BadSubfieldCodeWarning",
    "BaseAddressInvalid",
    "BaseAddressNotFound",
    "EncodingError",
    "EndOfRecordNotFound",
    "FatalReaderError",
    "FieldNotFound",
    "InvalidField",
    "InvalidIndicator",
    "JsonError",
    "MrrcException",
    "RecordDirectoryInvalid",
    "RecordLeaderInvalid",
    "RecordLengthInvalid",
    "TruncatedRecord",
    "WriterError",
    "XmlError",
]
