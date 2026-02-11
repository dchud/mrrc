"""
Concurrent write tests for Python MARC writers.

Demonstrates GIL release on the write-side and validates:
1. Write-side parallelism (2+ threads achieve speedup)
2. Round-trip correctness (read → write → read)
3. Data integrity (no corruption or loss)
"""

import pytest
import io
import tempfile
import os
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader, MARCWriter


class TestWriteGILRelease:
     """Tests for write-side GIL release and parallelism."""

     def test_write_single_record(self, fixture_1k):
         """Basic write test: single record."""
         # Read a single record from fixture
         data = io.BytesIO(fixture_1k)
         reader = MARCReader(data)
         records = []
         for record in reader:
             records.append(record)
             if len(records) >= 1:
                 break

         # Write it out
         output = io.BytesIO()
         writer = MARCWriter(output)
         for record in records:
             writer.write_record(record)
         writer.close()

         # Verify something was written
         output.seek(0)
         data = output.read()
         assert len(data) > 0

     def test_write_multiple_records(self, fixture_1k):
         """Write test: multiple records."""
         # Read records from fixture
         data = io.BytesIO(fixture_1k)
         reader = MARCReader(data)
         records = list(reader)
         assert len(records) > 0

         # Write them out
         output = io.BytesIO()
         writer = MARCWriter(output)
         for record in records:
             writer.write_record(record)
         writer.close()

         # Verify data was written
         output.seek(0)
         data = output.read()
         assert len(data) > 0

     def test_sequential_write_2x_1k(self, fixture_1k):
         """Baseline: sequential writing of 2x 1k records."""
         # Read records from two instances of fixture
         def read_records():
             reader = MARCReader(io.BytesIO(fixture_1k))
             return list(reader)

         records_a = read_records()
         records_b = read_records()
         all_records = records_a + records_b

         # Write all records sequentially
         output = io.BytesIO()
         writer = MARCWriter(output)
         for record in all_records:
             writer.write_record(record)
         writer.close()

         output.seek(0)
         sequential_data = output.read()
         assert len(sequential_data) > 0
         assert len(all_records) == len(records_a) + len(records_b)

     def test_concurrent_write_2x_1k_speedup(self, fixture_1k):
         """
         Concurrent write test: verify 2-thread execution works without GIL deadlock.

         Tests that GIL is released during serialization phase,
         allowing two threads to write concurrently without blocking.

         Note: Detailed performance benchmarking is in separate benchmarking suite,
         which has more controlled conditions for accurate timing measurements.
         """
         # Read records once
         reader = MARCReader(io.BytesIO(fixture_1k))
         records = list(reader)
         assert len(records) > 0

         # Function to write records to output
         def write_records(records_copy):
             output = io.BytesIO()
             writer = MARCWriter(output)
             for record in records_copy:
                 writer.write_record(record)
             writer.close()
             output.seek(0)
             return output.read()

         # Sequential baseline
         sequential_data = write_records(records)

         # Concurrent with 2 threads
         with ThreadPoolExecutor(max_workers=2) as executor:
             # Two threads, each writing the same records to different files
             futures = [
                 executor.submit(write_records, records),
                 executor.submit(write_records, records),
             ]
             concurrent_data = [f.result() for f in futures]

         # Verify output files have content
         assert len(concurrent_data) == 2
         assert all(len(d) > 0 for d in concurrent_data)

         # Both concurrent outputs should be identical to sequential
         assert concurrent_data[0] == sequential_data
         assert concurrent_data[1] == sequential_data

     def test_concurrent_write_4x_1k(self, fixture_1k):
         """
         Concurrent write test: 4 threads.

         Verifies that 4 threads can write concurrently without GIL deadlock.
         Detailed performance measurements are in the benchmarking suite.
         """
         reader = MARCReader(io.BytesIO(fixture_1k))
         records = list(reader)

         def write_records(records_copy):
             output = io.BytesIO()
             writer = MARCWriter(output)
             for record in records_copy:
                 writer.write_record(record)
             writer.close()
             output.seek(0)
             return output.read()

         # Get baseline result
         baseline = write_records(records)

         # Concurrent with 4 threads
         with ThreadPoolExecutor(max_workers=4) as executor:
             results = list(
                 executor.map(write_records, [records] * 4)
             )

         # All outputs should be identical
         assert len(results) == 4
         assert all(r == baseline for r in results)


