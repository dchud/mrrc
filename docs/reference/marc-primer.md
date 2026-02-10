# MARC Primer

An introduction to MARC record structure for developers.

## What is MARC?

**MARC** (MAchine-Readable Cataloging) is a data format for bibliographic records. Created by the Library of Congress in the 1960s, it remains the standard for library systems worldwide.

A MARC record describes a single item—a book, journal, map, recording, or other material—with structured metadata about its title, author, publisher, subjects, and more.

**Why MARC matters:**

- Over 200 million records in WorldCat alone
- Standard interchange format for library systems (ILS, OCLC, Z39.50)
- Most library metadata exists in MARC format

## Record Structure

A MARC record has three parts:

```
┌─────────────────────────────────────────┐
│  Leader (24 bytes)                      │
├─────────────────────────────────────────┤
│  Directory (variable length)            │
├─────────────────────────────────────────┤
│  Fields (variable length)               │
└─────────────────────────────────────────┘
```

### Leader

The leader is exactly 24 characters containing record metadata:

| Position | Length | Description |
|----------|--------|-------------|
| 00-04 | 5 | Record length |
| 05 | 1 | Record status (n=new, c=corrected, d=deleted) |
| 06 | 1 | Type of record (a=language material, e=map, etc.) |
| 07 | 1 | Bibliographic level (m=monograph, s=serial) |
| 08-09 | 2 | Type of control |
| 10 | 1 | Indicator count (always "2") |
| 11 | 1 | Subfield code count (always "2") |
| 12-16 | 5 | Base address of data |
| 17-19 | 3 | Encoding level, descriptive form, linked record |
| 20-23 | 4 | Entry map (always "4500") |

**Python:**
```python
leader = record.leader
print(leader.record_status)  # 'n', 'c', or 'd'
print(leader.type_of_record)  # 'a' for language material
```

**Rust:**
```rust
let leader = record.leader();
println!("{}", leader.record_status());
println!("{}", leader.type_of_record());
```

### Directory

The directory is an index listing each field's tag, length, and starting position. MRRC handles directory parsing automatically—you work directly with fields.

## Field Types

MARC has two types of fields:

### Control Fields (001-009)

Control fields contain unstructured data—no indicators or subfields.

| Tag | Name | Example |
|-----|------|---------|
| 001 | Control Number | `ocm12345678` |
| 003 | Control Number Identifier | `OCoLC` |
| 005 | Date/Time Last Modified | `20240115120000.0` |
| 006 | Fixed-Length Data Elements | (additional material) |
| 007 | Physical Description | (format-specific codes) |
| 008 | Fixed-Length Data Elements | (40 character coded data) |

**Python:**
```python
control_number = record["001"]
if control_number:
    print(control_number.value)  # "ocm12345678"
```

**Rust:**
```rust
if let Some(field) = record.field("001") {
    if let Some(data) = field.data() {
        println!("{}", data);
    }
}
```

### Data Fields (010-999)

Data fields have a 3-digit tag, two indicators, and one or more subfields.

```
Tag   Ind1  Ind2  Subfields
245   1     0     $a Title : $b subtitle / $c author.
```

#### Indicators

Each data field has two single-character indicators that modify the field's meaning:

| Field | Ind1 | Ind2 | Meaning |
|-------|------|------|---------|
| 245 (Title) | 0-1 | 0-9 | Title added entry; Nonfiling characters |
| 100 (Author) | 0-3 | (blank) | Type of name |
| 650 (Subject) | (blank) | 0-7 | Level; Thesaurus |

Example: `245 14` means:
- Ind1=1: Title added entry
- Ind2=4: Skip first 4 characters when filing ("The " in "The Book")

#### Subfields

Subfields are identified by a delimiter (`$` or `‡`) and a single-character code:

```
245 10 $a Programming in Rust :$b a comprehensive guide /$c by Jane Smith.
       ↑                        ↑                         ↑
       $a = title proper        $b = subtitle             $c = statement of responsibility
```

