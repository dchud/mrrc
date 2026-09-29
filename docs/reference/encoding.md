# Character Encoding

MRRC supports both MARC-8 (legacy) and UTF-8 character encodings, with automatic conversion.

## Encoding Overview

| Encoding | Leader Position 09 | Description |
|----------|-------------------|-------------|
| MARC-8 | (blank/space) | Legacy encoding with escape sequences for non-Latin scripts |
| UTF-8 | `a` | Unicode, modern standard |

**MRRC handles encoding automatically:**

- Chooses each record's encoding from leader position 09, as pymarc does:
  `a` is UTF-8 and any other value is MARC-8 (see
  [Choosing the Encoding](#choosing-the-encoding) to override this)
- Converts MARC-8 to UTF-8 when reading
- Stores all strings internally as UTF-8
- Writes UTF-8 (MARC-8 output is not currently supported)

## UTF-8 (Modern Standard)

UTF-8 is the recommended encoding for new records. It supports all Unicode characters directly without escape sequences.

**Reading UTF-8 records:**

=== "Python"

    ```python
    from mrrc import MARCReader

    for record in MARCReader("utf8_records.mrc"):
        # All strings are already UTF-8
        print(record.title)
    ```

=== "Rust"

    ```rust
    use mrrc::{MarcReader, RecordHelpers};

    let mut reader = MarcReader::new(file);
    while let Some(record) = reader.read_record()? {
        // All strings are Rust String (UTF-8)
        println!("{:?}", record.title());
    }
    ```

## MARC-8 (Legacy)

MARC-8 is a Library of Congress encoding that predates Unicode. It uses escape sequences to switch between character sets.

### Supported Character Sets

MRRC supports all standard MARC-8 character sets:

| Character Set | Code | Description |
|---------------|------|-------------|
| Basic Latin | 42 (B) | ASCII characters |
| Extended Latin (ANSEL) | 45 (E) | Diacritics and extended Latin |
| Basic Hebrew | 32 (2) | Hebrew alphabet |
| Basic Arabic | 33 (3) | Arabic script |
| Extended Arabic | 34 (4) | Extended Arabic variants |
| Basic Cyrillic | 4E (N) | Cyrillic alphabet |
| Extended Cyrillic | 51 (Q) | Extended Cyrillic |
| Basic Greek | 53 (S) | Greek alphabet |
| Subscript | 62 (b) | Mathematical subscripts |
| Superscript | 70 (p) | Mathematical superscripts |
| Greek Symbols | 67 (g) | Greek letters in symbols |
| EACC | 31 (1) | East Asian (CJK, 15,000+ characters) |

### Escape Sequences

MARC-8 uses escape sequences (starting with 0x1B) to switch character sets:

```
ESC + intermediate chars + final char → Switch character set
```

For example:
- `ESC ( B` → Switch G0 to Basic Latin
- `ESC $ 1` → Switch G0 to EACC (East Asian)

**You don't need to handle escape sequences manually** - MRRC decodes them automatically.

### Combining Marks (Diacritics)

MARC-8 represents diacritics as combining marks that precede their base character:

```
MARC-8:  [combining acute] + e → é
Unicode: e + [combining acute] → é (or precomposed é)
```

MRRC normalizes these to Unicode combining sequences.

## Encoding Detection

Check a record's declared encoding via the leader:

=== "Python"

    ```python
    from mrrc import MARCReader

    for record in MARCReader("records.mrc"):
        # Check what encoding the record declares
        leader = record.leader
        if leader.character_coding == 'a':
            print("Record declares UTF-8")
        else:
            print("Record declares MARC-8")
    ```

=== "Rust"

    ```rust
    use mrrc::encoding::MarcEncoding;

    // Leader position 9 (`character_coding`) declares the scheme
    let encoding = MarcEncoding::from_leader_char(record.leader.character_coding)?;
    match encoding {
        MarcEncoding::Utf8 => println!("UTF-8"),
        MarcEncoding::Marc8 => println!("MARC-8"),
    }
    ```

## Choosing the Encoding

By default each record is decoded in the encoding its leader position 09
declares, which is pymarc's rule. Files in the wild don't always get position
09 right, most often a UTF-8 record whose leader still says MARC-8, so the
readers take a `character_coding` option:

| `character_coding` | Behavior |
|---|---|
| `"leader"` (default) | Position 09 `a` is UTF-8; any other value is MARC-8. |
| `"utf-8"` | Every record is UTF-8, whatever position 09 says. pymarc spells this `force_utf8=True`, which mrrc also accepts. |
| `"detect"` | A record whose field data is valid UTF-8 containing non-ASCII bytes is UTF-8; otherwise position 09 decides. Suited to files that mix MARC-8 records with mislabelled UTF-8 ones, since MARC-8 text almost never forms valid multibyte UTF-8. |

=== "Python"

    ```python
    from mrrc import MARCReader

    # pymarc's spelling
    reader = MARCReader("records.mrc", force_utf8=True)

    # A file mixing MARC-8 records with UTF-8 records labelled MARC-8
    reader = MARCReader("records.mrc", character_coding="detect")
    ```

=== "Rust"

    ```rust
    use mrrc::{CharacterCoding, MarcReader};

    let mut reader = MarcReader::new(file).with_character_coding(CharacterCoding::Detect);
    ```

`AuthorityMARCReader` and `HoldingsMARCReader` (and their Rust counterparts)
take the same option.

A MARC-8 character with no mapping in the active character set, or an escape
sequence cut off by the end of a value, becomes `U+FFFD` under the default
`validation_level="structural"` (pymarc substitutes a space). Under
`validation_level="strict_marc"` it raises
[E302 `marc8_invalid`](error-codes.md#E302) instead, as invalid UTF-8 raises
[E301](error-codes.md#E301).

## Writing

MRRC writes UTF-8 and sets leader position 09 to `a` on output. Records read
from MARC-8 sources are converted to UTF-8 on the way in, so written output is
always UTF-8 regardless of the source encoding. Writing MARC-8 output is not
currently supported.

## Mixed Encoding Handling

Some legacy records have inconsistent encoding - the leader says MARC-8 but some fields contain UTF-8 (or vice versa).

In Rust, the encoding validator can detect this programmatically:

```rust
use mrrc::encoding::EncodingValidator;

let analysis = EncodingValidator::analyze_encoding(&record)?;
match analysis {
    EncodingAnalysis::Consistent(enc) => {
        println!("Consistent encoding: {:?}", enc);
    }
    EncodingAnalysis::Mixed { primary, .. } => {
        println!("Warning: mixed encoding detected");
    }
    EncodingAnalysis::Undetermined => {
        println!("Could not determine encoding");
    }
}
```

In Python, MRRC handles encoding conversion automatically when reading records. For a file that mixes MARC-8 records with UTF-8 records labelled MARC-8, read with `character_coding="detect"` (see [Choosing the Encoding](#choosing-the-encoding)). If you encounter other encoding issues, check the leader's `character_coding` property and compare it with the actual content.

## Common Issues

### Mojibake (Garbled Text)

If you see garbled text like `Ã©` instead of `é`, the encoding may be misdetected:

- Record declares UTF-8 but contains MARC-8
- Record declares MARC-8 but contains UTF-8
- File was saved with wrong encoding

**Solution**: Check the leader position 9 and verify it matches the actual data. If the data is UTF-8 but the leader says MARC-8, read with `character_coding="utf-8"` (pymarc's `force_utf8=True`), or with `character_coding="detect"` when only some records are affected.

### Missing Characters

If characters display as `?` or `\uFFFD`:

- The character may not be in the MARC-8 character tables
- The character may be from an unsupported script
- The data may be corrupted

Under `validation_level="strict_marc"`, an undecodable MARC-8 character raises
[E302](error-codes.md#E302) rather than becoming `\uFFFD`.

### East Asian Text (CJK)

MARC-8 uses EACC (East Asian Character Code) for Chinese, Japanese, and Korean:

- Each character is 3 bytes, after the `ESC $ 1` escape sequence
- MRRC supports 15,000+ EACC characters
- Modern records should use UTF-8 for CJK

## Best Practices

1. **Use UTF-8 for new records** - Simpler, universal character support

2. **Expect UTF-8 on round-trip** - Reading then re-writing a MARC-8 record normalizes it to UTF-8; MRRC does not reproduce the original MARC-8 bytes

3. **Validate encoding before batch processing** - Check a sample of records for consistency

4. **Handle encoding errors gracefully** - Some legacy records have encoding issues

## See Also

- [MARC Primer](marc-primer.md) - Record structure overview
- [Library of Congress MARC-8 Specification](https://www.loc.gov/marc/specifications/speccharmarc8.html)
- [Unicode Character Tables](https://unicode.org/charts/)
