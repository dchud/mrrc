#!/usr/bin/env python3
"""
Simple comprehensive benchmark of all modes pre/post Phase 1.

Focuses on extracting throughput and latency metrics from existing benchmark tools.
"""

import subprocess
import json
import sys
import os
import re
from pathlib import Path
from datetime import datetime
from typing import Dict, Any, Tuple
import time

REPO_ROOT = Path(__file__).parent.parent
PHASE_1_COMMIT = "b1b2ac17"
PHASE_1_PARENT = f"{PHASE_1_COMMIT}~1"

# ============================================================================
# UTILITIES
# ============================================================================

def run_command(cmd, cwd=None, capture=True):
    """Run a shell command."""
    if cwd is None:
        cwd = REPO_ROOT
    
    result = subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=capture,
        text=True,
        shell=isinstance(cmd, str),
    )
    return result.returncode, result.stdout, result.stderr


def checkout_commit(commit):
    """Checkout a specific commit."""
    print(f"  Checking out {commit}...")
    rc, _, err = run_command(["git", "checkout", commit])
    if rc != 0:
        print(f"    ERROR: {err}")
        return False
    time.sleep(0.5)
    return True


def get_current_commit():
    """Get current HEAD commit hash."""
    rc, stdout, _ = run_command(["git", "rev-parse", "HEAD"])
    return stdout.strip() if rc == 0 else "unknown"


def build_release():
    """Build release binary."""
    print("  Building release...")
    rc, _, err = run_command(["cargo", "build", "--release"])
    if rc != 0:
        print(f"    ERROR: {err[:200]}")
        return False
    return True


def build_python_extension():
    """Build Python extension."""
    print("  Building Python extension...")
    rc, _, err = run_command(["maturin", "develop"], cwd=REPO_ROOT / "src-python")
    if rc != 0:
        print(f"    WARNING: {err[:100]}")
        # Don't fail, might be cached
    return True


def run_rust_single_thread():
    """Run Rust single-threaded benchmark."""
    print("    Rust single-threaded...", end=" ", flush=True)
    rc, stdout, err = run_command(["cargo", "bench", "--bench", "profiling_harness", "--", "--nocapture"])
    
    if rc != 0:
        print(f"ERROR: {err[:100]}")
        return {}
    
    # Extract metrics
    metrics = {}
    
    # Find average throughput
    match = re.search(r"Average throughput:\s*([\d.]+)\s*rec/sec", stdout)
    if match:
        metrics["throughput_rec_sec"] = float(match.group(1))
    
    # Find average latency
    match = re.search(r"Average time per record:\s*([\d.]+)\s*µs", stdout)
    if match:
        metrics["latency_us"] = float(match.group(1))
    
    if metrics:
        print(f"✓ {metrics.get('throughput_rec_sec', 0):.0f} rec/sec, {metrics.get('latency_us', 0):.2f} µs/rec")
    else:
        print("ERROR: No metrics extracted")
    
    return metrics


def run_rust_concurrent():
    """Run Rust concurrent benchmark."""
    print("    Rust concurrent...", end=" ", flush=True)
    rc, stdout, err = run_command(["cargo", "bench", "--bench", "parallel_benchmarks", "--", "--nocapture"])
    
    if rc != 0:
        print(f"ERROR: {err[:100]}")
        return {}
    
    metrics = {}
    
    # Extract metrics from parallel_benchmarks output
    match = re.search(r"Average throughput.*?:\s*([\d.]+)", stdout)
    if match:
        metrics["throughput_rec_sec"] = float(match.group(1))
    
    match = re.search(r"Average time per record.*?:\s*([\d.]+)", stdout)
    if match:
        metrics["latency_us"] = float(match.group(1))
    
    if metrics:
        print(f"✓ {metrics.get('throughput_rec_sec', 0):.0f} rec/sec, {metrics.get('latency_us', 0):.2f} µs/rec")
    else:
        print("ERROR: No metrics extracted")
    
    return metrics


def run_python_single_thread():
    """Run Python single-threaded benchmark."""
    print("    Python single-threaded...", end=" ", flush=True)
    
    script = REPO_ROOT / "scripts" / "profile_pymrrc_single_thread.py"
    rc, stdout, err = run_command(["python3", str(script)])
    
    if rc != 0:
        print(f"ERROR: {err[:100]}")
        return {}
    
    metrics = {}
    
    # Look for throughput in output
    match = re.search(r"Throughput:\s*([\d.]+)\s*rec/s", stdout)
    if match:
        metrics["throughput_rec_sec"] = float(match.group(1))
    
    # Look for JSON output file
    json_file = REPO_ROOT / ".benchmarks" / "pymrrc_single_thread_profile.json"
    if json_file.exists():
        try:
            with open(json_file) as f:
                data = json.load(f)
                # Try to extract metrics from JSON
                if "summary" in data:
                    metrics.update(data["summary"])
        except:
            pass
    
    if metrics:
        print(f"✓ {metrics.get('throughput_rec_sec', 0):.0f} rec/sec")
    else:
        print("ERROR: No metrics extracted")
    
    return metrics


