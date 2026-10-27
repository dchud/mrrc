# Reading Records (Rust)

Learn to read MARC records from files and work with their contents.

## Basic Reading

```rust
--8<-- "doc_examples/reading_records.rs:basic-reading"
```

## Reading from Memory

```rust
--8<-- "doc_examples/reading_records.rs:reading-from-memory"
```

## Accessing Fields

```rust
--8<-- "doc_examples/reading_records.rs:accessing-fields"
```

## Control Fields

Control fields (001-009) contain unstructured data:

```rust
--8<-- "doc_examples/reading_records.rs:control-fields"
```

## Convenience Methods

```rust
--8<-- "doc_examples/reading_records.rs:convenience-methods"
```

## Working with Subfields

```rust
--8<-- "doc_examples/reading_records.rs:working-with-subfields"
```

## Working with Indicators

```rust
--8<-- "doc_examples/reading_records.rs:working-with-indicators"
```

## Error Handling

```rust
--8<-- "doc_examples/reading_records.rs:error-handling"
```

## Complete Example

```rust
--8<-- "doc_examples/reading_records.rs:complete-example"
```

## Next Steps

- [Writing Records](writing-records.md) - Create and modify records
- [Querying Fields](querying-fields.md) - Advanced field searching
- [Rust API Reference](../../reference/rust-api.md) - Full API documentation
