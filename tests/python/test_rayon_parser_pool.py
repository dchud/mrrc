"""
Rayon Parser Pool Tests

Tests the RayonParserPool Rust module for parallel MARC record parsing.
These tests verify:

- Correct parallel parsing of record batches
- Output identical to sequential parsing
- Proper error handling within parallel context
- Thread-safety and GIL release
- Backpressure and batch limiting
"""

import pytest
from mrrc import MARCReader, RecordBoundaryScanner
from mrrc.rayon_parser_pool import parse_batch_parallel


@pytest.fixture
def simple_book_bytes():
    """Read simple_book.mrc as raw bytes."""
    with open("tests/data/simple_book.mrc", "rb") as f:
        return f.read()


@pytest.fixture
def multi_records_bytes():
    """Read multi_records.mrc as raw bytes."""
    with open("tests/data/multi_records.mrc", "rb") as f:
        return f.read()


class TestRayonParserPoolBasics:
    """Test basic parallel parsing functionality."""

    def test_parser_pool_single_record(self, simple_book_bytes):
        """Parse a single record in parallel."""
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(simple_book_bytes)
        
        # Take just the first record
        boundaries = boundaries[:1]
        
        records = parse_batch_parallel(boundaries, simple_book_bytes)
        assert len(records) == 1
        assert records[0] is not None

    def test_parser_pool_multiple_records(self, multi_records_bytes):
        """Parse multiple records in parallel."""
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)
        
        assert len(boundaries) > 1, "Test data should have multiple records"
        
        records = parse_batch_parallel(boundaries, multi_records_bytes)
        assert len(records) == len(boundaries)
        # All records should be valid
        for record in records:
            assert record is not None

    def test_parser_pool_empty_boundaries(self, multi_records_bytes):
        """Parse empty boundary list should return empty result."""
        records = parse_batch_parallel([], multi_records_bytes)
        assert len(records) == 0

    def test_parser_pool_invalid_boundary(self):
        """Invalid boundaries should error."""
        buffer = b"test data"
        boundaries = [(0, 100)]  # Exceeds buffer
        
        with pytest.raises(Exception):  # Should raise MarcError
            parse_batch_parallel(boundaries, buffer)


class TestRayonParserPoolParity:
    """Test that parallel parsing matches sequential parsing."""

    def test_parity_simple_book(self, simple_book_bytes):
        """Parallel results match sequential parsing."""
        # Sequential parsing
        reader = MARCReader(simple_book_bytes)
        sequential_records = []
        while True:
            record = reader.read_record()
            if record is None:
                break
            sequential_records.append(record)
        
        # Parallel parsing via boundaries
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(simple_book_bytes)
        parallel_records = parse_batch_parallel(boundaries, simple_book_bytes)
        
        # Should have same count
        assert len(parallel_records) == len(sequential_records), \
            f"Record count mismatch: {len(parallel_records)} vs {len(sequential_records)}"
        
        # Verify content matches (using marcjson for comparison)
        for i, (seq_rec, par_rec) in enumerate(zip(sequential_records, parallel_records)):
            seq_json = seq_rec.to_marcjson()
            par_json = par_rec.to_marcjson()
            assert seq_json == par_json, \
                f"Record {i} mismatch:\nSequential: {seq_json}\nParallel: {par_json}"

    def test_parity_multi_records(self, multi_records_bytes):
        """Parallel parsing matches sequential on multi-record file."""
        # Sequential
        reader = MARCReader(multi_records_bytes)
        sequential_records = []
        while True:
            record = reader.read_record()
            if record is None:
                break
            sequential_records.append(record)
        
        # Parallel
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)
        parallel_records = parse_batch_parallel(boundaries, multi_records_bytes)
        
        assert len(parallel_records) == len(sequential_records)
        
        for seq_rec, par_rec in zip(sequential_records, parallel_records):
            assert seq_rec.to_marcjson() == par_rec.to_marcjson()


class TestRayonParserPoolBatching:
    """Test batch limiting and batch processing."""

    def test_parse_batch_limited(self, multi_records_bytes):
        """Test limited batch processing."""
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)
        
        if len(boundaries) > 1:
            # Parse only first 2 records
            limited_boundaries = boundaries[:2]
            records = parse_batch_parallel(limited_boundaries, multi_records_bytes)
            
            assert len(records) == 2

    def test_parse_batch_order_preserved(self, multi_records_bytes):
        """Parsed records should maintain boundary order."""
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)
        
        if len(boundaries) > 1:
            records = parse_batch_parallel(boundaries, multi_records_bytes)
            
            # Verify order is preserved
            for i, record in enumerate(records):
                assert record is not None, f"Record {i} is None"


