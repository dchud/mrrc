#!/usr/bin/env python3
"""Compare wall-clock throughput of mrrc (pymrrc) against pymarc.

This is the Python-to-Python comparison behind any "Nx pymarc" claim: it
measures the mrrc Python wrapper against a pinned pymarc over identical
fixtures and operations, and emits a Markdown report carrying the full
run context that ``docs/benchmarks/results.md`` requires for a citable
figure.

Absolute Rust-library throughput is a *separate* measurement
(``cargo bench --bench marc_benchmarks``). Rust-vs-Python is not an
apples-to-apples comparison, so it is deliberately out of scope here.

Operations, each applied to every record in the file:

* ``read``      -- parse every record, no field access.
* ``extract``   -- parse, then touch fields the way a real pymarc loop
  does (``record.title`` plus ``field.value()`` for every field), the
  path where the wrapper's field-handle protocol costs the most.
* ``roundtrip`` -- parse, then re-encode each record with ``as_marc()``.

For each (fixture, operation, library) the harness runs ``--repeat``
measured repetitions after discarding one cache-warming run, and reports
the median records/second plus the run-to-run spread. It checks that both
libraries read the same record count from each fixture before comparing.

Wall-clock numbers are only worth citing from a quiet machine: AC power,
networking off, other apps closed. CI CodSpeed simulation models
instruction counts, not wall-clock, and cannot back a records/second
figure.

Usage::

    uv run --with 'pymarc>=5.3' python scripts/benchmark_comparison.py \\
        tests/data/fixtures/1k_records.mrc \\
        tests/data/fixtures/10k_records.mrc \\
        --repeat 7 --output docs/benchmarks/comparison-results.md
"""

from __future__ import annotations

import argparse
import datetime
import importlib.metadata
import os
import platform
import statistics
import subprocess
import sys
import time
from pathlib import Path

import mrrc
from mrrc import MARCReader as MrrcReader
from mrrc import RecordBoundaryScanner, parse_batch_parallel

try:
    import pymarc
except ImportError:
    sys.exit(
        "pymarc is not installed. Run the harness with it available, e.g.:\n"
        "  uv run --with 'pymarc>=5.3' python "
        "scripts/benchmark_comparison.py FILE [FILE ...]"
    )


# --- operations -----------------------------------------------------------
#
# Each operation returns (record_count, elapsed_seconds) for one full pass
# over the file. The two libraries do the same work per record so the
# comparison is fair.


def mrrc_read(path: Path) -> tuple[int, float]:
    start = time.perf_counter()
    count = 0
    for _record in MrrcReader(str(path)):
        count += 1
    return count, time.perf_counter() - start


def mrrc_read_bulk(path: Path) -> tuple[int, float]:
    # mrrc's fastest read path: scan record boundaries and parse the whole
    # file in one parallel Rust call (rayon), instead of one record per Python
    # iteration. pymarc has no batch equivalent, so this is compared against
    # pymarc's per-record read — each library at its best.
    start = time.perf_counter()
    with open(path, "rb") as handle:
        buffer = handle.read()
    boundaries = RecordBoundaryScanner().scan(buffer)
    records = parse_batch_parallel(boundaries, buffer)
    return len(records), time.perf_counter() - start


def mrrc_extract(path: Path) -> tuple[int, float]:
    start = time.perf_counter()
    count = 0
    acc = 0
    for record in MrrcReader(str(path)):
        count += 1
        title = record.title
        if title:
            acc += len(title)
        for field in record.get_fields():
            acc += len(field.value())
    return count, time.perf_counter() - start


def mrrc_roundtrip(path: Path) -> tuple[int, float]:
    start = time.perf_counter()
    count = 0
    acc = 0
    for record in MrrcReader(str(path)):
        count += 1
        acc += len(record.as_marc())
    return count, time.perf_counter() - start


def pymarc_read(path: Path) -> tuple[int, float]:
    start = time.perf_counter()
    count = 0
    with open(path, "rb") as handle:
        for record in pymarc.MARCReader(handle):
            if record is not None:
                count += 1
    return count, time.perf_counter() - start


def pymarc_extract(path: Path) -> tuple[int, float]:
    start = time.perf_counter()
    count = 0
    acc = 0
    with open(path, "rb") as handle:
        for record in pymarc.MARCReader(handle):
            if record is None:
                continue
            count += 1
            title = record.title
            if title:
                acc += len(title)
            for field in record.get_fields():
                acc += len(field.value())
    return count, time.perf_counter() - start


