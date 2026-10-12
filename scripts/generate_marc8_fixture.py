#!/usr/bin/env python3
"""Generate a MARC-8 encoded fixture for benchmarking MARC-8 decoding.

The other benchmark fixtures declare UTF-8 (leader position 09 = 'a') and are
almost entirely ASCII, so no benchmark exercises the MARC-8 decode path: ANSEL
combining diacritics, the extended Latin letters, and escape sequences into
other character sets.

This builds realistic-shaped records on the realistic generator's core record,
puts Latin diacritics in names, places, and subject headings, and gives some
records a Cyrillic or Greek 880 pair for the 100 and 245 with a romanized 245.
Every value is encoded to MARC-8 (combining marks before their base letter,
G0 escape sequences for Cyrillic and Greek) and the records are written with
leader position 09 blank. The encoder inverts pymarc's MARC-8 tables; the
script reads the output back with pymarc and stops if any value does not
decode to the text it started from. Records are generated deterministically
from a fixed seed.

Usage:
    uv run python scripts/generate_marc8_fixture.py [--records N] [--out PATH]
"""

from __future__ import annotations

import argparse
import random
import sys
import unicodedata
from itertools import zip_longest
from pathlib import Path

from pymarc import Field, MARCReader, Record, Subfield, marc8_mapping

sys.path.insert(0, str(Path(__file__).parent))
from generate_realistic_fixture import make_record

ASCII, ANSEL, CYRILLIC, GREEK = 0x42, 0x45, 0x4E, 0x53
# G0 designations. ANSEL is the default G1 set, so its bytes (0xA1-0xFE) need
# no escape.
G0_ESCAPE = {ASCII: b"\x1b(B", CYRILLIC: b"\x1b(N", GREEK: b"\x1b(S"}

TABLES = {
    cs: {
        chr(uc): code
        for code, (uc, _combining) in marc8_mapping.CODESETS[cs].items()
    }
    for cs in (ASCII, ANSEL, CYRILLIC, GREEK)
}

NAMES = [
    "Müller, Jürgen",
    "Dvořák, Antonín",
    "Núñez, José",
    "Lefèvre, François",
    "Łukasiewicz, Jan",
    "Søndergaard, Mette",
    "Gödel, Kurt",
    "Çelik, Ayşe",
    "Đorđević, Milan",
    "Ångström, Anders",
    "Brontë, Charlotte",
    "Ibáñez, Vicente",
    "Škvorecký, Josef",
    "Ørsted, Hans Christian",
]
PLACES = [
    "Kraków",
    "München",
    "Zürich",
    "São Paulo",
    "Montréal",
    "Reykjavík",
    "Bogotá",
    "Malmö",
    "Göttingen",
    "Łódź",
    "Besançon",
    "Århus",
]
SUBJECTS = [
    "Économie politique",
    "Literatura española",
    "Philosophie française",
    "Geschichte Österreichs",
    "Música popular",
    "Kultura polská",
    "Littérature québécoise",
    "Folklore islandés",
]
# (romanized title, original-script title, original-script author)
ALT_SCRIPT = [
    ("Voĭna i mir", "Война и мир", "Толстой, Лев Николаевич"),
    (
        "Istorii︠a︡ gosudarstva Rossiĭskogo",
        "История государства Российского",
        "Карамзин, Николай Михайлович",
    ),
    (
        "Sobranie sochineniĭ",
        "Собрание сочинений",
        "Пушкин, Александр Сергеевич",
    ),
    # U+02B9 MODIFIER LETTER PRIME is ALA-LC's romanization of the soft sign.
    (
        "Zapiski iz podpolʹi︠a︡",
        "Записки из подполья",  # noqa: RUF001
        "Достоевский, Федор Михайлович",
    ),
    (
        "Historia tēs Hellados",
        "Ιστορία της Ελλάδος",
        "Παπαρρηγόπουλος, Κωνσταντίνος",
    ),
    ("Poiēmata", "Ποιήματα", "Καβάφης, Κωνσταντίνος"),
]


def encode_char(ch: str, g0: int) -> tuple[int, bytes]:
    """Return (new G0 set, bytes) for one character in the current G0 state."""
    if ch == " ":
        # pymarc's decoder takes 0x20 only in the ASCII G0 set, so switch back
        # for spaces inside Cyrillic or Greek text.
        if g0 == ASCII:
            return g0, b" "
        return ASCII, G0_ESCAPE[ASCII] + b" "
    if ch in TABLES[g0]:
        return g0, bytes([TABLES[g0][ch]])
    if ch in TABLES[ANSEL]:
        return g0, bytes([TABLES[ANSEL][ch]])
    for cs in (ASCII, CYRILLIC, GREEK):
        if ch in TABLES[cs]:
            return cs, G0_ESCAPE[cs] + bytes([TABLES[cs][ch]])
    raise ValueError(f"no MARC-8 encoding for {ch!r} (U+{ord(ch):04X})")


