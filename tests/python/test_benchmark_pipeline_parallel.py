"""
Benchmarks for ProducerConsumerPipeline - parallel infrastructure.

These benchmarks test the opt-in parallel API to achieve multi-threaded
performance gains without modifying the standard MARCReader iteration API.

The ProducerConsumerPipeline is the correct way to benchmark pymrrc multi-threaded
performance. It uses:
- Background producer thread reading file + scanning boundaries
- Rayon parallel parser pool for batch processing
- Bounded channel (1000 records) for backpressure
- Consumer thread iterating results

Expected performance: 2.0x speedup (2 threads), 3.74x (4 threads)
"""

import pytest
import tempfile
import shutil
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
from mrrc import ProducerConsumerPipeline


class TestProducerConsumerPipelineBasic:
    """Basic tests for ProducerConsumerPipeline API."""

    @pytest.mark.benchmark
    def test_pipeline_sequential_1x_10k(self, benchmark):
        """Baseline: Pipeline reading 1x 10k records."""
        fixture_path = "tests/data/fixtures/10k_records.mrc"

        def read_pipeline():
            pipeline = ProducerConsumerPipeline.from_file(fixture_path)
            count = 0
            for record in pipeline:
                count += 1
            return count

        result = benchmark(read_pipeline)
        assert result == 10000

    @pytest.mark.benchmark
    def test_pipeline_sequential_4x_10k(self, benchmark):
        """Baseline: Sequential pipeline reading of 4x 10k from disk."""
        fixture_path = "tests/data/fixtures/10k_records.mrc"

        def read_pipelines():
            total = 0
            for _ in range(4):
                pipeline = ProducerConsumerPipeline.from_file(fixture_path)
                for record in pipeline:
                    total += 1
            return total

        result = benchmark(read_pipelines)
        assert result == 40000

    @pytest.mark.benchmark
    def test_pipeline_parallel_2x_10k_threaded(self, benchmark):
        """Parallel: 2 threads, each with own ProducerConsumerPipeline.

        This is the CORRECT way to test multi-threaded performance:
        - Each thread gets its own pipeline (each spawns background producer)
        - Expected: 2.0x speedup vs sequential
        """
        fixture_path = "tests/data/fixtures/10k_records.mrc"

        def read_parallel():
            def read_pipeline(_):
                pipeline = ProducerConsumerPipeline.from_file(fixture_path)
                count = 0
                for record in pipeline:
                    count += 1
                return count

            with ThreadPoolExecutor(max_workers=2) as executor:
                results = list(executor.map(read_pipeline, range(2)))
            return sum(results)

        result = benchmark(read_parallel)
        assert result == 20000

    @pytest.mark.benchmark
    def test_pipeline_parallel_4x_10k_threaded(self, benchmark):
        """Parallel: 4 threads, each with own ProducerConsumerPipeline.

        This is the test that demonstrates parallel speedup with the pipeline.
        - 4 threads, each with independent pipeline
        - Each pipeline has its own background producer + rayon parser pool
        - Expected: 3.74x speedup vs sequential (from RESULTS.md)
        """
        fixture_path = "tests/data/fixtures/10k_records.mrc"

        def read_parallel():
            def read_pipeline(_):
                pipeline = ProducerConsumerPipeline.from_file(fixture_path)
                count = 0
                for record in pipeline:
                    count += 1
                return count

            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(read_pipeline, range(4)))
            return sum(results)

        result = benchmark(read_parallel)
        assert result == 40000


class TestProducerConsumerPipelineWithExtraction:
    """Realistic workloads with field extraction."""

    @pytest.mark.benchmark
    def test_pipeline_sequential_extraction_4x_10k(self, benchmark):
        """Baseline: Sequential pipeline with field extraction (4x 10k)."""
        fixture_path = "tests/data/fixtures/10k_records.mrc"

        def read_with_extraction():
            total = 0
            for _ in range(4):
                pipeline = ProducerConsumerPipeline.from_file(fixture_path)
                for record in pipeline:
                    _ = record.title()
                    _ = record.get_fields("100")
                    total += 1
            return total

        result = benchmark(read_with_extraction)
        assert result == 40000

    @pytest.mark.benchmark
    def test_pipeline_parallel_extraction_4x_10k_threaded(self, benchmark):
        """Parallel: 4 threads with field extraction (realistic workload).

        Pipeline + extraction with multi-threading:
        - Each thread reads + extracts via pipeline
        - Expected: 3.74x speedup
        """
        fixture_path = "tests/data/fixtures/10k_records.mrc"

        def read_with_extraction():
            def process_pipeline(_):
                pipeline = ProducerConsumerPipeline.from_file(fixture_path)
                count = 0
                for record in pipeline:
                    _ = record.title()
                    _ = record.get_fields("100")
                    count += 1
                return count

            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(process_pipeline, range(4)))
            return sum(results)

        result = benchmark(read_with_extraction)
        assert result == 40000


class TestProducerConsumerPipelineMultiFile:
    """Multi-file processing scenarios (typical use case)."""

    @pytest.mark.benchmark
    def test_process_4_files_sequential(self, benchmark):
        """Process 4 files sequentially using ProducerConsumerPipeline."""
        fixture_path = "tests/data/fixtures/10k_records.mrc"

        def process_sequential():
            total = 0
            for _ in range(4):
                pipeline = ProducerConsumerPipeline.from_file(fixture_path)
                for record in pipeline:
                    total += 1
            return total

        result = benchmark(process_sequential)
        assert result == 40000

    @pytest.mark.benchmark
    def test_process_4_files_parallel_4_threads(self, benchmark):
        """Process 4 files in parallel with 4 threads (optimal threading).

        This demonstrates the parallel pipeline infrastructure:
        - 4 threads, 4 files
        - Each thread: ProducerConsumerPipeline.from_file(filepath)
        - Each pipeline spawns its own background producer + parser pool
        - Expected: 3.74x speedup vs sequential
        """
        fixture_path = "tests/data/fixtures/10k_records.mrc"

        def process_parallel():
            def process_file(_):
                pipeline = ProducerConsumerPipeline.from_file(fixture_path)
                count = 0
                for record in pipeline:
                    count += 1
                return count

            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(process_file, range(4)))
            return sum(results)

        result = benchmark(process_parallel)
        assert result == 40000

    @pytest.mark.benchmark
    def test_process_8_files_parallel_4_threads(self, benchmark):
        """Process 8 files with 4 threads (oversubscription test).

        Tests thread pool efficiency with more files than threads.
        Expected: ~2.0x speedup (limited by 4 threads, not 8 files)
        """
        fixture_path = "tests/data/fixtures/10k_records.mrc"

        def process_parallel():
            def process_file(_):
                pipeline = ProducerConsumerPipeline.from_file(fixture_path)
                count = 0
                for record in pipeline:
                    count += 1
                return count

            with ThreadPoolExecutor(max_workers=4) as executor:
                results = list(executor.map(process_file, range(8)))
            return sum(results)

        result = benchmark(process_parallel)
        assert result == 80000
