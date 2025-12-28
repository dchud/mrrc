#!/usr/bin/env python3
"""
Benchmark comparison script: mrrc (Python wrapper) vs pymarc.

This script compares performance between:
1. pymarc - Pure Python MARC library (baseline)
2. mrrc - Rust-backed Python wrapper via PyO3 (this project)

Run with:
    pip install pymarc
    python scripts/benchmark_comparison.py
"""

import sys
import time
import json
from pathlib import Path
from io import BytesIO
import statistics

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


def main():
    """Run comprehensive benchmarks."""
    print("=" * 70)
    print("MARC Library Benchmarking: pymarc vs mrrc (Rust-backed)")
    print("=" * 70)
    
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
    
    results = {}
    
    # Test 1: Pure reading - 1k records
    print("\n" + "-" * 70)
    print("Test 1: Reading 1,000 records")
    print("-" * 70)
    b1_pymarc = benchmark_pymarc_read(data_1k, iterations=3)
    b1_mrrc = benchmark_mrrc_read(data_1k, iterations=3)
    b1_pymarc.report()
    b1_mrrc.report()
    comp1 = compare_benchmarks(b1_pymarc, b1_mrrc)
    print_comparison("Comparison", comp1)
    results['read_1k'] = comp1
    
    # Test 2: Pure reading - 10k records
    print("\n" + "-" * 70)
    print("Test 2: Reading 10,000 records")
    print("-" * 70)
    b2_pymarc = benchmark_pymarc_read(data_10k, iterations=3)
    b2_mrrc = benchmark_mrrc_read(data_10k, iterations=3)
    b2_pymarc.report()
    b2_mrrc.report()
    comp2 = compare_benchmarks(b2_pymarc, b2_mrrc)
    print_comparison("Comparison", comp2)
    results['read_10k'] = comp2
    
    # Test 3: Field extraction - 1k records
    print("\n" + "-" * 70)
    print("Test 3: Reading + extracting titles (1,000 records)")
    print("-" * 70)
    b3_pymarc = benchmark_pymarc_extract_titles(data_1k, iterations=3)
    b3_mrrc = benchmark_mrrc_extract_titles(data_1k, iterations=3)
    b3_pymarc.report()
    b3_mrrc.report()
    comp3 = compare_benchmarks(b3_pymarc, b3_mrrc)
    print_comparison("Comparison", comp3)
    results['extract_titles_1k'] = comp3
    
    # Test 4: Field extraction - 10k records
    print("\n" + "-" * 70)
    print("Test 4: Reading + extracting titles (10,000 records)")
    print("-" * 70)
    b4_pymarc = benchmark_pymarc_extract_titles(data_10k, iterations=3)
    b4_mrrc = benchmark_mrrc_extract_titles(data_10k, iterations=3)
    b4_pymarc.report()
    b4_mrrc.report()
    comp4 = compare_benchmarks(b4_pymarc, b4_mrrc)
    print_comparison("Comparison", comp4)
    results['extract_titles_10k'] = comp4
    
    # Summary
    print("\n" + "=" * 70)
    print("SUMMARY")
    print("=" * 70)
    
    speedups = [r['speedup'] for r in results.values() if r]
    if speedups:
        avg_speedup = statistics.mean(speedups)
        print(f"\nAverage speedup: {avg_speedup:.1f}x faster")
        print(f"Best case: {max(speedups):.1f}x")
        print(f"Worst case: {min(speedups):.1f}x")
    
    # Save results to file
    results_file = Path(__file__).parent.parent / '.benchmarks' / 'comparison.json'
    results_file.parent.mkdir(parents=True, exist_ok=True)
    with open(results_file, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"\nResults saved to: {results_file}")


if __name__ == '__main__':
    main()
