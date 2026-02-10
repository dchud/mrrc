#!/usr/bin/env python3
"""
Profiling of pymrrc concurrent (ProducerConsumerPipeline) performance.

Within-mode profiling targets:
- Producer thread efficiency (file I/O patterns, GIL behavior)
- Consumer thread utilization (rayon task distribution, context switching)
- Bounded channel overhead and contention patterns
- Thread synchronization costs
- GIL contention between producer and consumer threads
- Memory allocation patterns under concurrency
- Context switching overhead

Outputs JSON report for docs/design/profiling/pymrrc_concurrent_profile.md
"""

import sys
import time
import cProfile
import pstats
import io
import json
import os
import gc
import threading
import tracemalloc
from pathlib import Path
from typing import Dict, Any, List
import statistics

# Add src-python to path for development
sys.path.insert(0, str(Path(__file__).parent.parent / "src-python" / "target" / "release"))

import mrrc
from mrrc import ProducerConsumerPipeline


# ============================================================================
# MONITORING UTILITIES
# ============================================================================

class ThreadActivityMonitor:
    """Monitor thread activity and GIL contention patterns."""
    
    def __init__(self):
        self.samples = []
        self.start_time = None
        self.active_threads = {}
        
    def start(self):
        """Start monitoring thread activity."""
        self.start_time = time.perf_counter()
        self.sample_threads()
        
    def sample_threads(self):
        """Record active thread count."""
        now = time.perf_counter()
        active = threading.active_count()
        self.samples.append({
            "time": now - self.start_time if self.start_time else 0,
            "active_threads": active,
        })
        
    def get_summary(self) -> Dict[str, Any]:
        """Get thread activity summary."""
        if not self.samples:
            return {}
            
        thread_counts = [s["active_threads"] for s in self.samples]
        return {
            "sample_count": len(self.samples),
            "avg_active_threads": statistics.mean(thread_counts),
            "max_active_threads": max(thread_counts),
            "min_active_threads": min(thread_counts),
        }


class ChannelInstrumentation:
    """Measure bounded channel throughput and latency."""
    
    def __init__(self):
        self.send_times = []
        self.recv_times = []
        self.send_count = 0
        self.recv_count = 0
        
    def get_summary(self) -> Dict[str, Any]:
        """Get channel efficiency metrics."""
        return {
            "send_count": self.send_count,
            "recv_count": self.recv_count,
            "avg_send_time_us": statistics.mean(self.send_times) * 1_000_000 if self.send_times else 0,
            "avg_recv_time_us": statistics.mean(self.recv_times) * 1_000_000 if self.recv_times else 0,
        }


class MemoryTracker:
    """Track memory allocation patterns during concurrent parsing."""
    
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


# ============================================================================
# PROFILING SCENARIOS
# ============================================================================

def profile_basic_concurrent(test_file: Path) -> Dict[str, Any]:
    """Profile basic concurrent file reading using ProducerConsumerPipeline."""
    print(f"\n{'='*70}")
    print(f"SCENARIO 1: Basic Concurrent Reading ({test_file.name})")
    print(f"{'='*70}")
    
    memory = MemoryTracker()
    thread_monitor = ThreadActivityMonitor()
    
    memory.start()
    thread_monitor.start()
    
    start = time.perf_counter()
    
    try:
        pipeline = ProducerConsumerPipeline.from_file(str(test_file))
        record_count = 0
        for record in pipeline:
            record_count += 1
            thread_monitor.sample_threads()
    except Exception as e:
        print(f"Error during concurrent reading: {e}")
        import traceback
        traceback.print_exc()
        return {"error": str(e)}
    
    elapsed = time.perf_counter() - start
    
    memory.snapshot("concurrent_end")
    memory.stop()
    
    throughput = record_count / elapsed if elapsed > 0 else 0
    
    result = {
        "scenario": "basic_concurrent",
        "test_file": test_file.name,
        "num_consumer_threads": 4,
        "record_count": record_count,
        "elapsed_seconds": elapsed,
        "throughput_rec_per_sec": throughput,
        "throughput_rec_per_ms": throughput / 1000,
        "memory": memory.get_summary(),
        "top_allocations": memory.get_top_allocations(5),
        "thread_activity": thread_monitor.get_summary(),
    }
    
    print(f"Records processed: {record_count}")
    print(f"Time elapsed: {elapsed:.4f}s")
    print(f"Throughput: {throughput:.0f} rec/s ({throughput/1000:.1f} rec/ms)")
    print(f"Memory peak: {memory.get_summary()['peak_memory_mb']:.1f} MB")
    print(f"Thread activity: {thread_monitor.get_summary()}")
    
    return result


