"""Verification that mrrc's exception class names match pymarc's so
that pymarc-shaped ``except`` clauses keep working unchanged after
a port.

This is a *verification + documentation pass*: every exception class
name a pymarc-default loop could plausibly catch by name has a mrrc
class of the same name, importable from ``mrrc.exceptions`` (and
``mrrc`` for the ones re-exported there). This test pins that
promise. Inheritance-hierarchy divergences from pymarc are documented
separately in ``docs/guides/migration-from-pymarc.md``.

The pymarc reference is pymarc 5.3.1's
``pymarc/exceptions.py`` (see
https://gitlab.com/pymarc/pymarc/-/raw/main/pymarc/exceptions.py).
"""

from __future__ import annotations

from pathlib import Path

import pytest

import mrrc
import mrrc.exceptions as mexc

_REPO_ROOT = Path(__file__).resolve().parents[2]


# ---------------------------------------------------------------------
# Names every pymarc port plausibly catches by name
# ---------------------------------------------------------------------

# Classes a typical pymarc-shaped error-handling loop could `except` by
# name. Every entry MUST be importable from `mrrc.exceptions`, be a
# subclass of `Exception` (so it's catchable), and be a subclass of
# `mrrc.MrrcException` (so callers can also catch the base class).
PYMARC_CLASS_NAMES_MRRC_PROVIDES = [
    "RecordLengthInvalid",
    "RecordLeaderInvalid",
    "RecordDirectoryInvalid",
    "BaseAddressInvalid",
    "BaseAddressNotFound",
    "EndOfRecordNotFound",
    "TruncatedRecord",
    "FieldNotFound",
    "FatalReaderError",
    # Warning, not Exception — pymarc 5.3.1 ships it as a Warning
    # subclass; mrrc mirrors that shape.
    "BadSubfieldCodeWarning",
]


# Classes pymarc has that mrrc deliberately does NOT mirror. Listed
# here so the test surfaces any drift (a future addition to pymarc
# adding to this list would need to be re-evaluated against mrrc).
# Each entry includes a one-line reason for the divergence.
PYMARC_CLASS_NAMES_MRRC_OMITS = {
    "NoFieldsFound": (
        "mrrc does not raise on records with zero fields; an empty "
        "Record is a valid in-memory state."
    ),
    "WriteNeedsRecord": (
        "mrrc.MARCWriter is type-annotated; passing a non-Record is "
        "caught by static type checking, not a runtime exception."
    ),
    "NoActiveFile": (
        "mrrc.MARCWriter manages file lifecycle via context manager; "
        "operating on a closed writer raises RuntimeError, not a "
        "typed mrrc exception."
    ),
    "BadLeaderValue": (
        "mrrc.Leader validates fields at construction via its typed "
        "field accessors; bad values surface as ValueError, not as "
        "a typed mrrc exception."
    ),
    "MissingLinkedFields": (
        "880-linkage validation isn't implemented in mrrc; linked-"
        "field consistency is out of scope for the parser."
    ),
}


# ---------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------


@pytest.mark.parametrize("name", PYMARC_CLASS_NAMES_MRRC_PROVIDES)
def test_pymarc_compatible_name_is_importable_from_exceptions(name: str) -> None:
    """Every pymarc-shape name a port could catch is importable from
    ``mrrc.exceptions``."""
    assert hasattr(mexc, name), (
        f"mrrc.exceptions is missing {name} — pymarc-shape error-handling "
        f"code that catches {name} by name would fail at import."
    )


@pytest.mark.parametrize("name", PYMARC_CLASS_NAMES_MRRC_PROVIDES)
def test_pymarc_compatible_class_is_catchable(name: str) -> None:
    """Each shared-name class is a proper Exception (or Warning)
    subclass — so a pymarc-shape ``except`` clause can catch it."""
    cls = getattr(mexc, name)
    # Warning is a separate hierarchy in CPython; allow both.
    assert issubclass(cls, (Exception, Warning)), (
        f"{name} ({cls!r}) is not catchable as Exception/Warning"
    )


