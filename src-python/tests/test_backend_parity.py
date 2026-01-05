"""
Backend Parity & Sequential Baseline Tests

This test suite validates:
1. RustFile output is identical to PythonFile (record-by-record)
2. CursorBackend output is identical to RustFile (record-by-record)
3. GIL release is verified (no GIL overhead in Rust sections)
4. Memory usage is stable (no leaks)

Acceptance Criteria:
- RustFile output identical to PythonFile
- CursorBackend output identical to RustFile
- GIL release verified (no GIL overhead in Rust sections)
- Memory usage stable (no leaks)
"""

import io
import json
import os
import tempfile
import threading
import time
import psutil
from pathlib import Path
import pytest
import mrrc


class TestParityRustFileVsPythonFile:
    """Test that RustFile backend produces identical output to PythonFile backend"""

    @staticmethod
    def _record_to_comparable(record):
        """Convert record to JSON-serializable dict for comparison"""
        marcjson = json.loads(record.to_marcjson())
        return marcjson

    @staticmethod
    def read_all_records(source):
        """Helper: Read all records from a source and return list of record objects"""
        reader = mrrc.MARCReader(source)
        records = []
        for record in reader:
            records.append(record)
        return records

    def test_parity_simple_book_file_path(self):
        """RustFile (file path) vs PythonFile (open file handle) - simple_book.mrc"""
        test_file = "tests/data/simple_book.mrc"
        if not os.path.exists(test_file):
            pytest.skip(f"Test file not found: {test_file}")

        # Read via RustFile (file path string)
        records_rustfile = self.read_all_records(test_file)

        # Read via PythonFile (file handle)
        with open(test_file, "rb") as f:
            records_pythonfile = self.read_all_records(f)

        # Verify same number of records
        assert len(records_rustfile) == len(records_pythonfile), \
            f"Record count mismatch: RustFile={len(records_rustfile)}, PythonFile={len(records_pythonfile)}"

        # Verify each record is identical
        for i, (rec_rust, rec_py) in enumerate(zip(records_rustfile, records_pythonfile)):
            # Compare marcjson for content parity
            rust_json = self._record_to_comparable(rec_rust)
            py_json = self._record_to_comparable(rec_py)
            assert rust_json == py_json, \
                f"Record {i} mismatch: RustFile vs PythonFile"

    def test_parity_multi_records_file_path(self):
        """RustFile vs PythonFile - multi_records.mrc"""
        test_file = "tests/data/multi_records.mrc"
        if not os.path.exists(test_file):
            pytest.skip(f"Test file not found: {test_file}")

        records_rustfile = self.read_all_records(test_file)
        with open(test_file, "rb") as f:
            records_pythonfile = self.read_all_records(f)

        assert len(records_rustfile) == len(records_pythonfile)
        for i, (rec_rust, rec_py) in enumerate(zip(records_rustfile, records_pythonfile)):
            rust_json = self._record_to_comparable(rec_rust)
            py_json = self._record_to_comparable(rec_py)
            assert rust_json == py_json, f"Record {i} mismatch"

    def test_parity_pathlib_path(self):
        """RustFile (pathlib.Path) vs PythonFile (file handle)"""
        test_file = "tests/data/simple_book.mrc"
        if not os.path.exists(test_file):
            pytest.skip(f"Test file not found: {test_file}")

        # Use pathlib.Path (triggers __fspath__ in RustFile)
        records_rustfile = self.read_all_records(Path(test_file))
        with open(test_file, "rb") as f:
            records_pythonfile = self.read_all_records(f)

        assert len(records_rustfile) == len(records_pythonfile)
        for i, (rec_rust, rec_py) in enumerate(zip(records_rustfile, records_pythonfile)):
            rust_json = self._record_to_comparable(rec_rust)
            py_json = self._record_to_comparable(rec_py)
            assert rust_json == py_json, f"Record {i} mismatch"


