#!/usr/bin/env python3
"""
Benchmark comparison script: mrrc (Rust) vs pymrrc (Python wrapper) vs pymarc.

This script compares performance between:
1. mrrc - Pure Rust MARC library (maximum performance)
2. pymrrc - Rust-backed Python wrapper via PyO3 (this project)
3. pymarc - Pure Python MARC library (baseline)

Rust benchmarks are obtained from Criterion.rs output files.

Run with:
    pip install pymarc
    cargo bench --release  # Generate Criterion.rs results
    python scripts/benchmark_comparison.py
"""

import sys
import time
import json
import re
import subprocess
import os
from pathlib import Path
from io import BytesIO
import statistics

# Add scripts directory to path for local imports
sys.path.insert(0, str(Path(__file__).parent))

try:
    from criterion_extractor import CriterionExtractor
    HAS_CRITERION = True
except ImportError:
    HAS_CRITERION = False

try:
    import mrrc
    HAS_MRRC = True
except ImportError:
    HAS_MRRC = False
    print("Warning: mrrc not installed. Install with: maturin develop", file=sys.stderr)

try:
    from pymarc import MARCReader as PymarcReader
    HAS_PYMARC = True
except ImportError:
    HAS_PYMARC = False
    print("Warning: pymarc not installed. Install with: pip install pymarc", file=sys.stderr)


class Benchmark:
    """Simple benchmarking utility."""
    
    def __init__(self, name, iterations=1):
        self.name = name
        self.iterations = iterations
        self.times = []
    
    def __enter__(self):
        self._start = time.perf_counter()
        return self
    
    def __exit__(self, *args):
        elapsed = time.perf_counter() - self._start
        self.times.append(elapsed)
    
    def run(self, func):
        """Run function multiple times and collect times."""
        for _ in range(self.iterations):
            with self:
                func()
    
    def stats(self):
        """Return benchmark statistics."""
        return {
            'min': min(self.times),
            'max': max(self.times),
            'mean': statistics.mean(self.times),
            'stddev': statistics.stdev(self.times) if len(self.times) > 1 else 0.0,
            'median': statistics.median(self.times),
            'runs': len(self.times),
        }
    
    def report(self):
        """Print benchmark results."""
        stats = self.stats()
        print(f"\n{self.name}:")
        print(f"  Runs:    {stats['runs']}")
        print(f"  Min:     {stats['min']*1000:.2f} ms")
        print(f"  Max:     {stats['max']*1000:.2f} ms")
        print(f"  Mean:    {stats['mean']*1000:.2f} ms")
        print(f"  Median:  {stats['median']*1000:.2f} ms")
        print(f"  StdDev:  {stats['stddev']*1000:.2f} ms")


def load_fixture(fixture_path, size='1k'):
    """Load a test fixture."""
    path = Path(fixture_path) / f'{size}_records.mrc'
    if not path.exists():
        raise FileNotFoundError(f"Fixture not found: {path}")
    
    with open(path, 'rb') as f:
        return f.read()


def benchmark_pymarc_read(data, iterations=3):
    """Benchmark pymarc reading."""
    if not HAS_PYMARC:
        return None
    
    bench = Benchmark("pymarc - Read records", iterations)
    
    def read():
        reader = PymarcReader(BytesIO(data))
        count = 0
        for record in reader:
            count += 1
        return count
    
    bench.run(read)
    return bench


def benchmark_mrrc_read(data, iterations=3):
    """Benchmark mrrc reading."""
    if not HAS_MRRC:
        return None
    
    bench = Benchmark("mrrc - Read records", iterations)
    
    def read():
        reader = mrrc.MARCReader(BytesIO(data))
        count = 0
        while record := reader.read_record():
            count += 1
        return count
    
    bench.run(read)
    return bench


def benchmark_pymarc_extract_titles(data, iterations=3):
    """Benchmark pymarc reading with field extraction."""
    if not HAS_PYMARC:
        return None
    
    bench = Benchmark("pymarc - Read + extract titles", iterations)
    
    def read_and_extract():
        reader = PymarcReader(BytesIO(data))
        titles = []
        for record in reader:
            try:
                title = record['245']['a'] if '245' in record else 'Unknown'
            except:
                title = 'Unknown'
            titles.append(title)
        return titles
    
    bench.run(read_and_extract)
    return bench


