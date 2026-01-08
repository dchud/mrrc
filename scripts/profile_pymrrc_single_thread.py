#!/usr/bin/env python3
"""
Profiling of pymrrc single-threaded performance to identify bottlenecks.

Within-mode profiling targets:
- GIL acquisition/release patterns
- FFI call overhead (Rust ↔ Python boundary crossing)
- Python object creation cost per record
- Memory allocation patterns
- Garbage collection impact on throughput
- Record parsing vs iteration overhead

Outputs JSON report for docs/design/profiling/pymrrc_single_thread_profile.md
"""

import sys
import time
import cProfile
import pstats
import io
import json
import os
import gc
import tracemalloc
from pathlib import Path
from typing import Dict, Any, List
import statistics

# Add src-python to path for development
sys.path.insert(0, str(Path(__file__).parent.parent / "src-python" / "target" / "release"))

import mrrc
from mrrc import MARCReader


# ============================================================================
# PROFILING: GIL and FFI Overhead Detection
# ============================================================================

class GILMonitor:
    """Monitor GIL release/acquisition patterns during FFI calls."""
    
    def __init__(self):
        self.gil_releases = 0
        self.ffi_calls = 0
        self.sample_times = []
        
    def sample_gil_state(self):
        """Attempt to detect GIL state via timing patterns."""
        # This is heuristic: GIL-released operations have higher variance
        # due to thread switching
        import sys
        return sys._getframe().f_lasti
    
    def get_summary(self):
        return {
            "ffi_calls": self.ffi_calls,
            "sample_count": len(self.sample_times),
            "gil_contention_score": len(self.sample_times) > 0,
        }


class MemoryTracker:
    """Track memory allocation patterns during parsing."""
    
    def __init__(self):
        self.snapshots: List[tracemalloc.Snapshot] = []
        self.peak_memory = 0
        self.total_allocated = 0
        
    def start(self):
        """Start memory tracing."""
        tracemalloc.start()
        
    def snapshot(self, label: str = ""):
        """Take a memory snapshot."""
        snapshot = tracemalloc.take_snapshot()
        self.snapshots.append(snapshot)
        current, peak = tracemalloc.get_traced_memory()
        self.peak_memory = max(self.peak_memory, peak)
        
    def stop(self):
        """Stop memory tracing."""
        tracemalloc.stop()
        
    def get_top_allocations(self, n: int = 5) -> List[Dict[str, Any]]:
        """Get top n memory allocations from last snapshot."""
        if not self.snapshots:
            return []
        
        snapshot = self.snapshots[-1]
        top_stats = snapshot.statistics('lineno')[:n]
        
        result = []
        for stat in top_stats:
            result.append({
                "file": str(stat.traceback[0].filename),
                "line": stat.traceback[0].lineno,
                "size_mb": stat.size / 1024 / 1024,
                "count": stat.count,
            })
        return result
        
    def get_summary(self) -> Dict[str, Any]:
        """Get memory profiling summary."""
        if not self.snapshots:
            return {"peak_memory_mb": 0}
            
        current, peak = tracemalloc.get_traced_memory()
        return {
            "peak_memory_mb": peak / 1024 / 1024,
            "current_memory_mb": current / 1024 / 1024,
            "snapshot_count": len(self.snapshots),
        }


class FFICallCounter:
    """Count and time FFI boundary crossings."""
    
    def __init__(self):
        self.call_count = 0
        self.total_ffi_time = 0.0
        self.call_times: List[float] = []
        
    def record_ffi_call(self, duration: float):
        """Record an FFI call duration."""
        self.call_count += 1
        self.total_ffi_time += duration
        self.call_times.append(duration)
        
    def get_summary(self) -> Dict[str, Any]:
        """Get FFI overhead analysis."""
        if not self.call_times:
            return {
                "ffi_calls": 0,
                "total_ffi_time_ms": 0,
                "avg_ffi_time_us": 0,
            }
            
        call_times_us = [t * 1_000_000 for t in self.call_times]
        return {
            "ffi_calls": self.call_count,
            "total_ffi_time_ms": self.total_ffi_time * 1000,
            "avg_ffi_time_us": statistics.mean(call_times_us),
            "min_ffi_time_us": min(call_times_us),
            "max_ffi_time_us": max(call_times_us),
            "median_ffi_time_us": statistics.median(call_times_us),
            "stdev_ffi_time_us": statistics.stdev(call_times_us) if len(call_times_us) > 1 else 0,
        }


