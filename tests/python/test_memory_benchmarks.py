"""
Memory usage benchmarks for MARC record processing.

Measures peak memory consumption during various operations to track
memory efficiency of the Python wrapper.
"""

import pytest
import io
import sys
import tracemalloc
from mrrc import MARCReader, MARCWriter, Record, Field


class TestMemoryBenchmarks:
    """Memory usage benchmarks for record operations."""

    def measure_peak_memory(self, func):
        """Helper to measure peak memory usage of a function."""
        tracemalloc.start()
        try:
            result = func()
            current, peak = tracemalloc.get_traced_memory()
            return result, peak
        finally:
            tracemalloc.stop()

    @pytest.mark.benchmark
    def test_memory_read_1k_records(self, fixture_1k):
        """Measure peak memory when reading 1,000 records."""
        def read_all():
            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            records = []
            while record := reader.read_record():
                records.append(record)
            return records
        
        records, peak_memory = self.measure_peak_memory(read_all)
        
        assert len(records) == 1000
        # Rough estimate: ~1KB per record in memory
        # 1000 records should use less than 10MB
        assert peak_memory < 10 * 1024 * 1024, f"Peak memory {peak_memory / 1024 / 1024:.2f}MB exceeds 10MB for 1k records"

    @pytest.mark.benchmark
    def test_memory_read_10k_records(self, fixture_10k):
        """Measure peak memory when reading 10,000 records."""
        def read_all():
            data = io.BytesIO(fixture_10k)
            reader = MARCReader(data)
            records = []
            while record := reader.read_record():
                records.append(record)
            return records
        
        records, peak_memory = self.measure_peak_memory(read_all)
        
        assert len(records) == 10000
        # 10k records should use less than 100MB
        assert peak_memory < 100 * 1024 * 1024, f"Peak memory {peak_memory / 1024 / 1024:.2f}MB exceeds 100MB for 10k records"

    @pytest.mark.benchmark
    def test_memory_streaming_read_10k(self, fixture_10k):
        """Measure memory with streaming (not storing all records)."""
        def stream_and_process():
            data = io.BytesIO(fixture_10k)
            reader = MARCReader(data)
            count = 0
            while record := reader.read_record():
                # Process but don't store
                _ = record.title()
                count += 1
            return count

        count, peak_memory = self.measure_peak_memory(stream_and_process)

        assert count == 10000
        # Streaming should use minimal memory (just one record at a time)
        # Allow up to 10MB for FFI overhead, internal buffers, and Python objects
        assert peak_memory < 10 * 1024 * 1024, f"Peak memory {peak_memory / 1024 / 1024:.2f}MB exceeds 10MB for streaming"

    @pytest.mark.benchmark
    def test_memory_field_creation_bulk(self):
        """Measure memory usage when creating many fields."""
        def create_many_fields():
            fields = []
            for i in range(10000):
                field = Field('650', ' ', '0')
                field.add_subfield('a', f'Subject {i}')
                fields.append(field)
            return fields
        
        fields, peak_memory = self.measure_peak_memory(create_many_fields)
        
        assert len(fields) == 10000
        # 10k fields should use less than 10MB
        assert peak_memory < 10 * 1024 * 1024, f"Peak memory {peak_memory / 1024 / 1024:.2f}MB exceeds 10MB for 10k fields"

    @pytest.mark.benchmark
    def test_memory_record_creation_bulk(self):
        """Measure memory usage when creating many records."""
        def create_many_records():
            records = []
            for i in range(1000):
                record = Record()
                record.add_control_field('001', f'id-{i}')
                field = Field('245', '1', '0')
                field.add_subfield('a', f'Title {i}')
                record.add_field(field)
                records.append(record)
            return records
        
        records, peak_memory = self.measure_peak_memory(create_many_records)
        
        assert len(records) == 1000
        # 1000 records should use less than 10MB
        assert peak_memory < 10 * 1024 * 1024, f"Peak memory {peak_memory / 1024 / 1024:.2f}MB exceeds 10MB for 1k records"

    @pytest.mark.benchmark
    def test_memory_serialization_1k(self, fixture_1k):
        """Measure memory when serializing 1k records to MARC21."""
        def serialize_all():
            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            marc_outputs = []
            while record := reader.read_record():
                marc_bytes = record.to_marc21()
                marc_outputs.append(marc_bytes)
            return marc_outputs
        
        outputs, peak_memory = self.measure_peak_memory(serialize_all)
        
        assert len(outputs) == 1000
        # Serializing 1k records should use less than 20MB
        assert peak_memory < 20 * 1024 * 1024, f"Peak memory {peak_memory / 1024 / 1024:.2f}MB exceeds 20MB for serialization"

    @pytest.mark.benchmark
    def test_memory_json_serialization_1k(self, fixture_1k):
        """Measure memory when serializing 1k records to JSON."""
        def json_serialize_all():
            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            json_outputs = []
            while record := reader.read_record():
                json_str = record.to_json()
                json_outputs.append(json_str)
            return json_outputs
        
        outputs, peak_memory = self.measure_peak_memory(json_serialize_all)
        
        assert len(outputs) == 1000
        # JSON serialization of 1k records should use less than 50MB
        assert peak_memory < 50 * 1024 * 1024, f"Peak memory {peak_memory / 1024 / 1024:.2f}MB exceeds 50MB for JSON serialization"

    @pytest.mark.benchmark
    def test_memory_roundtrip_serialize_deserialize_1k(self, fixture_1k):
        """Measure memory for round-trip serialization/deserialization."""
        def roundtrip_all():
            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            roundtrip_records = []
            
            for record in reader:
                # Serialize to MARC21
                marc_bytes = record.to_marc21()
                
                # Deserialize back
                restored_reader = MARCReader(io.BytesIO(marc_bytes))
                restored_record = restored_reader.read_record()
                
                if restored_record:
                    roundtrip_records.append(restored_record)
            
            return roundtrip_records
        
        records, peak_memory = self.measure_peak_memory(roundtrip_all)
        
        assert len(records) == 1000
        # Round-trip should use less than 30MB for 1k records
        assert peak_memory < 30 * 1024 * 1024, f"Peak memory {peak_memory / 1024 / 1024:.2f}MB exceeds 30MB for round-trip"

    @pytest.mark.benchmark
    def test_memory_multiple_format_conversions_1k(self, fixture_1k):
        """Measure memory when converting 1k records to multiple formats."""
        def multi_format_convert():
            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            conversions = []
            
            while record := reader.read_record():
                formats = {
                    'json': record.to_json(),
                    'xml': record.to_xml(),
                    'marcjson': record.to_marcjson(),
                }
                conversions.append(formats)
            
            return conversions
        
        conversions, peak_memory = self.measure_peak_memory(multi_format_convert)
        
        assert len(conversions) == 1000
        # Multiple format conversions should use less than 100MB
        assert peak_memory < 100 * 1024 * 1024, f"Peak memory {peak_memory / 1024 / 1024:.2f}MB exceeds 100MB for format conversions"

    @pytest.mark.benchmark
    def test_memory_field_access_patterns_1k(self, fixture_1k):
        """Measure memory overhead of field access patterns."""
        def access_patterns():
            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            results = []
            
            while record := reader.read_record():
                # Various access patterns
                title = record.title()
                author = record.author()
                subjects = record.subjects()
                fields_245 = record.get_fields('245')
                
                results.append((title, author, len(subjects), len(fields_245)))
            
            return results
        
        results, peak_memory = self.measure_peak_memory(access_patterns)
        
        assert len(results) == 1000
        # Field access patterns should have minimal overhead
        assert peak_memory < 10 * 1024 * 1024, f"Peak memory {peak_memory / 1024 / 1024:.2f}MB exceeds 10MB for field access"


