#!/usr/bin/env python3
"""
Measure ISO 2709 (MARC binary) baseline performance.

NOTE: This script uses Python pymarc to measure baseline performance.
For Rust-native mrrc evaluation, use: cargo bench --bench marc_benchmarks

Metrics:
- Read throughput (records/sec)
- Write throughput (records/sec)
- File size (raw, gzipped, compression ratio)
- Peak memory usage

Output: docs/design/format-research/BASELINE_ISO2709.md

EVALUATION FOCUS: Rust mrrc is the primary implementation platform.
All binary format evaluations should measure Rust performance using the same environment.
Python measurements are secondary and used only for infrastructure validation.
"""

import subprocess
import sys
import time
import os
import platform
import gzip
from pathlib import Path

# Add src-python to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'src-python'))

from pymarc import MARCReader, MARCWriter
from io import BytesIO


def get_system_info() -> dict:
    """Collect system information."""
    info = {}
    
    # OS
    info['os_name'] = platform.system()
    info['os_version'] = platform.release()
    
    # Architecture
    info['architecture'] = platform.machine()
    
    # CPU info
    if platform.system() == 'Darwin':  # macOS
        try:
            info['cpu'] = subprocess.check_output(['sysctl', '-n', 'machdep.cpu.brand_string']).decode().strip()
            info['cores'] = subprocess.check_output(['sysctl', '-n', 'hw.physicalcpu']).decode().strip()
            ram_bytes = int(subprocess.check_output(['sysctl', '-n', 'hw.memsize']).decode().strip())
            info['ram_gb'] = f"{ram_bytes / (1024**3):.1f}"
        except Exception as e:
            info['cpu'] = f"Error: {e}"
    else:  # Linux
        try:
            with open('/proc/cpuinfo') as f:
                for line in f:
                    if line.startswith('model name'):
                        info['cpu'] = line.split(':')[1].strip()
                        break
            with open('/proc/cpuinfo') as f:
                info['cores'] = sum(1 for line in f if line.startswith('processor'))
            with open('/proc/meminfo') as f:
                for line in f:
                    if line.startswith('MemTotal'):
                        ram_kb = int(line.split(':')[1].strip().split()[0])
                        info['ram_gb'] = f"{ram_kb / (1024**2):.1f}"
                        break
        except Exception as e:
            info['cpu'] = f"Error: {e}"
    
    # Python version
    info['python'] = platform.python_version()
    
    # Storage type (best effort)
    info['storage'] = 'Unknown'
    if Path('/').stat().st_dev != Path('/tmp').stat().st_dev:
        info['storage'] = 'SSD/Fast Storage'
    
    return info


def measure_read_throughput(test_file: Path, num_records: int) -> float:
    """Measure read throughput in records/sec."""
    with open(test_file, 'rb') as f:
        reader = MARCReader(f)
        
        start = time.perf_counter()
        count = 0
        for record in reader:
            count += 1
        end = time.perf_counter()
    
    elapsed = end - start
    throughput = count / elapsed if elapsed > 0 else 0
    return throughput


def measure_write_throughput(test_file: Path, num_records: int) -> float:
    """Measure write throughput in records/sec."""
    # Read records first
    with open(test_file, 'rb') as f:
        reader = MARCReader(f)
        records = list(reader)
    
    # Write to memory
    start = time.perf_counter()
    output = BytesIO()
    writer = MARCWriter(output)
    for record in records:
        writer.write(record)
    end = time.perf_counter()
    
    elapsed = end - start
    throughput = len(records) / elapsed if elapsed > 0 else 0
    return throughput


def measure_compression(test_file: Path) -> tuple:
    """Measure compression ratio and gzip performance."""
    with open(test_file, 'rb') as f:
        raw_data = f.read()
    
    raw_size = len(raw_data)
    
    start = time.perf_counter()
    gzipped = gzip.compress(raw_data, compresslevel=9)
    gzip_time = time.perf_counter() - start
    
    gzip_size = len(gzipped)
    ratio = (1 - gzip_size / raw_size) * 100 if raw_size > 0 else 0
    
    return raw_size, gzip_size, ratio, gzip_time


