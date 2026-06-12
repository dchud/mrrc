"""Happy-path reading tests for AuthorityMARCReader and HoldingsMARCReader.

These mirror the Rust integration tests for the same fixtures
(``tests/data/simple_authority.mrc`` and ``tests/data/simple_holdings.mrc``)
and exercise the typed accessors on AuthorityRecord and HoldingsRecord.
Synthetic records built in-memory cover the accessors that the minimal
fixtures do not populate (tracings, locations, captions, enumeration,
textual holdings).
"""

import io

import pytest

import mrrc

AUTHORITY_FIXTURE = "tests/data/simple_authority.mrc"
HOLDINGS_FIXTURE = "tests/data/simple_holdings.mrc"

FIELD_TERMINATOR = b"\x1e"
RECORD_TERMINATOR = b"\x1d"
SUBFIELD_DELIMITER = b"\x1f"


def build_record_bytes(record_type, control_fields, data_fields):
    """Build a minimal ISO 2709 record with the given leader type byte.

    ``control_fields`` is a list of ``(tag, value)`` pairs; ``data_fields``
    is a list of ``(tag, ind1, ind2, [(code, value), ...])`` tuples.
    """
    directory = b""
    data = b""
    for tag, value in control_fields:
        payload = value.encode() + FIELD_TERMINATOR
        directory += f"{tag}{len(payload):04}{len(data):05}".encode()
        data += payload
    for tag, ind1, ind2, subfields in data_fields:
        payload = (ind1 + ind2).encode()
        for code, value in subfields:
            payload += SUBFIELD_DELIMITER + code.encode() + value.encode()
        payload += FIELD_TERMINATOR
        directory += f"{tag}{len(payload):04}{len(data):05}".encode()
        data += payload
    directory += FIELD_TERMINATOR
    base_address = 24 + len(directory)
    record_length = base_address + len(data) + 1
    leader = f"{record_length:05}n{record_type}|  22{base_address:05}1a 4500".encode()
    assert len(leader) == 24
    return leader + directory + data + RECORD_TERMINATOR


def read_authority(data):
    reader = mrrc.AuthorityMARCReader(io.BytesIO(data))
    record = reader.read_record()
    assert record is not None
    return record


def read_holdings(data):
    reader = mrrc.HoldingsMARCReader(io.BytesIO(data))
    record = reader.read_record()
    assert record is not None
    return record


class TestAuthorityFixtureReading:
    """Read the simple_authority.mrc fixture and check every populated accessor."""

    @pytest.fixture()
    def record(self):
        with open(AUTHORITY_FIXTURE, "rb") as fh:
            reader = mrrc.AuthorityMARCReader(fh)
            record = reader.read_record()
        assert record is not None
        return record

    def test_leader_record_type(self, record):
        assert record.leader.record_type == "z"
        assert record.record_type() == "z"

    def test_control_field(self, record):
        assert record.get_control_field("001") == "n79021800"
        assert record.get_control_field("999") is None

    def test_heading(self, record):
        heading = record.heading()
        assert heading is not None
        assert heading.tag == "100"
        assert heading.subfields_by_code("a") == ["Smith, John"]

    def test_heading_text(self, record):
        assert record.heading_text() == "Smith, John"

    def test_get_fields(self, record):
        fields = record.get_fields("100")
        assert fields is not None
        assert len(fields) == 1
        assert fields[0].tag == "100"
        assert record.get_fields("999") is None

    def test_get_field(self, record):
        field = record.get_field("100")
        assert field is not None
        assert field.tag == "100"
        assert record.get_field("999") is None

    def test_get_field_or_err_success(self, record):
        field = record.get_field_or_err("100")
        assert field.tag == "100"

    def test_no_parse_errors(self, record):
        assert record.errors == []

    def test_accessors_empty_on_minimal_fixture(self, record):
        assert record.see_from_tracings() == []
        assert record.see_also_tracings() == []
        assert record.notes() == []
        assert record.linking_entries() == []

    def test_repr_and_str(self, record):
        assert "AuthorityRecord" in repr(record)
        assert str(record)


