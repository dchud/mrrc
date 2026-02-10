"""
Test suite for Producer-Consumer Pipeline with Backpressure

Tests the ProducerConsumerPipeline class that implements high-performance
batch reading from Rust file I/O backend with backpressure management.
"""

import pytest
import tempfile
import os
from pathlib import Path
import mrrc
from mrrc import (
    ProducerConsumerPipeline,
    RecordBoundaryScanner,
    parse_batch_parallel,
)


# Fixtures

@pytest.fixture
def simple_book_mrc():
    """Path to simple_book.mrc test file."""
    return Path("tests/data/simple_book.mrc")


@pytest.fixture
def multi_records_mrc():
    """Path to multi_records.mrc test file."""
    return Path("tests/data/multi_records.mrc")


@pytest.fixture
def large_5k_mrc():
    """Path to 5k records MARC file."""
    return Path("tests/data/fixtures/5k_records.mrc")


@pytest.fixture
def large_10k_mrc():
    """Path to 10k records MARC file."""
    return Path("tests/data/fixtures/10k_records.mrc")


class TestProducerConsumerPipelineBasics:
    """Test basic pipeline creation and file I/O."""

    def test_pipeline_creation_from_file(self, simple_book_mrc):
        """Test creating a pipeline from a valid file path."""
        pipeline = ProducerConsumerPipeline.from_file(str(simple_book_mrc))
        assert pipeline is not None
        assert repr(pipeline) == "ProducerConsumerPipeline(active)"

    def test_pipeline_file_not_found(self):
        """Test that FileNotFoundError is raised for missing file."""
        with pytest.raises((FileNotFoundError, RuntimeError)):
            ProducerConsumerPipeline.from_file("/nonexistent/path/file.mrc")

    def test_pipeline_from_pathlib_path(self, simple_book_mrc):
        """Test creating pipeline from pathlib.Path object."""
        path = Path(str(simple_book_mrc))
        pipeline = ProducerConsumerPipeline.from_file(str(path))
        assert pipeline is not None

    def test_pipeline_with_custom_config(self, simple_book_mrc):
        """Test pipeline with custom buffer and channel sizes."""
        pipeline = ProducerConsumerPipeline.from_file(
            str(simple_book_mrc),
            buffer_size=256 * 1024,  # 256 KB instead of 512 KB
            channel_capacity=500,     # 500 records instead of 1000
        )
        assert pipeline is not None


class TestProducerConsumerPipelineIteration:
    """Test record iteration via producer-consumer pipeline."""

    def test_blocking_next_simple_book(self, simple_book_mrc):
        """Test blocking next() iteration on simple_book.mrc."""
        pipeline = ProducerConsumerPipeline.from_file(str(simple_book_mrc))

        record_count = 0
        while True:
            record = pipeline.next()
            if record is None:
                break
            record_count += 1
            assert record is not None

        # simple_book.mrc contains 1 record
        assert record_count == 1

    def test_blocking_next_multi_records(self, multi_records_mrc):
        """Test blocking next() on multi_records.mrc."""
        pipeline = ProducerConsumerPipeline.from_file(str(multi_records_mrc))

        record_count = 0
        while True:
            record = pipeline.next()
            if record is None:
                break
            record_count += 1

        # multi_records.mrc contains multiple records
        assert record_count >= 2

    def test_try_next_basic(self, multi_records_mrc):
        """Test non-blocking try_next() method."""
        pipeline = ProducerConsumerPipeline.from_file(str(multi_records_mrc))

        # Producer should have buffered some records quickly
        # (but we can't guarantee due to timing)
        record = pipeline.try_next()
        if record is not None:
            # Successfully got a record without blocking
            assert isinstance(record, mrrc.Record)

    def test_pythonic_for_loop_iteration(self, multi_records_mrc):
        """Test iteration using Python for loop."""
        pipeline = ProducerConsumerPipeline.from_file(str(multi_records_mrc))

        record_count = 0
        for record in pipeline:
            record_count += 1
            # Record should have to_marcjson method
            assert hasattr(record, 'to_marcjson')

        assert record_count >= 1

    def test_eof_returns_none_idempotent(self, simple_book_mrc):
        """Test that subsequent calls after EOF return None."""
        pipeline = ProducerConsumerPipeline.from_file(str(simple_book_mrc))

        # Drain all records
        while True:
            record = pipeline.next()
            if record is None:
                break

        # Subsequent calls should also return None (idempotent)
        assert pipeline.next() is None
        assert pipeline.next() is None

    def test_record_content_accuracy(self, multi_records_mrc):
        """Test that records from pipeline have correct content."""
        pipeline = ProducerConsumerPipeline.from_file(str(multi_records_mrc))

        record = pipeline.next()
        assert record is not None

        # Verify it's a valid record
        assert hasattr(record, 'to_marcjson')
        marcjson = record.to_marcjson()
        assert marcjson is not None
        assert len(marcjson) > 0


