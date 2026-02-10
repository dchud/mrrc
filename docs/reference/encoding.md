# Character Encoding

MRRC supports both MARC-8 (legacy) and UTF-8 character encodings, with automatic conversion.

## Encoding Overview

| Encoding | Leader Position 09 | Description |
|----------|-------------------|-------------|
| MARC-8 | (blank/space) | Legacy encoding with escape sequences for non-Latin scripts |
| UTF-8 | `a` | Unicode, modern standard |

**MRRC handles encoding automatically:**

- Detects encoding from leader position 09
- Converts MARC-8 to UTF-8 when reading
- Stores all strings internally as UTF-8
- Can write to either encoding

## UTF-8 (Modern Standard)

UTF-8 is the recommended encoding for new records. It supports all Unicode characters directly without escape sequences.

**Reading UTF-8 records:**

=== "Python"

    ```python
    from mrrc import MARCReader

    for record in MARCReader("utf8_records.mrc"):
        # All strings are already UTF-8
        print(record.title())
    ```

=== "Rust"

    ```rust
    use mrrc::MarcReader;

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
        leader = record.leader()
        if leader.character_coding == 'a':
            print("Record declares UTF-8")
        else:
            print("Record declares MARC-8")
    ```

=== "Rust"

    ```rust
    use mrrc::{MarcReader, encoding::MarcEncoding};

    let encoding = MarcEncoding::from_leader_char(leader.position_9())?;
    match encoding {
        MarcEncoding::Utf8 => println!("UTF-8"),
        MarcEncoding::Marc8 => println!("MARC-8"),
    }
    ```

## Writing with Specific Encoding

By default, MRRC writes UTF-8. To write MARC-8:

=== "Python"

    ```python
    from mrrc import MARCWriter, Encoding

    with MARCWriter("output.mrc", encoding=Encoding.MARC8) as writer:
        writer.write(record)
    ```

=== "Rust"

    ```rust
    use mrrc::{MarcWriter, encoding::MarcEncoding};

    let mut writer = MarcWriter::with_encoding(
        File::create("output.mrc")?,
        MarcEncoding::Marc8
    );
    writer.write_record(&record)?;
    ```

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

In Python, MRRC handles encoding conversion automatically when reading records. If you encounter encoding issues, check the leader's `character_coding` property and compare it with the actual content.

## Common Issues

### Mojibake (Garbled Text)

If you see garbled text like `Ã©` instead of `é`, the encoding may be misdetected:

- Record declares UTF-8 but contains MARC-8
- Record declares MARC-8 but contains UTF-8
- File was saved with wrong encoding

**Solution**: Check the leader position 9 and verify it matches the actual data.

### Missing Characters

If characters display as `?` or `\uFFFD`:

- The character may not be in the MARC-8 character tables
- The character may be from an unsupported script
- The data may be corrupted

### East Asian Text (CJK)

MARC-8 uses EACC (East Asian Character Code) for Chinese, Japanese, and Korean:

- Uses 3-byte sequences (escape + 2 bytes)
- MRRC supports 15,000+ EACC characters
- Modern records should use UTF-8 for CJK

## Best Practices

1. **Use UTF-8 for new records** - Simpler, universal character support

2. **Preserve original encoding when round-tripping** - Read MARC-8, write MARC-8 if you need exact byte preservation

3. **Validate encoding before batch processing** - Check a sample of records for consistency

4. **Handle encoding errors gracefully** - Some legacy records have encoding issues

## See Also

- [MARC Primer](marc-primer.md) - Record structure overview
- [Library of Congress MARC-8 Specification](https://www.loc.gov/marc/specifications/speccharmarc8.html)
- [Unicode Character Tables](https://unicode.org/charts/)
