# Format Conversion (Rust)

Learn to convert MARC records between different formats.

## Core Formats

All formats are available without feature flags:

```rust
--8<-- "doc_examples/format_conversion.rs:core-formats"
```

## BIBFRAME Conversion

Convert MARC to BIBFRAME RDF:

```rust
--8<-- "doc_examples/format_conversion.rs:bibframe-conversion"
```

## Batch Conversion

```rust
--8<-- "doc_examples/format_conversion.rs:batch-conversion"
```

## Format Selection

| Format | Best For |
|--------|----------|
| ISO 2709 | Library system interchange |
| JSON | Web APIs, debugging |
| MARCXML | MARCXML pipelines |
| CSV | Spreadsheet export |
| Dublin Core | Simple metadata exchange |
| MODS | Detailed metadata crosswalks |
| BIBFRAME | Linked data, RDF |

## Complete Example

```rust
--8<-- "doc_examples/format_conversion.rs:complete-example"
```

## Next Steps

- [Reading Records](reading-records.md) - Basic record access
- [BIBFRAME Conversion](../../guides/bibframe-conversion.md) - RDF conversion
- [Format Support](../../reference/formats.md) - Full format reference
