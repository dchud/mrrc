# Concurrency (Rust)

Learn to process MARC records in parallel using Rayon.

## Why Rayon?

Rayon provides data parallelism for Rust with minimal code changes:

- Parallel iterators that look like sequential code
- Work-stealing for automatic load balancing
- No manual thread management

## Setup

Add Rayon to your `Cargo.toml`:

```toml
[dependencies]
mrrc = "0.9"
rayon = "1.8"
```

## Parallel Processing

### Process Records in Parallel

```rust
--8<-- "doc_examples/concurrency.rs:process-records-in-parallel"
```

### Batch Parsing with Rayon

For maximum parallelism, use batch parsing:

```rust
--8<-- "doc_examples/concurrency.rs:batch-parsing"
```

## Processing Multiple Files

```rust
--8<-- "doc_examples/concurrency.rs:processing-multiple-files"
```

## Parallel Aggregation

```rust
--8<-- "doc_examples/concurrency.rs:parallel-aggregation"
```

## Thread Pool Configuration

```rust
--8<-- "doc_examples/concurrency.rs:thread-pool-configuration"
```

## Complete Example

```rust
--8<-- "doc_examples/concurrency.rs:complete-example"
```

## Benchmarks

Typical speedups on a 4-core system:

| Approach | Speedup | Use Case |
|----------|---------|----------|
| Sequential | 1x | Baseline |
| `par_iter()` | 3-4x | In-memory records |
| Batch parsing | 4-5x | Large files |
| Multiple files | Near-linear | Many files |

## Next Steps

- [Reading Records](reading-records.md) - Basic record access
- [Performance Tuning](../../guides/performance-tuning.md) - Optimization tips
- [Rust API Reference](../../reference/rust-api.md) - Full API documentation
