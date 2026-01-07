#!/usr/bin/env python3
"""
Baseline performance measurement for current PyMARCWriter (pre-refactoring).

This script measures current writer performance to establish a baseline for
comparison after Phase D refactoring. Metrics captured:
- Write 10K records to BytesIO (Python file object)
- Write 10K records to temp file
- Concurrent write (2 threads × 5K records)
- Timing and peak memory usage

Results saved to writer_baseline.txt with timestamp and system info.
"""

import sys
import os
import io
import tempfile
import time
import threading
from concurrent.futures import ThreadPoolExecutor
import tracemalloc
import json
from pathlib import Path

# Add parent to path to import mrrc
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from mrrc import MARCReader, MARCWriter, Record, Field, Leader


def get_system_info():
    """Capture system information for context."""
    import platform
    import subprocess
    
    info = {
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "python_version": platform.python_version(),
        "platform": platform.platform(),
        "processor": platform.processor(),
    }
    
    # Try to get Rust version
    try:
        result = subprocess.run(["rustc", "--version"], capture_output=True, text=True)
        info["rustc_version"] = result.stdout.strip()
    except:
        info["rustc_version"] = "unknown"
    
    # Try to get CPU info (macOS)
    try:
        result = subprocess.run(["sysctl", "-n", "hw.ncpu"], capture_output=True, text=True)
        info["cpu_count"] = int(result.stdout.strip())
    except:
        info["cpu_count"] = os.cpu_count() or "unknown"
    
    return info


def read_test_records(fixture_path: str) -> list:
    """Read all records from a test fixture file."""
    records = []
    with open(fixture_path, 'rb') as f:
        reader = MARCReader(f)
        for record in reader:
            records.append(record)
    return records


def measure_bytesio_write(records: list) -> dict:
    """Measure writing records to BytesIO (Python file object)."""
    print("Testing BytesIO write (10K records)...", end=" ", flush=True)
    
    tracemalloc.start()
    start_time = time.perf_counter()
    
    output = io.BytesIO()
    writer = MARCWriter(output)
    for record in records:
        writer.write_record(record)
    writer.close()
    
    elapsed = time.perf_counter() - start_time
    current, peak = tracemalloc.get_traced_memory()
    tracemalloc.stop()
    
    result = {
        "name": "BytesIO write (10K records)",
        "time_seconds": round(elapsed, 3),
        "records_per_second": round(len(records) / elapsed, 1),
        "peak_memory_mb": round(peak / 1024 / 1024, 1),
        "total_bytes": output.tell(),
    }
    
    print(f"{elapsed:.3f}s, {peak/1024/1024:.1f}MB peak, {len(records)/elapsed:.1f} rec/s")
    return result


def measure_tempfile_write(records: list) -> dict:
    """Measure writing records to a temporary file."""
    print("Testing temp file write (10K records)...", end=" ", flush=True)
    
    tracemalloc.start()
    start_time = time.perf_counter()
    
    with tempfile.NamedTemporaryFile(delete=False, suffix='.mrc') as tmp:
        tmp_path = tmp.name
    
    try:
        with open(tmp_path, 'wb') as f:
            writer = MARCWriter(f)
            for record in records:
                writer.write_record(record)
            writer.close()
        
        elapsed = time.perf_counter() - start_time
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        
        file_size = os.path.getsize(tmp_path)
        
        result = {
            "name": "Temp file write (10K records)",
            "time_seconds": round(elapsed, 3),
            "records_per_second": round(len(records) / elapsed, 1),
            "peak_memory_mb": round(peak / 1024 / 1024, 1),
            "file_size_bytes": file_size,
        }
        
        print(f"{elapsed:.3f}s, {peak/1024/1024:.1f}MB peak, {len(records)/elapsed:.1f} rec/s")
        return result
    finally:
        os.unlink(tmp_path)


def write_records_to_file(records: list, output_path: str, thread_id: int = None):
    """Helper to write records to a file (used by concurrent test)."""
    with open(output_path, 'wb') as f:
        writer = MARCWriter(f)
        for record in records:
            writer.write_record(record)
        writer.close()