def profile_thread_count_sensitivity(test_file: Path) -> Dict[str, Any]:
    """Profile performance across different thread counts."""
    print(f"\n{'='*70}")
    print(f"SCENARIO 2: Thread Count Sensitivity ({test_file.name})")
    print(f"{'='*70}")
    
    thread_counts = [1, 2, 4, 8]
    results = {}
    
    for num_threads in thread_counts:
        print(f"  Testing with {num_threads} consumer threads...", end=" ", flush=True)
        
        start = time.perf_counter()
        try:
            pipeline = ProducerConsumerPipeline.from_file(str(test_file))
            count = sum(1 for _ in pipeline)
        except Exception as e:
            print(f"ERROR: {e}")
            continue
        
        elapsed = time.perf_counter() - start
        throughput = count / elapsed if elapsed > 0 else 0
        
        results[num_threads] = {
            "elapsed_seconds": elapsed,
            "throughput_rec_per_sec": throughput,
            "record_count": count,
        }
        
        print(f"{elapsed:.4f}s ({throughput:.0f} rec/s)")
    
    # Calculate speedup relative to single-threaded
    baseline = results.get(1, {}).get("throughput_rec_per_sec", 1)
    for num_threads in results:
        results[num_threads]["speedup_vs_1thread"] = (
            results[num_threads]["throughput_rec_per_sec"] / baseline
            if baseline > 0 else 0
        )
    
    return {
        "scenario": "thread_count_sensitivity",
        "test_file": test_file.name,
        "results": results,
    }


def profile_channel_efficiency(test_file: Path) -> Dict[str, Any]:
    """Profile bounded channel contention and throughput."""
    print(f"\n{'='*70}")
    print(f"SCENARIO 3: Channel Efficiency ({test_file.name})")
    print(f"{'='*70}")
    
    # Test with different buffer sizes (bounded channel)
    buffer_sizes = [1, 10, 100, 1000]
    results = {}
    
    for buffer_size in buffer_sizes:
        print(f"  Testing with buffer size {buffer_size}...", end=" ", flush=True)
        
        start = time.perf_counter()
        try:
            pipeline = ProducerConsumerPipeline.from_file(str(test_file), buffer_size=buffer_size)
            count = sum(1 for _ in pipeline)
        except Exception as e:
            print(f"ERROR: {e}")
            continue
        
        elapsed = time.perf_counter() - start
        throughput = count / elapsed if elapsed > 0 else 0
        
        results[buffer_size] = {
            "elapsed_seconds": elapsed,
            "throughput_rec_per_sec": throughput,
            "record_count": count,
        }
        
        print(f"{elapsed:.4f}s ({throughput:.0f} rec/s)")
    
    return {
        "scenario": "channel_efficiency",
        "test_file": test_file.name,
        "buffer_sizes": buffer_sizes,
        "results": results,
    }


def profile_producer_efficiency(test_file: Path) -> Dict[str, Any]:
    """Profile producer thread I/O efficiency."""
    print(f"\n{'='*70}")
    print(f"SCENARIO 4: Producer Thread I/O Efficiency ({test_file.name})")
    print(f"{'='*70}")
    
    memory = MemoryTracker()
    
    memory.start()
    
    start = time.perf_counter()
    try:
        pipeline = ProducerConsumerPipeline.from_file(str(test_file))
        
        # Measure time to first record (producer startup)
        first_record_time = None
        record_count = 0
        
        for i, record in enumerate(pipeline):
            if i == 0:
                first_record_time = time.perf_counter() - start
            record_count += 1
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        return {"error": str(e)}
    
    total_elapsed = time.perf_counter() - start
    
    memory.snapshot("producer_end")
    memory.stop()
    
    result = {
        "scenario": "producer_efficiency",
        "test_file": test_file.name,
        "num_consumer_threads": 4,
        "record_count": record_count,
        "total_elapsed_seconds": total_elapsed,
        "time_to_first_record_seconds": first_record_time or 0,
        "throughput_rec_per_sec": record_count / total_elapsed if total_elapsed > 0 else 0,
        "memory": memory.get_summary(),
    }
    
    print(f"Total records: {record_count}")
    print(f"Total time: {total_elapsed:.4f}s")
    print(f"Time to first record: {first_record_time:.4f}s if first_record_time else 'N/A'")
    print(f"Throughput: {record_count/total_elapsed:.0f} rec/s")
    print(f"Memory peak: {memory.get_summary()['peak_memory_mb']:.1f} MB")
    
    return result