class TestProducerConsumerPipelineBackpressure:
    """Test backpressure mechanism preventing OOM."""

    def test_backpressure_with_large_file(self, large_5k_mrc):
        """Test that pipeline handles backpressure with large files.

        With default channel capacity of 1000, the producer should block
        if the consumer is not draining fast enough.
        """
        pipeline = ProducerConsumerPipeline.from_file(str(large_5k_mrc))

        # Slowly consume records (simulating slow consumer)
        record_count = 0
        while True:
            record = pipeline.next()
            if record is None:
                break
            record_count += 1

        # Should have processed all records
        # (5k_records.mrc contains 4957 records)
        assert record_count == 4957

    def test_channel_capacity_respected(self, large_5k_mrc):
        """Test that channel capacity is respected during iteration."""
        # With a small channel (100 records), producer should block frequently
        pipeline = ProducerConsumerPipeline.from_file(
            str(large_5k_mrc),
            channel_capacity=100,  # Very small
        )

        record_count = 0
        while True:
            record = pipeline.next()
            if record is None:
                break
            record_count += 1

        # Should still process all records despite small channel
        assert record_count == 4957


class TestProducerConsumerPipelineMemory:
    """Test memory stability and no unbounded growth."""

    def test_memory_stability_large_file(self, large_10k_mrc):
        """Test that memory doesn't grow unboundedly with large files."""
        import psutil
        import os

        pipeline = ProducerConsumerPipeline.from_file(str(large_10k_mrc))

        process = psutil.Process(os.getpid())
        initial_memory = process.memory_info().rss

        record_count = 0
        peak_memory = initial_memory
        while True:
            record = pipeline.next()
            if record is None:
                break
            record_count += 1

            # Check memory periodically
            if record_count % 100 == 0:
                current_memory = process.memory_info().rss
                peak_memory = max(peak_memory, current_memory)

        # Memory growth should be bounded (< 200 MB for 10k records)
        memory_growth = peak_memory - initial_memory
        assert memory_growth < 200 * 1024 * 1024  # 200 MB limit
        assert record_count == 10000  # Should process all records


class TestProducerConsumerPipelineErrorHandling:
    """Test error handling in pipeline."""

    def test_malformed_record_handling(self):
        """Test that pipeline handles malformed records gracefully.

        Creates a file with a truncated record (missing record terminator).
        """
        # Create a temporary file with truncated MARC data
        with tempfile.NamedTemporaryFile(delete=False, suffix=".mrc") as f:
            # Write an incomplete record (just raw bytes without proper structure)
            f.write(b"This is not valid MARC data" * 10)
            temp_path = f.name

        try:
            pipeline = ProducerConsumerPipeline.from_file(temp_path)
            # Attempting to read should not crash
            record = pipeline.next()
            # May be None or a parse error is acceptable
        finally:
            os.unlink(temp_path)

    def test_empty_file_handling(self):
        """Test that pipeline handles empty files gracefully."""
        with tempfile.NamedTemporaryFile(delete=False, suffix=".mrc") as f:
            temp_path = f.name
            # File is empty

        try:
            pipeline = ProducerConsumerPipeline.from_file(temp_path)
            record = pipeline.next()
            # Empty file should return None immediately
            assert record is None
        finally:
            os.unlink(temp_path)


