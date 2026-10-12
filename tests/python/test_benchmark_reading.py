"""
Benchmark tests for MARC record reading performance.
"""

import io

import pytest

from mrrc import MARCReader


class TestReadingBenchmarks:
    """Benchmarks for reading operations."""

    @pytest.mark.benchmark
    def test_read_1k_records(self, benchmark, fixture_1k):
        """Benchmark reading 1,000 records."""

        def read_all():
            # Create fresh BytesIO each time
            import io

            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            records = []
            while (record := reader.read_record()) is not None:
                records.append(record)
            return records

        result = benchmark(read_all)
        assert len(result) == 1000

    @pytest.mark.benchmark
    def test_read_10k_records(self, benchmark, fixture_10k):
        """Benchmark reading 10,000 records."""

        def read_all():
            data = io.BytesIO(fixture_10k)
            reader = MARCReader(data)
            records = []
            while (record := reader.read_record()) is not None:
                records.append(record)
            return records

        result = benchmark(read_all)
        assert len(result) == 10000

    @pytest.mark.benchmark
    def test_read_1k_records_from_path(self, benchmark, fixture_1k_path):
        """Benchmark reading 1,000 records from a file path (RustFile backend)."""

        def read_all():
            reader = MARCReader(fixture_1k_path)
            records = []
            while (record := reader.read_record()) is not None:
                records.append(record)
            return records

        result = benchmark(read_all)
        assert len(result) == 1000

    @pytest.mark.benchmark
    def test_read_10k_records_from_path(self, benchmark, fixture_10k_path):
        """Benchmark reading 10,000 records from a file path (RustFile backend)."""

        def read_all():
            reader = MARCReader(fixture_10k_path)
            records = []
            while (record := reader.read_record()) is not None:
                records.append(record)
            return records

        result = benchmark(read_all)
        assert len(result) == 10000

    @pytest.mark.benchmark
    def test_read_and_extract_titles_1k(self, benchmark, fixture_1k):
        """Benchmark reading records and extracting titles."""

        def read_with_extraction():
            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            titles = []
            while (record := reader.read_record()) is not None:
                # Try to get title from field 245
                title = record.title or "Unknown"
                titles.append(title)
            return titles

        result = benchmark(read_with_extraction)
        assert len(result) == 1000

    @pytest.mark.benchmark
    def test_read_and_extract_titles_10k(self, benchmark, fixture_10k):
        """Benchmark reading 10k records and extracting titles."""

        def read_with_extraction():
            data = io.BytesIO(fixture_10k)
            reader = MARCReader(data)
            titles = []
            while (record := reader.read_record()) is not None:
                title = record.title or "Unknown"
                titles.append(title)
            return titles

        result = benchmark(read_with_extraction)
        assert len(result) == 10000


class TestRecordShapeReadingBenchmarks:
    """Reading benchmarks over fixtures of realistic, large, and MARC-8 records.

    The 1k and 10k fixtures hold records of about 6 fields; these cover
    records of 12-30 fields, records of 150-1,000 fields, and MARC-8 decoding.
    """

    @staticmethod
    def read_all(data):
        reader = MARCReader(io.BytesIO(data))
        records = []
        while (record := reader.read_record()) is not None:
            records.append(record)
        return records

    @pytest.mark.benchmark
    def test_read_realistic_records(self, benchmark, fixture_realistic):
        """Benchmark reading 2,000 records of 12-30 fields."""
        result = benchmark(self.read_all, fixture_realistic)
        assert len(result) == 2000

    @pytest.mark.benchmark
    def test_read_large_records(self, benchmark, fixture_large):
        """Benchmark reading 100 records of 150-250 fields."""
        result = benchmark(self.read_all, fixture_large)
        assert len(result) == 100

    @pytest.mark.benchmark
    def test_read_very_large_records(self, benchmark, fixture_very_large):
        """Benchmark reading 10 records of 800-1,000 fields."""
        result = benchmark(self.read_all, fixture_very_large)
        assert len(result) == 10

    @pytest.mark.benchmark
    def test_read_marc8_records(self, benchmark, fixture_marc8):
        """Benchmark reading 1,000 MARC-8 records with diacritics and
        Cyrillic and Greek text."""
        result = benchmark(self.read_all, fixture_marc8)
        assert len(result) == 1000


class TestIterationBenchmarks:
    """Benchmarks for different iteration patterns."""

    @pytest.mark.benchmark
    def test_iterator_vs_while_loop_1k(self, benchmark, fixture_1k):
        """Benchmark using iterator vs while loop."""

        def iterate_records():
            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            count = 0
            while reader.read_record() is not None:
                count += 1
            return count

        result = benchmark(iterate_records)
        assert result == 1000

    @pytest.mark.benchmark
    def test_collect_all_records_1k(self, benchmark, fixture_1k):
        """Benchmark collecting all records into a list."""

        def collect_all():
            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            records = []
            while (record := reader.read_record()) is not None:
                records.append(record)
            return records

        result = benchmark(collect_all)
        assert len(result) == 1000
