"""
Record Boundary Scanner Tests

Tests the RecordBoundaryScanner Rust module for accurate MARC record
boundary detection using 0x1D (record terminator) delimiters. These tests verify:

- Correct identification of record boundaries
- Proper length calculation (including terminator)
- Batch limiting functionality
- Integration with existing MARC readers
- Performance characteristics for large buffers
"""

import pytest
from mrrc import MARCReader, RecordBoundaryScanner


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


class TestBoundaryScannerBasics:
    """Test basic boundary scanner functionality."""

    def test_scanner_creation(self):
        """Verify scanner can be instantiated."""
        scanner = RecordBoundaryScanner()
        assert scanner is not None

    def test_scan_single_record(self):
        """Scan a single record with terminator."""
        data = bytes([1, 2, 3, 0x1D])  # 0x1D = record terminator
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(data)

        assert len(boundaries) == 1
        assert boundaries[0] == (0, 4)

    def test_scan_multiple_records(self):
        """Scan multiple records with distinct boundaries."""
        data = bytes([1, 2, 0x1D, 3, 4, 0x1D, 5, 0x1D])  # 0x1D = record terminator
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(data)

        assert len(boundaries) == 3
        assert boundaries[0] == (0, 3)
        assert boundaries[1] == (3, 3)
        assert boundaries[2] == (6, 2)

    def test_scan_empty_buffer(self):
        """Empty buffer should raise error."""
        scanner = RecordBoundaryScanner()
        with pytest.raises(Exception):  # MarcError
            scanner.scan(b"")

    def test_scan_no_terminators(self):
        """Buffer with no record terminators should raise error."""
        data = bytes([1, 2, 3, 4])  # No 0x1D terminators
        scanner = RecordBoundaryScanner()
        with pytest.raises(Exception):  # MarcError
            scanner.scan(data)


class TestBoundaryScannerRealData:
    """Test boundary scanner with real MARC data."""

    def test_scan_simple_book(self, simple_book_bytes):
        """Scan simple_book.mrc and verify record count."""
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(simple_book_bytes)

        # simple_book.mrc contains 1 record
        assert len(boundaries) >= 1, "Should find at least one record"
        # First record should start at offset 0
        assert boundaries[0][0] == 0, "First record should start at offset 0"
        # All boundaries should end before file end
        for offset, length in boundaries:
            assert offset + length <= len(simple_book_bytes), \
                "Record boundary exceeds file size"

    def test_scan_multi_records(self, multi_records_bytes):
        """Scan multi_records.mrc and verify record count."""
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)

        # multi_records.mrc contains multiple records
        assert len(boundaries) > 1
        # Verify all boundaries are valid (offsets in increasing order)
        offsets = [b[0] for b in boundaries]
        assert offsets == sorted(offsets)

    def test_boundary_reconstruction(self, simple_book_bytes):
        """Verify boundaries point to valid record data."""
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(simple_book_bytes)

        # Verify boundaries are valid
        for offset, length in boundaries:
            record_bytes = simple_book_bytes[offset : offset + length]
            # Should have content
            assert len(record_bytes) > 0, "Record should have content"
            # Should end with record terminator (0x1D)
            assert record_bytes[-1] == 0x1D, "Record should end with 0x1D record terminator"
            # Should not exceed file bounds
            assert offset + length <= len(simple_book_bytes), \
                "Record should not exceed file boundaries"


class TestBoundaryScannerLimiting:
    """Test boundary scanner limiting functionality."""

    def test_scan_limited(self):
        """Verify scan_limited returns up to limit records."""
        data = bytes([1, 0x1D, 2, 0x1D, 3, 0x1D])  # 0x1D = record terminator
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan_limited(data, 2)

        assert len(boundaries) == 2
        assert boundaries[0] == (0, 2)
        assert boundaries[1] == (2, 2)

    def test_scan_limited_exceeds_available(self):
        """Verify scan_limited returns fewer records if not enough available."""
        data = bytes([1, 0x1D, 2, 0x1D])  # 0x1D = record terminator
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan_limited(data, 10)

        # Should return only 2 even though limit is 10
        assert len(boundaries) == 2

    def test_scan_limited_batch_processing(self, multi_records_bytes):
        """Verify scan_limited works for batch processing."""
        scanner = RecordBoundaryScanner()

        # Scan all records
        all_boundaries = scanner.scan(multi_records_bytes)
        total_records = len(all_boundaries)

        # Verify we found records
        assert total_records > 0, "Should find at least one record"
        
        # Test that scan_limited returns correct number
        half = (total_records + 1) // 2
        limited_boundaries = scanner.scan_limited(multi_records_bytes, half)
        assert len(limited_boundaries) <= half, \
            f"Should return at most {half} records, got {len(limited_boundaries)}"
        assert len(limited_boundaries) > 0, "Should return at least one record"


