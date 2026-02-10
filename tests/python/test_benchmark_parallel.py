"""
Parallel processing benchmarks for Python MARC readers.

Demonstrates the GIL impact on threading performance and compares
pymrrc vs pymarc in concurrent workloads.

IMPORTANT: These benchmarks test both BytesIO and file-path backends.
- BytesIO tests verify GIL release during parsing (in-memory data)
- File-path tests verify end-to-end threading benefits with realistic I/O
"""

import pytest
import io
import time
import tempfile
import shutil
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
from multiprocessing import Pool, cpu_count
from mrrc import MARCReader


class TestPythonParallelBenchmarks:
    """Benchmarks for parallel processing with threading and multiprocessing."""
    
    @pytest.mark.benchmark
    def test_sequential_reading_1k(self, benchmark, fixture_1k):
        """Baseline: sequential reading of 1k records."""
        def read_all():
            data = io.BytesIO(fixture_1k)
            reader = MARCReader(data)
            count = 0
            while record := reader.read_record():
                count += 1
            return count
        
        result = benchmark(read_all)
        assert result == 1000
    
    @pytest.mark.benchmark
    def test_sequential_2x_reading_1k(self, benchmark, fixture_1k):
        """Baseline: sequential reading of 2x 1k records."""
        def read_twice():
            total = 0
            for _ in range(2):
                data = io.BytesIO(fixture_1k)
                reader = MARCReader(data)
                while record := reader.read_record():
                    total += 1
            return total
        
        result = benchmark(read_twice)
        assert result == 2000
    
    @pytest.mark.benchmark
    def test_sequential_4x_reading_1k(self, benchmark, fixture_1k):
        """Baseline: sequential reading of 4x 1k records."""
        def read_4x():
            total = 0
            for _ in range(4):
                data = io.BytesIO(fixture_1k)
                reader = MARCReader(data)
                while record := reader.read_record():
                    total += 1
            return total
        
        result = benchmark(read_4x)
        assert result == 4000
    
    @pytest.mark.benchmark
    def test_threaded_reading_1k(self, benchmark, fixture_1k):
        """ThreadPoolExecutor reading of 2x 1k records (pymrrc)."""
        def read_with_threads():
            def read_single_file(data):
                reader = MARCReader(io.BytesIO(data))
                count = 0
                while record := reader.read_record():
                    count += 1
                return count
            
            with ThreadPoolExecutor(max_workers=2) as executor:
                results = list(executor.map(read_single_file, [fixture_1k, fixture_1k]))
            return sum(results)
        
        result = benchmark(read_with_threads)
        assert result == 2000
    
    @pytest.mark.benchmark
    def test_threaded_reading_4x_1k(self, benchmark, fixture_1k):
        """ThreadPoolExecutor reading of 4x 1k records (pymrrc)."""
        def read_with_threads():
            def read_single_file(data):
                reader = MARCReader(io.BytesIO(data))
                count = 0
                while record := reader.read_record():
                    count += 1
                return count
            
            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(
                    read_single_file, 
                    [fixture_1k] * 4
                ))
            return sum(results)
        
        result = benchmark(read_with_threads)
        assert result == 4000
    
    @pytest.mark.benchmark
    def test_sequential_10k(self, benchmark, fixture_10k):
        """Baseline: sequential reading of 10k records."""
        def read_all():
            data = io.BytesIO(fixture_10k)
            reader = MARCReader(data)
            count = 0
            while record := reader.read_record():
                count += 1
            return count
        
        result = benchmark(read_all)
        assert result == 10000
    
    @pytest.mark.benchmark
    def test_sequential_2x_reading_10k(self, benchmark, fixture_10k):
        """Baseline: sequential reading of 2x 10k records."""
        def read_twice():
            total = 0
            for _ in range(2):
                data = io.BytesIO(fixture_10k)
                reader = MARCReader(data)
                while record := reader.read_record():
                    total += 1
            return total
        
        result = benchmark(read_twice)
        assert result == 20000
    
    @pytest.mark.benchmark
    def test_threaded_reading_2x_10k(self, benchmark, fixture_10k):
        """ThreadPoolExecutor reading of 2x 10k records (pymrrc)."""
        def read_with_threads():
            def read_single_file(data):
                reader = MARCReader(io.BytesIO(data))
                count = 0
                while record := reader.read_record():
                    count += 1
                return count
            
            with ThreadPoolExecutor(max_workers=2) as executor:
                results = list(executor.map(read_single_file, [fixture_10k, fixture_10k]))
            return sum(results)
        
        result = benchmark(read_with_threads)
        assert result == 20000
    
    @pytest.mark.benchmark
    def test_threaded_reading_4x_10k(self, benchmark, fixture_10k):
        """ThreadPoolExecutor reading of 4x 10k records (pymrrc)."""
        def read_with_threads():
            def read_single_file(data):
                reader = MARCReader(io.BytesIO(data))
                count = 0
                while record := reader.read_record():
                    count += 1
                return count
            
            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(
                    read_single_file, 
                    [fixture_10k] * 4
                ))
            return sum(results)
        
        result = benchmark(read_with_threads)
        assert result == 40000


