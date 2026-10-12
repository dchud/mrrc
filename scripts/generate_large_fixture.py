#!/usr/bin/env python3
"""Generate fixtures of large MARC records for benchmarking.

`realistic.mrc` holds records of 12-30 fields. Some real records are far
larger: collected works with long contents notes, archival and serial records
with hundreds of subject headings and added entries, records carrying 880
alternate-script fields. Costs that grow with a record's field count (tag
lookups, field iteration, conversion) only show on records like these.

This builds each record on the realistic generator's core record and adds
notes, subject headings, added entries, links, and 880 fields up to a target
field count. Fields are ordered by tag block (0XX, 1XX, ... 8XX, then 880),
and within the 5XX, 6XX, and 7XX blocks they are shuffled, as cataloging
tools often leave them, so a repeated tag is not always contiguous. Records
are generated deterministically from a fixed seed with pymarc.

Usage:
    uv run python scripts/generate_large_fixture.py [--preset large|very-large]
        [--records N] [--min-fields N] [--max-fields N] [--out PATH]

Presets: `large` (default) writes 100 records of 150-250 fields to
`large_records.mrc`; `very-large` writes 10 records of 800-1000 fields to
`very_large_records.mrc`.
"""

from __future__ import annotations

import argparse
import random
import sys
from pathlib import Path

from pymarc import Field, Record, Subfield

sys.path.insert(0, str(Path(__file__).parent))
from generate_realistic_fixture import (
    initial,
    make_record,
    phrase,
)

PRESETS = {
    "large": {
        "records": 100,
        "min_fields": 150,
        "max_fields": 250,
        "out": "tests/data/fixtures/large_records.mrc",
    },
    "very-large": {
        "records": 10,
        "min_fields": 800,
        "max_fields": 1000,
        "out": "tests/data/fixtures/very_large_records.mrc",
    },
}

# ISO 2709 record length is a five-digit field.
MAX_RECORD_BYTES = 99_999

PLACES = [
    "France",
    "Germany",
    "Japan",
    "Brazil",
    "Canada",
    "India",
    "Egypt",
    "Mexico",
    "Italy",
    "Kenya",
    "Peru",
    "Norway",
]
ALT_SCRIPT = [
    "История и теория",
    "Собрание сочинений",
    "Ιστορία και θεωρία",
    "Συλλογή έργων",
    "Очерки по истории",
    "Δοκίμια",
]


def person(rng: random.Random) -> str:
    return f"{phrase(rng, 1)}, {initial(rng)}."


def note_500(rng: random.Random) -> Field:
    return Field(
        tag="500",
        indicators=[" ", " "],
        subfields=[Subfield("a", phrase(rng, rng.randint(6, 16)) + ".")],
    )


def contents_505(rng: random.Random) -> Field:
    parts = [phrase(rng, rng.randint(3, 6)) for _ in range(rng.randint(4, 10))]
    return Field(
        tag="505",
        indicators=["0", " "],
        subfields=[Subfield("a", " -- ".join(parts) + ".")],
    )


def subject(rng: random.Random) -> Field:
    kind = rng.random()
    if kind < 0.15:
        return Field(
            tag="600",
            indicators=["1", "0"],
            subfields=[
                Subfield("a", person(rng)),
                Subfield("d", f"{rng.randint(1800, 1950)}-"),
                Subfield("x", phrase(rng, 1) + "."),
            ],
        )
    if kind < 0.25:
        return Field(
            tag="610",
            indicators=["2", "0"],
            subfields=[Subfield("a", phrase(rng, 2) + " (Firm)")],
        )
    if kind < 0.40:
        return Field(
            tag="651",
            indicators=[" ", "0"],
            subfields=[
                Subfield("a", rng.choice(PLACES)),
                Subfield("x", "History."),
            ],
        )
    if kind < 0.50:
        return Field(
            tag="655",
            indicators=[" ", "7"],
            subfields=[
                Subfield("a", phrase(rng, 1) + "."),
                Subfield("2", "lcgft"),
            ],
        )
    return Field(
        tag="650",
        indicators=[" ", "0"],
        subfields=[
            Subfield("a", phrase(rng, rng.randint(1, 3))),
            Subfield("x", phrase(rng, 1)),
            Subfield("z", rng.choice(PLACES)),
            Subfield("v", "Congresses."),
        ],
    )


