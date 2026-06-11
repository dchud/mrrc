#!/usr/bin/env python3
"""
Measure wall-clock read throughput for MARC files via mrrc.

Reads each file through MARCReader(path) — the RustFile backend — and
reports records/second per repetition plus the median. Wall-clock numbers
from a quiet machine are the payoff measurement for read-path changes;
CI CodSpeed simulation tracks regressions but cannot back records/sec
claims.

Usage:
    uv run python scripts/bench_read_throughput.py FILE [FILE ...] [--repeat N]

The first repetition warms the OS page cache, so all repetitions measure
the parse path rather than disk; pass --include-first to keep it anyway.
"""

import argparse
import statistics
import sys
import time
from pathlib import Path

from mrrc import MARCReader


def read_all(path):
    """Drain one file; return (records, seconds)."""
    start = time.perf_counter()
    reader = MARCReader(str(path))
    count = 0
    while reader.read_record() is not None:
        count += 1
    elapsed = time.perf_counter() - start
    return count, elapsed


def bench_file(path, repeat, include_first):
    runs = []
    count = None
    for i in range(repeat + (0 if include_first else 1)):
        n, elapsed = read_all(path)
        if count is None:
            count = n
            if not include_first:
                continue  # discard the cache-warming repetition
        elif n != count:
            sys.exit(f"{path}: record count changed between runs ({count} vs {n})")
        runs.append(n / elapsed)
    return count, runs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("files", nargs="+", type=Path, help=".mrc files to read")
    parser.add_argument(
        "--repeat", type=int, default=5, help="measured repetitions per file (default 5)"
    )
    parser.add_argument(
        "--include-first",
        action="store_true",
        help="measure the first repetition instead of discarding it as cache warm-up",
    )
    args = parser.parse_args()

    for path in args.files:
        if not path.exists():
            sys.exit(f"No such file: {path}")
        count, runs = bench_file(path, args.repeat, args.include_first)
        median = statistics.median(runs)
        spread = (max(runs) - min(runs)) / median * 100
        print(f"{path}: {count:,} records")
        print(f"  runs (rec/s): {', '.join(f'{r:,.0f}' for r in runs)}")
        print(f"  median: {median:,.0f} rec/s  (spread {spread:.1f}%)")


if __name__ == "__main__":
    main()