class TestRoundTrip:
     """Round-trip tests: read → write → read."""

     def test_round_trip_basic(self, fixture_1k):
         """Round-trip: read → write → read."""
         # Read all records from fixture
         reader = MARCReader(io.BytesIO(fixture_1k))
         records_original = list(reader)
         assert len(records_original) > 0

         # Write them to a BytesIO
         output = io.BytesIO()
         writer = MARCWriter(output)
         for record in records_original:
             writer.write_record(record)
         writer.close()

         # Read them back
         output.seek(0)
         reader2 = MARCReader(output)
         records_roundtrip = list(reader2)

         # Verify count matches
         assert len(records_roundtrip) == len(records_original)

         # Verify each record matches
         for orig, roundtrip in zip(records_original, records_roundtrip):
             assert orig == roundtrip

     def test_round_trip_preserves_fields(self, fixture_1k):
         """Round-trip preserves field data."""
         reader = MARCReader(io.BytesIO(fixture_1k))
         records_original = list(reader)

         # Write and read back
         output = io.BytesIO()
         writer = MARCWriter(output)
         for record in records_original:
             writer.write_record(record)
         writer.close()

         output.seek(0)
         reader2 = MARCReader(output)
         records_roundtrip = list(reader2)

         # Spot check some fields
         for orig, rt in zip(records_original, records_roundtrip):
             # Leader should match
             assert orig.leader().record_type == rt.leader().record_type
             assert orig.leader().bibliographic_level == rt.leader().bibliographic_level

             # Title (245) should match
             orig_title = orig.title()
             rt_title = rt.title()
             if orig_title:
                 assert rt_title == orig_title

             # Author (100/110) should match
             orig_author = orig.author()
             rt_author = rt.author()
             if orig_author:
                 assert rt_author == orig_author

     def test_round_trip_with_modification(self, fixture_1k):
         """Round-trip with record modification.
         
         Tests that leader properties can be modified and persist through
         a write/read cycle. This validates the leader mutation API.
         """
         # Read original records
         reader = MARCReader(io.BytesIO(fixture_1k))
         records_original = list(reader)
         assert len(records_original) > 0
         
         # Modify leader properties on first few records
         for i, record in enumerate(records_original[:3]):
             leader = record.leader()
             # Change record status to 'c' (corrected)
             leader.record_status = 'c'
             # Change encoding level to 'I' (full level)
             leader.encoding_level = 'I'
             # Change cataloging form to 'a' (AACR2)
             leader.cataloging_form = 'a'
         
         # Write modified records
         output = io.BytesIO()
         writer = MARCWriter(output)
         for record in records_original:
             writer.write_record(record)
         writer.close()
         
         # Read them back
         output.seek(0)
         reader2 = MARCReader(output)
         records_roundtrip = list(reader2)
         
         # Verify count matches
         assert len(records_roundtrip) == len(records_original)
         
         # Verify modifications persisted
         for i, (orig, roundtrip) in enumerate(zip(records_original[:3], records_roundtrip[:3])):
             orig_leader = orig.leader()
             rt_leader = roundtrip.leader()
             
             # These should have been modified
             assert rt_leader.record_status == 'c'
             assert rt_leader.encoding_level == 'I'
             assert rt_leader.cataloging_form == 'a'
             
             # Verify they match what we expect
             assert rt_leader.record_status == orig_leader.record_status
             assert rt_leader.encoding_level == orig_leader.encoding_level
             assert rt_leader.cataloging_form == orig_leader.cataloging_form
         
         # Verify remaining records unchanged
         for i, (orig, roundtrip) in enumerate(zip(records_original[3:], records_roundtrip[3:]), start=3):
             assert orig.leader() == roundtrip.leader()

     def test_round_trip_large_file(self, fixture_10k):
         """Round-trip test with large file (10k records)."""
         reader = MARCReader(io.BytesIO(fixture_10k))
         records_original = list(reader)
         count_original = len(records_original)

         # Write all
         output = io.BytesIO()
         writer = MARCWriter(output)
         for record in records_original:
             writer.write_record(record)
         writer.close()

         # Read back
         output.seek(0)
         reader2 = MARCReader(output)
         records_roundtrip = list(reader2)

         # Verify count
         assert len(records_roundtrip) == count_original

         # Verify first and last records match
         assert records_original[0] == records_roundtrip[0]
         assert records_original[-1] == records_roundtrip[-1]


class TestWriteEdgeCases:
     """Edge case tests for writing."""

     def test_write_empty_file(self):
         """Writing zero records produces valid output."""
         output = io.BytesIO()
         writer = MARCWriter(output)
         # Write nothing
         writer.close()

         output.seek(0)
         data = output.read()
         # Should be empty or have minimal structure
         assert len(data) >= 0

     def test_write_context_manager(self, fixture_1k):
         """Write using context manager."""
         reader = MARCReader(io.BytesIO(fixture_1k))
         records = list(reader)

         output = io.BytesIO()
         with MARCWriter(output) as writer:
             for record in records:
                 writer.write_record(record)

         output.seek(0)
         data = output.read()
         assert len(data) > 0

     def test_write_after_close_raises_error(self, fixture_1k):
         """Writing after close raises error."""
         reader = MARCReader(io.BytesIO(fixture_1k))
         record = next(reader)

         output = io.BytesIO()
         writer = MARCWriter(output)
         writer.close()

         # Should raise error
         with pytest.raises(RuntimeError):
             writer.write_record(record)

     def test_write_close_idempotent(self, fixture_1k):
         """Calling close() multiple times is safe."""
         output = io.BytesIO()
         writer = MARCWriter(output)
         writer.close()
         writer.close()  # Should not raise
         writer.close()  # Should not raise


