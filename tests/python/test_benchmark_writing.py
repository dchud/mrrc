"""
Benchmark tests for MARC record writing performance.
"""

import pytest
import io
from mrrc import MARCReader, MARCWriter


class TestWritingBenchmarks:
    """Benchmarks for writing operations."""
    
    @pytest.mark.benchmark
    def test_roundtrip_1k_records(self, benchmark, fixture_1k):
        """Benchmark reading and writing 1,000 records."""
        def roundtrip():
            # Read all records
            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            records = []
            while record := reader.read_record():
                records.append(record)
            
            # Write all records
            output = io.BytesIO()
            writer = MARCWriter(output)
            for record in records:
                writer.write_record(record)
            
            return output.getvalue()
        
        result = benchmark(roundtrip)
        # Check that we got some data back
        assert len(result) > 0
        assert len(result) > len(fixture_1k) * 0.9  # Should be similar size
    
    @pytest.mark.benchmark
    def test_write_only_1k_records(self, benchmark, fixture_1k):
        """Benchmark writing 1,000 pre-loaded records."""
        # Pre-load records outside of benchmark
        data = io.BytesIO(fixture_1k)
        reader = MARCReader(data)
        records = []
        while record := reader.read_record():
            records.append(record)
        
        def write_all():
            output = io.BytesIO()
            writer = MARCWriter(output)
            for record in records:
                writer.write_record(record)
            return output.getvalue()
        
        result = benchmark(write_all)
        assert len(result) > 0
    
    @pytest.mark.benchmark
    def test_write_only_10k_records(self, benchmark, fixture_10k):
        """Benchmark writing 10,000 pre-loaded records."""
        # Pre-load records
        data = io.BytesIO(fixture_10k)
        reader = MARCReader(data)
        records = []
        while record := reader.read_record():
            records.append(record)
        
        def write_all():
            output = io.BytesIO()
            writer = MARCWriter(output)
            for record in records:
                writer.write_record(record)
            return output.getvalue()
        
        result = benchmark(write_all)
        assert len(result) > 0
    
    @pytest.mark.benchmark
    @pytest.mark.slow
    def test_roundtrip_100k_records(self, benchmark, fixture_100k):
        """Benchmark reading and writing 100,000 records (full roundtrip)."""
        def roundtrip():
            # Read all records
            data = io.BytesIO(fixture_100k)
            reader = MARCReader(data)
            records = []
            while record := reader.read_record():
                records.append(record)
            
            # Write all records
            output = io.BytesIO()
            writer = MARCWriter(output)
            for record in records:
                writer.write_record(record)
            
            return output.getvalue()
        
        result = benchmark(roundtrip)
        assert len(result) > 0
        # Result should be roughly the same size as input
        assert len(result) > len(fixture_100k) * 0.9


class TestIncrementalWriting:
    """Benchmarks for incremental/streaming write patterns."""
    
    @pytest.mark.benchmark
    def test_stream_write_1k(self, benchmark, fixture_1k):
        """Benchmark streaming write pattern (read, modify, write)."""
        def stream_and_write():
            data = io.BytesIO(fixture_1k)
            input_reader = MARCReader(data)
            output = io.BytesIO()
            writer = MARCWriter(output)
            
            count = 0
            while record := input_reader.read_record():
                # Simulate a simple modification
                # (in real use, you might add fields, update data, etc.)
                writer.write_record(record)
                count += 1
            
            return output.getvalue(), count
        
        result, count = benchmark(stream_and_write)
        assert count == 1000
        assert len(result) > 0
