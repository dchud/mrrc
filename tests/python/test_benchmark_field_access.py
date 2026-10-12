"""
Benchmark tests for field access on parsed records.

Records are read once, outside the timed region; each benchmark times one
access pattern over every record. The fixtures run from records of 12-30
fields to records of 800-1,000, so access costs that grow with a record's
field count show up as the fixtures grow.
"""

import io

import pytest

from mrrc import MARCReader

FIXTURES = ["fixture_realistic", "fixture_large", "fixture_very_large"]


@pytest.fixture(scope="module", params=FIXTURES)
def records(request):
    data = request.getfixturevalue(request.param)
    return list(MARCReader(io.BytesIO(data)))


def getitem_245(records):
    return sum(1 for record in records if record["245"] is not None)


def get_fields_650(records):
    return sum(len(record.get_fields("650")) for record in records)


def field_handle_loop(records):
    """pymarc's canonical loop: every field as a live handle, reading $a."""
    found = 0
    for record in records:
        for field in record.get_fields():
            if not field.is_control_field() and field["a"] is not None:
                found += 1
    return found


class TestFieldAccessBenchmarks:
    """Benchmarks for reading fields from parsed records."""

    @pytest.mark.benchmark
    def test_getitem_245(self, benchmark, records):
        """Benchmark record["245"] on every record."""
        assert benchmark(getitem_245, records) == len(records)

    @pytest.mark.benchmark
    def test_get_fields_650(self, benchmark, records):
        """Benchmark record.get_fields("650") on every record."""
        assert benchmark(get_fields_650, records) > 0

    @pytest.mark.benchmark
    def test_field_handle_loop(self, benchmark, records):
        """Benchmark `for f in record.get_fields(): f["a"]` on every record."""
        assert benchmark(field_handle_loop, records) > 0
