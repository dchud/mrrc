#!/usr/bin/env python3
"""
Extract Criterion.rs benchmark results from target/criterion/ directory.

This module parses Criterion.rs JSON output files to extract mean times
without needing to re-run benchmarks. Much faster than running cargo bench.
"""

import json
from pathlib import Path
from typing import Optional, Dict
import statistics
import os
from datetime import datetime, timedelta


class CriterionExtractor:
    """Extract and parse Criterion.rs benchmark results."""
    
    def __init__(self, project_root: Optional[Path] = None):
        """
        Initialize extractor.
        
        Args:
            project_root: Path to project root. Defaults to parent of this script.
        """
        if project_root is None:
            project_root = Path(__file__).parent.parent
        
        self.project_root = project_root
        self.criterion_dir = project_root / "target" / "criterion"
    
    def get_benchmark_result(self, bench_name: str) -> Optional[float]:
        """
        Get mean time (in seconds) for a benchmark.
        
        Args:
            bench_name: Criterion benchmark name (e.g., "read_1k_records")
        
        Returns:
            Mean time in seconds, or None if not found.
        """
        estimates_file = self.criterion_dir / bench_name / "base" / "estimates.json"
        
        if not estimates_file.exists():
            return None
        
        try:
            with open(estimates_file) as f:
                data = json.load(f)
            
            # Extract mean time in nanoseconds
            mean_ns = data.get("mean", {}).get("point_estimate")
            if mean_ns is None:
                return None
            
            # Convert nanoseconds to seconds
            return mean_ns / 1_000_000_000
        
        except (json.JSONDecodeError, KeyError, TypeError):
            return None
    
    def get_all_benchmarks(self) -> Dict[str, float]:
        """
        Get all available benchmark results.
        
        Returns:
            Dictionary mapping benchmark names to mean times (seconds).
        """
        results = {}
        
        if not self.criterion_dir.exists():
            return results
        
        for bench_dir in self.criterion_dir.iterdir():
            if not bench_dir.is_dir():
                continue
            
            bench_name = bench_dir.name
            mean_time = self.get_benchmark_result(bench_name)
            
            if mean_time is not None:
                results[bench_name] = mean_time
        
        return results
    
    def get_available_benchmarks(self) -> list:
        """Get list of benchmark names that have cached results."""
        return sorted(self.get_all_benchmarks().keys())
    
    def is_cached(self, bench_name: str) -> bool:
        """Check if a benchmark has cached results."""
        return self.get_benchmark_result(bench_name) is not None
    
    def is_stale(self, max_age_hours: int = 24) -> bool:
        """
        Check if benchmark cache is stale.
        
        Args:
            max_age_hours: Maximum age of cache in hours before considered stale
        
        Returns:
            True if cache is older than max_age_hours or doesn't exist.
        """
        if not self.criterion_dir.exists():
            return True
        
        # Find newest benchmark result
        newest_mtime = 0
        for bench_dir in self.criterion_dir.iterdir():
            if bench_dir.is_dir():
                estimates_file = bench_dir / "base" / "estimates.json"
                if estimates_file.exists():
                    mtime = estimates_file.stat().st_mtime
                    newest_mtime = max(newest_mtime, mtime)
        
        if newest_mtime == 0:
            return True
        
        # Check if source files are newer than benchmark results
        src_dir = self.project_root / "benches"
        if src_dir.exists():
            for rs_file in src_dir.glob("*.rs"):
                if rs_file.stat().st_mtime > newest_mtime:
                    return True
        
        # Check age in hours
        cache_age = datetime.now().timestamp() - newest_mtime
        max_age_seconds = max_age_hours * 3600
        
        return cache_age > max_age_seconds
    
    def cache_summary(self) -> Dict:
        """
        Get summary of what's cached.
        
        Returns:
            Dictionary with cache status info.
        """
        benchmarks = self.get_all_benchmarks()
        stale = self.is_stale()
        
        return {
            'total_benchmarks': len(benchmarks),
            'benchmarks': sorted(benchmarks.keys()),
            'criterion_dir_exists': self.criterion_dir.exists(),
            'is_stale': stale,
        }


def extract_rust_benchmark_cached(bench_name: str, project_root: Optional[Path] = None) -> Optional[float]:
    """
    Convenience function to extract a single benchmark result.
    
    Args:
        bench_name: Criterion benchmark name
        project_root: Path to project root
    
    Returns:
        Mean time in seconds, or None if not found.
    """
    extractor = CriterionExtractor(project_root)
    return extractor.get_benchmark_result(bench_name)


if __name__ == '__main__':
    import sys
    
    extractor = CriterionExtractor()
    
    # Print cached benchmarks
    print("Cached Criterion.rs Benchmarks")
    print("=" * 70)
    
    summary = extractor.cache_summary()
    
    if not summary['criterion_dir_exists']:
        print("\nNo Criterion results found.")
        print("Run: cargo bench --release")
        sys.exit(1)
    
    print(f"\nFound {summary['total_benchmarks']} benchmarks:\n")
    
    benchmarks = extractor.get_all_benchmarks()
    for bench_name in sorted(benchmarks.keys()):
        mean_time = benchmarks[bench_name]
        
        # Format time nicely
        if mean_time < 0.001:
            time_str = f"{mean_time * 1_000_000:.2f} µs"
        elif mean_time < 1:
            time_str = f"{mean_time * 1000:.2f} ms"
        else:
            time_str = f"{mean_time:.2f} s"
        
        print(f"  {bench_name:40s} {time_str:>12s}")