class TestMemoryLeaks:
    """Tests to detect potential memory leaks."""

    @pytest.mark.benchmark
    def test_repeated_record_creation_no_leak(self):
        """Verify no memory leak in repeated record creation."""
        tracemalloc.start()
        
        # Create records in batches and check memory doesn't grow unbounded
        measurements = []
        
        for batch in range(10):
            for i in range(100):
                record = Record()
                field = Field('245', '1', '0')
                field.add_subfield('a', f'Title {i}')
                record.add_field(field)
            
            current, peak = tracemalloc.get_traced_memory()
            measurements.append(peak)
        
        tracemalloc.stop()
        
        # Memory should not grow significantly between batches
        # Allow up to 2x growth (due to Python overhead)
        assert measurements[-1] < measurements[0] * 2, \
            f"Memory leak detected: {measurements[0]} -> {measurements[-1]}"

    @pytest.mark.benchmark
    def test_repeated_serialization_no_leak(self, fixture_1k):
        """Verify no memory leak in repeated serialization."""
        def serialize_once():
            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            while record := reader.read_record():
                _ = record.to_json()

        # Measure peak memory for each iteration independently
        measurements = []
        for _ in range(5):
            tracemalloc.start()
            serialize_once()
            current, peak = tracemalloc.get_traced_memory()
            tracemalloc.stop()
            measurements.append(peak)

        # Memory per iteration should be relatively stable
        # Allow up to 2x variance between iterations (due to GC timing, etc.)
        assert max(measurements) < min(measurements) * 2, \
            f"Possible memory leak in serialization: {measurements}"


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
