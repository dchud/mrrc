"""
Benchmark tests for format reader/writer performance.

Measures Python wrapper overhead and throughput for all supported formats:
- ISO 2709 MARC (baseline)
- Protobuf
- Arrow
- FlatBuffers
- MessagePack
"""

import io
import os
import tempfile
import pytest
import mrrc


# Path to test fixtures
FIXTURES_DIR = os.path.join(os.path.dirname(__file__), "..", "data", "fixtures")


@pytest.fixture(scope="module")
def fixture_1k():
    """Load 1k records fixture as bytes."""
    with open(os.path.join(FIXTURES_DIR, "1k_records.mrc"), "rb") as f:
        return f.read()


@pytest.fixture(scope="module")
def records_1k(fixture_1k):
    """Parse 1k records into a list."""
    data = io.BytesIO(fixture_1k)
    reader = mrrc.MARCReader(data)
    return list(reader)


class TestFormatReadBenchmarks:
    """Benchmarks for reading records in different formats."""

    @pytest.mark.benchmark
    def test_read_marc_1k(self, benchmark, fixture_1k):
        """Baseline: Read 1k records from ISO 2709 MARC."""

        def read_marc():
            data = io.BytesIO(fixture_1k)
            reader = mrrc.MARCReader(data)
            return list(reader)

        result = benchmark(read_marc)
        assert len(result) == 1000

    @pytest.mark.benchmark
    def test_read_protobuf_1k(self, benchmark, records_1k):
        """Read 1k records from Protobuf format."""
        # First write to protobuf
        with tempfile.NamedTemporaryFile(suffix=".pb", delete=False) as f:
            pb_path = f.name

        try:
            mrrc.write(records_1k, pb_path)

            def read_protobuf():
                return list(mrrc.read(pb_path))

            result = benchmark(read_protobuf)
            assert len(result) == 1000
        finally:
            os.unlink(pb_path)

    @pytest.mark.benchmark
    def test_read_arrow_1k(self, benchmark, records_1k):
        """Read 1k records from Arrow format."""
        with tempfile.NamedTemporaryFile(suffix=".arrow", delete=False) as f:
            arrow_path = f.name

        try:
            mrrc.write(records_1k, arrow_path)

            def read_arrow():
                return list(mrrc.read(arrow_path))

            result = benchmark(read_arrow)
            assert len(result) == 1000
        finally:
            os.unlink(arrow_path)

    @pytest.mark.benchmark
    def test_read_flatbuffers_1k(self, benchmark, records_1k):
        """Read 1k records from FlatBuffers format."""
        with tempfile.NamedTemporaryFile(suffix=".fb", delete=False) as f:
            fb_path = f.name

        try:
            mrrc.write(records_1k, fb_path)

            def read_flatbuffers():
                return list(mrrc.read(fb_path))

            result = benchmark(read_flatbuffers)
            assert len(result) == 1000
        finally:
            os.unlink(fb_path)

    @pytest.mark.benchmark
    def test_read_messagepack_1k(self, benchmark, records_1k):
        """Read 1k records from MessagePack format."""
        with tempfile.NamedTemporaryFile(suffix=".msgpack", delete=False) as f:
            mp_path = f.name

        try:
            mrrc.write(records_1k, mp_path)

            def read_messagepack():
                return list(mrrc.read(mp_path))

            result = benchmark(read_messagepack)
            assert len(result) == 1000
        finally:
            os.unlink(mp_path)


class TestFormatWriteBenchmarks:
    """Benchmarks for writing records in different formats."""

    @pytest.mark.benchmark
    def test_write_marc_1k(self, benchmark, records_1k):
        """Baseline: Write 1k records to ISO 2709 MARC."""

        def write_marc():
            with tempfile.NamedTemporaryFile(suffix=".mrc", delete=True) as f:
                mrrc.write(records_1k, f.name)
                return f.name

        benchmark(write_marc)

    @pytest.mark.benchmark
    def test_write_protobuf_1k(self, benchmark, records_1k):
        """Write 1k records to Protobuf format."""

        def write_protobuf():
            with tempfile.NamedTemporaryFile(suffix=".pb", delete=True) as f:
                mrrc.write(records_1k, f.name)
                return f.name

        benchmark(write_protobuf)

    @pytest.mark.benchmark
    def test_write_arrow_1k(self, benchmark, records_1k):
        """Write 1k records to Arrow format."""

        def write_arrow():
            with tempfile.NamedTemporaryFile(suffix=".arrow", delete=True) as f:
                mrrc.write(records_1k, f.name)
                return f.name

        benchmark(write_arrow)

    @pytest.mark.benchmark
    def test_write_flatbuffers_1k(self, benchmark, records_1k):
        """Write 1k records to FlatBuffers format."""

        def write_flatbuffers():
            with tempfile.NamedTemporaryFile(suffix=".fb", delete=True) as f:
                mrrc.write(records_1k, f.name)
                return f.name

        benchmark(write_flatbuffers)

    @pytest.mark.benchmark
    def test_write_messagepack_1k(self, benchmark, records_1k):
        """Write 1k records to MessagePack format."""

        def write_messagepack():
            with tempfile.NamedTemporaryFile(suffix=".msgpack", delete=True) as f:
                mrrc.write(records_1k, f.name)
                return f.name

        benchmark(write_messagepack)


