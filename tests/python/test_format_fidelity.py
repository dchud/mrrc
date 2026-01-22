"""Round-trip fidelity tests for all format readers/writers.

Tests that records can be written and read back with full fidelity
across all supported formats (Protobuf, Arrow, FlatBuffers, MessagePack).
"""

import os
import tempfile
import pytest
import mrrc


# Path to the fidelity test set (100 diverse MARC records)
FIDELITY_TEST_SET = os.path.join(
    os.path.dirname(__file__), "..", "data", "fixtures", "fidelity_test_100.mrc"
)


def records_equal(rec1, rec2) -> bool:
    """Compare two records for equality, checking all fields and subfields."""
    # Compare control fields
    cf1 = dict(rec1._inner.control_fields())
    cf2 = dict(rec2._inner.control_fields())
    if cf1 != cf2:
        return False

    # Compare data fields
    fields1 = rec1._inner.fields()
    fields2 = rec2._inner.fields()

    if len(fields1) != len(fields2):
        return False

    for f1, f2 in zip(fields1, fields2):
        if f1.tag != f2.tag:
            return False
        if f1.indicator1 != f2.indicator1:
            return False
        if f1.indicator2 != f2.indicator2:
            return False

        subs1 = f1.subfields()
        subs2 = f2.subfields()
        if len(subs1) != len(subs2):
            return False

        for s1, s2 in zip(subs1, subs2):
            if s1.code != s2.code or s1.value != s2.value:
                return False

    return True


@pytest.fixture(scope="module")
def fidelity_records():
    """Load the fidelity test set records."""
    records = list(mrrc.read(FIDELITY_TEST_SET))
    assert len(records) > 0, f"Expected records, got {len(records)}"
    return records


class TestProtobufFidelity:
    """Test round-trip fidelity for Protobuf format."""

    def test_single_record_roundtrip(self, fidelity_records):
        """Test single record serialization/deserialization."""
        record = fidelity_records[0]

        # Serialize and deserialize
        pb_bytes = mrrc.record_to_protobuf(record._inner)
        restored = mrrc.protobuf_to_record(pb_bytes)

        # Create wrapper for comparison
        restored_wrapper = mrrc.Record()
        restored_wrapper._inner = restored

        assert records_equal(record, restored_wrapper)

    def test_all_records_roundtrip(self, fidelity_records):
        """Test round-trip fidelity for all 100 records via file."""
        with tempfile.NamedTemporaryFile(suffix=".pb", delete=False) as f:
            pb_path = f.name

        try:
            # Write all records
            count = mrrc.write(fidelity_records, pb_path)
            assert count == len(fidelity_records)

            # Read back and compare
            restored = list(mrrc.read(pb_path))
            assert len(restored) == len(fidelity_records)

            for i, (orig, rest) in enumerate(zip(fidelity_records, restored)):
                # Create wrapper for restored record
                rest_wrapper = mrrc.Record()
                rest_wrapper._inner = rest
                assert records_equal(orig, rest_wrapper), f"Record {i} mismatch"

        finally:
            os.unlink(pb_path)


class TestArrowFidelity:
    """Test round-trip fidelity for Arrow format."""

    def test_all_records_roundtrip(self, fidelity_records):
        """Test round-trip fidelity for all 100 records via file."""
        with tempfile.NamedTemporaryFile(suffix=".arrow", delete=False) as f:
            arrow_path = f.name

        try:
            # Write all records
            count = mrrc.write(fidelity_records, arrow_path)
            assert count == len(fidelity_records)

            # Read back and compare
            restored = list(mrrc.read(arrow_path))
            assert len(restored) == len(fidelity_records)

            for i, (orig, rest) in enumerate(zip(fidelity_records, restored)):
                rest_wrapper = mrrc.Record()
                rest_wrapper._inner = rest
                assert records_equal(orig, rest_wrapper), f"Record {i} mismatch"

        finally:
            os.unlink(arrow_path)