def format_size(bytes: int) -> str:
    """Format bytes as human-readable size."""
    if bytes < 1024:
        return f"{bytes} B"
    elif bytes < 1024**2:
        return f"{bytes / 1024:.2f} KB"
    elif bytes < 1024**3:
        return f"{bytes / (1024**2):.2f} MB"
    else:
        return f"{bytes / (1024**3):.2f} GB"


def main():
    test_file = Path(__file__).parent.parent / 'tests' / 'data' / 'fixtures' / '10k_records.mrc'
    output_file = Path(__file__).parent.parent / 'docs' / 'design' / 'format-research' / 'BASELINE_ISO2709.md'
    
    if not test_file.exists():
        print(f"Error: Test file not found: {test_file}")
        sys.exit(1)
    
    output_file.parent.mkdir(parents=True, exist_ok=True)
    
    print("Measuring ISO 2709 baseline performance...")
    print(f"Test file: {test_file}")
    print()
    
    # Get system info
    sys_info = get_system_info()
    
    # Count records
    with open(test_file, 'rb') as f:
        reader = MARCReader(f)
        num_records = sum(1 for _ in reader)
    
    print(f"Total records: {num_records:,}")
    
    # Measure performance
    print("Measuring read throughput...", end='', flush=True)
    read_throughput = measure_read_throughput(test_file, num_records)
    print(f" ✓ {read_throughput:.0f} records/sec")
    
    print("Measuring write throughput...", end='', flush=True)
    write_throughput = measure_write_throughput(test_file, num_records)
    print(f" ✓ {write_throughput:.0f} records/sec")
    
    print("Measuring compression...", end='', flush=True)
    raw_size, gzip_size, ratio, gzip_time = measure_compression(test_file)
    print(f" ✓ {ratio:.1f}% compression")
    
    print()
    
    # Generate markdown report
    report = f"""# ISO 2709 Baseline Performance

Baseline performance measurements for ISO 2709 (MARC binary format) on the mrrc library.

## Test Dataset

- **File:** `tests/data/fixtures/10k_records.mrc`
- **Records:** {num_records:,}
- **Size:** {format_size(raw_size)} (uncompressed)

## System Environment

| Property | Value |
|----------|-------|
| **OS** | {sys_info.get('os_name', 'Unknown')} {sys_info.get('os_version', '')} |
| **CPU** | {sys_info.get('cpu', 'Unknown')} |
| **Cores** | {sys_info.get('cores', 'Unknown')} |
| **RAM** | {sys_info.get('ram_gb', 'Unknown')} GB |
| **Architecture** | {sys_info.get('architecture', 'Unknown')} |
| **Storage** | {sys_info.get('storage', 'Unknown')} |
| **Python** | {sys_info.get('python', 'Unknown')} |

## Performance Metrics

| Metric | Value |
|--------|-------|
| **Read throughput** | {read_throughput:.0f} records/sec |
| **Write throughput** | {write_throughput:.0f} records/sec |
| **File size (raw)** | {format_size(raw_size)} |
| **File size (gzipped)** | {format_size(gzip_size)} |
| **Compression ratio** | {ratio:.1f}% |
| **Gzip time** | {gzip_time:.2f}s |

## Interpretation

- **Read throughput:** Lower is slower; higher is faster
- **Write throughput:** Lower is slower; higher is faster
- **Compression ratio:** Higher percentage = more compressible (better for storage)
- All other format evaluations will be compared against these metrics

## Date & Commit

- **Measured:** {time.strftime('%Y-%m-%d %H:%M:%S')}
- **Commit:** {subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=Path(__file__).parent.parent).decode().strip()[:7] if subprocess.run(['git', 'rev-parse', 'HEAD'], capture_output=True, cwd=Path(__file__).parent.parent).returncode == 0 else 'N/A'}

---

This baseline is frozen and used as the reference for all subsequent format evaluations.
Retroactive adjustments to this baseline are not permitted (to prevent cherry-picking results).
"""
    
    # Write report
    with open(output_file, 'w') as f:
        f.write(report)
    
    print(f"✓ Baseline documented: {output_file}")
    print()
    print("Performance Summary:")
    print(f"  Read:  {read_throughput:.0f} records/sec")
    print(f"  Write: {write_throughput:.0f} records/sec")
    print(f"  Compression: {ratio:.1f}%")


if __name__ == '__main__':
    main()