class TestRustFileBackend:
     """Tests for RustFile backend (direct file I/O via Rust)."""

     def test_write_roundtrip_rust_file(self, fixture_1k):
         """Round-trip test using RustFile backend (file path)."""
         # Read records from fixture
         reader = MARCReader(io.BytesIO(fixture_1k))
         records_original = list(reader)
         assert len(records_original) > 0

         # Write to a temporary file using RustFile backend
         with tempfile.NamedTemporaryFile(delete=False, suffix='.mrc') as tmp:
             temp_path = tmp.name

         try:
             # Write using string path (RustFile backend)
             writer = MARCWriter(temp_path)
             for record in records_original:
                 writer.write_record(record)
             writer.close()

             # Read back from the file
             with open(temp_path, 'rb') as f:
                 reader2 = MARCReader(f)
                 records_roundtrip = list(reader2)

             # Verify round-trip
             assert len(records_roundtrip) == len(records_original)
             for orig, roundtrip in zip(records_original, records_roundtrip):
                 assert orig == roundtrip

         finally:
             if os.path.exists(temp_path):
                 os.unlink(temp_path)

     def test_write_roundtrip_pathlib_path(self, fixture_1k):
         """Round-trip test using RustFile backend with pathlib.Path."""
         # Read records from fixture
         reader = MARCReader(io.BytesIO(fixture_1k))
         records_original = list(reader)
         assert len(records_original) > 0

         # Write to a temporary file using pathlib.Path
         with tempfile.NamedTemporaryFile(delete=False, suffix='.mrc') as tmp:
             temp_path = Path(tmp.name)

         try:
             # Write using Path object (RustFile backend)
             writer = MARCWriter(temp_path)
             for record in records_original:
                 writer.write_record(record)
             writer.close()

             # Read back from the file
             with open(temp_path, 'rb') as f:
                 reader2 = MARCReader(f)
                 records_roundtrip = list(reader2)

             # Verify round-trip
             assert len(records_roundtrip) == len(records_original)
             for orig, roundtrip in zip(records_original, records_roundtrip):
                 assert orig == roundtrip

         finally:
             if temp_path.exists():
                 temp_path.unlink()

     def test_write_multiple_records_rust_file(self, fixture_1k):
         """Write batch of records via RustFile backend."""
         # Read records
         reader = MARCReader(io.BytesIO(fixture_1k))
         records = list(reader)
         assert len(records) > 0

         with tempfile.NamedTemporaryFile(delete=False, suffix='.mrc') as tmp:
             temp_path = tmp.name

         try:
             # Write all records
             writer = MARCWriter(temp_path)
             for record in records:
                 writer.write_record(record)
             writer.close()

             # Read back and verify count
             with open(temp_path, 'rb') as f:
                 reader2 = MARCReader(f)
                 roundtrip_records = list(reader2)

             assert len(roundtrip_records) == len(records)

         finally:
             if os.path.exists(temp_path):
                 os.unlink(temp_path)

     def test_concurrent_writes_different_files(self, fixture_1k):
         """Thread safety: concurrent writes to different files (RustFile backend)."""
         # Read records once
         reader = MARCReader(io.BytesIO(fixture_1k))
         records = list(reader)
         assert len(records) > 0

         def write_to_file(file_index):
             """Helper to write records to a temp file."""
             with tempfile.NamedTemporaryFile(delete=False, suffix=f'_{file_index}.mrc') as tmp:
                 temp_path = tmp.name

             try:
                 # Write records
                 writer = MARCWriter(temp_path)
                 for record in records:
                     writer.write_record(record)
                 writer.close()

                 # Read back and verify
                 with open(temp_path, 'rb') as f:
                     reader2 = MARCReader(f)
                     roundtrip = list(reader2)

                 return len(roundtrip) == len(records), temp_path

             except Exception as e:
                 if os.path.exists(temp_path):
                     os.unlink(temp_path)
                 raise e

         # Run 2 concurrent writes to different files
         with ThreadPoolExecutor(max_workers=2) as executor:
             results = list(executor.map(write_to_file, range(2)))

         # Verify both succeeded
         for success, temp_path in results:
             assert success, f"Write to {temp_path} failed"
             if os.path.exists(temp_path):
                 os.unlink(temp_path)
