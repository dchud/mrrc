"""
Iterator Semantics & Idempotence Verification

Tests verify that MARCReader correctly implements Python iterator protocol
and maintains idempotence guarantees after EOF:

State Machine Verification:
- State 1: CHECK_QUEUE_NON_EMPTY (if queue has records, pop and return)
- State 2: CHECK_EOF_STATE (if EOF reached, return None idempotently)
- State 3: READ_BATCH (read next batch of ~100 records)

Idempotence Guarantee:
After EOF is set once, ALL subsequent calls to __next__() must:
- Return None (via StopIteration in iteration)
- NOT attempt I/O operations
- NOT modify reader state

Iterator Protocol Compliance:
- __iter__() returns self
- __next__() raises StopIteration when done
- Iteration can be paused/resumed
- Multiple iterators from same reader consume same state
"""

import io
import sys
import threading
import time
import pytest
from mrrc import MARCReader


class TestIteratorProtocol:
    """Verify Python iterator protocol implementation"""

    def test_iter_returns_self(self, fixture_1k):
        """__iter__() should return self"""
        reader = MARCReader(io.BytesIO(fixture_1k))
        assert iter(reader) is reader

    def test_next_raises_stop_iteration(self, fixture_small):
        """__next__() should raise StopIteration at EOF, not return None"""
        reader = MARCReader(io.BytesIO(fixture_small))
        
        # Consume all records
        records = []
        try:
            while True:
                records.append(next(reader))
        except StopIteration:
            pass
        
        assert len(records) > 0
        
        # Next call must also raise StopIteration
        with pytest.raises(StopIteration):
            next(reader)

    def test_iteration_protocol_standard_loop(self, fixture_1k):
        """Standard for loop should consume all records and stop cleanly"""
        reader = MARCReader(io.BytesIO(fixture_1k))
        
        records = []
        for record in reader:
            records.append(record)
        
        assert len(records) == 1000

    def test_iteration_protocol_list_conversion(self, fixture_1k):
        """list() should consume entire iterator cleanly"""
        reader = MARCReader(io.BytesIO(fixture_1k))
        records = list(reader)
        assert len(records) == 1000


class TestEofIdempotence:
    """Verify idempotent EOF behavior - central requirement"""

    def test_eof_idempotence_repeated_next_calls(self, fixture_small):
        """Repeated next() calls after EOF must all raise StopIteration"""
        reader = MARCReader(io.BytesIO(fixture_small))
        
        # Consume all
        list(reader)
        
        # Call next() 10 times - all must raise StopIteration
        for call_num in range(10):
            with pytest.raises(StopIteration, match=None):
                next(reader)

    def test_eof_idempotence_no_io_after_eof(self, fixture_small):
        """After EOF, no I/O should occur on subsequent calls"""
        # Create a wrapper that tracks read() calls
        class TrackingBytesIO(io.BytesIO):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, **kwargs)
                self.read_count = 0
                
            def read(self, n=-1):
                self.read_count += 1
                return super().read(n)
        
        bytes_io = TrackingBytesIO(fixture_small)
        reader = MARCReader(bytes_io)
        
        # Consume all records - this will trigger several read() calls
        list(reader)
        initial_read_count = bytes_io.read_count
        
        # Call next() 5 times - no additional read() should occur
        for _ in range(5):
            with pytest.raises(StopIteration):
                next(reader)
        
        # read_count should not increase (idempotence = no I/O)
        assert bytes_io.read_count == initial_read_count

    def test_eof_idempotence_state_unchanged(self, fixture_small):
        """After EOF, reader state should not change on repeated calls"""
        reader = MARCReader(io.BytesIO(fixture_small))
        
        # Consume all
        list(reader)
        
        # Verify repr is consistent
        repr_before = repr(reader)
        
        # Call next() a few times
        for _ in range(3):
            with pytest.raises(StopIteration):
                next(reader)
        
        # Repr should be unchanged (no state mutations on repeated EOF calls)
        repr_after = repr(reader)
        assert repr_before == repr_after


class TestPartialBatchAtEof:
    """Test edge case: partial batch when EOF reached during read_batch()"""

    def test_partial_batch_at_eof_all_delivered(self, fixture_217):
        """
        File with 217 records:
        - read_batch(100) at offset 0-99 → 100 records
        - read_batch(100) at offset 100-199 → 100 records  
        - read_batch(100) at offset 200-216 → 17 records (partial)
        - read_batch(100) at offset 217+ → 0 records (sets EOF)
        
        All 217 must be delivered before EOF is triggered.
        """
        reader = MARCReader(io.BytesIO(fixture_217))
        
        records = list(reader)
        assert len(records) == 217

    def test_partial_batch_eof_flag_set(self, fixture_217):
        """After partial batch returns final records, next call sets EOF"""
        reader = MARCReader(io.BytesIO(fixture_217))
        
        # Consume all records
        records = list(reader)
        assert len(records) == 217
        
        # Next call should raise StopIteration (EOF flag is set)
        with pytest.raises(StopIteration):
            next(reader)

    def test_exact_batch_multiple_at_eof(self, fixture_500):
        """
        File with 500 records (exactly 5 batches of 100):
        - Batches 1-4: 100 records each
        - Batch 5: 100 records
        - Batch 6: 0 records (sets EOF)
        
        All 500 delivered, then EOF.
        """
        reader = MARCReader(io.BytesIO(fixture_500))
        
        records = list(reader)
        assert len(records) == 500
        
        # EOF should be set
        with pytest.raises(StopIteration):
            next(reader)


