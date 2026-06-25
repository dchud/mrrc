#!/usr/bin/env python3
"""Generate a realistic-shaped MARC fixture for benchmarking.

The committed synthetic fixtures (`tests/data/fixtures/*_records.mrc`) hold
tiny ~6-field, ~260-byte records. Those understate parsing-bound performance
because per-record object construction dominates the time, not parsing — so a
fast Rust parser looks no faster than pure-Python pymarc on them.

This builds records with the field count and text lengths of real
bibliographic data (~12-30 varied fields, ~0.8-1.6 KB each), so a benchmark
measures parsing rather than fixed per-record overhead. Records are generated
deterministically from a fixed seed (re-running yields an identical file),
using pymarc to construct and serialize them.

Usage:
    uv run python scripts/generate_realistic_fixture.py [--records N] [--out PATH]
"""

from __future__ import annotations

import argparse
import random
from pathlib import Path

from pymarc import Field, Record, Subfield

WORDS = [
    "analysis", "history", "theory", "practice", "modern", "critical",
    "study", "research", "method", "society", "culture", "politics",
    "economic", "social", "development", "system", "structure", "language",
    "literature", "science", "nature", "human", "world", "century", "essays",
    "sources", "perspectives", "readings", "documents", "collection",
    "edition", "revised", "introduction",
]


def phrase(rng: random.Random, n: int) -> str:
    return " ".join(rng.choice(WORDS) for _ in range(n)).capitalize()


def initial(rng: random.Random) -> str:
    return chr(65 + rng.randint(0, 25))


def make_record(rng: random.Random) -> Record:
    rec = Record()
    year = rng.randint(1950, 2024)
    rec.add_field(Field(
        tag="008",
        data=f"{rng.randint(800000, 999999):06d}s{year}"
             "    nyu           000 0 eng d",
    ))
    rec.add_field(Field(
        tag="020", indicators=[" ", " "],
        subfields=[Subfield("a", f"97801{rng.randint(0, 99999):05d}"
                                 f"{rng.randint(0, 9)}")],
    ))
    rec.add_field(Field(
        tag="100", indicators=["1", " "],
        subfields=[
            Subfield("a", f"{phrase(rng, 1)}, {initial(rng)}.,"),
            Subfield("d", f"{rng.randint(1900, 1970)}-"),
        ],
    ))
    rec.add_field(Field(
        tag="245", indicators=["1", "0"],
        subfields=[
            Subfield("a", phrase(rng, rng.randint(4, 9)) + " :"),
            Subfield("b", phrase(rng, rng.randint(3, 8)) + " /"),
            Subfield("c", "by " + phrase(rng, 2) + "."),
        ],
    ))
    if rng.random() < 0.4:
        edn = rng.choice(["Second", "Third", "Revised"])
        rec.add_field(Field(
            tag="250", indicators=[" ", " "],
            subfields=[Subfield("a", f"{edn} edition.")],
        ))
    rec.add_field(Field(
        tag="264", indicators=[" ", "1"],
        subfields=[
            Subfield("a", phrase(rng, 1) + " :"),
            Subfield("b", phrase(rng, 2) + ","),
            Subfield("c", f"{rng.randint(1990, 2024)}."),
        ],
    ))
    rec.add_field(Field(
        tag="300", indicators=[" ", " "],
        subfields=[
            Subfield("a", f"{rng.randint(100, 800)} pages :"),
            Subfield("b", "illustrations ;"),
            Subfield("c", "24 cm"),
        ],
    ))
    for tag, a, b, two in (
        ("336", "text", "txt", "rdacontent"),
        ("337", "unmediated", "n", "rdamedia"),
        ("338", "volume", "nc", "rdacarrier"),
    ):
        rec.add_field(Field(
            tag=tag, indicators=[" ", " "],
            subfields=[Subfield("a", a), Subfield("b", b), Subfield("2", two)],
        ))
    for _ in range(rng.randint(1, 4)):
        rec.add_field(Field(
            tag="500", indicators=[" ", " "],
            subfields=[Subfield("a", phrase(rng, rng.randint(6, 16)) + ".")],
        ))
    if rng.random() < 0.7:
        rec.add_field(Field(
            tag="520", indicators=[" ", " "],
            subfields=[Subfield("a", phrase(rng, rng.randint(15, 40)) + ".")],
        ))
    for _ in range(rng.randint(1, 6)):
        rec.add_field(Field(
            tag="650", indicators=[" ", "0"],
            subfields=[
                Subfield("a", phrase(rng, 1)),
                Subfield("x", phrase(rng, 1) + "."),
            ],
        ))
    for _ in range(rng.randint(0, 4)):
        rec.add_field(Field(
            tag="700", indicators=["1", " "],
            subfields=[Subfield("a", f"{phrase(rng, 1)}, {initial(rng)}.")],
        ))
    return rec


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--records", type=int, default=2000)
    parser.add_argument(
        "--out", type=Path,
        default=Path("tests/data/fixtures/realistic.mrc"),
    )
    parser.add_argument("--seed", type=int, default=20260625)
    args = parser.parse_args()

    rng = random.Random(args.seed)
    data = bytearray()
    sizes = []
    for _ in range(args.records):
        encoded = make_record(rng).as_marc()
        data += encoded
        sizes.append(len(encoded))

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_bytes(bytes(data))
    avg = sum(sizes) // len(sizes)
    print(
        f"wrote {args.out}: {args.records:,} records, {len(data):,} bytes, "
        f"avg {avg} B/record (min {min(sizes)}, max {max(sizes)})"
    )


if __name__ == "__main__":
    main()