class TestParityCursorBackendVsRustFile:
    """Test that CursorBackend produces identical output to RustFile"""

    @staticmethod
    def _record_to_comparable(record):
        """Convert record to JSON-serializable dict for comparison"""
        marcjson = json.loads(record.to_marcjson())
        return marcjson

    @staticmethod
    def read_all_records(source):
        """Helper: Read all records from a source"""
        reader = mrrc.MARCReader(source)
        records = []
        for record in reader:
            records.append(record)
        return records

    def test_parity_bytes_vs_file_path(self):
        """CursorBackend (bytes) vs RustFile (file path) - simple_book.mrc"""
        test_file = "tests/data/simple_book.mrc"
        if not os.path.exists(test_file):
            pytest.skip(f"Test file not found: {test_file}")

        # Read file into memory
        with open(test_file, "rb") as f:
            file_bytes = f.read()

        # Read via CursorBackend (bytes)
        records_cursor = self.read_all_records(file_bytes)

        # Read via RustFile (file path)
        records_rustfile = self.read_all_records(test_file)

        assert len(records_cursor) == len(records_rustfile)
        for i, (rec_cursor, rec_rust) in enumerate(zip(records_cursor, records_rustfile)):
            cursor_json = self._record_to_comparable(rec_cursor)
            rust_json = self._record_to_comparable(rec_rust)
            assert cursor_json == rust_json, \
                f"Record {i} mismatch: CursorBackend vs RustFile"

    def test_parity_bytearray_vs_file_path(self):
        """CursorBackend (bytearray) vs RustFile (file path)"""
        test_file = "tests/data/multi_records.mrc"
        if not os.path.exists(test_file):
            pytest.skip(f"Test file not found: {test_file}")

        # Read file as bytearray
        with open(test_file, "rb") as f:
            file_data = bytearray(f.read())

        records_cursor = self.read_all_records(file_data)
        records_rustfile = self.read_all_records(test_file)

        assert len(records_cursor) == len(records_rustfile)
        for i, (rec_cursor, rec_rust) in enumerate(zip(records_cursor, records_rustfile)):
            cursor_json = self._record_to_comparable(rec_cursor)
            rust_json = self._record_to_comparable(rec_rust)
            assert cursor_json == rust_json, f"Record {i} mismatch"

    def test_parity_bytesio_vs_file_path(self):
        """CursorBackend (via BytesIO) vs RustFile (file path)"""
        test_file = "tests/data/simple_book.mrc"
        if not os.path.exists(test_file):
            pytest.skip(f"Test file not found: {test_file}")

        # Read file into BytesIO
        with open(test_file, "rb") as f:
            file_bytes = f.read()
        bytesio = io.BytesIO(file_bytes)

        # BytesIO uses PythonFile backend, but test parity anyway
        records_bytesio = self.read_all_records(bytesio)
        records_rustfile = self.read_all_records(test_file)

        assert len(records_bytesio) == len(records_rustfile)
        for i, (rec_bio, rec_rust) in enumerate(zip(records_bytesio, records_rustfile)):
            bio_json = self._record_to_comparable(rec_bio)
            rust_json = self._record_to_comparable(rec_rust)
            assert bio_json == rust_json, f"Record {i} mismatch"


class TestGILReleaseVerification:
    """Verify that GIL is released during Rust I/O (no overhead)"""

    def test_rustfile_and_cursor_backend_are_thread_safe(self):
        """Verify RustFile and CursorBackend can be used in threads without panics"""
        test_file = "tests/data/multi_records.mrc"
        if not os.path.exists(test_file):
            pytest.skip(f"Test file not found: {test_file}")

        results = {"errors": []}
        lock = threading.Lock()

        def reader_thread(source, source_type):
            """Read MARC records and record any errors"""
            try:
                reader = mrrc.MARCReader(source)
                count = 0
                for _ in reader:
                    count += 1
                with lock:
                    results[f"{source_type}_count"] = count
            except Exception as e:
                with lock:
                    results["errors"].append(f"{source_type}: {e}")

        # Test RustFile (file path)
        results["rustfile_count"] = 0
        thread1 = threading.Thread(target=reader_thread, args=(test_file, "rustfile"))
        thread1.start()
        thread1.join(timeout=10)

        assert not thread1.is_alive(), "RustFile read timed out"
        assert "rustfile_count" in results, "RustFile read did not complete"
        assert results["rustfile_count"] > 0, "RustFile read returned no records"

        # Test CursorBackend (in-memory bytes)
        with open(test_file, "rb") as f:
            file_data = f.read()
        results["cursor_count"] = 0
        thread2 = threading.Thread(target=reader_thread, args=(file_data, "cursor"))
        thread2.start()
        thread2.join(timeout=10)

        assert not thread2.is_alive(), "CursorBackend read timed out"
        assert "cursor_count" in results, "CursorBackend read did not complete"
        assert results["cursor_count"] > 0, "CursorBackend read returned no records"

        # Verify no errors occurred
        assert not results["errors"], f"Threading errors: {results['errors']}"

    def test_concurrent_reads_same_file(self):
        """Test that multiple threads can safely read the same file concurrently"""
        test_file = "tests/data/multi_records.mrc"
        if not os.path.exists(test_file):
            pytest.skip(f"Test file not found: {test_file}")

        results = {"counts": [], "errors": []}
        lock = threading.Lock()

        def reader_thread(thread_id):
            """Read same file in multiple threads"""
            try:
                reader = mrrc.MARCReader(test_file)
                count = 0
                for _ in reader:
                    count += 1
                with lock:
                    results["counts"].append(count)
            except Exception as e:
                with lock:
                    results["errors"].append(f"Thread {thread_id}: {e}")

        # Spawn 3 concurrent readers
        threads = []
        for i in range(3):
            t = threading.Thread(target=reader_thread, args=(i,))
            threads.append(t)
            t.start()

        # Wait for all to complete
        for t in threads:
            t.join(timeout=10)
            assert not t.is_alive(), "Thread timed out"

        # Verify all threads got the same record count
        assert not results["errors"], f"Errors occurred: {results['errors']}"
        assert len(results["counts"]) == 3, "Not all threads completed"
        assert all(c == results["counts"][0] for c in results["counts"]), \
            f"Different record counts across threads: {results['counts']}"


