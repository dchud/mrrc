#!/usr/bin/env python3
"""
Comprehensive benchmarking across all implementation modes pre/post Phase 1.

Test matrix:
1. Pure Rust - Single-Threaded
2. Pure Rust - Concurrent (rayon, 1/2/4/8/16 threads)
3. Python Wrapper - Single-Threaded
4. Python Wrapper - Concurrent (ProducerConsumerPipeline, 1/2/4/8 workers)

Runs on both pre-Phase 1 baseline and current HEAD, saves results for comparison.
"""

import subprocess
import json
import sys
import os
import shutil
from pathlib import Path
from datetime import datetime
from typing import Dict, Any, List, Tuple
import time

# ============================================================================
# CONFIGURATION
# ============================================================================

REPO_ROOT = Path(__file__).parent.parent
PHASE_1_COMMIT = "b1b2ac17"
PHASE_1_PARENT = f"{PHASE_1_COMMIT}~1"

# Test datasets
TEST_FILES = ["1k_records.mrc", "10k_records.mrc"]
TEST_FILES_CONCURRENT = ["1k_records.mrc", "10k_records.mrc", "100k_records.mrc"]

# Rust thread counts for concurrent testing
RUST_THREAD_COUNTS = [1, 2, 4, 8, 16]

# Python worker counts for concurrent testing
PYTHON_WORKER_COUNTS = [1, 2, 4, 8]

# ============================================================================
# UTILITIES
# ============================================================================

def run_command(cmd: List[str], cwd: Path = None, capture: bool = True) -> Tuple[int, str, str]:
    """Run a shell command and return (returncode, stdout, stderr)."""
    if cwd is None:
        cwd = REPO_ROOT
    
    result = subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=capture,
        text=True,
    )
    return result.returncode, result.stdout, result.stderr


def checkout_commit(commit: str) -> bool:
    """Checkout a specific commit."""
    print(f"  Checking out {commit}...")
    rc, _, err = run_command(["git", "checkout", commit])
    if rc != 0:
        print(f"    ERROR: {err}")
        return False
    time.sleep(1)  # Let filesystem settle
    return True


def get_current_commit() -> str:
    """Get current HEAD commit hash."""
    rc, stdout, _ = run_command(["git", "rev-parse", "HEAD"])
    if rc == 0:
        return stdout.strip()
    return "unknown"


def build_release() -> bool:
    """Build Rust release binary."""
    print("  Building release binary...")
    rc, _, err = run_command(["cargo", "build", "--release"])
    if rc != 0:
        print(f"    ERROR: {err}")
        return False
    
    # Also build Python extension
    print("  Building Python extension...")
    rc, _, err = run_command(["maturin", "develop"], cwd=REPO_ROOT / "src-python")
    if rc != 0:
        print(f"    WARNING: maturin failed (might already be built): {err[:200]}")
        # Don't fail on this, as it might be cached
    
    return True


def run_rust_single_thread_benchmark() -> Dict[str, Any]:
    """Run single-threaded Rust benchmark."""
    print("    Running Rust single-threaded...")
    
    # Run via cargo bench
    rc, stdout, err = run_command(["cargo", "bench", "--bench", "profiling_harness", "--", "--nocapture"])
    
    if rc != 0:
        print(f"      ERROR: {err}")
        return {}
    
    # Parse output
    results = parse_profiling_harness_output(stdout)
    return results


def run_rust_concurrent_benchmark() -> Dict[str, Any]:
    """Run concurrent Rust benchmark (rayon)."""
    print("    Running Rust concurrent (rayon)...")
    
    # Run via cargo bench
    rc, stdout, err = run_command(["cargo", "bench", "--bench", "parallel_benchmarks", "--", "--nocapture"])
    
    if rc != 0:
        print(f"      ERROR: {err}")
        return {}
    
    results = parse_parallel_benchmarks_output(stdout)
    return results


def run_python_single_thread_benchmark() -> Dict[str, Any]:
    """Run single-threaded Python benchmark."""
    print("    Running Python single-threaded...")
    
    script = REPO_ROOT / "scripts" / "profile_pymrrc_single_thread.py"
    rc, stdout, err = run_command(["python3", str(script)])
    
    if rc != 0:
        print(f"      ERROR: {err}")
        return {}
    
    # Parse JSON output or metrics from script
    results = parse_python_single_thread_output(stdout)
    return results


def run_python_concurrent_benchmark() -> Dict[str, Any]:
    """Run concurrent Python benchmark (ProducerConsumerPipeline)."""
    print("    Running Python concurrent (ProducerConsumerPipeline)...")
    
    script = REPO_ROOT / "scripts" / "profile_pymrrc_concurrent.py"
    rc, stdout, err = run_command(["python3", str(script)])
    
    if rc != 0:
        print(f"      ERROR: {err}")
        return {}
    
    results = parse_python_concurrent_output(stdout)
    return results


def parse_profiling_harness_output(output: str) -> Dict[str, Any]:
    """Extract metrics from profiling harness output."""
    results = {}
    
    for line in output.split('\n'):
        if 'Average throughput:' in line:
            try:
                throughput = float(line.split(':')[1].strip().split()[0])
                results['throughput_rec_sec'] = throughput
            except:
                pass
        if 'Average time per record:' in line:
            try:
                latency = float(line.split(':')[1].strip().split()[0])
                results['latency_us'] = latency
            except:
                pass
    
    return results


def parse_parallel_benchmarks_output(output: str) -> Dict[str, Any]:
    """Extract metrics from parallel benchmarks output."""
    # Similar to profiling harness parsing
    return parse_profiling_harness_output(output)


