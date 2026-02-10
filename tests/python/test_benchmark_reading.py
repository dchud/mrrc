"""
Benchmark tests for MARC record reading performance.
"""

import pytest
import io
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
            while record := reader.read_record():
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
            while record := reader.read_record():
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
            while record := reader.read_record():
                # Try to get title from field 245
                title = record.title() or "Unknown"
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
            while record := reader.read_record():
                title = record.title() or "Unknown"
                titles.append(title)
            return titles
        
        result = benchmark(read_with_extraction)
        assert len(result) == 10000
    

class TestIterationBenchmarks:
    """Benchmarks for different iteration patterns."""
    
    @pytest.mark.benchmark
    def test_iterator_vs_while_loop_1k(self, benchmark, fixture_1k):
        """Benchmark using iterator vs while loop."""
        def iterate_records():
            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            count = 0
            while record := reader.read_record():
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
            while record := reader.read_record():
                records.append(record)
            return records
        
        result = benchmark(collect_all)
        assert len(result) == 1000