# ============================================================================
# PROFILING SCENARIOS
# ============================================================================

def profile_simple_sequential(test_file: Path) -> Dict[str, Any]:
    """Profile basic sequential file reading."""
    print(f"\n{'='*70}")
    print(f"SCENARIO 1: Simple Sequential Reading ({test_file.name})")
    print(f"{'='*70}")
    
    # Enable cProfile
    pr = cProfile.Profile()
    memory = MemoryTracker()
    
    memory.start()
    pr.enable()
    
    start = time.perf_counter()
    with open(test_file, "rb") as f:
        reader = MARCReader(f)
        record_count = 0
        for record in reader:
            record_count += 1
    elapsed = time.perf_counter() - start
    
    pr.disable()
    memory.snapshot("sequential_end")
    memory.stop()
    
    # Get profiling stats
    s = io.StringIO()
    ps = pstats.Stats(pr, stream=s).sort_stats('cumulative')
    ps.print_stats(10)
    profile_output = s.getvalue()
    
    throughput = record_count / elapsed if elapsed > 0 else 0
    
    result = {
        "scenario": "simple_sequential",
        "test_file": test_file.name,
        "record_count": record_count,
        "elapsed_seconds": elapsed,
        "throughput_rec_per_sec": throughput,
        "throughput_rec_per_ms": throughput / 1000,
        "profile_output": profile_output,
        "memory": memory.get_summary(),
        "top_allocations": memory.get_top_allocations(5),
    }
    
    print(f"Records processed: {record_count}")
    print(f"Time elapsed: {elapsed:.4f}s")
    print(f"Throughput: {throughput:.0f} rec/s ({throughput/1000:.1f} rec/ms)")
    print(f"Memory peak: {memory.get_summary()['peak_memory_mb']:.1f} MB")
    
    return result


def profile_with_object_creation(test_file: Path) -> Dict[str, Any]:
    """Profile object creation overhead in Python."""
    print(f"\n{'='*70}")
    print(f"SCENARIO 2: Object Creation Overhead ({test_file.name})")
    print(f"{'='*70}")
    
    pr = cProfile.Profile()
    memory = MemoryTracker()
    
    memory.start()
    pr.enable()
    
    start = time.perf_counter()
    with open(test_file, "rb") as f:
        reader = MARCReader(f)
        records = []
        for record in reader:
            # Materialize record as dict to measure object conversion cost
            record_dict = {
                "leader": record.leader(),
                "fields": [],
            }
            for field in record.fields():
                record_dict["fields"].append({
                    "tag": field.tag,
                    "indicators": str(field.indicators) if hasattr(field, 'indicators') else None,
                    "subfields": field.subfields() if hasattr(field, 'subfields') else None,
                })
            records.append(record_dict)
    elapsed = time.perf_counter() - start
    
    pr.disable()
    memory.snapshot("object_creation_end")
    memory.stop()
    
    s = io.StringIO()
    ps = pstats.Stats(pr, stream=s).sort_stats('cumulative')
    ps.print_stats(10)
    profile_output = s.getvalue()
    
    throughput = len(records) / elapsed if elapsed > 0 else 0
    
    result = {
        "scenario": "object_creation",
        "test_file": test_file.name,
        "record_count": len(records),
        "elapsed_seconds": elapsed,
        "throughput_rec_per_sec": throughput,
        "profile_output": profile_output,
        "memory": memory.get_summary(),
    }
    
    print(f"Records processed: {len(records)}")
    print(f"Time elapsed: {elapsed:.4f}s")
    print(f"Throughput: {throughput:.0f} rec/s")
    print(f"Memory peak: {memory.get_summary()['peak_memory_mb']:.1f} MB")
    
    return result


