# Querying Fields (Rust)

Learn to use MRRC's Query DSL for complex field searching.

## Basic Field Access

```rust
--8<-- "doc_examples/querying_fields.rs:basic-field-access"
```

## FieldQuery (Tag Matching)

```rust
--8<-- "doc_examples/querying_fields.rs:field-query"
```

## Tag Range Query

Find fields within a tag range:

```rust
--8<-- "doc_examples/querying_fields.rs:tag-range-query"
```

## Filtering by Indicators

```rust
--8<-- "doc_examples/querying_fields.rs:filtering-by-indicators"
```

## Subfield Pattern Query

`SubfieldPatternQuery` matches a subfield against a regular expression, so you
don't have to wire up `regex` yourself. Construction returns a `Result` (the
error is a `regex::Error`) because the pattern is compiled up front:

```rust
--8<-- "doc_examples/querying_fields.rs:subfield-pattern-query"
```

## Subfield Value Query

`SubfieldValueQuery` matches a subfield's value directly. Use `new` for an exact
match and `partial` for a substring (both case-sensitive):

```rust
--8<-- "doc_examples/querying_fields.rs:subfield-value-query"
```

To test a single field instead of scanning the whole record, call
`query.matches(&field)`, which returns a `bool`.

## Combining Queries

```rust
--8<-- "doc_examples/querying_fields.rs:combining-queries"
```

## Complete Example

```rust
--8<-- "doc_examples/querying_fields.rs:complete-example"
```

## Performance Tips

- Use specific tags when possible (faster than ranges)
- Filter by tag first, then by indicator, then by content
- For large datasets, consider parallel processing with Rayon

## Next Steps

- [Reading Records](reading-records.md) - Basic record access
- [Concurrency](concurrency.md) - Parallel processing
- [Rust API Reference](../../reference/rust-api.md) - Full API documentation
