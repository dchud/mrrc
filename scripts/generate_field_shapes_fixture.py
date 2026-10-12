#!/usr/bin/env python3
"""Generate the field-shapes fixture for the pymarc parity oracle.

The oracle (tests/python/test_pymarc_parity.py) reads each record through
both mrrc and pymarc and compares field text: value(), format_field(), and
str(). The benchmark corpora it also reads have few of the field shapes
where those methods do something other than join subfield values with
spaces, so this fixture supplies them:

- 6XX fields with $v, $x, $y, and $z subdivisions, including a subdivision
  before $a and empty subdivisions
- $6 linkage in 1XX, 2XX, and 6XX fields, with the linked 880 fields
- $8 field links, which format_field() keeps
- subfield values with leading or trailing whitespace, and empty values
- control fields containing spaces

Fields with the same tag are kept adjacent, so field order does not depend
on how a reader orders repeated tags. The records are written with pymarc.

Usage:
    uv run python scripts/generate_field_shapes_fixture.py [--out PATH]
"""

from __future__ import annotations

import argparse
from pathlib import Path

from pymarc import Field, Indicators, Leader, Record, Subfield


def control(tag: str, data: str) -> Field:
    return Field(tag=tag, data=data)


def data(tag: str, ind: str, *subfields: tuple[str, str]) -> Field:
    return Field(
        tag=tag,
        indicators=Indicators(ind[0], ind[1]),
        subfields=[Subfield(code, value) for code, value in subfields],
    )


def record(*fields: Field) -> Record:
    rec = Record(leader=Leader("00000nam a2200000 i 4500"))
    rec.add_field(*fields)
    return rec


def subject_record() -> Record:
    """Subject subdivisions and $6 linkage to 880 fields."""
    return record(
        control("001", "shape0001"),
        control("003", "XxU"),
        control("005", "20260101120000.0"),
        control("008", "850101s1985    ja a          000 0 jpn d"),
        data("020", "  ", ("a", "0-306-40615-2 (pbk.)")),
        data(
            "100",
            "1 ",
            ("6", "880-01"),
            ("a", "Tanaka, Hiroshi,"),
            ("d", "1950-"),
        ),
        data(
            "245",
            "10",
            ("6", "880-02"),
            ("a", "Neko no rekishi /"),
            ("c", "Tanaka Hiroshi."),
        ),
        data(
            "600",
            "10",
            ("a", "Natsume, Soseki,"),
            ("d", "1867-1916"),
            ("x", "Criticism and interpretation."),
        ),
        data("650", " 0", ("a", "Cats"), ("x", "History.")),
        data(
            "650",
            " 0",
            ("a", "Cats"),
            ("z", "Japan"),
            ("x", "History"),
            ("y", "20th century"),
            ("v", "Pictorial works."),
        ),
        data(
            "650",
            " 0",
            ("6", "880-03"),
            ("a", "Cats"),
            ("x", "Folklore."),
        ),
        data(
            "650",
            " 7",
            ("a", "Cats."),
            ("2", "fast"),
            ("0", "(OCoLC)fst00849374"),
        ),
        data(
            "651",
            " 0",
            ("a", "Japan"),
            ("x", "Social life and customs"),
            ("y", "1868-1912."),
        ),
        data("655", " 7", ("a", "Picture books."), ("2", "lcgft")),
        data(
            "880",
            "1 ",
            ("6", "100-01/$1"),
            ("a", "田中, 浩,"),
            ("d", "1950-"),
        ),
        data(
            "880",
            "10",
            ("6", "245-02/$1"),
            ("a", "猫の歴史 /"),
            ("c", "田中浩."),
        ),
        data("880", " 0", ("6", "650-03/$1"), ("a", "猫"), ("x", "民話.")),
    )


def whitespace_record() -> Record:
    """Whitespace, empty subfields, $8 links, and leading subdivisions."""
    return record(
        control("001", "shape0002"),
        control("008", "200101s2020    xxu           000 0 eng  "),
        data(
            "245",
            "00",
            ("a", "  Padded title\u00a0 "),
            ("b", ""),
            ("c", "edited by\tSomeone. "),
        ),
        data("500", "  ", ("a", "Note with a trailing space. ")),
        data(
            "541",
            "  ",
            ("8", "1\\c"),
            ("a", "Gift of the author."),
        ),
        data("650", " 4", ("a", ""), ("x", ""), ("b", "Empty subdivisions")),
        data("650", " 4", ("a", " Spaced "), ("x", " subdivision ")),
        data("690", "  ", ("x", "Leading subdivision"), ("a", "Local term")),
        data(
            "700",
            "1 ",
            ("8", "1\\c"),
            ("a", "Doe, Jane,"),
            ("e", "donor."),
        ),
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--out",
        type=Path,
        default=Path("tests/data/field_shapes.mrc"),
    )
    args = parser.parse_args()
    records = [subject_record(), whitespace_record()]
    args.out.write_bytes(b"".join(r.as_marc() for r in records))
    print(f"wrote {len(records)} records to {args.out}")


if __name__ == "__main__":
    main()