def encode_marc8(text: str) -> bytes:
    """Encode text as MARC-8, ending in the default (ASCII) G0 state."""
    clusters: list[list] = []
    for ch in unicodedata.normalize("NFD", text):
        if unicodedata.combining(ch) and clusters:
            clusters[-1][1].append(ch)
        else:
            clusters.append([ch, []])
    out = bytearray()
    g0 = ASCII
    for base, marks in clusters:
        # Switch G0 for the base letter first, so the marks sit directly
        # before it: MARC-8 puts combining marks before their base.
        new_g0, base_bytes = encode_char(base, g0)
        if new_g0 != g0:
            out += G0_ESCAPE[new_g0]
            base_bytes = base_bytes[len(G0_ESCAPE[new_g0]) :]
            g0 = new_g0
        for mark in marks:
            g0, mark_bytes = encode_char(mark, g0)
            out += mark_bytes
        out += base_bytes
    if g0 != ASCII:
        out += G0_ESCAPE[ASCII]
    return bytes(out)


def add_multilingual(rng: random.Random, rec: Record) -> None:
    """Put diacritics and, for some records, an 880 pair into a record."""
    if rng.random() < 0.6:
        rec["100"]["a"] = rng.choice(NAMES)
    if rng.random() < 0.5:
        rec["264"]["a"] = rng.choice(PLACES) + " :"
    for field in rec.get_fields("650"):
        if rng.random() < 0.3:
            field["a"] = rng.choice(SUBJECTS)
    for field in rec.get_fields("700"):
        if rng.random() < 0.3:
            field["a"] = rng.choice(NAMES)
    if rng.random() < 0.3:
        romanized, title, author = rng.choice(ALT_SCRIPT)
        rec["100"].subfields.insert(0, Subfield("6", "880-01"))
        rec["245"]["a"] = romanized + " :"
        rec["245"].subfields.insert(0, Subfield("6", "880-02"))
        rec.add_field(
            Field(
                tag="880",
                indicators=["1", " "],
                subfields=[Subfield("6", "100-01"), Subfield("a", author)],
            ),
            Field(
                tag="880",
                indicators=["1", "0"],
                subfields=[Subfield("6", "245-02"), Subfield("a", title)],
            ),
        )


def to_marc8_record(rec: Record) -> Record:
    """Copy a Unicode record into a MARC-8 record (leader position 09 blank).

    pymarc writes a non-UTF-8 record's values as ISO 8859-1, so each value is
    carried as its MARC-8 bytes decoded one byte per character.
    """
    out = Record(to_unicode=False, force_utf8=False)
    for field in rec.fields:
        if field.is_control_field():
            out.add_field(Field(tag=field.tag, data=field.data))
            continue
        out.add_field(
            Field(
                tag=field.tag,
                indicators=list(field.indicators),
                subfields=[
                    Subfield(sf.code, encode_marc8(sf.value).decode("latin-1"))
                    for sf in field.subfields
                ],
            )
        )
    return out


def values(rec: Record) -> list[str]:
    return [
        unicodedata.normalize("NFC", sf.value)
        for field in rec.fields
        if not field.is_control_field()
        for sf in field.subfields
    ]


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--records", type=int, default=1000)
    parser.add_argument(
        "--out",
        type=Path,
        default=Path("tests/data/fixtures/realistic_marc8.mrc"),
    )
    parser.add_argument("--seed", type=int, default=20261012)
    args = parser.parse_args()

    rng = random.Random(args.seed)
    originals = []
    data = bytearray()
    for _ in range(args.records):
        rec = make_record(rng)
        add_multilingual(rng, rec)
        originals.append(rec)
        data += to_marc8_record(rec).as_marc()

    # Read the output back with pymarc's MARC-8 decoder and require every
    # value to come back unchanged.
    decoded = list(MARCReader(bytes(data), to_unicode=True, force_utf8=False))
    if len(decoded) != len(originals):
        sys.exit(f"read back {len(decoded)} records, wrote {len(originals)}")
    for i, (want, got) in enumerate(zip(originals, decoded, strict=True)):
        if str(got.leader)[9] != " ":
            sys.exit(
                f"record {i}: leader position 09 is {str(got.leader)[9]!r}"
            )
        if values(want) != values(got):
            bad = next(
                (w, g)
                for w, g in zip_longest(values(want), values(got))
                if w != g
            )
            sys.exit(f"record {i}: {bad[0]!r} decoded as {bad[1]!r}")

    non_ascii = sum(1 for b in data if b >= 0x80)
    escapes = data.count(0x1B)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_bytes(bytes(data))
    print(
        f"wrote {args.out}: {args.records:,} records, {len(data):,} bytes, "
        f"{non_ascii:,} non-ASCII bytes, {escapes:,} escape sequences; "
        "pymarc decodes every value unchanged"
    )


if __name__ == "__main__":
    main()