def measure_concurrent_writes(records_5k: list) -> dict:
    """Measure concurrent writes (2 threads, 5K records each)."""
    print("Testing concurrent writes (2 threads × 5K records)...", end=" ", flush=True)
    
    tracemalloc.start()
    start_time = time.perf_counter()
    
    # Create temp files for both threads
    tmp_files = []
    for _ in range(2):
        tmp = tempfile.NamedTemporaryFile(delete=False, suffix='.mrc')
        tmp_files.append(tmp.name)
        tmp.close()
    
    try:
        # Sequential baseline (for comparison)
        seq_start = time.perf_counter()
        for tmp_path in tmp_files:
            write_records_to_file(records_5k, tmp_path)
        seq_time = time.perf_counter() - seq_start
        
        # Concurrent write
        with ThreadPoolExecutor(max_workers=2) as executor:
            futures = [
                executor.submit(write_records_to_file, records_5k, tmp_path, i)
                for i, tmp_path in enumerate(tmp_files)
            ]
            for future in futures:
                future.result()
        
        elapsed = time.perf_counter() - start_time
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        
        speedup = seq_time / elapsed if elapsed > 0 else 0
        total_records = len(records_5k) * 2
        
        result = {
            "name": "Concurrent writes (2 threads × 5K records)",
            "sequential_time_seconds": round(seq_time, 3),
            "concurrent_time_seconds": round(elapsed, 3),
            "speedup": round(speedup, 2),
            "records_per_second": round(total_records / elapsed, 1),
            "peak_memory_mb": round(peak / 1024 / 1024, 1),
        }
        
        print(f"seq {seq_time:.3f}s → {elapsed:.3f}s, speedup {speedup:.2f}x, {total_records/elapsed:.1f} rec/s")
        return result
    finally:
        for tmp_path in tmp_files:
            os.unlink(tmp_path)


def main():
    """Run all baseline measurements."""
    fixture_path = Path(__file__).parent.parent.parent / "tests" / "data" / "fixtures" / "10k_records.mrc"
    
    if not fixture_path.exists():
        print(f"Error: Test fixture not found at {fixture_path}")
        sys.exit(1)
    
    print(f"Loading test fixture from {fixture_path}...")
    records_10k = read_test_records(str(fixture_path))
    records_5k = records_10k[:5000]
    
    print(f"Loaded {len(records_10k)} records\n")
    
    # Capture system info
    system_info = get_system_info()
    print(f"System: {system_info['platform']}")
    print(f"Python: {system_info['python_version']}")
    print(f"Rustc: {system_info['rustc_version']}")
    print(f"CPU count: {system_info['cpu_count']}\n")
    
    # Run measurements
    results = []
    results.append(measure_bytesio_write(records_10k))
    results.append(measure_tempfile_write(records_10k))
    results.append(measure_concurrent_writes(records_5k))
    
    # Save results
    output_dir = Path(__file__).parent.parent / "performance"
    output_dir.mkdir(exist_ok=True)
    
    output_file = output_dir / "writer_baseline.txt"
    
    with open(output_file, 'w') as f:
        f.write("=" * 80 + "\n")
        f.write("WRITER BASELINE MEASUREMENTS (Pre-Refactoring)\n")
        f.write("=" * 80 + "\n\n")
        
        # System info
        f.write("SYSTEM INFO\n")
        f.write("-" * 80 + "\n")
        for key, value in system_info.items():
            f.write(f"{key:20s}: {value}\n")
        f.write("\n")
        
        # Results
        f.write("MEASUREMENTS\n")
        f.write("-" * 80 + "\n")
        for result in results:
            f.write(f"\n{result['name']}\n")
            for key, value in result.items():
                if key != 'name':
                    f.write(f"  {key:30s}: {value}\n")
        
        f.write("\n" + "=" * 80 + "\n")
        f.write("ACCEPTANCE CRITERIA (Phase D.6)\n")
        f.write("=" * 80 + "\n")
        f.write("RustFile backend:     >= 1.2x faster than baseline\n")
        f.write("PythonFile backend:   >= baseline (no regression)\n")
        f.write("Concurrent speedup:   >= 1.8x for 2 threads\n")
        f.write("\n")
        
        # Save as JSON for easier comparison
        f.write("RAW DATA (JSON)\n")
        f.write("-" * 80 + "\n")
        f.write(json.dumps({
            "system_info": system_info,
            "measurements": results,
        }, indent=2))
    
    print(f"\nResults saved to {output_file}")
    
    # Also print summary to console
    print("\n" + "=" * 80)
    print("SUMMARY")
    print("=" * 80)
    for result in results:
        print(f"\n{result['name']}")
        for key, value in result.items():
            if key != 'name':
                print(f"  {key:30s}: {value}")


if __name__ == '__main__':
    main()
