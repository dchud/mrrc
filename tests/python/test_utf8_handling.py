"""Invalid UTF-8 in a record decoded as UTF-8 follows pymarc's
``utf8_handling``: ``"strict"`` (the default) makes it an E301 error for
the recovery mode to handle, ``"replace"`` substitutes U+FFFD, and
``"ignore"`` drops the bytes."""

from __future__ import annotations

import pytest

import mrrc

from .test_marc8_reading import iso2709, subfield_a

# A UTF-8 record whose 245$a holds one invalid byte (0xFF).
DATA = iso2709(b"a", [("001", b"id1"), ("245", subfield_a(b"Caf\xffe"))])


def test_default_recovery_drops_the_field_and_records_e301():
    (record,) = list(mrrc.MARCReader(DATA))
    assert "245" not in record
    assert record["001"].data == "id1"
    assert [type(e) for e in record.errors] == [mrrc.EncodingError]
    assert record.errors[0].code == "E301"


def test_permissive_yields_none_like_pymarc():
    reader = mrrc.MARCReader(DATA, permissive=True)
    assert next(reader) is None
    assert isinstance(reader.current_exception, mrrc.EncodingError)


def test_strict_recovery_raises():
    reader = mrrc.MARCReader(DATA, recovery_mode="strict")
    with pytest.raises(mrrc.EncodingError):
        next(reader)


def test_strict_is_independent_of_validation_level():
    for level in ("structural", "strict_marc"):
        reader = mrrc.MARCReader(
            DATA, recovery_mode="strict", validation_level=level
        )
        with pytest.raises(mrrc.EncodingError):
            next(reader)


def test_replace_substitutes_the_replacement_character():
    (record,) = list(mrrc.MARCReader(DATA, utf8_handling="replace"))
    assert record["245"]["a"] == "Caf�e"
    assert record.errors == []


def test_ignore_drops_the_invalid_bytes():
    (record,) = list(mrrc.MARCReader(DATA, utf8_handling="ignore"))
    assert record["245"]["a"] == "Cafe"


def test_replace_holds_under_strict_marc():
    reader = mrrc.MARCReader(
        DATA,
        recovery_mode="strict",
        validation_level="strict_marc",
        utf8_handling="replace",
    )
    assert next(reader)["245"]["a"] == "Caf�e"


def test_invalid_value_raises():
    with pytest.raises(ValueError, match="utf8_handling"):
        mrrc.MARCReader(DATA, utf8_handling="backslashreplace")


@pytest.mark.parametrize(
    "reader_class, kind, tag",
    [
        (mrrc.AuthorityMARCReader, b"nz  ", "100"),
        (mrrc.HoldingsMARCReader, b"ny  ", "852"),
    ],
)
def test_authority_and_holdings_readers(reader_class, kind, tag):
    data = iso2709(b"a", [(tag, subfield_a(b"Caf\xffe"))], kind=kind)
    with pytest.raises(mrrc.EncodingError):
        next(iter(reader_class(data, recovery_mode="strict")))
    record = next(iter(reader_class(data, utf8_handling="replace")))
    assert record.errors == []
