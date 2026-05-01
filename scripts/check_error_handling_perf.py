#!/usr/bin/env python3
"""Compare current error-handling bench numbers against a baseline.

Two modes, sharing the threshold-evaluation logic:

* **Default (developer-local):** baseline is the means recorded in a
  JSON file (benches/baselines/error_handling_v080.json by default).
  Runs cargo bench in the working tree, reads target/criterion, and
  compares. Use this on the machine class that captured the baseline
  (cross-machine comparison is not statistically reliable at the
  2%/5% threshold).

* **CI same-runner before/after:** baseline is a criterion output
  directory captured on the same CI runner from a different commit
  (typically `main`). Use ``--baseline-criterion-dir`` plus
  ``--current-criterion-dir`` to compare two such directories
  directly. The CI workflow checks out main, runs benches, captures
  target/criterion, then checks out the PR HEAD, runs benches again,
  and invokes this script with both paths.

Exit codes are the same in either mode:
    0   all scenarios within tolerance (≤ warn threshold)
    1   one or more scenarios exceed warn threshold (default 2%) but
        none exceed fail threshold
    2   one or more scenarios exceed fail threshold (default 5%)
    3   bench output missing or unreadable
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BASELINE = REPO_ROOT / "benches" / "baselines" / "error_handling_v080.json"


def run_benches() -> None:
    """Invoke `cargo bench --bench error_handling_benchmarks`. Raises if
    cargo is missing or the bench run exits non-zero."""
    if shutil.which("cargo") is None:
        sys.exit("cargo not found on PATH")
    cmd = ["cargo", "bench", "--bench", "error_handling_benchmarks"]
    print(f"$ {' '.join(cmd)}", flush=True)
    subprocess.run(cmd, cwd=REPO_ROOT, check=True)


def read_mean_ns_from_criterion_dir(criterion_dir: Path, scenario: str) -> float | None:
    estimates = criterion_dir / scenario / "new" / "estimates.json"
    if not estimates.exists():
        return None
    data = json.loads(estimates.read_text())
    return data["mean"]["point_estimate"]


def default_criterion_dir() -> Path:
    return REPO_ROOT / "target" / "criterion"


def fmt_ns_ms(ns: float) -> str:
    return f"{ns / 1e6:.3f} ms"


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument(
        "--baseline",
        type=Path,
        default=DEFAULT_BASELINE,
        help="path to the baseline JSON file (default: %(default)s)",
    )
    p.add_argument(
        "--baseline-criterion-dir",
        type=Path,
        default=None,
        help=(
            "use this criterion directory as the baseline instead of the JSON "
            "file. The script reads scenario names and thresholds from --baseline "
            "but takes the baseline mean for each scenario from this directory's "
            "estimates.json files. Use for same-runner before/after comparison."
        ),
    )
    p.add_argument(
        "--current-criterion-dir",
        type=Path,
        default=None,
        help=(
            "read current bench numbers from this directory instead of "
            "target/criterion. Implies --no-bench."
        ),
    )
    p.add_argument(
        "--no-bench",
        action="store_true",
        help="skip running cargo bench; read existing target/criterion output",
    )
    args = p.parse_args()

    if not args.baseline.exists():
        return _exit(3, f"baseline file not found: {args.baseline}")

    baseline = json.loads(args.baseline.read_text())
    warn_pct = baseline["thresholds"]["warn_pct"]
    fail_pct = baseline["thresholds"]["fail_pct"]
    scenarios: dict[str, dict] = baseline["scenarios"]

    current_dir = args.current_criterion_dir or default_criterion_dir()
    if args.current_criterion_dir is None and not args.no_bench:
        run_benches()

    rows: list[tuple[str, float | None, float | None, float | None]] = []
    worst_pct = 0.0
    for name in scenarios:
        if args.baseline_criterion_dir is not None:
            baseline_mean = read_mean_ns_from_criterion_dir(args.baseline_criterion_dir, name)
        else:
            baseline_mean = scenarios[name]["mean_ns"]
        current_mean = read_mean_ns_from_criterion_dir(current_dir, name)
        if baseline_mean is None or current_mean is None:
            rows.append((name, current_mean, baseline_mean, None))
            continue
        delta_pct = ((current_mean - baseline_mean) / baseline_mean) * 100.0
        rows.append((name, current_mean, baseline_mean, delta_pct))
        if delta_pct > worst_pct:
            worst_pct = delta_pct

    print()
    print(
        f"{'Scenario':<40} {'Current':<14} {'Baseline':<14} {'Delta':>10}  Status"
    )
    print("-" * 92)

    any_missing = False
    fail = False
    warn = False
    for name, cur, base, delta_pct in rows:
        if cur is None or base is None or delta_pct is None:
            cur_s = fmt_ns_ms(cur) if cur is not None else "—"
            base_s = fmt_ns_ms(base) if base is not None else "—"
            print(f"{name:<40} {cur_s:<14} {base_s:<14} {'—':>10}  MISSING")
            any_missing = True
            continue
        if delta_pct > fail_pct:
            status = f"FAIL (> {fail_pct}%)"
            fail = True
        elif delta_pct > warn_pct:
            status = f"WARN (> {warn_pct}%)"
            warn = True
        elif delta_pct < -warn_pct:
            status = "improvement"
        else:
            status = "ok"
        sign = "+" if delta_pct >= 0 else ""
        print(
            f"{name:<40} {fmt_ns_ms(cur):<14} {fmt_ns_ms(base):<14} {sign}{delta_pct:>8.2f}%  {status}"
        )

    print()
    if any_missing:
        return _exit(
            3,
            "one or more scenarios produced no output; "
            "did `cargo bench --bench error_handling_benchmarks` fail?",
        )
    if fail:
        return _exit(
            2,
            f"FAIL: cumulative regression exceeds {fail_pct}% on at least one "
            f"scenario (worst: +{worst_pct:.2f}%). Phase B work should not "
            f"land until the regression is recovered or an explicit trade-off "
            f"is accepted (and the baseline updated as a deliberate decision).",
        )
    if warn:
        return _exit(
            1,
            f"WARN: cumulative regression exceeds {warn_pct}% on at least one "
            f"scenario (worst: +{worst_pct:.2f}%). Pause and assess whether "
            f"the trade-off is justified before merging.",
        )
    print(f"OK: all scenarios within ±{warn_pct}% of baseline.")
    return 0


def _exit(code: int, message: str) -> int:
    print(message, file=sys.stderr)
    return code


if __name__ == "__main__":
    sys.exit(main())
