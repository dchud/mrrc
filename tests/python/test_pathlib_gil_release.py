"""
Deterministic tests for pathlib.Path backend routing and GIL release.

Verifies that pathlib.Path input routes to the RustFile backend (same as str),
which enables GIL release during parsing. Replaces the previous timing-based
proxy that was flaky on CI runners.

See: src-python/src/backend.rs for the __fspath__() detection logic.
"""

import io
import tempfile
import shutil
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor

import pytest
from mrrc import MARCReader


class TestBackendTypeRouting:
    """Verify that each input type routes to the correct backend."""

    @pytest.fixture
    def mrc_file(self, fixture_1k, tmp_path):
        """Write fixture to a temp .mrc file, return its path."""
        path = tmp_path / "test.mrc"
        path.write_bytes(fixture_1k)
        return path

    def test_str_path_uses_rust_file(self, mrc_file):
        reader = MARCReader(str(mrc_file))
        assert reader.backend_type == "rust_file"

    def test_pathlib_path_uses_rust_file(self, mrc_file):
        reader = MARCReader(mrc_file)  # pathlib.Path
        assert reader.backend_type == "rust_file"

    def test_bytes_uses_cursor(self, fixture_1k):
        reader = MARCReader(fixture_1k)
        assert reader.backend_type == "cursor"

    def test_file_object_uses_python_file(self, mrc_file):
        with open(mrc_file, "rb") as f:
            reader = MARCReader(f)
            assert reader.backend_type == "python_file"

    def test_bytesio_uses_python_file(self, fixture_1k):
        reader = MARCReader(io.BytesIO(fixture_1k))
        assert reader.backend_type == "python_file"

    def test_pathlib_and_str_same_backend(self, mrc_file):
        """The actual invariant: pathlib.Path and str produce the same backend."""
        str_reader = MARCReader(str(mrc_file))
        path_reader = MARCReader(mrc_file)
        assert str_reader.backend_type == path_reader.backend_type == "rust_file"


class TestPathlibThreadingCorrectness:
    """Verify pathlib.Path works correctly under threading (no speedup assertions)."""

    @pytest.fixture
    def temp_pathlib_fixtures(self, fixture_1k):
        """Create temporary files, returning pathlib.Path objects."""
        tmpdir = Path(tempfile.mkdtemp())
        file_paths = []
        try:
            for i in range(4):
                filepath = tmpdir / f"pathlib_test_{i}.mrc"
                filepath.write_bytes(fixture_1k)
                file_paths.append(filepath)
            yield file_paths
        finally:
            shutil.rmtree(tmpdir)

    def test_pathlib_sequential_reads_correct(self, temp_pathlib_fixtures):
        """Baseline: sequential reading with pathlib.Path produces correct counts."""
        total = 0
        for path in temp_pathlib_fixtures[:2]:
            assert isinstance(path, Path)
            for _ in MARCReader(path):
                total += 1
        assert total == 2000

    def test_pathlib_parallel_reads_correct(self, temp_pathlib_fixtures):
        """Parallel reading with pathlib.Path completes without error."""
        def read_file(path):
            count = 0
            for _ in MARCReader(path):
                count += 1
            return count

        with ThreadPoolExecutor(max_workers=4) as executor:
            results = list(executor.map(read_file, temp_pathlib_fixtures))

        assert all(r == 1000 for r in results)
        assert sum(results) == 4000