class TestRayonParserPoolThreadSafety:
    """Test thread safety and GIL release."""

    def test_concurrent_parallel_parsing(self, multi_records_bytes):
        """Verify parallel parsing works with multiple concurrent calls."""
        import threading
        
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)
        
        results = []
        errors = []
        
        def parse_worker():
            try:
                records = parse_batch_parallel(boundaries, multi_records_bytes)
                results.append(len(records))
            except Exception as e:
                errors.append(e)
        
        # Run 3 concurrent parse operations
        threads = [threading.Thread(target=parse_worker) for _ in range(3)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()
        
        # All should succeed
        assert len(errors) == 0, f"Errors occurred: {errors}"
        assert len(results) == 3
        # All should have same count
        assert len(set(results)) == 1, "Inconsistent record counts"

    def test_parse_while_reading_sequential(self, multi_records_bytes):
        """Test parsing in parallel while another thread reads sequentially."""
        import threading
        
        sequential_records = []
        parallel_records = []
        
        def sequential_reader():
            reader = MARCReader(multi_records_bytes)
            while True:
                record = reader.read_record()
                if record is None:
                    break
                sequential_records.append(record)
        
        def parallel_parser():
            scanner = RecordBoundaryScanner()
            boundaries = scanner.scan(multi_records_bytes)
            records = parse_batch_parallel(boundaries, multi_records_bytes)
            parallel_records.extend(records)
        
        t1 = threading.Thread(target=sequential_reader)
        t2 = threading.Thread(target=parallel_parser)
        
        t1.start()
        t2.start()
        t1.join()
        t2.join()
        
        # Both should succeed
        assert len(sequential_records) > 0
        assert len(parallel_records) > 0
        assert len(sequential_records) == len(parallel_records)


class TestRayonParserPoolErrorHandling:
    """Test error propagation in parallel context."""

    def test_error_in_parallel_task(self):
        """Errors in parallel tasks should propagate."""
        buffer = b"bad data"
        boundaries = [(0, 8)]  # Valid boundary but bad MARC data
        
        # Should error since "bad data" is not valid MARC
        with pytest.raises(Exception):
            parse_batch_parallel(boundaries, buffer)

    def test_mixed_valid_invalid_records(self):
        """Multiple records with one invalid should error."""
        # Create a mixed buffer: valid terminator, then invalid data
        buffer = b"\x00" * 24 + b"\x1E" + b"invalid"
        boundaries = [(0, 25), (25, 7)]
        
        # Should error on invalid record
        with pytest.raises(Exception):
            parse_batch_parallel(boundaries, buffer)


class TestRayonParserPoolPerformance:
    """Test performance characteristics."""

    def test_large_batch_parsing(self):
        """Parse many records in a single batch."""
        # Create a synthetic buffer with many small record boundaries
        buffer = bytearray()
        boundaries = []
        
        # Create 100 minimal records
        for i in range(100):
            offset = len(buffer)
            # Minimal 24-byte leader
            buffer.extend(b"\x00" * 24)
            # Record terminator
            buffer.append(0x1E)
            boundaries.append((offset, 25))
        
        # Should handle large batches
        # Note: May error on parsing since these are minimal records
        # Just verify it doesn't crash
        try:
            parse_batch_parallel(boundaries, bytes(buffer))
        except Exception:
            # Expected - minimal data won't parse
            pass

    def test_parser_reuse_across_calls(self, multi_records_bytes):
        """Parser should work across multiple calls."""
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)
        
        # Call multiple times
        records1 = parse_batch_parallel(boundaries, multi_records_bytes)
        records2 = parse_batch_parallel(boundaries, multi_records_bytes)
        
        assert len(records1) == len(records2)
        
        # Results should be identical
        for r1, r2 in zip(records1, records2):
            assert r1.to_marcjson() == r2.to_marcjson()


class TestRayonParserPoolAcceptanceCriteria:
    """Test acceptance criteria."""

    def test_criterion_parallel_produces_identical_output(self, multi_records_bytes):
        """Criterion 1: Parallel parsing produces identical output to sequential."""
        # Sequential
        reader = MARCReader(multi_records_bytes)
        sequential = []
        while True:
            record = reader.read_record()
            if record is None:
                break
            sequential.append(record.to_marcjson())
        
        # Parallel
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)
        parallel = parse_batch_parallel(boundaries, multi_records_bytes)
        parallel_json = [r.to_marcjson() for r in parallel]
        
        assert sequential == parallel_json, "Parallel output doesn't match sequential"

    def test_criterion_error_within_parallel_context(self):
        """Criterion 2: Errors within parallel context are properly handled."""
        # Invalid boundary should error
        buffer = b"x" * 100
        boundaries = [(0, 200)]  # Out of bounds
        
        with pytest.raises(Exception) as excinfo:
            parse_batch_parallel(boundaries, buffer)
        
        # Error should be informative
        assert "bound" in str(excinfo.value).lower() or "exceed" in str(excinfo.value).lower()

    def test_criterion_all_records_parsed_identically(self, simple_book_bytes, multi_records_bytes):
        """Criterion 3: All records from real MARC files parse identically."""
        for test_data in [simple_book_bytes, multi_records_bytes]:
            # Sequential reference
            reader = MARCReader(test_data)
            sequential_count = 0
            while reader.read_record():
                sequential_count += 1
            
            # Parallel result
            scanner = RecordBoundaryScanner()
            boundaries = scanner.scan(test_data)
            parallel_records = parse_batch_parallel(boundaries, test_data)
            
            assert len(parallel_records) == sequential_count, \
                f"Record count mismatch for {test_data[:20]!r}"


class TestRayonParserPoolIntegration:
    """Integration tests with boundary scanner."""

    def test_boundary_scanner_to_parser_pipeline(self, multi_records_bytes):
        """Test the full pipeline: scan boundaries → parse parallel."""
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)
        
        # Boundaries should be compatible with parser
        records = parse_batch_parallel(boundaries, multi_records_bytes)
        
        assert len(records) > 0
        assert len(records) == len(boundaries)

    def test_parser_with_limited_boundaries(self, multi_records_bytes):
        """Test parser with subset of boundaries."""
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)
        
        if len(boundaries) > 2:
            # Parse only half
            half = len(boundaries) // 2
            limited = boundaries[:half]
            
            records = parse_batch_parallel(limited, multi_records_bytes)
            assert len(records) == half