def added_entry(rng: random.Random) -> Field:
    kind = rng.random()
    if kind < 0.2:
        return Field(
            tag="710",
            indicators=["2", " "],
            subfields=[Subfield("a", phrase(rng, 3) + ".")],
        )
    if kind < 0.3:
        return Field(
            tag="730",
            indicators=["0", " "],
            subfields=[Subfield("a", phrase(rng, rng.randint(2, 5)) + ".")],
        )
    return Field(
        tag="700",
        indicators=["1", " "],
        subfields=[
            Subfield("a", person(rng)),
            Subfield("d", f"{rng.randint(1900, 1990)}-"),
            Subfield(
                "e", rng.choice(["editor.", "contributor.", "translator."])
            ),
        ],
    )


def link_856(rng: random.Random) -> Field:
    return Field(
        tag="856",
        indicators=["4", "0"],
        subfields=[
            Subfield("u", f"https://example.org/item/{rng.randint(0, 10**9)}"),
            Subfield("z", "Online version."),
        ],
    )


def series_830(rng: random.Random) -> Field:
    return Field(
        tag="830",
        indicators=[" ", "0"],
        subfields=[
            Subfield("a", phrase(rng, rng.randint(2, 4)) + " ;"),
            Subfield("v", f"v. {rng.randint(1, 200)}."),
        ],
    )


EXTRA = [
    # (weight, builder)
    (8, note_500),
    (6, contents_505),
    (40, subject),
    (36, added_entry),
    (6, link_856),
    (4, series_830),
]


def block(field: Field) -> int:
    """Sort key placing fields by tag block, with 880 last."""
    return 10 if field.tag == "880" else int(field.tag[0])


def make_large_record(rng: random.Random, n_fields: int) -> Record:
    rec = make_record(rng)
    rec.add_field(Field(tag="001", data=f"ocm{rng.randint(10**7, 10**8 - 1)}"))
    rec.add_field(Field(tag="003", data="OCoLC"))
    rec.add_field(Field(tag="005", data="20260101120000.0"))

    weights = [w for w, _ in EXTRA]
    builders = [b for _, b in EXTRA]
    extra: list[Field] = []
    target = n_fields - len(rec.fields)
    n_880 = max(1, target // 20)
    while len(extra) < target - n_880:
        builder = rng.choices(builders, weights)[0]
        extra.append(builder(rng))

    # Link some 700 fields to 880 alternate-script fields through $6.
    linkable = [f for f in extra if f.tag == "700"]
    for occ, linked in enumerate(
        rng.sample(linkable, min(n_880, len(linkable))), 1
    ):
        linked.subfields.insert(0, Subfield("6", f"880-{occ:02d}"))
        extra.append(
            Field(
                tag="880",
                indicators=["1", " "],
                subfields=[
                    Subfield("6", f"700-{occ:02d}"),
                    Subfield("a", rng.choice(ALT_SCRIPT)),
                ],
            )
        )

    fields = rec.fields + extra
    shuffled = {5, 6, 7}
    rank = {
        id(f): rng.random() if block(f) in shuffled else i
        for i, f in enumerate(fields)
    }
    fields.sort(
        key=lambda f: (
            block(f),
            f.tag if block(f) not in shuffled else "",
            rank[id(f)],
        )
    )
    rec.fields = fields

    # Stay under the ISO 2709 record-length limit by dropping 505 and 500
    # notes, the longest added fields, until the record fits.
    while len(rec.as_marc()) > MAX_RECORD_BYTES:
        notes = [f for f in rec.fields if f.tag in ("505", "500")]
        rec.remove_field(rng.choice(notes))
    return rec


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--preset", choices=sorted(PRESETS), default="large")
    parser.add_argument("--records", type=int)
    parser.add_argument("--min-fields", type=int)
    parser.add_argument("--max-fields", type=int)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--seed", type=int, default=20261012)
    args = parser.parse_args()
    preset = PRESETS[args.preset]
    records = args.records or preset["records"]
    lo = args.min_fields or preset["min_fields"]
    hi = args.max_fields or preset["max_fields"]
    out = args.out or Path(preset["out"])

    rng = random.Random(args.seed)
    data = bytearray()
    sizes = []
    counts = []
    for _ in range(records):
        rec = make_large_record(rng, rng.randint(lo, hi))
        encoded = rec.as_marc()
        data += encoded
        sizes.append(len(encoded))
        counts.append(len(rec.fields))

    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(bytes(data))
    print(
        f"wrote {out}: {records:,} records, {len(data):,} bytes, "
        f"{sum(counts) / records:.0f} fields/record (min {min(counts)}, "
        f"max {max(counts)}), max {max(sizes):,} B/record"
    )


if __name__ == "__main__":
    main()