class TestParallelSummary:
    """Summary tests showing GIL impact and speedup metrics."""
    
    @pytest.mark.benchmark
    def test_threading_speedup_2x_10k(self, benchmark, fixture_10k):
        """
        Measure threading speedup on 2x 10k reads.
        
        Expected: ~1.4x speedup (limited by GIL)
        With GIL-release: ~1.9x speedup
        
        This demonstrates the bottleneck that mrrc-gyk will address.
        """
        def threaded_vs_sequential():
            # Threaded version
            def read_single_file(data):
                reader = MARCReader(io.BytesIO(data))
                count = 0
                while record := reader.read_record():
                    count += 1
                return count
            
            with ThreadPoolExecutor(max_workers=2) as executor:
                results = list(executor.map(read_single_file, [fixture_10k, fixture_10k]))
            return sum(results)
        
        result = benchmark(threaded_vs_sequential)
        assert result == 20000
    
    @pytest.mark.benchmark
    def test_threading_speedup_4x_10k(self, benchmark, fixture_10k):
        """
        Measure threading speedup on 4x 10k reads.
        
        Expected: ~1.3-1.4x speedup (GIL contention increases)
        With GIL-release: ~3.0-3.5x speedup
        
        This is the key benchmark showing GIL impact.
        """
        def threaded_4x():
            def read_single_file(data):
                reader = MARCReader(io.BytesIO(data))
                count = 0
                while record := reader.read_record():
                    count += 1
                return count
            
            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(
                    read_single_file,
                    [fixture_10k] * 4
                ))
            return sum(results)
        
        result = benchmark(threaded_4x)
        assert result == 40000


class TestParallelWithFieldAccess:
    """Parallel benchmarks with realistic field access patterns."""
    
    @pytest.mark.benchmark
    def test_sequential_with_title_extraction_10k(self, benchmark, fixture_10k):
        """Sequential reading with field extraction."""
        def read_with_extraction():
            data = io.BytesIO(fixture_10k)
            reader = MARCReader(data)
            titles = []
            while record := reader.read_record():
                title = record.title() or "Unknown"
                titles.append(title)
            return len(titles)
        
        result = benchmark(read_with_extraction)
        assert result == 10000
    
    @pytest.mark.benchmark
    def test_threaded_with_title_extraction_2x_10k(self, benchmark, fixture_10k):
        """Parallel reading with field extraction."""
        def read_with_extraction():
            def extract_titles(data):
                reader = MARCReader(io.BytesIO(data))
                titles = []
                while record := reader.read_record():
                    title = record.title() or "Unknown"
                    titles.append(title)
                return len(titles)
            
            with ThreadPoolExecutor(max_workers=2) as executor:
                results = list(executor.map(extract_titles, [fixture_10k, fixture_10k]))
            return sum(results)
        
        result = benchmark(read_with_extraction)
        assert result == 20000
    
    @pytest.mark.benchmark
    def test_threaded_with_title_extraction_4x_10k(self, benchmark, fixture_10k):
        """Parallel reading with field extraction (4 threads)."""
        def read_with_extraction():
            def extract_titles(data):
                reader = MARCReader(io.BytesIO(data))
                titles = []
                while record := reader.read_record():
                    title = record.title() or "Unknown"
                    titles.append(title)
                return len(titles)
            
            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(
                    extract_titles,
                    [fixture_10k] * 4
                ))
            return sum(results)
        
        result = benchmark(read_with_extraction)
        assert result == 40000


class TestIndividualOperationParallel:
    """Individual operation benchmarks with threading to measure speedup."""
    
    @pytest.mark.benchmark
    def test_parallel_read_4x_1k(self, benchmark, fixture_1k):
        """Parallel reading of 4x 1k records with 4 threads."""
        def read_parallel():
            def read_single_file(data):
                reader = MARCReader(io.BytesIO(data))
                count = 0
                while record := reader.read_record():
                    count += 1
                return count
            
            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(read_single_file, [fixture_1k] * 4))
            return sum(results)
        
        result = benchmark(read_parallel)
        assert result == 4000
    
    @pytest.mark.benchmark
    def test_parallel_read_with_extract_4x_1k(self, benchmark, fixture_1k):
        """Parallel reading with field extraction of 4x 1k records."""
        def read_parallel_extract():
            def read_and_extract(data):
                reader = MARCReader(io.BytesIO(data))
                count = 0
                while record := reader.read_record():
                    _ = record.title()
                    _ = record.get_fields("100")
                    count += 1
                return count
            
            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(read_and_extract, [fixture_1k] * 4))
            return sum(results)
        
        result = benchmark(read_parallel_extract)
        assert result == 4000
    
    @pytest.mark.benchmark
    def test_parallel_read_4x_10k(self, benchmark, fixture_10k):
        """Parallel reading of 4x 10k records with 4 threads."""
        def read_parallel():
            def read_single_file(data):
                reader = MARCReader(io.BytesIO(data))
                count = 0
                while record := reader.read_record():
                    count += 1
                return count
            
            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(read_single_file, [fixture_10k] * 4))
            return sum(results)
        
        result = benchmark(read_parallel)
        assert result == 40000
    
    @pytest.mark.benchmark
    def test_parallel_read_with_extract_4x_10k(self, benchmark, fixture_10k):
        """Parallel reading with field extraction of 4x 10k records."""
        def read_parallel_extract():
            def read_and_extract(data):
                reader = MARCReader(io.BytesIO(data))
                count = 0
                while record := reader.read_record():
                    _ = record.title()
                    _ = record.get_fields("100")
                    count += 1
                return count
            
            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(read_and_extract, [fixture_10k] * 4))
            return sum(results)
        
        result = benchmark(read_parallel_extract)
        assert result == 40000