def profile_with_gc_impact(test_file: Path) -> Dict[str, Any]:
    """Profile garbage collection impact on performance."""
    print(f"\n{'='*70}")
    print(f"SCENARIO 3: Garbage Collection Impact ({test_file.name})")
    print(f"{'='*70}")
    
    # Get GC stats
    gc_stats_before = gc.get_stats() if hasattr(gc, 'get_stats') else []
    collections_before = gc.get_count()
    
    pr = cProfile.Profile()
    
    # Test with GC enabled
    gc.enable()
    pr.enable()
    
    start = time.perf_counter()
    with open(test_file, "rb") as f:
        reader = MARCReader(f)
        record_count_gc_on = sum(1 for _ in reader)
    elapsed_gc_on = time.perf_counter() - start
    
    pr.disable()
    
    collections_with_gc = gc.get_count()
    gc_collections = [
        collections_with_gc[i] - collections_before[i]
        for i in range(len(collections_before))
    ]
    
    # Test with GC disabled
    gc.disable()
    pr2 = cProfile.Profile()
    pr2.enable()
    
    start = time.perf_counter()
    with open(test_file, "rb") as f:
        reader = MARCReader(f)
        record_count_gc_off = sum(1 for _ in reader)
    elapsed_gc_off = time.perf_counter() - start
    
    pr2.disable()
    gc.enable()
    
    # Calculate overhead
    gc_overhead = elapsed_gc_on - elapsed_gc_off
    gc_overhead_pct = (gc_overhead / elapsed_gc_off * 100) if elapsed_gc_off > 0 else 0
    
    result = {
        "scenario": "gc_impact",
        "test_file": test_file.name,
        "record_count": record_count_gc_on,
        "gc_enabled": {
            "elapsed_seconds": elapsed_gc_on,
            "throughput_rec_per_sec": record_count_gc_on / elapsed_gc_on if elapsed_gc_on > 0 else 0,
            "gc_collections": gc_collections,
        },
        "gc_disabled": {
            "elapsed_seconds": elapsed_gc_off,
            "throughput_rec_per_sec": record_count_gc_off / elapsed_gc_off if elapsed_gc_off > 0 else 0,
        },
        "gc_overhead": {
            "absolute_seconds": gc_overhead,
            "percentage": gc_overhead_pct,
        },
    }
    
    print(f"Records processed: {record_count_gc_on}")
    print(f"With GC: {elapsed_gc_on:.4f}s ({record_count_gc_on/elapsed_gc_on:.0f} rec/s)")
    print(f"Without GC: {elapsed_gc_off:.4f}s ({record_count_gc_off/elapsed_gc_off:.0f} rec/s)")
    print(f"GC overhead: {gc_overhead_pct:.1f}% ({gc_overhead:.4f}s)")
    print(f"GC collections: gen0={gc_collections[0]}, gen1={gc_collections[1]}, gen2={gc_collections[2]}")
    
    return result


def profile_parsing_vs_iteration(test_file: Path) -> Dict[str, Any]:
    """Profile cost of parsing vs iteration/object creation."""
    print(f"\n{'='*70}")
    print(f"SCENARIO 4: Parsing vs Iteration Cost ({test_file.name})")
    print(f"{'='*70}")
    
    # Just iteration (no object access)
    pr1 = cProfile.Profile()
    pr1.enable()
    
    start = time.perf_counter()
    with open(test_file, "rb") as f:
        reader = MARCReader(f)
        count_iter = 0
        for _ in reader:
            count_iter += 1
    elapsed_iter = time.perf_counter() - start
    
    pr1.disable()
    
    # Full parsing and property access
    pr2 = cProfile.Profile()
    pr2.enable()
    
    start = time.perf_counter()
    with open(test_file, "rb") as f:
        reader = MARCReader(f)
        count_parse = 0
        for record in reader:
            # Access properties to force materialization
            _ = record.leader()
            _ = record.control_fields()
            count_parse += 1
    elapsed_parse = time.perf_counter() - start
    
    pr2.disable()
    
    # Overhead of parsing vs bare iteration
    parse_overhead = elapsed_parse - elapsed_iter
    parse_overhead_pct = (parse_overhead / elapsed_iter * 100) if elapsed_iter > 0 else 0
    
    result = {
        "scenario": "parsing_vs_iteration",
        "test_file": test_file.name,
        "record_count": count_parse,
        "iteration_only": {
            "elapsed_seconds": elapsed_iter,
            "throughput_rec_per_sec": count_iter / elapsed_iter if elapsed_iter > 0 else 0,
        },
        "full_parsing": {
            "elapsed_seconds": elapsed_parse,
            "throughput_rec_per_sec": count_parse / elapsed_parse if elapsed_parse > 0 else 0,
        },
        "parsing_overhead": {
            "absolute_seconds": parse_overhead,
            "percentage": parse_overhead_pct,
        },
    }
    
    print(f"Iteration only: {elapsed_iter:.4f}s ({count_iter/elapsed_iter:.0f} rec/s)")
    print(f"Full parsing: {elapsed_parse:.4f}s ({count_parse/elapsed_parse:.0f} rec/s)")
    print(f"Parsing overhead: {parse_overhead_pct:.1f}% ({parse_overhead:.4f}s)")
    
    return result


