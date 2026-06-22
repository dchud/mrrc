#!/usr/bin/env python3
"""Parallel-throughput measurement for mrrc's GIL-releasing parse path.

mrrc's headline concurrency feature — releasing the GIL while parsing in
pure Rust — cannot be measured by the CodSpeed simulation gate, because
Valgrind serializes threads under simulation. This script is the
real-walltime path: it sweeps thread counts T = 1..cores with a
``ThreadPoolExecutor``, each task parsing an independent in-memory copy of
a synthetic MARC fixture via ``mrrc.MARCReader`` over ``bytes`` (the
``CursorBackend`` path, which holds no GIL during parsing), and reports the
median wall-clock throughput (records/sec) at each thread count plus the
speedup versus the single-thread baseline.

The numbers are wall-clock and machine-specific — NOT a portable
records/sec claim — and the fixtures are synthetic, so they exercise
parsing throughput rather than real-world MARC variety. Run on a quiet
machine and record the hardware (the printed header lists what to capture).

Usage:
    uv run maturin develop --release       # measure a release build
    uv run python scripts/parallel_throughput.py
    uv run python scripts/parallel_throughput.py --fixture tests/data/fixtures/5k_records.mrc
    uv run python scripts/parallel_throughput.py --gil-check
"""

from __future__ import annotations

import argparse
import os
import platform
import statistics
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import mrrc

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_FIXTURE = REPO_ROOT / "tests/data/fixtures/10k_records.mrc"


def machine_header() -> str:
    """Describe the host. Record the CPU model and RAM by hand — neither is
    reliably available from the stdlib (``platform.processor()`` is often
    blank on macOS)."""
    cores = os.cpu_count() or 1
    impl = platform.python_implementation()
    return (
        f"machine : {platform.platform()}\n"
        f"python  : {platform.python_version()} ({impl})\n"
        f"cpu     : {platform.processor() or '<record CPU model by hand>'} "
        f"- {cores} logical cores\n"
        f"NOTE    : also record the physical CPU model, core count, and RAM"
    )


def count_records(data: bytes) -> int:
    """Parse one in-memory copy of the fixture and return its record count.

    ``bytes`` input takes the CursorBackend path, which copies the buffer
    into Rust and releases the GIL while parsing.
    """
    return sum(1 for _ in mrrc.MARCReader(data))


def time_run(data: bytes, threads: int, tasks: int) -> tuple[int, float]:
    """Parse ``tasks`` fixture copies across ``threads`` workers; return
    (total_records, wall_seconds)."""
    payloads = [data] * tasks
    start = time.perf_counter()
    with ThreadPoolExecutor(max_workers=threads) as pool:
        total = sum(pool.map(count_records, payloads))
    return total, time.perf_counter() - start


def throughput_curve(
    data: bytes, runs: int, tasks_per_thread: int, max_threads: int
) -> None:
    """Print the records/sec-vs-threads speedup curve (weak scaling: each
    thread parses a fixed number of fixture copies, so total work grows with
    the thread count and ideal scaling is linear)."""
    per_copy = count_records(data)
    print(machine_header())
    print(
        f"\nfixture : {per_copy} records/copy, {len(data):,} bytes"
        f"\nmethod  : {tasks_per_thread} copies/thread, median of {runs} timed"
        f" runs (1 warmup discarded)\n"
    )
    print(f"{'threads':>7}  {'records/s':>13}  {'speedup':>8}")
    print(f"{'-' * 7:>7}  {'-' * 13:>13}  {'-' * 8:>8}")
    baseline: float | None = None
    for threads in range(1, max_threads + 1):
        tasks = threads * tasks_per_thread
        time_run(data, threads, tasks)  # warmup, untimed
        rates = []
        for _ in range(runs):
            total, secs = time_run(data, threads, tasks)
            rates.append(total / secs)
        rps = statistics.median(rates)
        if baseline is None:
            baseline = rps
        print(f"{threads:>7}  {rps:>13,.0f}  {rps / baseline:>7.2f}x")


def gil_check(data: bytes, window: float) -> None:
    """GIL-release detector. One thread parses in a loop (the parse releases
    the GIL); a second thread spins a pure-Python counter. A high counter
    total means the counter thread ran *during* the parses, i.e. the GIL was
    available — a yes/no detector, not a throughput number."""
    stop = threading.Event()
    counter = 0

    def spin() -> None:
        nonlocal counter
        while not stop.is_set():
            counter += 1

    def parse_loop() -> None:
        while not stop.is_set():
            for _ in mrrc.MARCReader(data):
                pass

    print(machine_header())
    print(
        f"\nGIL-release detector: {window:.0f}s window,"
        f" 1 parse thread + 1 pure-Python counter thread"
    )
    spinner = threading.Thread(target=spin)
    parser = threading.Thread(target=parse_loop)
    spinner.start()
    parser.start()
    time.sleep(window)
    stop.set()
    spinner.join()
    parser.join()
    print(f"counter iterations during parsing: {counter:,}")
    print(
        "higher = GIL was more available during parsing; the parser holds the"
        " GIL only to hand records back to Python"
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="mrrc parallel-throughput measurement (real wall-clock)"
    )
    parser.add_argument("--fixture", type=Path, default=DEFAULT_FIXTURE)
    parser.add_argument("--runs", type=int, default=7, help="timed runs per thread count")
    parser.add_argument("--tasks-per-thread", type=int, default=4)
    parser.add_argument(
        "--max-threads", type=int, default=os.cpu_count() or 1
    )
    parser.add_argument(
        "--gil-check",
        action="store_true",
        help="run the GIL-release detector instead of the throughput curve",
    )
    parser.add_argument(
        "--window", type=float, default=2.0, help="--gil-check window, seconds"
    )
    args = parser.parse_args()

    if not args.fixture.exists():
        print(
            f"fixture not found: {args.fixture}\n"
            "regenerate with scripts/generate_benchmark_fixtures.py",
            file=sys.stderr,
        )
        return 1
    data = args.fixture.read_bytes()

    if args.gil_check:
        gil_check(data, args.window)
    else:
        throughput_curve(
            data, args.runs, args.tasks_per_thread, args.max_threads
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