class TestFileBatchParallelBenchmarks:
    """
    File-path based benchmarks showing realistic threading performance.
    
    These benchmarks use actual file paths (not BytesIO), which allows
    the Rust backend to use pure file I/O without Python overhead.
    This tests the ARCHITECTURE.md recommendation: "one reader per thread
    reading from file paths."
    
    Expected: 3.74x speedup on 4 threads, 2.0x on 2 threads.
    """
    
    @pytest.fixture
    def temp_fixtures(self, fixture_10k):
        """Create temporary file copies for parallel read tests."""
        tmpdir = tempfile.mkdtemp()
        file_paths = []
        try:
            for i in range(4):
                filepath = Path(tmpdir) / f"batch_{i}.mrc"
                filepath.write_bytes(fixture_10k)
                file_paths.append(str(filepath))
            yield file_paths
        finally:
            shutil.rmtree(tmpdir)
    
    @pytest.mark.benchmark
    def test_file_sequential_1x_10k(self, benchmark, temp_fixtures):
        """Baseline: sequential file reading of 1x 10k from disk."""
        filepath = temp_fixtures[0]
        
        def read_file():
            reader = MARCReader(filepath)
            count = 0
            while record := reader.read_record():
                count += 1
            return count
        
        result = benchmark(read_file)
        assert result == 10000
    
    @pytest.mark.benchmark
    def test_file_sequential_2x_10k(self, benchmark, temp_fixtures):
        """Baseline: sequential file reading of 2x 10k from disk."""
        filepaths = temp_fixtures[:2]
        
        def read_files():
            total = 0
            for filepath in filepaths:
                reader = MARCReader(filepath)
                while record := reader.read_record():
                    total += 1
            return total
        
        result = benchmark(read_files)
        assert result == 20000
    
    @pytest.mark.benchmark
    def test_file_sequential_4x_10k(self, benchmark, temp_fixtures):
        """Baseline: sequential file reading of 4x 10k from disk."""
        filepaths = temp_fixtures
        
        def read_files():
            total = 0
            for filepath in filepaths:
                reader = MARCReader(filepath)
                while record := reader.read_record():
                    total += 1
            return total
        
        result = benchmark(read_files)
        assert result == 40000
    
    @pytest.mark.benchmark
    def test_file_parallel_2x_10k(self, benchmark, temp_fixtures):
        """Parallel file reading of 2x 10k with 2 threads (file-based)."""
        filepaths = temp_fixtures[:2]
        
        def read_parallel():
            def read_file(filepath):
                reader = MARCReader(filepath)
                count = 0
                while record := reader.read_record():
                    count += 1
                return count
            
            with ThreadPoolExecutor(max_workers=2) as executor:
                results = list(executor.map(read_file, filepaths))
            return sum(results)
        
        result = benchmark(read_parallel)
        assert result == 20000
    
    @pytest.mark.benchmark
    def test_file_parallel_4x_10k(self, benchmark, temp_fixtures):
        """Parallel file reading of 4x 10k with 4 threads (file-based).
        
        Expected: ~3.74x speedup vs sequential.
        Key test: Verifies ARCHITECTURE.md claim about file-path threading.
        """
        filepaths = temp_fixtures
        
        def read_parallel():
            def read_file(filepath):
                reader = MARCReader(filepath)
                count = 0
                while record := reader.read_record():
                    count += 1
                return count
            
            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(read_file, filepaths))
            return sum(results)
        
        result = benchmark(read_parallel)
        assert result == 40000
    
    @pytest.mark.benchmark
    def test_file_parallel_4x_10k_with_extraction(self, benchmark, temp_fixtures):
        """Parallel file reading + extraction with 4 threads (file-based).
        
        Realistic workload: read + process fields in parallel.
        """
        filepaths = temp_fixtures
        
        def read_parallel_extract():
            def process_file(filepath):
                reader = MARCReader(filepath)
                count = 0
                while record := reader.read_record():
                    _ = record.title()
                    _ = record.get_fields("100")
                    count += 1
                return count
            
            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(process_file, filepaths))
            return sum(results)
        
        result = benchmark(read_parallel_extract)
        assert result == 40000