class TestSingleRecordSerializationBenchmarks:
    """Benchmarks for single-record serialization overhead."""

    @pytest.mark.benchmark
    def test_serialize_protobuf_single(self, benchmark, records_1k):
        """Measure single-record Protobuf serialization overhead."""
        record = records_1k[0]

        def serialize():
            return mrrc.record_to_protobuf(record._inner)

        result = benchmark(serialize)
        assert len(result) > 0

    @pytest.mark.benchmark
    def test_serialize_flatbuffers_single(self, benchmark, records_1k):
        """Measure single-record FlatBuffers serialization overhead."""
        record = records_1k[0]

        def serialize():
            return mrrc.record_to_flatbuffers(record._inner)

        result = benchmark(serialize)
        assert len(result) > 0

    @pytest.mark.benchmark
    def test_serialize_messagepack_single(self, benchmark, records_1k):
        """Measure single-record MessagePack serialization overhead."""
        record = records_1k[0]

        def serialize():
            return mrrc.record_to_messagepack(record._inner)

        result = benchmark(serialize)
        assert len(result) > 0


class TestSingleRecordDeserializationBenchmarks:
    """Benchmarks for single-record deserialization overhead."""

    @pytest.mark.benchmark
    def test_deserialize_protobuf_single(self, benchmark, records_1k):
        """Measure single-record Protobuf deserialization overhead."""
        record = records_1k[0]
        pb_bytes = mrrc.record_to_protobuf(record._inner)

        def deserialize():
            return mrrc.protobuf_to_record(pb_bytes)

        result = benchmark(deserialize)
        assert result is not None

    @pytest.mark.benchmark
    def test_deserialize_flatbuffers_single(self, benchmark, records_1k):
        """Measure single-record FlatBuffers deserialization overhead."""
        record = records_1k[0]
        fb_bytes = mrrc.record_to_flatbuffers(record._inner)

        def deserialize():
            return mrrc.flatbuffers_to_record(fb_bytes)

        result = benchmark(deserialize)
        assert result is not None

    @pytest.mark.benchmark
    def test_deserialize_messagepack_single(self, benchmark, records_1k):
        """Measure single-record MessagePack deserialization overhead."""
        record = records_1k[0]
        mp_bytes = mrrc.record_to_messagepack(record._inner)

        def deserialize():
            return mrrc.messagepack_to_record(mp_bytes)

        result = benchmark(deserialize)
        assert result is not None


class TestRoundtripBenchmarks:
    """Benchmarks for complete round-trip operations."""

    @pytest.mark.benchmark
    def test_roundtrip_protobuf_1k(self, benchmark, records_1k):
        """Full round-trip: write and read 1k records via Protobuf."""

        def roundtrip():
            with tempfile.NamedTemporaryFile(suffix=".pb", delete=False) as f:
                path = f.name
            try:
                mrrc.write(records_1k, path)
                return list(mrrc.read(path))
            finally:
                os.unlink(path)

        result = benchmark(roundtrip)
        assert len(result) == 1000

    @pytest.mark.benchmark
    def test_roundtrip_flatbuffers_1k(self, benchmark, records_1k):
        """Full round-trip: write and read 1k records via FlatBuffers."""

        def roundtrip():
            with tempfile.NamedTemporaryFile(suffix=".fb", delete=False) as f:
                path = f.name
            try:
                mrrc.write(records_1k, path)
                return list(mrrc.read(path))
            finally:
                os.unlink(path)

        result = benchmark(roundtrip)
        assert len(result) == 1000

    @pytest.mark.benchmark
    def test_roundtrip_messagepack_1k(self, benchmark, records_1k):
        """Full round-trip: write and read 1k records via MessagePack."""

        def roundtrip():
            with tempfile.NamedTemporaryFile(suffix=".msgpack", delete=False) as f:
                path = f.name
            try:
                mrrc.write(records_1k, path)
                return list(mrrc.read(path))
            finally:
                os.unlink(path)

        result = benchmark(roundtrip)
        assert len(result) == 1000

    @pytest.mark.benchmark
    def test_roundtrip_arrow_1k(self, benchmark, records_1k):
        """Full round-trip: write and read 1k records via Arrow."""

        def roundtrip():
            with tempfile.NamedTemporaryFile(suffix=".arrow", delete=False) as f:
                path = f.name
            try:
                mrrc.write(records_1k, path)
                return list(mrrc.read(path))
            finally:
                os.unlink(path)

        result = benchmark(roundtrip)
        assert len(result) == 1000