def parse_python_single_thread_output(output: str) -> Dict[str, Any]:
    """Extract metrics from Python single-threaded benchmark."""
    results = {}
    
    for line in output.split('\n'):
        if 'throughput:' in line.lower():
            try:
                parts = line.split(':')
                if len(parts) > 1:
                    value = float(parts[1].strip().split()[0])
                    results['throughput_rec_sec'] = value
            except:
                pass
        if 'latency' in line.lower() or 'time per' in line.lower():
            try:
                parts = line.split(':')
                if len(parts) > 1:
                    value = float(parts[1].strip().split()[0])
                    results['latency_us'] = value
            except:
                pass
    
    return results


def parse_python_concurrent_output(output: str) -> Dict[str, Any]:
    """Extract metrics from Python concurrent benchmark."""
    return parse_python_single_thread_output(output)


def benchmark_phase(phase_name: str, commit: str = None) -> Dict[str, Any]:
    """Benchmark all modes for a given phase."""
    print(f"\n{'='*70}")
    print(f"BENCHMARKING PHASE: {phase_name}")
    print(f"{'='*70}")
    
    # Checkout commit if specified
    if commit:
        if not checkout_commit(commit):
            return {}
    
    commit_hash = get_current_commit()
    print(f"Current commit: {commit_hash}")
    
    # Build
    if not build_release():
        print("Build failed, skipping phase")
        return {}
    
    results = {
        "phase": phase_name,
        "commit": commit_hash,
        "timestamp": datetime.now().isoformat(),
        "modes": {},
    }
    
    # Run Rust single-threaded
    print("\n  RUST SINGLE-THREADED")
    results["modes"]["rust_single_thread"] = run_rust_single_thread_benchmark()
    
    # Run Rust concurrent
    print("\n  RUST CONCURRENT (rayon)")
    results["modes"]["rust_concurrent"] = run_rust_concurrent_benchmark()
    
    # Run Python single-threaded
    print("\n  PYTHON SINGLE-THREADED")
    results["modes"]["python_single_thread"] = run_python_single_thread_benchmark()
    
    # Run Python concurrent
    print("\n  PYTHON CONCURRENT (ProducerConsumerPipeline)")
    results["modes"]["python_concurrent"] = run_python_concurrent_benchmark()
    
    return results


def save_results(phase_results: Dict[str, Any], phase_name: str) -> Path:
    """Save benchmark results to JSON file."""
    output_dir = REPO_ROOT / "docs" / "design" / "profiling" / f"BASELINE_{phase_name.upper()}"
    output_dir.mkdir(parents=True, exist_ok=True)
    
    output_file = output_dir / "benchmark_results.json"
    
    with open(output_file, 'w') as f:
        json.dump(phase_results, f, indent=2)
    
    print(f"\nResults saved to: {output_file}")
    return output_file


def main():
    """Run comprehensive benchmarks across all modes for both phases."""
    
    print("""
╔════════════════════════════════════════════════════════════════════╗
║  COMPREHENSIVE BENCHMARK: Phase 1 Optimization Impact Analysis    ║
║  Tests: Rust single/concurrent + Python single/concurrent        ║
║  Compares: Pre-Phase 1 baseline vs Post-Phase 1 current code     ║
╚════════════════════════════════════════════════════════════════════╝
    """)
    
    # Save current state
    print("Saving current state...")
    rc, _, err = run_command(["git", "stash"])
    if rc != 0 and err:
        print(f"Note: {err[:100]}")
    
    try:
        # Benchmark pre-Phase 1
        pre_results = benchmark_phase("PRE_PHASE_1", PHASE_1_PARENT)
        if pre_results:
            save_results(pre_results, "pre_phase1")
        
        # Benchmark post-Phase 1
        post_results = benchmark_phase("POST_PHASE_1", PHASE_1_COMMIT)
        if post_results:
            save_results(post_results, "post_phase1")
        
        # Return to original state
        print("\nRestoring original state...")
        run_command(["git", "checkout", "-"])
        
        # Print comparison summary
        if pre_results and post_results:
            print_comparison_summary(pre_results, post_results)
        
        return 0
    
    except Exception as e:
        print(f"\nERROR: {e}")
        print("Attempting to restore original state...")
        run_command(["git", "checkout", "-"])
        return 1


def print_comparison_summary(pre: Dict[str, Any], post: Dict[str, Any]):
    """Print summary of before/after improvements."""
    print(f"\n{'='*70}")
    print("PERFORMANCE COMPARISON SUMMARY")
    print(f"{'='*70}\n")
    
    for mode in pre.get("modes", {}):
        pre_mode = pre["modes"].get(mode, {})
        post_mode = post["modes"].get(mode, {})
        
        if not pre_mode or not post_mode:
            continue
        
        print(f"\n{mode.upper().replace('_', ' ')}")
        print("-" * 70)
        
        # Throughput comparison
        pre_tput = pre_mode.get("throughput_rec_sec")
        post_tput = post_mode.get("throughput_rec_sec")
        
        if pre_tput and post_tput:
            improvement = ((post_tput - pre_tput) / pre_tput) * 100
            print(f"  Throughput: {pre_tput:.0f} → {post_tput:.0f} rec/sec ({improvement:+.1f}%)")
        
        # Latency comparison
        pre_lat = pre_mode.get("latency_us")
        post_lat = post_mode.get("latency_us")
        
        if pre_lat and post_lat:
            improvement = ((pre_lat - post_lat) / pre_lat) * 100
            print(f"  Latency: {pre_lat:.2f} → {post_lat:.2f} µs ({improvement:+.1f}%)")


if __name__ == "__main__":
    sys.exit(main())