def run_python_concurrent():
    """Run Python concurrent benchmark."""
    print("    Python concurrent...", end=" ", flush=True)
    
    script = REPO_ROOT / "scripts" / "profile_pymrrc_concurrent.py"
    rc, stdout, err = run_command(["python3", str(script)])
    
    if rc != 0:
        print(f"ERROR: {err[:100]}")
        return {}
    
    metrics = {}
    
    # Look for JSON output file
    json_file = REPO_ROOT / ".benchmarks" / "pymrrc_concurrent_profile.json"
    if json_file.exists():
        try:
            with open(json_file) as f:
                data = json.load(f)
                if "summary" in data:
                    metrics.update(data["summary"])
        except:
            pass
    
    if metrics:
        print(f"✓ {metrics.get('throughput_rec_sec', 0):.0f} rec/sec")
    else:
        print("ERROR: No metrics extracted")
    
    return metrics


def benchmark_phase(phase_name, commit=None):
    """Benchmark all modes for a phase."""
    print(f"\n{'='*70}")
    print(f"PHASE: {phase_name}")
    print(f"{'='*70}")
    
    if commit:
        if not checkout_commit(commit):
            return {}
    
    commit_hash = get_current_commit()
    print(f"Commit: {commit_hash}")
    
    # Build
    if not build_release():
        return {}
    
    build_python_extension()
    
    results = {
        "phase": phase_name,
        "commit": commit_hash,
        "timestamp": datetime.now().isoformat(),
        "modes": {},
    }
    
    # Run each mode
    results["modes"]["rust_single_thread"] = run_rust_single_thread()
    results["modes"]["rust_concurrent"] = run_rust_concurrent()
    results["modes"]["python_single_thread"] = run_python_single_thread()
    results["modes"]["python_concurrent"] = run_python_concurrent()
    
    return results


def save_results(results, phase_name):
    """Save results to JSON."""
    output_dir = REPO_ROOT / "docs" / "design" / "profiling" / f"BASELINE_{phase_name.upper()}"
    output_dir.mkdir(parents=True, exist_ok=True)
    
    output_file = output_dir / "benchmark_results.json"
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"  Saved: {output_file}")
    return output_file


def print_summary(pre, post):
    """Print before/after comparison."""
    print(f"\n{'='*70}")
    print("COMPARISON SUMMARY")
    print(f"{'='*70}\n")
    
    for mode in ["rust_single_thread", "rust_concurrent", "python_single_thread", "python_concurrent"]:
        pre_mode = pre.get("modes", {}).get(mode, {})
        post_mode = post.get("modes", {}).get(mode, {})
        
        if not pre_mode or not post_mode:
            continue
        
        mode_name = mode.upper().replace("_", " ")
        print(f"{mode_name:40s}", end="")
        
        pre_tput = pre_mode.get("throughput_rec_sec")
        post_tput = post_mode.get("throughput_rec_sec")
        
        if pre_tput and post_tput:
            improvement = ((post_tput - pre_tput) / pre_tput) * 100
            print(f"{improvement:+.1f}%")
        else:
            print("N/A")


def main():
    print("""
╔════════════════════════════════════════════════════════════════════╗
║  BENCHMARK: Phase 1 Optimization Impact (All Modes)              ║
║  Pre-Phase 1 vs Post-Phase 1                                       ║
╚════════════════════════════════════════════════════════════════════╝
    """)
    
    # Save current state
    print("Saving current state...")
    run_command(["git", "stash"])
    
    try:
        # Benchmark pre-Phase 1
        print("\n" + "="*70)
        print("PRE-PHASE 1 BASELINE")
        print("="*70)
        pre_results = benchmark_phase("PRE_PHASE_1", PHASE_1_PARENT)
        if pre_results:
            save_results(pre_results, "pre_phase1")
        
        # Benchmark post-Phase 1
        print("\n" + "="*70)
        print("POST-PHASE 1 CURRENT")
        print("="*70)
        post_results = benchmark_phase("POST_PHASE_1", PHASE_1_COMMIT)
        if post_results:
            save_results(post_results, "post_phase1")
        
        # Return to original state
        print("\nRestoring original state...")
        run_command(["git", "checkout", "-"])
        
        # Print summary
        if pre_results and post_results:
            print_summary(pre_results, post_results)
        
        print("\nBenchmarking complete!")
        return 0
    
    except Exception as e:
        print(f"\nERROR: {e}")
        run_command(["git", "checkout", "-"])
        return 1


if __name__ == "__main__":
    sys.exit(main())