@pytest.mark.parametrize("name", PYMARC_CLASS_NAMES_MRRC_PROVIDES)
def test_pymarc_compatible_class_is_mrrc_branded(name: str) -> None:
    """Each exception name (excluding the warning) inherits from
    ``MrrcException``, so callers can catch the mrrc base class
    instead of pymarc-style ``PymarcException``."""
    if name == "BadSubfieldCodeWarning":
        # Warning lives on the Warning hierarchy, not MrrcException.
        pytest.skip("BadSubfieldCodeWarning is a Warning, not an Exception")
    cls = getattr(mexc, name)
    assert issubclass(cls, mexc.MrrcException), (
        f"{name} is not a subclass of MrrcException — pymarc users "
        f"porting code with `except MrrcException:` won't catch it."
    )


@pytest.mark.parametrize("name", PYMARC_CLASS_NAMES_MRRC_PROVIDES)
def test_pymarc_compatible_name_re_exported_from_top_level_mrrc(name: str) -> None:
    """The pymarc-compatible names should also be reachable as
    ``mrrc.<Name>`` so ``from pymarc import RecordDirectoryInvalid``
    has a 1:1 ``from mrrc import RecordDirectoryInvalid`` replacement
    without callers having to know about ``mrrc.exceptions``."""
    if name == "BadSubfieldCodeWarning":
        # Optional re-export — record this as a documented gap if it
        # surfaces, but don't block on it for the warning case.
        if not hasattr(mrrc, name):
            pytest.skip(
                f"{name} not re-exported on top-level mrrc; documented "
                f"as accessible via mrrc.exceptions"
            )
    assert hasattr(mrrc, name), (
        f"mrrc.{name} not re-exported; pymarc users must change "
        f"`from pymarc import {name}` to "
        f"`from mrrc.exceptions import {name}` instead of "
        f"`from mrrc import {name}`."
    )


@pytest.mark.parametrize("name,reason", PYMARC_CLASS_NAMES_MRRC_OMITS.items())
def test_omitted_pymarc_name_is_documented(name: str, reason: str) -> None:
    """Names pymarc has but mrrc deliberately doesn't mirror: assert
    the name is NOT silently present (would be a docs vs reality
    drift) and that the omission has a recorded reason."""
    assert not hasattr(mexc, name), (
        f"mrrc.exceptions.{name} now exists but is documented as "
        f"deliberately omitted ({reason}). Update "
        f"PYMARC_CLASS_NAMES_MRRC_OMITS and the migration doc."
    )
    assert reason, f"Omission of {name} must carry a documented reason"


# ---------------------------------------------------------------------
# `except` clause semantics — real catch flow
# ---------------------------------------------------------------------


def test_specific_class_except_catches_mrrc_raised_exception() -> None:
    """A pymarc-shape ``except RecordDirectoryInvalid:`` actually
    catches what mrrc raises on a record with an invalid directory
    tag byte (the E101 non-ASCII-tag fixture)."""
    bad_bytes = (
        _REPO_ROOT / "tests" / "data" / "error_fixtures" / "e101_non_ascii_tag.bin"
    ).read_bytes()
    reader = mrrc.MARCReader(
        bad_bytes, recovery_mode="strict", validation_level="strict_marc"
    )
    try:
        next(reader)
    except mexc.RecordDirectoryInvalid:
        pass
    except Exception as e:
        pytest.fail(
            f"expected RecordDirectoryInvalid to be raised and caught; "
            f"got {type(e).__name__}: {e}"
        )
    else:
        pytest.fail("expected RecordDirectoryInvalid; nothing raised")


def test_mrrc_exception_base_catches_every_typed_variant() -> None:
    """A pymarc user porting ``except PymarcException:`` to
    ``except MrrcException:`` catches every typed mrrc exception."""
    bad_bytes = b"X0150nam a2200061   4500"  # non-digit in record_length
    reader = mrrc.MARCReader(bad_bytes)
    try:
        next(reader)
    except mexc.MrrcException:
        pass
    except Exception as e:
        pytest.fail(
            f"expected MrrcException to catch the parse failure; "
            f"got bare {type(e).__name__}: {e}"
        )
    else:
        pytest.fail("expected an exception; nothing raised")