def benchmark_mrrc_extract_titles(data, iterations=3):
    """Benchmark mrrc reading with field extraction."""
    if not HAS_MRRC:
        return None
    
    bench = Benchmark("mrrc - Read + extract titles", iterations)
    
    def read_and_extract():
        reader = mrrc.MARCReader(BytesIO(data))
        titles = []
        while record := reader.read_record():
            title = record.title() or 'Unknown'
            titles.append(title)
        return titles
    
    bench.run(read_and_extract)
    return bench


def benchmark_pymarc_roundtrip(data, iterations=3):
    """Benchmark pymarc read and write round-trip."""
    if not HAS_PYMARC:
        return None
    
    bench = Benchmark("pymarc - Read + write roundtrip", iterations)
    
    def roundtrip():
        reader = PymarcReader(BytesIO(data))
        records = list(reader)
        output = BytesIO()
        for record in records:
            output.write(record.as_marc())
        return output.getvalue()
    
    bench.run(roundtrip)
    return bench


def benchmark_mrrc_roundtrip(data, iterations=3):
    """Benchmark mrrc read and write round-trip."""
    if not HAS_MRRC:
        return None
    
    bench = Benchmark("mrrc - Read + write roundtrip", iterations)
    
    def roundtrip():
        reader = mrrc.MARCReader(BytesIO(data))
        records = []
        while record := reader.read_record():
            records.append(record)
        
        output = BytesIO()
        writer = mrrc.MARCWriter(output)
        for record in records:
            writer.write_record(record)
        
        return output.getvalue()
    
    bench.run(roundtrip)
    return bench


def benchmark_mrrc_json_serialization(data, iterations=3):
    """Benchmark mrrc JSON serialization."""
    if not HAS_MRRC:
        return None
    
    bench = Benchmark("mrrc - JSON serialization", iterations)
    
    def serialize():
        reader = mrrc.MARCReader(BytesIO(data))
        outputs = []
        while record := reader.read_record():
            outputs.append(record.to_json())
        return outputs
    
    bench.run(serialize)
    return bench


def benchmark_mrrc_xml_serialization(data, iterations=3):
    """Benchmark mrrc XML serialization."""
    if not HAS_MRRC:
        return None
    
    bench = Benchmark("mrrc - XML serialization", iterations)
    
    def serialize():
        reader = mrrc.MARCReader(BytesIO(data))
        outputs = []
        while record := reader.read_record():
            outputs.append(record.to_xml())
        return outputs
    
    bench.run(serialize)
    return bench


def extract_rust_benchmark(bench_name):
    """
    Extract Rust benchmark result from cached Criterion.rs output.
    Returns mean time in seconds, or None if not found.
    
    First tries to use cached Criterion results (fast, no compilation needed).
    Falls back to running cargo bench if cache not available.
    """
    # Try cached results first
    if HAS_CRITERION:
        try:
            extractor = CriterionExtractor(Path(__file__).parent.parent)
            cached_result = extractor.get_benchmark_result(bench_name)
            if cached_result is not None:
                return cached_result
        except Exception as e:
            pass  # Fall through to cargo bench
    
    # Fall back to running cargo bench
    try:
        print(f"  Generating Rust benchmark '{bench_name}' (no cached results)...", file=sys.stderr)
        
        result = subprocess.run(
            ['cargo', 'bench', '--release', '--', '--nocapture', bench_name],
            capture_output=True,
            text=True,
            timeout=120,
            cwd=Path(__file__).parent.parent
        )
        
        # Parse criterion output for mean time
        # Criterion format: "time:   [X.XXX s X.XXX s X.XXX s]"
        match = re.search(r'time:\s+\[\s*([\d.]+)\s+', result.stdout + result.stderr)
        if match:
            time_str = match.group(1)
            # Convert to seconds
            if 'ms' in result.stdout or 'ms' in result.stderr:
                return float(time_str) / 1000
            elif 'µs' in result.stdout or 'µs' in result.stderr or 'us' in result.stdout:
                return float(time_str) / 1_000_000
            else:
                return float(time_str)
    except Exception as e:
        print(f"  Warning: Could not extract Rust benchmark '{bench_name}': {e}", file=sys.stderr)
    
    return None


