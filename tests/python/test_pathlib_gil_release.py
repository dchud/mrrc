"""
Test to verify pathlib.Path achieves GIL release during parsing.

If pathlib.Path is handled correctly (routed to RustFile backend via __fspath__),
threading will achieve speedup. If it's handled as a file-like object
(routed to PythonFile backend), threading will show no speedup.

See: src-python/src/backend.rs lines 78-106 for the __fspath__() detection logic.
"""

import pytest
import tempfile
import shutil
import time
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
from mrrc import MARCReader


class TestPathlibGilRelease:
    """Verify pathlib.Path input achieves GIL release during parsing."""

    @pytest.fixture
    def temp_pathlib_fixtures(self, fixture_10k):
        """Create temporary files, returning pathlib.Path objects."""
        tmpdir = Path(tempfile.mkdtemp())
        file_paths = []
        try:
            for i in range(4):
                filepath = tmpdir / f"pathlib_test_{i}.mrc"
                filepath.write_bytes(fixture_10k)
                file_paths.append(filepath)  # Return Path objects, not strings
            yield file_paths
        finally:
            shutil.rmtree(tmpdir)

    def test_pathlib_path_sequential_baseline(self, temp_pathlib_fixtures):
        """Baseline: sequential reading with pathlib.Path."""
        paths = temp_pathlib_fixtures[:2]

        total = 0
        for path in paths:
            assert isinstance(path, Path), "Fixture should return Path objects"
            reader = MARCReader(path)  # Pass Path object directly
            while record := reader.read_record():
                total += 1

        assert total == 20000

    def test_pathlib_path_threading_equivalent_to_str(self, temp_pathlib_fixtures):
        """
        Verify pathlib.Path achieves the SAME threading behavior as str paths.

        This is the critical test: if pathlib.Path routes to RustFile backend
        (like str), both should show identical threading characteristics.

        We compare pathlib.Path speedup vs str speedup - they should match
        within tolerance, proving both use the same code path.
        """
        paths = temp_pathlib_fixtures
        str_paths = [str(p) for p in paths]

        def read_with_path(path: Path) -> int:
            reader = MARCReader(path)
            count = 0
            while record := reader.read_record():
                count += 1
            return count

        def read_with_str(path: str) -> int:
            reader = MARCReader(path)
            count = 0
            while record := reader.read_record():
                count += 1
            return count

        # Warm up to stabilize timings
        _ = read_with_path(paths[0])
        _ = read_with_str(str_paths[0])

        # Measure pathlib.Path: sequential vs parallel
        start = time.perf_counter()
        _ = sum(read_with_path(p) for p in paths)
        path_sequential = time.perf_counter() - start

        start = time.perf_counter()
        with ThreadPoolExecutor(max_workers=4) as executor:
            _ = list(executor.map(read_with_path, paths))
        path_parallel = time.perf_counter() - start

        path_speedup = path_sequential / path_parallel

        # Measure str: sequential vs parallel
        start = time.perf_counter()
        _ = sum(read_with_str(p) for p in str_paths)
        str_sequential = time.perf_counter() - start

        start = time.perf_counter()
        with ThreadPoolExecutor(max_workers=4) as executor:
            _ = list(executor.map(read_with_str, str_paths))
        str_parallel = time.perf_counter() - start

        str_speedup = str_sequential / str_parallel

        print(f"\nPathlib.Path vs str Threading Comparison:")
        print(f"  pathlib.Path: {path_sequential:.3f}s seq, {path_parallel:.3f}s par, {path_speedup:.2f}x speedup")
        print(f"  str:          {str_sequential:.3f}s seq, {str_parallel:.3f}s par, {str_speedup:.2f}x speedup")

        # Both should have similar speedup characteristics (within 30%)
        # This proves they use the same backend and GIL behavior
        speedup_ratio = path_speedup / str_speedup
        print(f"  Speedup ratio (path/str): {speedup_ratio:.2f}x")

        assert 0.7 < speedup_ratio < 1.3, (
            f"pathlib.Path and str have different threading behavior! "
            f"pathlib.Path speedup: {path_speedup:.2f}x, str speedup: {str_speedup:.2f}x. "
            f"This suggests they're using different backends."
        )