def profile_with_multiple_runs(test_file: Path, runs: int = 3) -> Dict[str, Any]:
    """Profile with multiple runs to calculate variance."""
    print(f"\n{'='*70}")
    print(f"SCENARIO 5: Variance Analysis ({test_file.name}, {runs} runs)")
    print(f"{'='*70}")
    
    times = []
    throughputs = []
    
    for run in range(runs):
        print(f"  Run {run + 1}/{runs}...", end=" ", flush=True)
        start = time.perf_counter()
        with open(test_file, "rb") as f:
            reader = MARCReader(f)
            count = sum(1 for _ in reader)
        elapsed = time.perf_counter() - start
        times.append(elapsed)
        throughputs.append(count / elapsed if elapsed > 0 else 0)
        print(f"{elapsed:.4f}s ({count/elapsed:.0f} rec/s)")
    
    avg_time = statistics.mean(times)
    avg_throughput = statistics.mean(throughputs)
    stdev_time = statistics.stdev(times) if len(times) > 1 else 0
    stdev_throughput = statistics.stdev(throughputs) if len(throughputs) > 1 else 0
    
    result = {
        "scenario": "variance_analysis",
        "test_file": test_file.name,
        "runs": runs,
        "times": times,
        "throughputs": throughputs,
        "statistics": {
            "avg_time_seconds": avg_time,
            "stdev_time_seconds": stdev_time,
            "cv_time": (stdev_time / avg_time * 100) if avg_time > 0 else 0,
            "avg_throughput_rec_per_sec": avg_throughput,
            "stdev_throughput": stdev_throughput,
            "cv_throughput": (stdev_throughput / avg_throughput * 100) if avg_throughput > 0 else 0,
        },
    }
    
    print(f"\nStatistics:")
    print(f"  Avg time: {avg_time:.4f}s ± {stdev_time:.4f}s (CV: {(stdev_time/avg_time*100):.1f}%)")
    print(f"  Avg throughput: {avg_throughput:.0f} ± {stdev_throughput:.0f} rec/s (CV: {(stdev_throughput/avg_throughput*100):.1f}%)")
    
    return result


# ============================================================================
# MAIN
# ============================================================================

def main():
    """Run Python single-threaded profiling for bottleneck identification."""
    # Find test files
    test_dir = Path(__file__).parent.parent / "tests" / "data"
    simple_book = test_dir / "simple_book.mrc"
    multi_records = test_dir / "multi_records.mrc"
    
    if not simple_book.exists():
        print(f"Error: Test file not found: {simple_book}")
        return
    
    report = {
        "title": "PyMRRC Single-Threaded Performance Profile",
        "objective": "Identify bottlenecks and optimization opportunities in sequential MARCReader",
        "timestamp": time.time(),
        "test_files": {
            "simple_book": str(simple_book),
            "multi_records": str(multi_records) if multi_records.exists() else None,
        },
        "scenarios": [],
    }
    
    # Run profiling scenarios
    report["scenarios"].append(profile_simple_sequential(simple_book))
    report["scenarios"].append(profile_with_object_creation(simple_book))
    report["scenarios"].append(profile_with_gc_impact(simple_book))
    report["scenarios"].append(profile_parsing_vs_iteration(simple_book))
    report["scenarios"].append(profile_with_multiple_runs(simple_book, runs=3))
    
    # Generate JSON report for profiling docs
    output_file = Path(__file__).parent.parent / ".benchmarks" / "pymrrc_single_thread_profile.json"
    output_file.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output_file, "w") as f:
        json.dump(report, f, indent=2, default=str)
    
    print(f"\n{'='*70}")
    print(f"PROFILING COMPLETE")
    print(f"{'='*70}")
    print(f"JSON data written to: {output_file}")
    print(f"Scenarios completed: {len(report['scenarios'])}")
    print(f"\nUse this data to create: docs/design/profiling/pymrrc_single_thread_profile.md")
    
    return report


if __name__ == "__main__":
    main()