def compare_three_way(rust_time_ms, pymarc_bench, pymrrc_bench):
    """
    Compare three benchmarks: Rust, Python wrapper, and pure Python.
    Times should be in milliseconds.
    """
    comparisons = {}
    
    if pymarc_bench and pymrrc_bench:
        pymarc_mean = pymarc_bench.stats()['mean']
        pymrrc_mean = pymrrc_bench.stats()['mean']
        
        comparisons['pymrrc_vs_pymarc'] = {
            'speedup': pymarc_mean / pymrrc_mean,
            'pymrrc_ms': pymrrc_mean * 1000,
            'pymarc_ms': pymarc_mean * 1000,
        }
    
    if rust_time_ms and pymrrc_bench:
        pymrrc_mean = pymrrc_bench.stats()['mean'] * 1000  # Convert to ms
        comparisons['pymrrc_vs_rust'] = {
            'speedup': rust_time_ms / pymrrc_mean,
            'rust_ms': rust_time_ms,
            'pymrrc_ms': pymrrc_mean,
        }
    
    if rust_time_ms and pymarc_bench:
        pymarc_mean = pymarc_bench.stats()['mean'] * 1000  # Convert to ms
        comparisons['rust_vs_pymarc'] = {
            'speedup': pymarc_mean / rust_time_ms,
            'rust_ms': rust_time_ms,
            'pymarc_ms': pymarc_mean,
        }
    
    return comparisons


def compare_benchmarks(pymarc_bench, mrrc_bench):
    """Compare two benchmarks and print results."""
    if not pymarc_bench or not mrrc_bench:
        return None
    
    pymarc_mean = pymarc_bench.stats()['mean']
    mrrc_mean = mrrc_bench.stats()['mean']
    
    speedup = pymarc_mean / mrrc_mean
    improvement = ((pymarc_mean - mrrc_mean) / pymarc_mean) * 100
    
    return {
        'speedup': speedup,
        'improvement': improvement,
        'pymarc_mean_ms': pymarc_mean * 1000,
        'mrrc_mean_ms': mrrc_mean * 1000,
    }


def print_comparison(name, comparison):
    """Print comparison results."""
    if comparison:
        print(f"\n{name}:")
        print(f"  pymarc: {comparison['pymarc_mean_ms']:.2f} ms")
        print(f"  mrrc:   {comparison['mrrc_mean_ms']:.2f} ms")
        print(f"  Speedup: {comparison['speedup']:.1f}x faster")
        print(f"  Improvement: {comparison['improvement']:.1f}%")


def print_three_way_comparison(name, rust_ms, pymrrc_bench, pymarc_bench):
    """Print three-way comparison results."""
    print(f"\n{name}:")
    
    if rust_ms:
        print(f"  Rust (mrrc):    {rust_ms:.2f} ms")
    else:
        print(f"  Rust (mrrc):    [benchmark skipped]")
    
    if pymrrc_bench:
        pymrrc_ms = pymrrc_bench.stats()['mean'] * 1000
        print(f"  Python (pymrrc): {pymrrc_ms:.2f} ms")
    
    if pymarc_bench:
        pymarc_ms = pymarc_bench.stats()['mean'] * 1000
        print(f"  Pure Python (pymarc): {pymarc_ms:.2f} ms")
    
    # Print speedups
    comparisons = compare_three_way(rust_ms, pymarc_bench, pymrrc_bench)
    if comparisons:
        if 'pymrrc_vs_pymarc' in comparisons:
            c = comparisons['pymrrc_vs_pymarc']
            print(f"  → pymrrc is {c['speedup']:.1f}x faster than pymarc")
        if 'rust_vs_pymarc' in comparisons:
            c = comparisons['rust_vs_pymarc']
            print(f"  → Rust is {c['speedup']:.1f}x faster than pymarc")
        if 'pymrrc_vs_rust' in comparisons:
            c = comparisons['pymrrc_vs_rust']
            print(f"  → Rust is {1/c['speedup']:.1f}x faster than pymrrc")