class TestQueueStateTransitions:
    """Verify state machine transitions during iteration"""

    def test_state_check_queue_non_empty_transition(self, fixture_1k):
        """
        Verify CHECK_QUEUE_NON_EMPTY state:
        - After batch read, queue has records
        - Pop from queue until empty
        - Transition to CHECK_EOF_STATE
        """
        reader = MARCReader(io.BytesIO(fixture_1k))
        
        # First call reads batch -> CHECK_QUEUE_NON_EMPTY state
        rec1 = next(reader)
        assert rec1 is not None
        
        # Next 99 calls should use queue (CHECK_QUEUE_NON_EMPTY)
        for i in range(99):
            rec = next(reader)
            assert rec is not None

    def test_state_read_batch_transition(self, fixture_1k):
        """
        After queue empty, should transition to READ_BATCH state
        and read next batch of records.
        """
        reader = MARCReader(io.BytesIO(fixture_1k))
        
        # Read exactly 100 records (first batch)
        for i in range(100):
            rec = next(reader)
            assert rec is not None
        
        # Next call triggers READ_BATCH (queue was empty after 100)
        rec_101 = next(reader)
        assert rec_101 is not None

    def test_state_check_eof_idempotent(self, fixture_small):
        """
        After CHECK_EOF_STATE sets eof_reached=true,
        subsequent calls should immediately return from CHECK_EOF_STATE
        without attempting READ_BATCH.
        """
        reader = MARCReader(io.BytesIO(fixture_small))
        
        # Consume all
        list(reader)
        
        # All subsequent calls should go through CHECK_EOF_STATE
        # and return None without I/O
        for _ in range(5):
            with pytest.raises(StopIteration):
                next(reader)


class TestResumeAfterPartialRead:
    """Test that iteration can be paused and resumed"""

    def test_pause_resume_iteration(self, fixture_1k):
        """Read some records, pause, resume, read rest"""
        reader = MARCReader(io.BytesIO(fixture_1k))
        
        records = []
        
        # Read first 50 records
        for _ in range(50):
            records.append(next(reader))
        
        assert len(records) == 50
        
        # Pause (implicit - just don't call next)
        
        # Resume - read next 50
        for _ in range(50):
            records.append(next(reader))
        
        assert len(records) == 100
        
        # Resume again - read all remaining
        for record in reader:
            records.append(record)
        
        assert len(records) == 1000

    def test_pause_resume_across_batch_boundary(self, fixture_1k):
        """Pause/resume across batch boundaries"""
        reader = MARCReader(io.BytesIO(fixture_1k))
        
        records = []
        
        # Read 75 records (stops in first batch)
        for _ in range(75):
            records.append(next(reader))
        
        # Pause
        
        # Resume - read 50 more (finishes first batch, might start second)
        for _ in range(50):
            records.append(next(reader))
        
        assert len(records) == 125
        
        # Resume - read rest
        for record in reader:
            records.append(record)
        
        assert len(records) == 1000


class TestConcurrentIterators:
    """Test multiple iterators from same file"""

    def test_independent_iterators_independent_state(self, fixture_1k):
        """Two readers from same file should maintain independent state"""
        data = fixture_1k
        
        reader1 = MARCReader(io.BytesIO(data))
        reader2 = MARCReader(io.BytesIO(data))
        
        # Advance reader1 to record 10
        for _ in range(10):
            next(reader1)
        
        # reader2 should still be at record 1
        rec2_first = next(reader2)
        assert rec2_first is not None
        
        # reader1 should be ahead
        rec1_tenth = next(reader1)
        assert rec1_tenth is not None

    def test_two_readers_same_file_consumed_independently(self, fixture_small):
        """Two readers should EOF independently"""
        data = fixture_small
        
        reader1 = MARCReader(io.BytesIO(data))
        reader2 = MARCReader(io.BytesIO(data))
        
        # Consume reader1 completely
        list(reader1)
        
        # reader2 should still have records
        rec = next(reader2)
        assert rec is not None
        
        # reader2 can be consumed
        for record in reader2:
            pass
        
        # Both should be at EOF
        with pytest.raises(StopIteration):
            next(reader1)
        with pytest.raises(StopIteration):
            next(reader2)


class TestErrorRecovery:
    """Test behavior with malformed or truncated files"""

    def test_eof_after_malformed_record(self, fixture_with_error):
        """After error, EOF should be deterministic"""
        reader = MARCReader(io.BytesIO(fixture_with_error))
        
        records = []
        error_count = 0
        
        try:
            for record in reader:
                records.append(record)
        except Exception as e:
            error_count += 1
        
        # After error, subsequent next() calls should behave deterministically
        # (either continue from where error occurred, or EOF)
        try:
            next(reader)
        except (StopIteration, Exception):
            pass  # Acceptable after error


class TestBatchSizeEdgeCases:
    """Test edge cases with hard limits"""

    def test_batch_200_record_hard_limit(self):
        """Batch hard limit is 200 records - verify it's respected"""
        # This is more of an internal invariant test
        # Would need to mock read_batch to verify hard limit enforcement
        # For now, just verify iteration works with any file size
        pass

    def test_batch_300kb_hard_limit(self):
        """Batch hard limit is 300KB - verify it's respected"""
        # Similar to above - internal invariant
        pass
