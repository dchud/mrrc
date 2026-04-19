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

from __future__ import annotations

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
)


class _MrrcExceptionBase:
    """Mixin providing positional context attributes, formatting, and pickle
    support for every mrrc exception class.

    Kept separate from the actual base ``MrrcException`` so subclasses can
    override ``_body_text`` independently of the inheritance chain to
    Exception itself.
    """

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
            return header

        label_width = max(len(label) for label, _ in rows)
        body = "\n".join(
            f"  {label}{' ' * (label_width - len(label) + 1)}{value}"
            for label, value in rows
        )
        return f"{header}\n{body}"

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


class MrrcException(_MrrcExceptionBase, Exception):
    """Base exception for all mrrc errors."""


class RecordLengthInvalid(MrrcException):
    """The leader's record-length field is invalid (non-numeric, too small, etc.)."""

    def _body_text(self) -> str:
        if self.found is not None and self.expected is not None:
            return f"invalid record length {self.found!r} — expected {self.expected}"
        return "invalid record length"


class RecordLeaderInvalid(MrrcException):
    """The 24-byte record leader is malformed."""

    def _body_text(self) -> str:
        if self.found is not None and self.expected is not None:
            return f"invalid leader: found {self.found!r} — expected {self.expected}"
        return "invalid leader"


class BaseAddressInvalid(MrrcException):
    """The leader's base-address-of-data field is invalid."""

    def _body_text(self) -> str:
        if self.found is not None and self.expected is not None:
            return f"invalid base address {self.found!r} — expected {self.expected}"
        return "invalid base address"


class BaseAddressNotFound(MrrcException):
    """The leader claims a base address of data that does not exist in the stream."""

    def _body_text(self) -> str:
        return "base address not found"


class RecordDirectoryInvalid(MrrcException):
    """A directory entry is structurally invalid (bad tag, length, or start position).

    Catches mrrc-specific subclasses ``InvalidIndicator``, ``BadSubfieldCode``,
    and ``InvalidField`` as well — pymarc-style ``except`` clauses keep
    working unchanged.
    """

    def _body_text(self) -> str:
        if self.found is not None and self.expected is not None:
            return f"invalid directory entry {self.found!r} — expected {self.expected}"
        return "invalid directory entry"


class EndOfRecordNotFound(MrrcException):
    """The end-of-record marker was not found where expected.

    Catches mrrc-specific subclass ``TruncatedRecord`` as well.
    """

    def _body_text(self) -> str:
        return "end-of-record marker not found"


class FieldNotFound(MrrcException):
    """A requested field was not present in the record (accessor error)."""

    def _body_text(self) -> str:
        return f"field {self.field_tag} not found" if self.field_tag else "field not found"


class FatalReaderError(MrrcException):
    """Unrecoverable error during record reading.

    Reserved for catastrophic states; not raised directly by the current
    Rust core but kept for pymarc compatibility and for future use.
    """


# --- mrrc-specific subclasses (extend the pymarc-named parents) -----------


class InvalidIndicator(RecordDirectoryInvalid):
    """An indicator byte was invalid for its position."""

    def _body_text(self) -> str:
        if self.found is not None and self.expected is not None:
            return f"invalid {self.found!r} — expected {self.expected}"
        return "invalid indicator"


class BadSubfieldCode(RecordDirectoryInvalid):
    """A subfield code byte was not a printable ASCII character."""

    def _body_text(self) -> str:
        if self.subfield_code is not None:
            return f"invalid subfield code 0x{self.subfield_code:02X}"
        return "invalid subfield code"


class InvalidField(RecordDirectoryInvalid):
    """A data field is structurally invalid in some way not covered by the more specific subclasses."""

    _pickle_extra_fields = ("message",)
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

    _pickle_extra_fields = ("expected_length", "actual_length")
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

    _pickle_extra_fields = ("message",)
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

    _pickle_extra_fields = ("message",)
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

    _pickle_extra_fields = ("message",)
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

    _pickle_extra_fields = ("message",)
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
