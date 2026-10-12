# Rust API Reference

Rust API reference for MRRC. See [docs.rs/mrrc](https://docs.rs/mrrc) for full auto-generated documentation.

## Core Types

### Record

A MARC bibliographic record.

```rust
--8<-- "doc_examples/rust_api.rs:record"
```

**Key members** (fields are accessed without parentheses; the `title`/`author`/`isbns` helpers require `use mrrc::RecordHelpers`):

| Member | Type / Returns | Description |
|--------|----------------|-------------|
| `leader` *(field)* | `Leader` | The record's leader |
| `get_control_field(tag)` | `Option<&str>` | Get a control field value |
| `get_field(tag)` | `Option<&Field>` | Get first field with tag |
| `fields_by_tag(tag)` | `impl Iterator<Item = &Field>` | Iterate fields with tag |
| `fields()` | `impl Iterator<Item = &Field>` | Iterate all data fields |
| `title()` | `Option<&str>` | Title from 245$a (`RecordHelpers`) |
| `author()` | `Option<&str>` | Author from 100/110/111 (`RecordHelpers`) |
| `isbns()` | `Vec<&str>` | All ISBNs (`RecordHelpers`) |

### RecordBuilder

Builder pattern for creating records.

```rust
--8<-- "doc_examples/rust_api.rs:record-builder"
```

### Field

A MARC data field with tag, indicators, and subfields.

```rust
--8<-- "doc_examples/rust_api.rs:field"
```

**Key members** (`tag`, `indicator1`, and `indicator2` are public fields, accessed without parentheses):

| Member | Type / Returns | Description |
|--------|----------------|-------------|
| `tag` *(field)* | `String` | 3-character field tag |
| `indicator1` *(field)* | `char` | First indicator |
| `indicator2` *(field)* | `char` | Second indicator |
| `get_subfield(code)` | `Option<&str>` | Get first subfield value |
| `subfields()` | `impl Iterator<Item = &Subfield>` | Iterate subfields |
| `add_subfield(code, value)` | `()` | Add a subfield (`code: char`, `value: String`) |

### Subfield

A subfield within a field.

```rust
--8<-- "doc_examples/rust_api.rs:subfield"
```

### Leader

The 24-byte MARC record header.

```rust
--8<-- "doc_examples/rust_api.rs:leader"
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `record_length` | `u32` | Record length (positions 00-04) |
| `record_status` | `char` | Status: n/c/d (position 05) |
| `record_type` | `char` | Type: a=language, c=music, etc. (position 06) |
| `bibliographic_level` | `char` | Level: m=monograph, s=serial (position 07) |
| `character_coding` | `char` | Encoding: space=MARC-8, a=UTF-8 (position 09) |

## Reader/Writer

### MarcReader

Reads MARC records from any `Read` source.

```rust
--8<-- "doc_examples/rust_api.rs:marc-reader"
```

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `new(reader)` | `MarcReader<R>` | Create from any `Read` |
| `read_record()` | `Result<Option<Record>>` | Read next record |

### MarcWriter

Writes MARC records to any `Write` sink.

```rust
--8<-- "doc_examples/rust_api.rs:marc-writer"
```

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `new(writer)` | `MarcWriter<W>` | Create from any `Write` |
| `write_record(record)` | `Result<()>` | Write a record |

### Parsing a single record from bytes

When you already hold one record's bytes in memory, parse it directly without
constructing a reader:

```rust
--8<-- "doc_examples/rust_api.rs:parse-from-bytes"
```

| Function | Returns | Description |
|----------|---------|-------------|
| `parse_record_from_bytes(bytes, recovery, validation)` | `Result<Option<Record>>` | Parse one record from an owned `Vec<u8>` |
| `parse_record_from_shared_bytes(&Arc<Vec<u8>>, recovery, validation)` | `Result<Option<Record>>` | Same, but borrows a shared buffer instead of taking ownership |

Unlike the streaming reader, a non-empty buffer shorter than the 24-byte leader
is a `TruncatedRecord` error here rather than end-of-stream.

## Specialized Record Types

### AuthorityRecord

MARC authority records for controlled vocabularies.

```rust
--8<-- "doc_examples/rust_api.rs:authority-record"
```

### HoldingsRecord

MARC holdings records for library holdings data.

```rust
--8<-- "doc_examples/rust_api.rs:holdings-record"
```

## Query DSL

Query records using a fluent API.

```rust
--8<-- "doc_examples/rust_api.rs:query-dsl"
```

## Format Conversion

### Core Formats (Always Available)

```rust
--8<-- "doc_examples/rust_api.rs:core-formats"
```

## BIBFRAME Conversion

Convert MARC to BIBFRAME 2.0 RDF:

```rust
--8<-- "doc_examples/rust_api.rs:bibframe-conversion"
```

## Character Encoding

```rust
--8<-- "doc_examples/rust_api.rs:character-encoding"
```

## Error Handling

```rust
--8<-- "doc_examples/rust_api.rs:error-handling"
```

**Error Types:**

`MarcError` carries one variant per stable error code (leader, directory,
field, encoding, serialization, and I/O failures), each with structured
positional context. See the [error codes reference](error-codes.md) for the
complete per-variant documentation, and
[docs.rs](https://docs.rs/mrrc/latest/mrrc/enum.MarcError.html) for the enum
definition.

## Parallel Processing

Use Rayon for parallel processing:

```rust
--8<-- "doc_examples/rust_api.rs:parallel-processing"
```

See the [Rust concurrency tutorial](../tutorials/rust/concurrency.md) for the
full worked example.

## See Also

- [Rust Quickstart](../getting-started/quickstart-rust.md)
- [Format Support](formats.md)
- [docs.rs/mrrc](https://docs.rs/mrrc) - Full API documentation