class TestProducerConsumerPipelineConsistency:
    """Test that pipeline produces consistent results."""

    def test_consistency_multiple_iterations(self, simple_book_mrc):
        """Test that multiple pipelines produce identical results."""
        # First pipeline
        pipeline1 = ProducerConsumerPipeline.from_file(str(simple_book_mrc))
        records1 = []
        while True:
            record = pipeline1.next()
            if record is None:
                break
            records1.append(record.to_marcjson())

        # Second pipeline
        pipeline2 = ProducerConsumerPipeline.from_file(str(simple_book_mrc))
        records2 = []
        while True:
            record = pipeline2.next()
            if record is None:
                break
            records2.append(record.to_marcjson())

        # Results should be identical
        assert len(records1) == len(records2)
        for r1, r2 in zip(records1, records2):
            assert r1 == r2

    def test_consistency_with_standard_reader(self, simple_book_mrc):
        """Test that pipeline output matches standard reader output."""
        # Standard reader
        with open(str(simple_book_mrc), "rb") as f:
            standard_reader = mrrc.MARCReader(f)
            standard_records = []
            for record in standard_reader:
                standard_records.append(record.to_marcjson())

        # Pipeline reader
        pipeline = ProducerConsumerPipeline.from_file(str(simple_book_mrc))
        pipeline_records = []
        while True:
            record = pipeline.next()
            if record is None:
                break
            pipeline_records.append(record.to_marcjson())

        # Results should match
        assert len(standard_records) == len(pipeline_records)
        for sr, pr in zip(standard_records, pipeline_records):
            assert sr == pr


class TestProducerConsumerPipelineAcceptanceCriteria:
    """Test acceptance criteria."""

    def test_gate_backpressure_works_as_designed(self, large_5k_mrc):
        """Acceptance Criterion 1: Backpressure works correctly.

        Producer should block when channel is full, preventing OOM.
        """
        pipeline = ProducerConsumerPipeline.from_file(
            str(large_5k_mrc),
            channel_capacity=100,  # Small channel to force blocking
        )

        record_count = 0
        while True:
            record = pipeline.next()
            if record is None:
                break
            record_count += 1

        # All records should be processed despite small channel
        assert record_count == 4957
        print(f"✓ Processed {record_count} records with 100-record channel")

    def test_gate_no_deadlocks(self, multi_records_mrc):
        """Acceptance Criterion 2: Pipeline doesn't deadlock.

        Both producer and consumer should make progress without deadlock.
        """
        import signal

        def timeout_handler(signum, frame):
            raise TimeoutError("Pipeline operation timed out (deadlock suspected)")

        # Set a 10-second timeout
        signal.signal(signal.SIGALRM, timeout_handler)
        signal.alarm(10)

        try:
            pipeline = ProducerConsumerPipeline.from_file(str(multi_records_mrc))

            record_count = 0
            while True:
                record = pipeline.next()
                if record is None:
                    break
                record_count += 1

            assert record_count >= 2
            print("✓ Pipeline completed without deadlock")
        finally:
            signal.alarm(0)  # Cancel timeout

    def test_gate_oom_prevented(self, large_10k_mrc):
        """Acceptance Criterion 3: Out-of-memory is prevented.

        With backpressure, pipeline should not consume unbounded memory
        even with 10k records.
        """
        import psutil
        import os

        pipeline = ProducerConsumerPipeline.from_file(
            str(large_10k_mrc),
            channel_capacity=500,
        )

        process = psutil.Process(os.getpid())
        initial_memory = process.memory_info().rss

        record_count = 0
        max_memory_increase = 0
        while True:
            record = pipeline.next()
            if record is None:
                break
            record_count += 1

            if record_count % 500 == 0:
                current_memory = process.memory_info().rss
                memory_increase = current_memory - initial_memory
                max_memory_increase = max(max_memory_increase, memory_increase)

        # Memory increase should be modest (< 300 MB for 10k records)
        assert max_memory_increase < 300 * 1024 * 1024
        assert record_count == 10000

        print(f"✓ OOM prevented: {max_memory_increase / 1024 / 1024:.1f} MB increase for {record_count} records")

    def test_regression_records_spanning_chunk_boundaries(self, large_10k_mrc):
        """Regression test for mrrc-0p0: Records spanning chunk boundaries.

        Previously, ProducerConsumerPipeline would stop at ~1985 records when
        processing a 10k record file because it didn't handle partial records
        that spanned file I/O chunk boundaries (512 KB default).

        The producer task now maintains a 'leftover' buffer to carry incomplete
        records from one chunk to the next, ensuring all records are processed.
        """
        pipeline = ProducerConsumerPipeline.from_file(str(large_10k_mrc))
        record_count = sum(1 for _ in pipeline)

        # Before the fix, this would be ~1985. After the fix, it should be 10000.
        assert record_count == 10000, (
            f"Expected 10000 records but got {record_count}. "
            "Check if records spanning chunk boundaries are being lost."
        )