**Python:**
```python
title_field = record["245"]
if title_field:
    print(title_field["a"])  # "Programming in Rust :"
    print(title_field["b"])  # "a comprehensive guide /"
    print(title_field["c"])  # "by Jane Smith."
    print(title_field.ind1)  # "1"
    print(title_field.ind2)  # "0"
```

**Rust:**
```rust
if let Some(field) = record.field("245") {
    println!("{:?}", field.subfield("a"));
    println!("{:?}", field.subfield("b"));
    println!("Indicators: {} {}", field.indicator1(), field.indicator2());
}
```

## Common Fields

### Identification

| Tag | Name | Subfields |
|-----|------|-----------|
| 001 | Control Number | (control field, no subfields) |
| 010 | LC Control Number | `$a` number |
| 020 | ISBN | `$a` ISBN, `$q` qualifier |
| 022 | ISSN | `$a` ISSN |
| 035 | System Control Number | `$a` number |

### Main Entry (Author)

| Tag | Name | Subfields |
|-----|------|-----------|
| 100 | Personal Name | `$a` name, `$d` dates, `$e` relator |
| 110 | Corporate Name | `$a` name, `$b` subordinate unit |
| 111 | Meeting Name | `$a` name, `$d` date, `$c` location |

### Title

| Tag | Name | Subfields |
|-----|------|-----------|
| 245 | Title Statement | `$a` title, `$b` subtitle, `$c` responsibility |
| 246 | Varying Form of Title | `$a` title, `$b` remainder |
| 130/240 | Uniform Title | `$a` title, `$l` language |

### Publication

| Tag | Name | Subfields |
|-----|------|-----------|
| 260 | Publication (older) | `$a` place, `$b` publisher, `$c` date |
| 264 | Production/Publication | `$a` place, `$b` name, `$c` date |

### Physical Description

| Tag | Name | Subfields |
|-----|------|-----------|
| 300 | Physical Description | `$a` extent, `$b` details, `$c` dimensions |

### Subject Access

| Tag | Name | Subfields |
|-----|------|-----------|
| 600 | Subject - Personal | `$a` name, `$d` dates, `$t` title |
| 610 | Subject - Corporate | `$a` name, `$b` subordinate |
| 650 | Subject - Topical | `$a` topic, `$x` general subdivision |
| 651 | Subject - Geographic | `$a` place, `$x` subdivision |

### Added Entries

| Tag | Name | Subfields |
|-----|------|-----------|
| 700 | Added Entry - Personal | `$a` name, `$e` relator |
| 710 | Added Entry - Corporate | `$a` name, `$b` subordinate |
| 856 | Electronic Location | `$u` URI, `$z` note |

## Using Convenience Methods

MRRC provides shortcuts for common fields:

**Python:**
```python
# Instead of parsing 245$a yourself:
print(record.title())

# Instead of checking 100, 110, 111:
print(record.author())

# Gets all ISBNs from 020$a:
for isbn in record.isbns():
    print(isbn)
```

**Rust:**
```rust
println!("{:?}", record.title());
println!("{:?}", record.author());
for isbn in record.isbns() {
    println!("{}", isbn);
}
```

## Character Encoding

MARC records use one of two encodings:

| Encoding | Leader position 09 | Description |
|----------|-------------------|-------------|
| MARC-8 | (blank) | Legacy encoding with escape sequences |
| UTF-8 | `a` | Unicode, modern standard |

MRRC handles encoding automatically, converting MARC-8 to UTF-8 when reading.

## ISO 2709 Binary Format

MARC records are commonly stored in ISO 2709 format (`.mrc` files):

- Records are concatenated with no separator
- Each record ends with a record terminator (ASCII 29)
- Fields within a record end with a field terminator (ASCII 30)
- Subfields are delimited by ASCII 31

MRRC handles all parsing automatically—you work with structured `Record` objects.

## See Also

- [Character Encoding Reference](encoding.md) - MARC-8 and UTF-8 details
- [Python Quickstart](../getting-started/quickstart-python.md) - Start working with records
- [Rust Quickstart](../getting-started/quickstart-rust.md) - Rust API introduction
- [LOC MARC Documentation](https://www.loc.gov/marc/) - Official MARC21 specifications