def pymarc_roundtrip(path: Path) -> tuple[int, float]:
    start = time.perf_counter()
    count = 0
    acc = 0
    with open(path, "rb") as handle:
        for record in pymarc.MARCReader(handle):
            if record is None:
                continue
            count += 1
            acc += len(record.as_marc())
    return count, time.perf_counter() - start


OPERATIONS = {
    "read": (mrrc_read, pymarc_read),
    "read_bulk": (mrrc_read_bulk, pymarc_read),
    "extract": (mrrc_extract, pymarc_extract),
    "roundtrip": (mrrc_roundtrip, pymarc_roundtrip),
}


# --- measurement ----------------------------------------------------------


def measure(op, path: Path, repeat: int, include_first: bool):
    """Return (count, [rec/s, ...]) over `repeat` measured repetitions."""
    rates: list[float] = []
    count: int | None = None
    total = repeat + (0 if include_first else 1)
    for i in range(total):
        n, elapsed = op(path)
        if not include_first and i == 0:
            count = n
            continue  # discard the cache-warming repetition
        if count is None:
            count = n
        elif n != count:
            sys.exit(
                f"{path}: record count changed between runs "
                f"({count} vs {n}); aborting"
            )
        rates.append(n / elapsed if elapsed else 0.0)
    return count, rates


# --- run context ----------------------------------------------------------


def _sysctl(key: str) -> str | None:
    try:
        out = subprocess.run(
            ["sysctl", "-n", key],
            capture_output=True,
            text=True,
            check=True,
        )
        return out.stdout.strip() or None
    except (OSError, subprocess.SubprocessError):
        return None


def cpu_model() -> str:
    if sys.platform == "darwin":
        return (
            _sysctl("machdep.cpu.brand_string")
            or _sysctl("hw.model")
            or platform.processor()
            or "unknown"
        )
    if sys.platform.startswith("linux"):
        try:
            for line in Path("/proc/cpuinfo").read_text().splitlines():
                if line.startswith("model name"):
                    return line.split(":", 1)[1].strip()
        except OSError:
            pass
    return platform.processor() or platform.machine() or "unknown"


def memory_gib() -> str:
    total: int | None = None
    if sys.platform == "darwin":
        raw = _sysctl("hw.memsize")
        total = int(raw) if raw and raw.isdigit() else None
    elif sys.platform.startswith("linux"):
        try:
            pages = os.sysconf("SC_PHYS_PAGES")
            page_size = os.sysconf("SC_PAGE_SIZE")
            total = pages * page_size
        except (ValueError, OSError):
            total = None
    return f"{total / 1024**3:.1f} GiB" if total else "unknown"


def rust_version() -> str:
    try:
        out = subprocess.run(
            ["rustc", "--version"],
            capture_output=True,
            text=True,
            check=True,
        )
        return out.stdout.strip() or "unknown"
    except (OSError, subprocess.SubprocessError):
        return "unknown"


def lib_version(name: str, fallback: str = "unknown") -> str:
    try:
        return importlib.metadata.version(name)
    except importlib.metadata.PackageNotFoundError:
        return fallback


def run_context(now: datetime.datetime) -> dict[str, str]:
    return {
        "Date": now.isoformat(timespec="seconds"),
        "Hardware": f"{cpu_model()}, {os.cpu_count()} logical cores, "
        f"{memory_gib()} RAM",
        "OS": platform.platform(),
        "Python": platform.python_version(),
        "Rust toolchain": rust_version(),
        "mrrc": lib_version("mrrc", getattr(mrrc, "__version__", "unknown")),
        "pymarc": lib_version("pymarc", getattr(pymarc, "__version__", "?")),
    }


# --- reporting ------------------------------------------------------------


def human_size(path: Path) -> str:
    size = path.stat().st_size
    for unit in ("B", "KB", "MB", "GB"):
        if size < 1024 or unit == "GB":
            return f"{size:.0f} {unit}" if unit == "B" else f"{size:.1f} {unit}"
        size /= 1024
    return f"{size:.1f} GB"


