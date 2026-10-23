# Rust Quickstart

Get started with MRRC in Rust in 5 minutes.

## Add Dependency

```toml
[dependencies]
mrrc = "0.9"
```

## Read Records

```rust
--8<-- "doc_examples/quickstart_rust.rs:read-records"
```

## Access Fields

```rust
--8<-- "doc_examples/quickstart_rust.rs:access-fields"
```

## Create Records

```rust
--8<-- "doc_examples/quickstart_rust.rs:create-record"
```

## Write Records

```rust
--8<-- "doc_examples/quickstart_rust.rs:write-record"
```

## Convert Formats

```rust
--8<-- "doc_examples/quickstart_rust.rs:convert-formats"
```

## Error Handling

MRRC uses a custom `Result` type:

```rust
--8<-- "doc_examples/quickstart_rust.rs:error-handling"
```

## Next Steps

- [Reading Records Tutorial](../tutorials/rust/reading-records.md) - Detailed reading guide
- [Writing Records Tutorial](../tutorials/rust/writing-records.md) - Builder pattern and traits
- [Concurrency Tutorial](../tutorials/rust/concurrency.md) - Parallel processing with Rayon
- [Rust API Reference](../reference/rust-api.md) - Full API documentation
