#!/usr/bin/env python3
"""
Detailed profiling analysis for Pure Rust MARC performance.

Extracts and analyzes benchmark results to identify bottlenecks.
"""

import json
import subprocess
import statistics
from pathlib import Path
from dataclasses import dataclass, field
from typing import Dict, List


@dataclass
class BenchmarkResult:
    """Single benchmark result"""
    name: str
    mean_ms: float
    stddev_ms: float
    min_ms: float
    max_ms: float
    samples: int
    
    @property
    def records_per_sec(self) -> float:
        # Infer record count from benchmark name
        if "1k" in self.name:
            records = 1000
        elif "10k" in self.name:
            records = 10000
        elif "100k" in self.name:
            records = 100000
        else:
            return 0
        
        return (records / (self.mean_ms / 1000.0)) if self.mean_ms > 0 else 0
    
    @property
    def us_per_record(self) -> float:
        rps = self.records_per_sec
        return (1_000_000 / rps) if rps > 0 else 0


@dataclass
class PerformanceProfile:
    """Complete profile for a benchmark configuration"""
    label: str
    results: List[BenchmarkResult] = field(default_factory=list)
    
    def add_result(self, result: BenchmarkResult):
        self.results.append(result)
    
    def avg_throughput(self) -> float:
        if not self.results:
            return 0
        return statistics.mean([r.records_per_sec for r in self.results])
    
    def avg_latency_us(self) -> float:
        if not self.results:
            return 0
        return statistics.mean([r.us_per_record for r in self.results])


def extract_criterion_results() -> Dict[str, BenchmarkResult]:
    """Extract results from criterion.rs JSON output"""
    results = {}
    
    criterion_dir = Path("target/criterion")
    if not criterion_dir.exists():
        print(f"⚠️  Criterion directory not found: {criterion_dir}")
        return results
    
    # Look for benchmark JSON files in estimates.json
    for json_file in criterion_dir.glob("*/base/estimates.json"):
        benchmark_name = json_file.parent.parent.name
        
        try:
            with open(json_file) as f:
                data = json.load(f)
            
            # Extract mean and std_dev from estimates
            # Values are in nanoseconds
            mean_ns = data.get("mean", {}).get("point_estimate", 0)
            stddev_ns = data.get("std_dev", {}).get("point_estimate", 0)
            
            # Convert nanoseconds to milliseconds
            mean_ms = mean_ns / 1_000_000
            stddev_ms = stddev_ns / 1_000_000
            
            # Try to extract min/max from sample.json
            sample_file = json_file.parent / "sample.json"
            min_ms = max_ms = mean_ms
            num_samples = 1
            
            if sample_file.exists():
                try:
                    with open(sample_file) as f:
                        sample_data = json.load(f)
                    if isinstance(sample_data, list):
                        sample_values = [v / 1_000_000 for v in sample_data]
                        if sample_values:
                            min_ms = min(sample_values)
                            max_ms = max(sample_values)
                            num_samples = len(sample_values)
                except Exception:
                    pass
            
            result = BenchmarkResult(
                name=benchmark_name,
                mean_ms=mean_ms,
                stddev_ms=stddev_ms,
                min_ms=min_ms,
                max_ms=max_ms,
                samples=num_samples
            )
            results[benchmark_name] = result
            
        except Exception as e:
            print(f"⚠️  Failed to parse {json_file}: {e}")
    
    return results


def analyze_read_operations(results: Dict[str, BenchmarkResult]):
    """Analyze read operation performance"""
    print("\n" + "="*70)
    print("READ OPERATIONS ANALYSIS")
    print("="*70)
    
    read_benchmarks = [k for k in results.keys() if k.startswith("read_") and "field" not in k]
    
    if not read_benchmarks:
        print("No read benchmarks found")
        return
    
    print(f"\n{'Benchmark':<35} {'Mean':<12} {'Stddev':<12} {'Rec/sec':<12}")
    print("-" * 70)
    
    for name in sorted(read_benchmarks):
        result = results[name]
        print(f"{name:<35} {result.mean_ms:>10.2f}ms {result.stddev_ms:>10.2f}ms {result.records_per_sec:>10.0f}")


