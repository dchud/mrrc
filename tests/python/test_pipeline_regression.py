"""
Pipeline regression tests.

Extracted from src-python/tests/test_producer_consumer_pipeline.py.
"""

import pytest
from mrrc import ProducerConsumerPipeline


@pytest.fixture
def large_10k_mrc(fixture_10k, tmp_path):
    """Write 10k fixture bytes to a temp file for pipeline testing."""
    path = tmp_path / "10k_records.mrc"
    path.write_bytes(fixture_10k)
    return path


def test_regression_records_spanning_chunk_boundaries(large_10k_mrc):
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