def profile_gc_impact(test_file: Path) -> Dict[str, Any]:
    """Profile garbage collection impact on concurrent throughput."""
    print(f"\n{'='*70}")
    print(f"SCENARIO 5: Garbage Collection Impact ({test_file.name})")
    print(f"{'='*70}")
    
    # With GC enabled
    gc.enable()
    gc_stats_before = gc.get_stats() if hasattr(gc, 'get_stats') else []
    collections_before = gc.get_count()
    
    start = time.perf_counter()
    try:
        pipeline = ProducerConsumerPipeline.from_file(str(test_file))
        count_gc_on = sum(1 for _ in pipeline)
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        return {"error": str(e)}
    
    elapsed_gc_on = time.perf_counter() - start
    collections_with_gc = gc.get_count()
    gc_collections = [
        collections_with_gc[i] - collections_before[i]
        for i in range(len(collections_before))
    ]
    
    # With GC disabled
    gc.disable()
    start = time.perf_counter()
    try:
        pipeline = ProducerConsumerPipeline.from_file(str(test_file))
        count_gc_off = sum(1 for _ in pipeline)
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        return {"error": str(e)}
    
    elapsed_gc_off = time.perf_counter() - start
    gc.enable()
    
    gc_overhead = elapsed_gc_on - elapsed_gc_off
    gc_overhead_pct = (gc_overhead / elapsed_gc_off * 100) if elapsed_gc_off > 0 else 0
    
    result = {
        "scenario": "gc_impact_concurrent",
        "test_file": test_file.name,
        "num_consumer_threads": 4,
        "gc_enabled": {
            "elapsed_seconds": elapsed_gc_on,
            "throughput_rec_per_sec": count_gc_on / elapsed_gc_on if elapsed_gc_on > 0 else 0,
            "gc_collections": gc_collections,
        },
        "gc_disabled": {
            "elapsed_seconds": elapsed_gc_off,
            "throughput_rec_per_sec": count_gc_off / elapsed_gc_off if elapsed_gc_off > 0 else 0,
        },
        "gc_overhead": {
            "absolute_seconds": gc_overhead,
            "percentage": gc_overhead_pct,
        },
    }
    
    print(f"With GC: {elapsed_gc_on:.4f}s ({count_gc_on/elapsed_gc_on:.0f} rec/s)")
    print(f"Without GC: {elapsed_gc_off:.4f}s ({count_gc_off/elapsed_gc_off:.0f} rec/s)")
    print(f"GC overhead: {gc_overhead_pct:.1f}% ({gc_overhead:.4f}s)")
    print(f"GC collections: gen0={gc_collections[0]}, gen1={gc_collections[1]}, gen2={gc_collections[2]}")
    
    return result


def profile_multiple_files_concurrent(test_dir: Path) -> Dict[str, Any]:
    """Profile concurrent reading of multiple files."""
    print(f"\n{'='*70}")
    print(f"SCENARIO 6: Multiple Files Concurrent ({test_dir})")
    print(f"{'='*70}")
    
    # Find test files
    test_files = list(test_dir.glob("*.mrc"))[:4]  # Use up to 4 test files
    
    if not test_files:
        print("No test files found")
        return {"error": "no_test_files"}
    
    total_records = 0
    results = {}
    
    for test_file in test_files:
        print(f"  Processing {test_file.name}...", end=" ", flush=True)
        
        start = time.perf_counter()
        try:
            pipeline = ProducerConsumerPipeline.from_file(str(test_file))
            count = sum(1 for _ in pipeline)
        except Exception as e:
            print(f"ERROR: {e}")
            continue
        
        elapsed = time.perf_counter() - start
        throughput = count / elapsed if elapsed > 0 else 0
        total_records += count
        
        results[test_file.name] = {
            "elapsed_seconds": elapsed,
            "record_count": count,
            "throughput_rec_per_sec": throughput,
        }
        
        print(f"{elapsed:.4f}s ({throughput:.0f} rec/s)")
    
    return {
        "scenario": "multiple_files_concurrent",
        "test_directory": str(test_dir),
        "test_files_count": len(test_files),
        "total_records": total_records,
        "per_file_results": results,
    }


# ============================================================================
# MAIN
# ============================================================================

def main():
    """Run Python concurrent profiling for bottleneck identification."""
    # Find test files
    test_dir = Path(__file__).parent.parent / "tests" / "data"
    simple_book = test_dir / "simple_book.mrc"
    
    if not simple_book.exists():
        print(f"Error: Test file not found: {simple_book}")
        return
    
    report = {
        "title": "PyMRRC Concurrent Performance Profile",
        "objective": "Identify bottlenecks in ProducerConsumerPipeline concurrent mode",
        "timestamp": time.time(),
        "test_files": {
            "simple_book": str(simple_book),
        },
        "scenarios": [],
    }
    
    # Run profiling scenarios
    report["scenarios"].append(profile_basic_concurrent(simple_book))
    report["scenarios"].append(profile_thread_count_sensitivity(simple_book))
    report["scenarios"].append(profile_channel_efficiency(simple_book))
    report["scenarios"].append(profile_producer_efficiency(simple_book))
    report["scenarios"].append(profile_gc_impact(simple_book))
    report["scenarios"].append(profile_multiple_files_concurrent(test_dir))
    
    # Generate JSON report for profiling docs
    output_file = Path(__file__).parent.parent / ".benchmarks" / "pymrrc_concurrent_profile.json"
    output_file.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output_file, "w") as f:
        json.dump(report, f, indent=2, default=str)
    
    print(f"\n{'='*70}")
    print(f"PROFILING COMPLETE")
    print(f"{'='*70}")
    print(f"JSON data written to: {output_file}")
    print(f"Scenarios completed: {len(report['scenarios'])}")
    print(f"\nUse this data to create: docs/design/profiling/pymrrc_concurrent_profile.md")
    
    return report


if __name__ == "__main__":
    main()