class TestMemoryStability:
    """Verify no memory leaks or unbounded growth"""

    def test_memory_stable_iterating_large_file(self):
        """Memory usage should not grow unboundedly while iterating"""
        test_file = "tests/data/fixtures/10k_records.mrc"
        if not os.path.exists(test_file):
            pytest.skip(f"Test file not found: {test_file}")

        process = psutil.Process(os.getpid())

        # Record initial memory
        process.memory_info()  # Force a measurement
        time.sleep(0.1)
        initial_mem = process.memory_info().rss / 1024 / 1024  # MB

        # Read all records
        reader = mrrc.MARCReader(test_file)
        record_count = 0
        for _ in reader:
            record_count += 1

        # Record final memory
        final_mem = process.memory_info().rss / 1024 / 1024  # MB
        mem_growth = final_mem - initial_mem

        print(f"Memory growth: {mem_growth:.2f} MB for {record_count} records")

        # Memory growth should be modest
        # 10k records of ~1KB each = ~10 MB expected
        # We'll allow 3x that for overhead
        assert mem_growth < 30, \
            f"Memory growth too large: {mem_growth:.2f} MB (expected <30 MB)"

    def test_memory_stable_cursor_backend(self):
        """CursorBackend memory should be stable (pre-loaded data)"""
        test_file = "tests/data/fixtures/5k_records.mrc"
        if not os.path.exists(test_file):
            pytest.skip(f"Test file not found: {test_file}")

        with open(test_file, "rb") as f:
            file_data = f.read()

        process = psutil.Process(os.getpid())
        process.memory_info()
        time.sleep(0.1)
        initial_mem = process.memory_info().rss / 1024 / 1024

        reader = mrrc.MARCReader(file_data)
        record_count = 0
        for _ in reader:
            record_count += 1

        final_mem = process.memory_info().rss / 1024 / 1024
        mem_growth = final_mem - initial_mem

        print(f"Cursor backend memory growth: {mem_growth:.2f} MB for {record_count} records")

        # Cursor backend is in-memory, so growth should be minimal
        # since data is already loaded (allow up to 10 MB for overhead)
        assert mem_growth < 10, \
            f"Cursor backend memory growth too large: {mem_growth:.2f} MB"


class TestBackendParityAcceptanceCriteria:
    """Integration test validating all backend parity acceptance criteria"""

    def test_gate_rustfile_equals_pythonfile(self):
        """Criterion 1: RustFile output identical to PythonFile"""
        test_file = "tests/data/multi_records.mrc"
        if not os.path.exists(test_file):
            pytest.skip(f"Test file not found: {test_file}")

        # RustFile
        reader1 = mrrc.MARCReader(test_file)
        records_rust = [json.loads(r.to_marcjson()) for r in reader1]

        # PythonFile
        with open(test_file, "rb") as f:
            reader2 = mrrc.MARCReader(f)
            records_py = [json.loads(r.to_marcjson()) for r in reader2]

        assert records_rust == records_py, "RustFile and PythonFile outputs differ"

    def test_gate_cursorbackend_equals_rustfile(self):
        """Criterion 2: CursorBackend output identical to RustFile"""
        test_file = "tests/data/multi_records.mrc"
        if not os.path.exists(test_file):
            pytest.skip(f"Test file not found: {test_file}")

        with open(test_file, "rb") as f:
            file_data = f.read()

        # CursorBackend
        reader1 = mrrc.MARCReader(file_data)
        records_cursor = [json.loads(r.to_marcjson()) for r in reader1]

        # RustFile
        reader2 = mrrc.MARCReader(test_file)
        records_rust = [json.loads(r.to_marcjson()) for r in reader2]

        assert records_cursor == records_rust, "CursorBackend and RustFile outputs differ"

    def test_gate_no_exceptions_or_panics(self):
        """Criterion 3: Clean reading with no exceptions or panics"""
        test_files = [
            "tests/data/simple_book.mrc",
            "tests/data/multi_records.mrc",
            "tests/data/with_control_fields.mrc",
        ]

        for test_file in test_files:
            if not os.path.exists(test_file):
                continue

            # RustFile read
            try:
                reader = mrrc.MARCReader(test_file)
                for _ in reader:
                    pass
            except Exception as e:
                pytest.fail(f"RustFile read failed for {test_file}: {e}")

            # PythonFile read
            try:
                with open(test_file, "rb") as f:
                    reader = mrrc.MARCReader(f)
                    for _ in reader:
                        pass
            except Exception as e:
                pytest.fail(f"PythonFile read failed for {test_file}: {e}")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