class TestAuthorityIteration:
    """Iterator and context-manager protocols on AuthorityMARCReader."""

    def test_iteration_yields_single_record(self):
        with open(AUTHORITY_FIXTURE, "rb") as fh:
            reader = mrrc.AuthorityMARCReader(fh)
            records = list(reader)
        assert len(records) == 1
        assert records[0].record_type() == "z"

    def test_next_raises_stop_iteration_at_eof(self):
        with open(AUTHORITY_FIXTURE, "rb") as fh:
            reader = mrrc.AuthorityMARCReader(fh)
            next(reader)
            with pytest.raises(StopIteration):
                next(reader)

    def test_read_record_returns_none_at_eof(self):
        with open(AUTHORITY_FIXTURE, "rb") as fh:
            reader = mrrc.AuthorityMARCReader(fh)
            assert reader.read_record() is not None
            assert reader.read_record() is None

    def test_context_manager(self):
        with open(AUTHORITY_FIXTURE, "rb") as fh, mrrc.AuthorityMARCReader(fh) as reader:
            record = next(reader)
        assert record.heading_text() == "Smith, John"

    def test_reads_from_bytesio(self):
        with open(AUTHORITY_FIXTURE, "rb") as fh:
            data = fh.read()
        record = read_authority(data)
        assert record.get_control_field("001") == "n79021800"

    def test_reads_multiple_records_from_stream(self):
        with open(AUTHORITY_FIXTURE, "rb") as fh:
            data = fh.read()
        reader = mrrc.AuthorityMARCReader(io.BytesIO(data * 3))
        records = list(reader)
        assert len(records) == 3
        assert all(r.heading_text() == "Smith, John" for r in records)


class TestAuthoritySyntheticAccessors:
    """Accessors for tracings, notes, and linking entries on a built record."""

    @pytest.fixture()
    def record(self):
        data = build_record_bytes(
            "z",
            [("001", "n79000001")],
            [
                ("100", "1", " ", [("a", "Twain, Mark"), ("d", "1835-1910")]),
                ("400", "1", " ", [("a", "Clemens, Samuel Langhorne")]),
                ("400", "1", " ", [("a", "Snodgrass, Quintus Curtius")]),
                ("500", "1", " ", [("a", "Conte, Louis de")]),
                ("667", " ", " ", [("a", "Machine-derived authority record")]),
                ("700", "1", " ", [("a", "Twain, Mark (alternate)")]),
            ],
        )
        return read_authority(data)

    def test_heading_with_multiple_subfields(self, record):
        heading = record.heading()
        assert heading is not None
        assert heading.subfields_by_code("a") == ["Twain, Mark"]
        assert heading.subfields_by_code("d") == ["1835-1910"]
        assert record.heading_text() == "Twain, Mark"

    def test_see_from_tracings(self, record):
        tracings = record.see_from_tracings()
        assert [f.tag for f in tracings] == ["400", "400"]
        values = [f.subfields_by_code("a")[0] for f in tracings]
        assert values == [
            "Clemens, Samuel Langhorne",
            "Snodgrass, Quintus Curtius",
        ]

    def test_see_also_tracings(self, record):
        tracings = record.see_also_tracings()
        assert [f.tag for f in tracings] == ["500"]
        assert tracings[0].subfields_by_code("a") == ["Conte, Louis de"]

    def test_notes(self, record):
        notes = record.notes()
        assert [f.tag for f in notes] == ["667"]
        assert notes[0].subfields_by_code("a") == [
            "Machine-derived authority record"
        ]

    def test_linking_entries(self, record):
        entries = record.linking_entries()
        assert [f.tag for f in entries] == ["700"]
        assert entries[0].subfields_by_code("a") == ["Twain, Mark (alternate)"]


class TestHoldingsFixtureReading:
    """Read the simple_holdings.mrc fixture and check every populated accessor."""

    @pytest.fixture()
    def record(self):
        with open(HOLDINGS_FIXTURE, "rb") as fh:
            reader = mrrc.HoldingsMARCReader(fh)
            record = reader.read_record()
        assert record is not None
        return record

    def test_leader_record_type(self, record):
        assert record.leader.record_type == "x"
        assert record.record_type() == "x"

    def test_control_field(self, record):
        assert record.get_control_field("001") == "n79021800"
        assert record.get_control_field("999") is None

    def test_get_fields(self, record):
        fields = record.get_fields("100")
        assert fields is not None
        assert len(fields) == 1
        assert fields[0].subfields_by_code("a") == ["Smith, John"]
        assert record.get_fields("999") is None

    def test_get_field(self, record):
        field = record.get_field("100")
        assert field is not None
        assert field.tag == "100"
        assert record.get_field("999") is None

    def test_get_field_or_err_success(self, record):
        field = record.get_field_or_err("100")
        assert field.tag == "100"

    def test_no_parse_errors(self, record):
        assert record.errors == []

    def test_accessors_empty_on_minimal_fixture(self, record):
        assert record.locations() == []
        assert record.captions_basic() == []
        assert record.captions_supplements() == []
        assert record.captions_indexes() == []
        assert record.enumeration_basic() == []
        assert record.enumeration_supplements() == []
        assert record.enumeration_indexes() == []
        assert record.textual_holdings_basic() == []
        assert record.textual_holdings_supplements() == []
        assert record.textual_holdings_indexes() == []

    def test_repr_and_str(self, record):
        assert "HoldingsRecord" in repr(record)
        assert str(record)