def render_markdown(context, fixtures, results, repeat) -> str:
    lines = [
        f"## mrrc vs pymarc — {context['Date'][:10]}",
        "",
        "**Run context**",
        "",
    ]
    lines += [f"- {key}: {value}" for key, value in context.items()]
    lines += [
        f"- Method: median of {repeat} measured repetitions per cell "
        "(one cache-warming run discarded); wall-clock records/second on the "
        "host above — a working machine, not a dedicated benchmark rig, so "
        "treat the figures as representative of the relative speedup rather "
        "than a sterile maximum.",
        "- Comparison is Python-to-Python (mrrc wrapper vs pymarc). Absolute "
        "Rust throughput is measured separately via "
        "`cargo bench --bench marc_benchmarks`.",
        "- `read` = per-record iteration (`for r in reader`), the pymarc-shaped "
        "path. `read_bulk` = mrrc's parallel `parse_batch_parallel` vs pymarc's "
        "per-record read — each library's fastest read path.",
        "",
    ]
    for fixture in fixtures:
        path = fixture["path"]
        lines += [
            f"### {path.name} — {fixture['count']:,} records "
            f"({human_size(path)})",
            "",
            "| Operation | mrrc (rec/s) | pymarc (rec/s) | speedup |",
            "|-----------|-------------:|---------------:|--------:|",
        ]
        for op_name in fixture["ops"]:
            cell = results[(path, op_name)]
            speed = (
                f"{cell['mrrc'] / cell['pymarc']:.2f}x"
                if cell["pymarc"]
                else "n/a"
            )
            lines.append(
                f"| {op_name} | {cell['mrrc']:,.0f} | "
                f"{cell['pymarc']:,.0f} | {speed} |"
            )
        lines.append("")
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "files", nargs="+", type=Path, help=".mrc fixtures to measure"
    )
    parser.add_argument(
        "--repeat",
        type=int,
        default=7,
        help="measured repetitions per cell (default 7)",
    )
    parser.add_argument(
        "--ops",
        default="read,read_bulk,extract,roundtrip",
        help="comma-separated operations (default: all)",
    )
    parser.add_argument(
        "--include-first",
        action="store_true",
        help="measure the first (cache-warming) repetition instead of "
        "discarding it",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="write the Markdown report here (also printed to stdout)",
    )
    args = parser.parse_args()

    ops = [op.strip() for op in args.ops.split(",") if op.strip()]
    unknown = [op for op in ops if op not in OPERATIONS]
    if unknown:
        sys.exit(f"Unknown operation(s): {', '.join(unknown)}")
    for path in args.files:
        if not path.exists():
            sys.exit(f"No such file: {path}")

    now = datetime.datetime.now().astimezone()
    context = run_context(now)
    print("Run context:", file=sys.stderr)
    for key, value in context.items():
        print(f"  {key}: {value}", file=sys.stderr)

    fixtures = []
    results: dict[tuple[Path, str], dict[str, float]] = {}
    for path in args.files:
        print(f"\n{path} ({human_size(path)})", file=sys.stderr)
        counts: set[int] = set()
        for op_name in ops:
            mrrc_op, pymarc_op = OPERATIONS[op_name]
            m_count, m_rates = measure(
                mrrc_op, path, args.repeat, args.include_first
            )
            p_count, p_rates = measure(
                pymarc_op, path, args.repeat, args.include_first
            )
            counts.update({m_count, p_count})
            m_median = statistics.median(m_rates)
            p_median = statistics.median(p_rates)
            results[(path, op_name)] = {
                "mrrc": m_median,
                "pymarc": p_median,
            }
            speed = m_median / p_median if p_median else float("nan")
            print(
                f"  {op_name:<10} mrrc {m_median:>12,.0f}  "
                f"pymarc {p_median:>12,.0f}  ({speed:.2f}x)",
                file=sys.stderr,
            )
            if m_count != p_count:
                print(
                    f"  WARNING: record counts differ for {op_name} "
                    f"(mrrc {m_count} vs pymarc {p_count}); the comparison "
                    "for this fixture is not apples-to-apples.",
                    file=sys.stderr,
                )
        fixtures.append(
            {"path": path, "count": max(counts), "ops": ops}
        )

    report = render_markdown(context, fixtures, results, args.repeat)
    print("\n" + report)
    if args.output:
        args.output.write_text(report + "\n")
        print(f"\nWrote {args.output}", file=sys.stderr)


if __name__ == "__main__":
    main()