def is_ci_environment():
    """Detect if running in CI environment."""
    return bool(os.environ.get('CI') or 
                os.environ.get('GITHUB_ACTIONS') or 
                os.environ.get('GITLAB_CI') or 
                os.environ.get('CIRCLECI') or
                os.environ.get('TRAVIS'))


def main():
    """Run comprehensive benchmarks."""
    print("=" * 70)
    print("MARC Library Benchmarking: Rust vs pymrrc vs pymarc")
    print("=" * 70)
    
    in_ci = is_ci_environment()
    if in_ci:
        print("\n[CI MODE] Running limited benchmarks (1k/10k only)")
        print("         For full suite including 100k, run locally")
    
    if not HAS_PYMARC:
        print("\nERROR: pymarc not installed. Install with: pip install pymarc")
        sys.exit(1)
    
    if not HAS_MRRC:
        print("\nERROR: mrrc not installed. Install with: maturin develop")
        sys.exit(1)
    
    fixture_dir = Path(__file__).parent.parent / 'tests' / 'data' / 'fixtures'
    
    print("\nLoading test fixtures...")
    data_1k = load_fixture(fixture_dir, '1k')
    data_10k = load_fixture(fixture_dir, '10k')
    
    print(f"  1k fixture: {len(data_1k) / 1024:.2f} KB")
    print(f"  10k fixture: {len(data_10k) / 1024:.2f} KB")
    
    if HAS_CRITERION:
        try:
            extractor = CriterionExtractor(Path(__file__).parent.parent)
            summary = extractor.cache_summary()
            
            if summary['criterion_dir_exists']:
                if summary['is_stale']:
                    print("\n⚠ Criterion.rs cache may be stale (>24h or source changed)")
                    print("  Run 'cargo bench --release' for fresh results")
                else:
                    print("\n✓ Using fresh cached Criterion.rs results")
                
                print(f"  Available: {summary['total_benchmarks']} benchmarks")
            else:
                print("\nNo Criterion.rs cache found")
                print("  Run 'cargo bench --release' to generate benchmarks")
        except Exception as e:
            print(f"\nCriterion check failed: {e}")
    else:
        print("\nNote: Criterion.rs module not available")
        print("      Rust benchmarks will be generated if needed")
    
    results = {}
    
    # Test 1: Pure reading - 1k records
    print("\n" + "-" * 70)
    print("Test 1: Reading 1,000 records")
    print("-" * 70)
    b1_pymarc = benchmark_pymarc_read(data_1k, iterations=3)
    b1_mrrc = benchmark_mrrc_read(data_1k, iterations=3)
    rust_1k_read = extract_rust_benchmark("read_1k_records")
    b1_pymarc.report()
    b1_mrrc.report()
    comp1 = compare_benchmarks(b1_pymarc, b1_mrrc)
    print_comparison("Python Comparison", comp1)
    print_three_way_comparison("Three-Way Comparison", rust_1k_read, b1_mrrc, b1_pymarc)
    results['read_1k'] = {
        'python': comp1,
        'three_way': compare_three_way(rust_1k_read, b1_pymarc, b1_mrrc),
    }
    
    # Test 2: Pure reading - 10k records
    print("\n" + "-" * 70)
    print("Test 2: Reading 10,000 records")
    print("-" * 70)
    b2_pymarc = benchmark_pymarc_read(data_10k, iterations=3)
    b2_mrrc = benchmark_mrrc_read(data_10k, iterations=3)
    rust_10k_read = extract_rust_benchmark("read_10k_records")
    b2_pymarc.report()
    b2_mrrc.report()
    comp2 = compare_benchmarks(b2_pymarc, b2_mrrc)
    print_comparison("Python Comparison", comp2)
    print_three_way_comparison("Three-Way Comparison", rust_10k_read, b2_mrrc, b2_pymarc)
    results['read_10k'] = {
        'python': comp2,
        'three_way': compare_three_way(rust_10k_read, b2_pymarc, b2_mrrc),
    }
    
    # Test 3: Field extraction - 1k records
    print("\n" + "-" * 70)
    print("Test 3: Reading + extracting titles (1,000 records)")
    print("-" * 70)
    b3_pymarc = benchmark_pymarc_extract_titles(data_1k, iterations=3)
    b3_mrrc = benchmark_mrrc_extract_titles(data_1k, iterations=3)
    rust_1k_field = extract_rust_benchmark("read_1k_with_field_access")
    b3_pymarc.report()
    b3_mrrc.report()
    comp3 = compare_benchmarks(b3_pymarc, b3_mrrc)
    print_comparison("Python Comparison", comp3)
    print_three_way_comparison("Three-Way Comparison", rust_1k_field, b3_mrrc, b3_pymarc)
    results['extract_titles_1k'] = {
        'python': comp3,
        'three_way': compare_three_way(rust_1k_field, b3_pymarc, b3_mrrc),
    }
    
    # Test 4: Field extraction - 10k records
    print("\n" + "-" * 70)
    print("Test 4: Reading + extracting titles (10,000 records)")
    print("-" * 70)
    b4_pymarc = benchmark_pymarc_extract_titles(data_10k, iterations=3)
    b4_mrrc = benchmark_mrrc_extract_titles(data_10k, iterations=3)
    rust_10k_field = extract_rust_benchmark("read_10k_with_field_access")
    b4_pymarc.report()
    b4_mrrc.report()
    comp4 = compare_benchmarks(b4_pymarc, b4_mrrc)
    print_comparison("Python Comparison", comp4)
    print_three_way_comparison("Three-Way Comparison", rust_10k_field, b4_mrrc, b4_pymarc)
    results['extract_titles_10k'] = {
        'python': comp4,
        'three_way': compare_three_way(rust_10k_field, b4_pymarc, b4_mrrc),
    }
    
    # Test 5: Round-trip (read + write) - 1k records
    print("\n" + "-" * 70)
    print("Test 5: Round-trip read + write (1,000 records)")
    print("-" * 70)
    b5_pymarc = benchmark_pymarc_roundtrip(data_1k, iterations=3)
    b5_mrrc = benchmark_mrrc_roundtrip(data_1k, iterations=3)
    rust_1k_rt = extract_rust_benchmark("roundtrip_1k_records")
    b5_pymarc.report()
    b5_mrrc.report()
    comp5 = compare_benchmarks(b5_pymarc, b5_mrrc)
    print_comparison("Python Comparison", comp5)
    print_three_way_comparison("Three-Way Comparison", rust_1k_rt, b5_mrrc, b5_pymarc)
    results['roundtrip_1k'] = {
        'python': comp5,
        'three_way': compare_three_way(rust_1k_rt, b5_pymarc, b5_mrrc),
    }
    
    # Test 6: Round-trip (read + write) - 10k records
    print("\n" + "-" * 70)
    print("Test 6: Round-trip read + write (10,000 records)")
    print("-" * 70)
    b6_pymarc = benchmark_pymarc_roundtrip(data_10k, iterations=3)
    b6_mrrc = benchmark_mrrc_roundtrip(data_10k, iterations=3)
    rust_10k_rt = extract_rust_benchmark("roundtrip_10k_records")
    b6_pymarc.report()
    b6_mrrc.report()
    comp6 = compare_benchmarks(b6_pymarc, b6_mrrc)
    print_comparison("Python Comparison", comp6)
    print_three_way_comparison("Three-Way Comparison", rust_10k_rt, b6_mrrc, b6_pymarc)
    results['roundtrip_10k'] = {
        'python': comp6,
        'three_way': compare_three_way(rust_10k_rt, b6_pymarc, b6_mrrc),
    }
    
    # Test 7: JSON serialization - 1k records
    print("\n" + "-" * 70)
    print("Test 7: JSON serialization (1,000 records)")
    print("-" * 70)
    print("Note: mrrc only (pymarc has no native JSON serialization)")
    b7_mrrc = benchmark_mrrc_json_serialization(data_1k, iterations=3)
    b7_mrrc.report()
    results['json_serialization_1k'] = {'mrrc_only': True}
    
    # Test 8: XML serialization - 1k records
    print("\n" + "-" * 70)
    print("Test 8: XML serialization (1,000 records)")
    print("-" * 70)
    print("Note: mrrc only (pymarc has no native XML serialization)")
    b8_mrrc = benchmark_mrrc_xml_serialization(data_1k, iterations=3)
    b8_mrrc.report()
    results['xml_serialization_1k'] = {'mrrc_only': True}
    
    # Test 9: Full 100k benchmark (local only, skip in CI)
    if not in_ci:
        print("\n" + "-" * 70)
        print("Test 9: Full 100k records (local-only comprehensive test)")
        print("-" * 70)
        try:
            data_100k = load_fixture(fixture_dir, '100k')
            print(f"  100k fixture: {len(data_100k) / 1024 / 1024:.2f} MB")
            
            b9_pymarc = benchmark_pymarc_read(data_100k, iterations=1)
            b9_mrrc = benchmark_mrrc_read(data_100k, iterations=1)
            rust_100k_read = extract_rust_benchmark("read_100k_records")
            
            b9_pymarc.report()
            b9_mrrc.report()
            
            comp9 = compare_benchmarks(b9_pymarc, b9_mrrc)
            print_comparison("Python Comparison", comp9)
            print_three_way_comparison("Three-Way Comparison", rust_100k_read, b9_mrrc, b9_pymarc)
            
            results['read_100k'] = {
                'python': comp9,
                'three_way': compare_three_way(rust_100k_read, b9_pymarc, b9_mrrc),
            }
        except FileNotFoundError:
            print("  ⚠ 100k fixture not available (run locally with full test suite)")
            results['read_100k'] = {'skipped': 'fixture_not_found'}
    else:
        results['read_100k'] = {'skipped': 'ci_environment'}
    
    # Summary
    print("\n" + "=" * 70)
    print("SUMMARY")
    print("=" * 70)
    
    # Extract speedups from Python comparisons
    python_speedups = []
    three_way_rust_speedups = []
    three_way_pymrrc_speedups = []
    
    for test_name, test_results in results.items():
        if isinstance(test_results, dict):
            if 'python' in test_results and test_results['python']:
                python_speedups.append(test_results['python'].get('speedup', 0))
            
            if 'three_way' in test_results and test_results['three_way']:
                tw = test_results['three_way']
                if 'rust_vs_pymarc' in tw and tw['rust_vs_pymarc']:
                    three_way_rust_speedups.append(tw['rust_vs_pymarc']['speedup'])
                if 'pymrrc_vs_pymarc' in tw and tw['pymrrc_vs_pymarc']:
                    three_way_pymrrc_speedups.append(tw['pymrrc_vs_pymarc']['speedup'])
    
    if python_speedups:
        avg_pymrrc_speedup = statistics.mean(python_speedups)
        print(f"\nPython Wrapper (pymrrc) vs Pure Python (pymarc):")
        print(f"  Average speedup: {avg_pymrrc_speedup:.1f}x faster")
        print(f"  Best case: {max(python_speedups):.1f}x")
        print(f"  Worst case: {min(python_speedups):.1f}x")
    
    if three_way_rust_speedups:
        avg_rust_speedup = statistics.mean(three_way_rust_speedups)
        print(f"\nRust (mrrc) vs Pure Python (pymarc):")
        print(f"  Average speedup: {avg_rust_speedup:.1f}x faster")
        print(f"  Best case: {max(three_way_rust_speedups):.1f}x")
        print(f"  Worst case: {min(three_way_rust_speedups):.1f}x")
    
    if three_way_pymrrc_speedups:
        avg_py_speedup = statistics.mean(three_way_pymrrc_speedups)
        print(f"\nPython Wrapper vs Rust (overhead):")
        print(f"  Average overhead: {statistics.mean([1/s for s in three_way_pymrrc_speedups]):.1%}")
    
    # Save results to file
    results_file = Path(__file__).parent.parent / '.benchmarks' / 'comparison.json'
    results_file.parent.mkdir(parents=True, exist_ok=True)
    with open(results_file, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"\nResults saved to: {results_file}")


if __name__ == '__main__':
    main()