class TestHoldingsIteration:
    """Iterator and context-manager protocols on HoldingsMARCReader."""

    def test_iteration_yields_single_record(self):
        with open(HOLDINGS_FIXTURE, "rb") as fh:
            reader = mrrc.HoldingsMARCReader(fh)
            records = list(reader)
        assert len(records) == 1
        assert records[0].record_type() == "x"

    def test_next_raises_stop_iteration_at_eof(self):
        with open(HOLDINGS_FIXTURE, "rb") as fh:
            reader = mrrc.HoldingsMARCReader(fh)
            next(reader)
            with pytest.raises(StopIteration):
                next(reader)

    def test_read_record_returns_none_at_eof(self):
        with open(HOLDINGS_FIXTURE, "rb") as fh:
            reader = mrrc.HoldingsMARCReader(fh)
            assert reader.read_record() is not None
            assert reader.read_record() is None

    def test_context_manager(self):
        with open(HOLDINGS_FIXTURE, "rb") as fh, mrrc.HoldingsMARCReader(fh) as reader:
            record = next(reader)
        assert record.record_type() == "x"

    def test_reads_from_bytesio(self):
        with open(HOLDINGS_FIXTURE, "rb") as fh:
            data = fh.read()
        record = read_holdings(data)
        assert record.get_control_field("001") == "n79021800"

    def test_reads_multiple_records_from_stream(self):
        with open(HOLDINGS_FIXTURE, "rb") as fh:
            data = fh.read()
        reader = mrrc.HoldingsMARCReader(io.BytesIO(data * 3))
        records = list(reader)
        assert len(records) == 3
        assert all(r.record_type() == "x" for r in records)


class TestHoldingsSyntheticAccessors:
    """Location, caption, enumeration, and textual-holdings accessors."""

    @pytest.fixture()
    def record(self):
        data = build_record_bytes(
            "x",
            [("001", "ocm00098765")],
            [
                ("852", "0", " ", [("a", "DLC"), ("b", "Main Library")]),
                ("853", "2", "0", [("a", "v."), ("b", "no.")]),
                ("854", "2", "0", [("a", "suppl.")]),
                ("855", "2", "0", [("a", "index")]),
                ("863", "4", "0", [("a", "1"), ("b", "1")]),
                ("864", "4", "0", [("a", "1")]),
                ("865", "4", "0", [("a", "1")]),
                ("866", "4", "1", [("a", "v.1-v.10 (1990-1999)")]),
                ("867", "4", "1", [("a", "suppl. 1990")]),
                ("868", "4", "1", [("a", "index 1990-1999")]),
            ],
        )
        return read_holdings(data)

    def test_control_field(self, record):
        assert record.get_control_field("001") == "ocm00098765"

    def test_locations(self, record):
        locations = record.locations()
        assert [f.tag for f in locations] == ["852"]
        assert locations[0].subfields_by_code("b") == ["Main Library"]

    def test_captions(self, record):
        assert [f.tag for f in record.captions_basic()] == ["853"]
        assert [f.tag for f in record.captions_supplements()] == ["854"]
        assert [f.tag for f in record.captions_indexes()] == ["855"]
        assert record.captions_basic()[0].subfields_by_code("a") == ["v."]

    def test_enumeration(self, record):
        assert [f.tag for f in record.enumeration_basic()] == ["863"]
        assert [f.tag for f in record.enumeration_supplements()] == ["864"]
        assert [f.tag for f in record.enumeration_indexes()] == ["865"]

    def test_textual_holdings(self, record):
        basic = record.textual_holdings_basic()
        assert [f.tag for f in basic] == ["866"]
        assert basic[0].subfields_by_code("a") == ["v.1-v.10 (1990-1999)"]
        assert [f.tag for f in record.textual_holdings_supplements()] == ["867"]
        assert [f.tag for f in record.textual_holdings_indexes()] == ["868"]

    def test_repeated_locations(self):
        data = build_record_bytes(
            "x",
            [("001", "ocm00098766")],
            [
                ("852", "0", " ", [("b", "Main Library")]),
                ("852", "0", " ", [("b", "Branch Library")]),
            ],
        )
        record = read_holdings(data)
        locations = record.locations()
        assert len(locations) == 2
        values = [f.subfields_by_code("b")[0] for f in locations]
        assert values == ["Main Library", "Branch Library"]