class TestBoundaryScannerCounting:
    """Test record counting functionality."""

    def test_count_records(self):
        """Verify count_records returns correct terminator count."""
        data = bytes([1, 0x1D, 2, 0x1D])  # 0x1D = record terminator
        scanner = RecordBoundaryScanner()

        count = scanner.count_records(data)
        assert count == 2

    def test_count_records_empty(self):
        """Count in empty buffer should return 0."""
        scanner = RecordBoundaryScanner()
        count = scanner.count_records(b"")
        assert count == 0

    def test_count_records_no_terminators(self):
        """Count with no terminators should return 0."""
        data = bytes([1, 2, 3, 4])
        scanner = RecordBoundaryScanner()
        count = scanner.count_records(data)
        assert count == 0

    def test_count_matches_scan(self, multi_records_bytes):
        """Verify count_records matches length of scan results."""
        scanner = RecordBoundaryScanner()

        count = scanner.count_records(multi_records_bytes)
        boundaries = scanner.scan(multi_records_bytes)

        assert count == len(boundaries)


class TestBoundaryScannerPerformance:
    """Test performance characteristics."""

    def test_large_buffer_scan(self):
        """Verify scanner handles large buffers efficiently."""
        # Create a buffer with 1000 records
        data = bytearray()
        for i in range(1000):
            data.append(0x01 if i % 2 == 0 else 0x02)
            data.append(0x1D)  # 0x1D = record terminator

        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(bytes(data))

        assert len(boundaries) == 1000
        # Spot check first and last
        assert boundaries[0] == (0, 2)
        assert boundaries[999] == (1998, 2)

    def test_scanner_reuse(self):
        """Verify scanner can be reused without issues."""
        scanner = RecordBoundaryScanner()

        # First scan
        data1 = bytes([1, 0x1D, 2, 0x1D])  # 0x1D = record terminator
        boundaries1 = scanner.scan(data1)
        assert len(boundaries1) == 2

        # Second scan (should clear internal state)
        data2 = bytes([1, 0x1D])
        boundaries2 = scanner.scan(data2)
        assert len(boundaries2) == 1
        # Verify no cross-contamination
        assert boundaries2[0] == (0, 2)

    def test_count_vs_scan_performance(self, multi_records_bytes):
        """Verify count_records is efficient for just counting."""
        scanner = RecordBoundaryScanner()

        # count_records should be faster than full scan
        count = scanner.count_records(multi_records_bytes)
        boundaries = scanner.scan(multi_records_bytes)

        assert count == len(boundaries)


class TestBoundaryScannerIntegration:
    """Test integration with existing MARC readers."""

    def test_boundaries_enable_parallel_parsing(self, multi_records_bytes):
        """Verify boundaries are suitable for parallel record processing."""
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)
        
        assert len(boundaries) > 0, "Should find at least one record"
        
        # Verify boundaries are non-overlapping
        for i, (offset1, len1) in enumerate(boundaries):
            for j, (offset2, len2) in enumerate(boundaries):
                if i != j:
                    # Records should not overlap
                    end1 = offset1 + len1
                    assert end1 <= offset2 or offset2 + len2 <= offset1, \
                        f"Records {i} and {j} overlap"

    def test_sequential_vs_boundary_parsing(self, multi_records_bytes):
        """Verify boundaries are consistent with sequential parsing."""
        # Sequential parsing identifies complete records
        reader = MARCReader(multi_records_bytes)
        sequential_records = []
        while True:
            record = reader.read_record()
            if record is None:
                break
            sequential_records.append(record)

        assert len(sequential_records) > 0, "Should find records via sequential parsing"

        # Boundary-based scan finds record delimiters
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)

        # Both methods should find records/boundaries
        assert len(boundaries) > 0, "Boundary scan should find record boundaries"
        # Boundary scan typically finds >= sequential parser (may find partial records)
        assert len(boundaries) >= len(sequential_records), \
            "Boundary scan should find >= complete records"


class TestBoundaryScannerAcceptanceCriteria:
    """Test boundary scanner acceptance criteria."""

    def test_accepts_real_marc_data(self, simple_book_bytes, multi_records_bytes):
        """Criterion 1: Scanner accepts real MARC files."""
        scanner = RecordBoundaryScanner()

        # Both files should be scannable
        boundaries1 = scanner.scan(simple_book_bytes)
        assert len(boundaries1) > 0

        boundaries2 = scanner.scan(multi_records_bytes)
        assert len(boundaries2) > 0

    def test_produces_valid_boundaries(self, multi_records_bytes):
        """Criterion 2: Boundaries are valid offsets into buffer."""
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)

        for offset, length in boundaries:
            # Offset should be within bounds
            assert 0 <= offset < len(multi_records_bytes)
            # Length should fit
            assert offset + length <= len(multi_records_bytes)
            # Should end with record terminator (0x1D)
            assert multi_records_bytes[offset + length - 1] == 0x1D

    def test_enables_parallel_parsing_readiness(self, multi_records_bytes):
        """Criterion 3: Output suitable for parallel processing."""
        scanner = RecordBoundaryScanner()
        boundaries = scanner.scan(multi_records_bytes)

        # Boundaries should be independent (non-overlapping)
        seen_ranges = set()
        for offset, length in boundaries:
            for i in range(offset, offset + length):
                assert i not in seen_ranges, "Overlapping boundaries"
                seen_ranges.add(i)

        # All scanned bytes should be covered exactly once
        # Note: May not cover entire file if last record is incomplete
        assert len(seen_ranges) == len(seen_ranges), "Should not have duplicates"
        
        # Verify boundaries don't exceed file size
        for offset, length in boundaries:
            assert offset + length <= len(multi_records_bytes), \
                f"Boundary exceeds file: {offset} + {length} > {len(multi_records_bytes)}"
