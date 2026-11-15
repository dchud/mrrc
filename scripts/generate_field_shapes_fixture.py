#!/usr/bin/env python3
"""Generate the field-shapes fixture for the pymarc parity oracle.

The oracle (tests/python/test_pymarc_parity.py) reads each record through
both mrrc and pymarc and compares field text (value(), format_field(), and
str()) and the Record properties (title, author, subjects, and the rest).
The benchmark corpora it also reads have few of the field shapes where
those differ from a plain join of subfield values, so this fixture
supplies them:

- 6XX fields with $v, $x, $y, and $z subdivisions, including a subdivision
  before $a and empty subdivisions
- $6 linkage in 1XX, 2XX, and 6XX fields, with the linked 880 fields
- $8 field links, which format_field() keeps
- subfield values with leading or trailing whitespace, and empty values
- control fields containing spaces
- the fields the Record properties read: 245 and 222 with $b, 110, 240,
  086, 022 $l, 264 with second indicators 4 and 1, 4XX/8XX series, 5XX notes
  inside and outside pymarc's list, 7XX added entries, and 852

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


def properties_record() -> Record:
    """The fields pymarc's Record properties read."""
    return record(
        control("001", "shape0003"),
        control("008", "200101t20202019dcu     o    f000 0 eng d"),
        data(
            "020", "  ", ("a", "978-0-306-40615-7 (hardcover)"), ("q", "print")
        ),
        data("022", "0 ", ("a", "0378-5955"), ("l", "0378-5955")),
        data(
            "086",
            "0 ",
            ("a", "Y 4.G 74/7:R 31"),
            ("z", "Y 4.G 74/7:R 30"),
        ),
        data(
            "110",
            "1 ",
            ("a", "United States."),
            ("b", "Congress."),
            ("b", "Senate."),
        ),
        data("222", " 0", ("a", "Journal of things"), ("b", "(Print)")),
        data(
            "240",
            "10",
            ("a", "Works."),
            ("k", "Selections."),
            ("l", "English"),
        ),
        data(
            "245",
            "10",
            ("a", "Collected things :"),
            ("b", "a reader /"),
            ("c", "edited by A. Editor."),
        ),
        data("264", " 4", ("c", "\u00a92019")),
        data(
            "264",
            " 1",
            ("a", "Washington :"),
            ("b", "U.S. Government Publishing Office,"),
            ("c", "[2020]"),
        ),
        data(
            "300",
            "  ",
            ("a", "xii, 300 pages :"),
            ("b", "illustrations ;"),
            ("c", "24 cm"),
        ),
        data("490", "1 ", ("a", "Things series ;"), ("v", "12")),
        data("500", "  ", ("a", "Includes index.")),
        data("504", "  ", ("a", "Includes bibliographical references.")),
        data("509", "  ", ("a", "A 5XX tag outside pymarc's notes list.")),
        data("590", "  ", ("a", "Local note.")),
        data("650", " 0", ("a", "Things"), ("v", "Periodicals.")),
        data("700", "1 ", ("a", "Editor, A.,"), ("e", "editor.")),
        data("740", "02", ("a", "Other things.")),
        data("830", " 0", ("a", "Things series ;"), ("v", "12.")),
        data("852", "  ", ("b", "Main"), ("h", "QA76"), ("i", ".T45")),
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--out",
        type=Path,
        default=Path("tests/data/field_shapes.mrc"),
    )
    args = parser.parse_args()
    records = [subject_record(), whitespace_record(), properties_record()]
    args.out.write_bytes(b"".join(r.as_marc() for r in records))
    print(f"wrote {len(records)} records to {args.out}")


if __name__ == "__main__":
    main()