def analyze_field_access(results: Dict[str, BenchmarkResult]):
    """Analyze field access overhead"""
    print("\n" + "="*70)
    print("FIELD ACCESS OVERHEAD")
    print("="*70)
    
    # Compare read vs read_with_field_access
    pairs = [
        ("read_1k_records", "read_1k_with_field_access"),
        ("read_10k_records", "read_10k_with_field_access"),
    ]
    
    print(f"\n{'Operation':<35} {'Time (ms)':<12} {'Overhead':<12}")
    print("-" * 70)
    
    for bare, with_access in pairs:
        if bare in results and with_access in results:
            bare_time = results[bare].mean_ms
            access_time = results[with_access].mean_ms
            overhead = ((access_time - bare_time) / bare_time) * 100
            
            print(f"{bare:<35} {bare_time:>10.2f}ms")
            print(f"  + field access (245, 100)        {access_time:>10.2f}ms (+{overhead:>6.1f}%)")


def analyze_serialization(results: Dict[str, BenchmarkResult]):
    """Analyze serialization overhead"""
    print("\n" + "="*70)
    print("SERIALIZATION PERFORMANCE")
    print("="*70)
    
    serialization_types = {
        "read_1k_records": "Parse only",
        "serialize_1k_to_json": "Parse + JSON",
        "serialize_1k_to_xml": "Parse + XML",
    }
    
    print(f"\n{'Operation':<35} {'Time (ms)':<12} {'Overhead':<12}")
    print("-" * 70)
    
    baseline_time = None
    for bench_name, label in serialization_types.items():
        if bench_name in results:
            result = results[bench_name]
            if baseline_time is None:
                baseline_time = result.mean_ms
                print(f"{label:<35} {result.mean_ms:>10.2f}ms (baseline)")
            else:
                overhead = ((result.mean_ms - baseline_time) / baseline_time) * 100
                print(f"{label:<35} {result.mean_ms:>10.2f}ms (+{overhead:>6.1f}%)")


def analyze_roundtrip(results: Dict[str, BenchmarkResult]):
    """Analyze roundtrip (read + write) performance"""
    print("\n" + "="*70)
    print("ROUNDTRIP (READ + WRITE)")
    print("="*70)
    
    roundtrip_benchmarks = [k for k in results.keys() if "roundtrip" in k]
    
    if not roundtrip_benchmarks:
        print("No roundtrip benchmarks found")
        return
    
    print(f"\n{'Benchmark':<35} {'Time (ms)':<12} {'Throughput':<12}")
    print("-" * 70)
    
    for name in sorted(roundtrip_benchmarks):
        result = results[name]
        print(f"{name:<35} {result.mean_ms:>10.2f}ms {result.records_per_sec:>10.0f} rec/sec")


def print_summary(results: Dict[str, BenchmarkResult]):
    """Print overall summary"""
    print("\n" + "="*70)
    print("OVERALL SUMMARY")
    print("="*70)
    
    if not results:
        print("No benchmarks available")
        return
    
    # Group by operation type
    read_results = [r for k, r in results.items() if "read_" in k and "field" not in k and "roundtrip" not in k]
    
    if read_results:
        avg_throughput = statistics.mean([r.records_per_sec for r in read_results])
        avg_latency = statistics.mean([r.us_per_record for r in read_results])
        
        print(f"\nRead Operations (averages across all file sizes):")
        print(f"  Average throughput: {avg_throughput:>10.0f} rec/sec")
        print(f"  Average latency:    {avg_latency:>10.2f} µs/record")
    
    print(f"\nTotal benchmarks analyzed: {len(results)}")


def main():
    print("Extracting Criterion.rs benchmark results...")
    
    results = extract_criterion_results()
    
    if not results:
        print("❌ No benchmark results found. Run 'cargo bench' first.")
        return 1
    
    print(f"✓ Found {len(results)} benchmark results\n")
    
    analyze_read_operations(results)
    analyze_field_access(results)
    analyze_serialization(results)
    analyze_roundtrip(results)
    print_summary(results)
    
    print("\n" + "="*70)
    print("For detailed profiling with flamegraph/perf, see docs/design/PROFILING_PLAN.md")
    print("="*70 + "\n")
    
    return 0


if __name__ == "__main__":
    exit(main())
