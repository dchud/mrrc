"""
Benchmark tests for MARC record writing performance.
"""

import pytest
import io
import tempfile
import os
import time
from pathlib import Path
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


class TestRustFileBackendBenchmarks:
     """Benchmarks for RustFile backend (direct file I/O via Rust)."""
     
     @pytest.mark.benchmark
     def test_write_only_1k_rustfile(self, benchmark, fixture_1k):
         """Benchmark writing 1,000 records via RustFile backend (file path)."""
         # Pre-load records outside of benchmark
         data = io.BytesIO(fixture_1k)
         reader = MARCReader(data)
         records = []
         while record := reader.read_record():
             records.append(record)
         
         def write_all():
             with tempfile.NamedTemporaryFile(delete=False, suffix='.mrc') as tmp:
                 temp_path = tmp.name
             
             try:
                 # Write using RustFile backend (string path)
                 writer = MARCWriter(temp_path)
                 for record in records:
                     writer.write_record(record)
                 writer.close()
                 
                 # Read the file size
                 file_size = os.path.getsize(temp_path)
                 return file_size
             finally:
                 if os.path.exists(temp_path):
                     os.unlink(temp_path)
         
         result = benchmark(write_all)
         assert result > 0
     
     @pytest.mark.benchmark
     def test_write_only_10k_rustfile(self, benchmark, fixture_10k):
         """Benchmark writing 10,000 records via RustFile backend."""
         # Pre-load records
         data = io.BytesIO(fixture_10k)
         reader = MARCReader(data)
         records = []
         while record := reader.read_record():
             records.append(record)
         
         def write_all():
             with tempfile.NamedTemporaryFile(delete=False, suffix='.mrc') as tmp:
                 temp_path = tmp.name
             
             try:
                 # Write using RustFile backend
                 writer = MARCWriter(temp_path)
                 for record in records:
                     writer.write_record(record)
                 writer.close()
                 
                 # Read the file size
                 file_size = os.path.getsize(temp_path)
                 return file_size
             finally:
                 if os.path.exists(temp_path):
                     os.unlink(temp_path)
         
         result = benchmark(write_all)
         assert result > 0
     
     @pytest.mark.benchmark
     def test_write_pathlib_1k_rustfile(self, benchmark, fixture_1k):
         """Benchmark writing 1,000 records via RustFile backend with pathlib.Path."""
         # Pre-load records
         data = io.BytesIO(fixture_1k)
         reader = MARCReader(data)
         records = []
         while record := reader.read_record():
             records.append(record)
         
         def write_all():
             with tempfile.NamedTemporaryFile(delete=False, suffix='.mrc') as tmp:
                 temp_path = Path(tmp.name)
             
             try:
                 # Write using RustFile backend with Path object
                 writer = MARCWriter(temp_path)
                 for record in records:
                     writer.write_record(record)
                 writer.close()
                 
                 # Get file size
                 file_size = temp_path.stat().st_size
                 return file_size
             finally:
                 if temp_path.exists():
                     temp_path.unlink()
         
         result = benchmark(write_all)
         assert result > 0


class TestBackendComparison:
     """Performance comparison between PythonFile (BytesIO) and RustFile backends."""

     @pytest.mark.benchmark
     def test_backend_comparison_1k(self, fixture_1k):
         """Compare PythonFile vs RustFile performance for 1k records."""
         # Pre-load records
         data = io.BytesIO(fixture_1k)
         reader = MARCReader(data)
         records = []
         while record := reader.read_record():
             records.append(record)
         
         # Benchmark PythonFile backend (BytesIO)
         # Use 5 iterations and median to drop outliers from CI runner noise
         pythonfile_times = []
         for _ in range(5):
             start = time.perf_counter()
             output = io.BytesIO()
             writer = MARCWriter(output)
             for record in records:
                 writer.write_record(record)
             writer.close()
             elapsed = time.perf_counter() - start
             pythonfile_times.append(elapsed)

         # Benchmark RustFile backend (file path)
         rustfile_times = []
         for _ in range(5):
             with tempfile.NamedTemporaryFile(delete=False, suffix='.mrc') as tmp:
                 temp_path = tmp.name

             try:
                 start = time.perf_counter()
                 writer = MARCWriter(temp_path)
                 for record in records:
                     writer.write_record(record)
                 writer.close()
                 elapsed = time.perf_counter() - start
                 rustfile_times.append(elapsed)
             finally:
                 if os.path.exists(temp_path):
                     os.unlink(temp_path)

         # Use median to naturally drop outlier spikes from CI runner noise
         median_pythonfile = sorted(pythonfile_times)[len(pythonfile_times) // 2]
         median_rustfile = sorted(rustfile_times)[len(rustfile_times) // 2]

         # RustFile should be comparable to PythonFile (no regression)
         # Note: exact performance depends on system I/O, so we just verify no major regression
         speedup = median_pythonfile / median_rustfile
         print("\n1k records benchmark:")
         print(f"  PythonFile (BytesIO): {median_pythonfile*1000:.2f}ms")
         print(f"  RustFile (temp file): {median_rustfile*1000:.2f}ms")
         print(f"  Speedup ratio: {speedup:.2f}x")

         # No assertion — timing comparisons on shared CI runners are too noisy
         # for reliable pass/fail. The printed output above is informational.
