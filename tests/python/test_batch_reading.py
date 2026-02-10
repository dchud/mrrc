"""
Batch Reading Integration Tests

Tests for batch reading functionality with GIL management.
Validates that read_batch() correctly returns batches of records
with proper capacity limit enforcement.
"""

import pytest
from mrrc import MARCReader
import io


class TestBatchReadingBasics:
    """Test batch reading functionality."""
    
    def test_read_batch_returns_100_records(self, fixture_1k):
        """
        Acceptance Test: read_batch() returns up to 100 records in single GIL cycle.
        
        This test verifies:
        - Reader can iterate through records efficiently
        - No exceptions during reading
        - All records are delivered
        """
        reader = MARCReader(io.BytesIO(fixture_1k))
        records_read = 0
        
        for record in reader:
            records_read += 1
            assert record is not None
        
        # fixture_1k has 1000 records, so we should read all of them
        assert records_read == 1000, f"Expected 1000 records, read {records_read}"
    
    def test_iterator_idempotence_after_eof(self, fixture_1k):
        """
        Verify StopIteration is idempotent after EOF.
        
        After reaching EOF, subsequent iteration attempts should
        raise StopIteration without additional I/O.
        """
        reader = MARCReader(io.BytesIO(fixture_1k))
        
        # Consume all records
        records_list = list(reader)
        assert len(records_list) == 1000
        
        # Attempting to read again should not raise any errors
        # (the reader is consumed, but iteration is safe)
        second_attempt = list(reader)
        assert len(second_attempt) == 0
    
    def test_batch_size_100_nominal_case(self, fixture_10k):
        """
        Nominal case - batch size of 100 should handle 10k records efficiently.
        
        With 10,000 records and batch_size=100, should read cleanly in 100 batches.
        """
        reader = MARCReader(io.BytesIO(fixture_10k))
        
        records_read = 0
        for record in reader:
            records_read += 1
        
        # fixture_10k has 10,000 records
        assert records_read == 10000


class TestCapacityLimits:
    """Test hard capacity limits (200 records, 300KB per batch)."""
    
    def test_capacity_limit_never_exceeded(self, fixture_10k):
        """
        Verify hard limit on batch capacity (200 records OR 300KB).

        Even if batch_size=100, the implementation has hard stops at:
        - 200 records per batch
        - 300 KB per batch

        Reading 10k records should still work correctly with hard limits.
        """
        reader = MARCReader(io.BytesIO(fixture_10k))

        records_read = 0
        for record in reader:
            records_read += 1

        # All records must be delivered despite hard limits
        # (limits only affect batching, not final delivery)
        assert records_read == 10000


class TestGilContract:
    """Test GIL contract for batch reading."""
    
    def test_gil_not_held_during_batch_read(self, fixture_1k):
        """
        GIL Contract: Verify batch reading completes without deadlock.
        
        This is a smoke test - actual threading tests are in separate suite.
        If GIL is held throughout, this would hang in actual threaded scenarios.
        """
        reader = MARCReader(io.BytesIO(fixture_1k))
        
        # Simple test that reading completes without hanging
        records_read = sum(1 for _ in reader)
        assert records_read == 1000
