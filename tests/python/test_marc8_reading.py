"""MARC-8 reading: records are decoded in the encoding leader position 09
declares, as in pymarc, and ``force_utf8`` (pymarc's option) or
``character_coding`` (mrrc's) override that choice."""

from __future__ import annotations

import pytest

import mrrc

FIELD_TERMINATOR = b"\x1e"
SUBFIELD_DELIMITER = b"\x1f"
RECORD_TERMINATOR = b"\x1d"

# "Café" in MARC-8: ANSEL 0xE2 (combining acute) precedes its base letter.
CAFE_MARC8 = b"Caf\xe2e"
CAFE = "Café"


def iso2709(coding: bytes, fields, kind: bytes = b"nam ") -> bytes:
    """One ISO 2709 record. ``kind`` is leader positions 05-08 and
    ``coding`` is position 09."""
    directory = b""
    data = b""
    for tag, value in fields:
        start = len(data)
        data += value + FIELD_TERMINATOR
        directory += tag.encode() + b"%04d%05d" % (len(value) + 1, start)
    directory += FIELD_TERMINATOR
    base = 24 + len(directory)
    total = base + len(data) + 1
    leader = b"%05d" % total + kind + coding + b"22%05d   4500" % base
    return leader + directory + data + RECORD_TERMINATOR


def subfield_a(value: bytes) -> bytes:
    return b"10" + SUBFIELD_DELIMITER + b"a" + value


def title(value: bytes, coding: bytes = b" ", **kwargs) -> str:
    data = iso2709(coding, [("245", subfield_a(value))])
    record = next(iter(mrrc.MARCReader(data, **kwargs)))
    return record["245"]["a"]


class TestLeaderDecidesByDefault:
    def test_marc8_leader_decodes_marc8(self):
        assert title(CAFE_MARC8) == CAFE

    def test_utf8_leader_decodes_utf8(self):
        assert title(CAFE.encode(), coding=b"a") == CAFE

    def test_control_fields_decode_as_marc8(self):
        data = iso2709(b" ", [("001", b"ab\xe2e")])
        record = next(iter(mrrc.MARCReader(data)))
        assert record["001"].data == "abé"

    def test_mislabelled_utf8_decodes_as_marc8_like_pymarc(self):
        # 0xC3 and 0xA9 are ANSEL's copyright and music flat signs.
        assert title(CAFE.encode()) == "Caf©♭"


class TestForceUtf8:
    def test_force_utf8_ignores_the_leader(self):
        assert title(CAFE.encode(), force_utf8=True) == CAFE

    def test_force_utf8_matches_character_coding_utf8(self):
        assert title(CAFE.encode(), character_coding="utf-8") == CAFE
        assert (
            title(CAFE.encode(), force_utf8=True, character_coding="utf-8")
            == CAFE
        )

    @pytest.mark.parametrize("coding", ["leader", "detect"])
    def test_force_utf8_conflicts_with_another_character_coding(self, coding):
        data = iso2709(b" ", [("245", subfield_a(b"x"))])
        with pytest.raises(ValueError, match="force_utf8"):
            mrrc.MARCReader(data, force_utf8=True, character_coding=coding)


class TestDetect:
    def test_detect_reads_mislabelled_utf8_as_utf8(self):
        assert title(CAFE.encode(), character_coding="detect") == CAFE

    def test_detect_reads_marc8_as_marc8(self):
        assert title(CAFE_MARC8, character_coding="detect") == CAFE

    def test_detect_reads_a_mixed_file(self):
        data = iso2709(b" ", [("245", subfield_a(CAFE_MARC8))]) + iso2709(
            b" ", [("245", subfield_a(CAFE.encode()))]
        )
        titles = [
            r["245"]["a"]
            for r in mrrc.MARCReader(data, character_coding="detect")
        ]
        assert titles == [CAFE, CAFE]


def test_invalid_character_coding_raises():
    data = iso2709(b" ", [("245", subfield_a(b"x"))])
    with pytest.raises(ValueError, match="character_coding"):
        mrrc.MARCReader(data, character_coding="latin-1")


class TestUndecodableMarc8:
    # 0xAF has no mapping in ANSEL.
    DATA = iso2709(b" ", [("245", subfield_a(b"a\xafb"))])

    def test_structural_substitutes_replacement_character(self):
        record = next(iter(mrrc.MARCReader(self.DATA)))
        assert record["245"]["a"] == "a�b"
        assert record.errors == []

    def test_strict_marc_raises_marc8_error(self):
        reader = mrrc.MARCReader(
            self.DATA, recovery_mode="strict", validation_level="strict_marc"
        )
        with pytest.raises(mrrc.Marc8Error) as excinfo:
            next(iter(reader))
        assert isinstance(excinfo.value, mrrc.EncodingError)
        assert excinfo.value.code == "E302"
        assert excinfo.value.slug == "marc8_invalid"

    def test_permissive_strict_marc_records_marc8_error(self):
        record = next(
            iter(mrrc.MARCReader(self.DATA, validation_level="strict_marc"))
        )
        assert [type(e) for e in record.errors] == [mrrc.Marc8Error]


class TestAuthorityAndHoldingsReaders:
    def test_authority_reader_decodes_marc8(self):
        data = iso2709(b" ", [("100", subfield_a(CAFE_MARC8))], kind=b"nz  ")
        record = next(iter(mrrc.AuthorityMARCReader(data)))
        assert record.heading().subfields_by_code("a") == [CAFE]

    def test_authority_reader_honours_force_utf8(self):
        data = iso2709(
            b" ", [("100", subfield_a(CAFE.encode()))], kind=b"nz  "
        )
        record = next(iter(mrrc.AuthorityMARCReader(data, force_utf8=True)))
        assert record.heading().subfields_by_code("a") == [CAFE]

    def test_holdings_reader_decodes_marc8(self):
        data = iso2709(b" ", [("852", subfield_a(CAFE_MARC8))], kind=b"ny  ")
        record = next(iter(mrrc.HoldingsMARCReader(data)))
        assert record.locations()[0].subfields_by_code("a") == [CAFE]

    def test_holdings_reader_honours_character_coding(self):
        data = iso2709(
            b" ", [("852", subfield_a(CAFE.encode()))], kind=b"ny  "
        )
        reader = mrrc.HoldingsMARCReader(data, character_coding="detect")
        location = next(iter(reader)).locations()[0]
        assert location.subfields_by_code("a") == [CAFE]

    def test_authority_reader_rejects_conflicting_options(self):
        data = iso2709(b" ", [("100", subfield_a(b"x"))], kind=b"nz  ")
        with pytest.raises(ValueError, match="force_utf8"):
            mrrc.AuthorityMARCReader(
                data, force_utf8=True, character_coding="detect"
            )


def test_escape_sequence_right_after_esc_s_is_recognised():
    # pymarc 5.3.1 reads the byte after ESC s as a character, dropping the
    # ESC of an escape sequence that follows immediately and emitting ")E".
    value = b"\x1b(NmIR\x1bs\x1b)E" + CAFE_MARC8
    assert title(value) == "Мир" + CAFE