class TestFlatbuffersFidelity:
    """Test round-trip fidelity for FlatBuffers format."""

    def test_single_record_roundtrip(self, fidelity_records):
        """Test single record serialization/deserialization."""
        record = fidelity_records[0]

        # Serialize and deserialize
        fb_bytes = mrrc.record_to_flatbuffers(record._inner)
        restored = mrrc.flatbuffers_to_record(fb_bytes)

        rest_wrapper = mrrc.Record()
        rest_wrapper._inner = restored

        assert records_equal(record, rest_wrapper)

    def test_all_records_roundtrip(self, fidelity_records):
        """Test round-trip fidelity for all 100 records via file."""
        with tempfile.NamedTemporaryFile(suffix=".fb", delete=False) as f:
            fb_path = f.name

        try:
            # Write all records
            count = mrrc.write(fidelity_records, fb_path)
            assert count == len(fidelity_records)

            # Read back and compare
            restored = list(mrrc.read(fb_path))
            assert len(restored) == len(fidelity_records)

            for i, (orig, rest) in enumerate(zip(fidelity_records, restored)):
                rest_wrapper = mrrc.Record()
                rest_wrapper._inner = rest
                assert records_equal(orig, rest_wrapper), f"Record {i} mismatch"

        finally:
            os.unlink(fb_path)


class TestMessagePackFidelity:
    """Test round-trip fidelity for MessagePack format."""

    def test_single_record_roundtrip(self, fidelity_records):
        """Test single record serialization/deserialization."""
        record = fidelity_records[0]

        # Serialize and deserialize
        mp_bytes = mrrc.record_to_messagepack(record._inner)
        restored = mrrc.messagepack_to_record(mp_bytes)

        rest_wrapper = mrrc.Record()
        rest_wrapper._inner = restored

        assert records_equal(record, rest_wrapper)

    def test_all_records_roundtrip(self, fidelity_records):
        """Test round-trip fidelity for all 100 records via file."""
        with tempfile.NamedTemporaryFile(suffix=".msgpack", delete=False) as f:
            mp_path = f.name

        try:
            # Write all records
            count = mrrc.write(fidelity_records, mp_path)
            assert count == len(fidelity_records)

            # Read back and compare
            restored = list(mrrc.read(mp_path))
            assert len(restored) == len(fidelity_records)

            for i, (orig, rest) in enumerate(zip(fidelity_records, restored)):
                rest_wrapper = mrrc.Record()
                rest_wrapper._inner = rest
                assert records_equal(orig, rest_wrapper), f"Record {i} mismatch"

        finally:
            os.unlink(mp_path)


class TestCrossFormatFidelity:
    """Test that records maintain fidelity when converted between formats."""

    def test_marc_to_protobuf_to_flatbuffers(self, fidelity_records):
        """Test MARC -> Protobuf -> FlatBuffers conversion chain."""
        record = fidelity_records[0]

        # MARC -> Protobuf
        pb_bytes = mrrc.record_to_protobuf(record._inner)
        pb_record = mrrc.protobuf_to_record(pb_bytes)

        # Protobuf -> FlatBuffers
        fb_bytes = mrrc.record_to_flatbuffers(pb_record)
        fb_record = mrrc.flatbuffers_to_record(fb_bytes)

        fb_wrapper = mrrc.Record()
        fb_wrapper._inner = fb_record

        assert records_equal(record, fb_wrapper)

    def test_marc_to_messagepack_to_arrow(self, fidelity_records):
        """Test MARC -> MessagePack -> Arrow conversion chain via files."""
        with tempfile.TemporaryDirectory() as tmpdir:
            mp_path = os.path.join(tmpdir, "test.msgpack")
            arrow_path = os.path.join(tmpdir, "test.arrow")

            # Write subset to MessagePack
            subset = fidelity_records[:10]
            mrrc.write(subset, mp_path)

            # Read from MessagePack and write to Arrow
            mp_records = list(mrrc.read(mp_path))
            mrrc.write(mp_records, arrow_path, format="arrow")

            # Read back from Arrow
            arrow_records = list(mrrc.read(arrow_path))

            assert len(arrow_records) == 10

            for i, (orig, rest) in enumerate(zip(subset, arrow_records)):
                rest_wrapper = mrrc.Record()
                rest_wrapper._inner = rest
                assert records_equal(orig, rest_wrapper), f"Record {i} mismatch"
