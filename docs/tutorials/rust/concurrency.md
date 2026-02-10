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
mrrc = "0.6"
rayon = "1.8"
```

## Parallel Processing

### Process Records in Parallel

```rust
use mrrc::MarcReader;
use rayon::prelude::*;
use std::fs::File;

fn main() -> mrrc::Result<()> {
    let file = File::open("records.mrc")?;
    let mut reader = MarcReader::new(file);

    // Collect records first
    let mut records = Vec::new();
    while let Some(record) = reader.read_record()? {
        records.push(record);
    }

    // Process in parallel
    let titles: Vec<_> = records
        .par_iter()
        .filter_map(|r| r.title())
        .collect();

    println!("Found {} titles", titles.len());
    Ok(())
}
```

### Batch Parsing with Rayon

For maximum parallelism, use batch parsing:

```rust
use mrrc::rayon_parser_pool::parse_batch_parallel;
use rayon::prelude::*;

fn main() -> mrrc::Result<()> {
    // Read raw record bytes
    let data = std::fs::read("records.mrc")?;

    // Split into individual record bytes (simplified)
    let record_bytes: Vec<&[u8]> = split_records(&data);

    // Parse in parallel
    let records = parse_batch_parallel(&record_bytes)?;

    // Process results in parallel
    let isbn_count: usize = records
        .par_iter()
        .filter(|r| r.isbn().is_some())
        .count();

    println!("Records with ISBN: {}", isbn_count);
    Ok(())
}
```

## Processing Multiple Files

```rust
use mrrc::MarcReader;
use rayon::prelude::*;
use std::fs::File;
use std::path::PathBuf;

fn process_file(path: &PathBuf) -> mrrc::Result<usize> {
    let file = File::open(path)?;
    let mut reader = MarcReader::new(file);
    let mut count = 0;

    while let Some(_record) = reader.read_record()? {
        count += 1;
    }

    Ok(count)
}

fn main() -> mrrc::Result<()> {
    let files: Vec<PathBuf> = glob::glob("data/*.mrc")?
        .filter_map(|r| r.ok())
        .collect();

    let results: Vec<_> = files
        .par_iter()
        .map(|path| (path.clone(), process_file(path)))
        .collect();

    for (path, result) in results {
        match result {
            Ok(count) => println!("{}: {} records", path.display(), count),
            Err(e) => eprintln!("{}: error - {}", path.display(), e),
        }
    }

    Ok(())
}
```

## Parallel Aggregation

```rust
use mrrc::MarcReader;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;

fn main() -> mrrc::Result<()> {
    let file = File::open("records.mrc")?;
    let mut reader = MarcReader::new(file);

    let mut records = Vec::new();
    while let Some(record) = reader.read_record()? {
        records.push(record);
    }

    // Count records by language (parallel reduction)
    let language_counts: HashMap<String, usize> = records
        .par_iter()
        .filter_map(|r| {
            r.control_field("008")
                .and_then(|f| f.get(35..38).map(|s| s.to_string()))
        })
        .fold(
            || HashMap::new(),
            |mut acc, lang| {
                *acc.entry(lang).or_insert(0) += 1;
                acc
            }
        )
        .reduce(
            || HashMap::new(),
            |mut a, b| {
                for (k, v) in b {
                    *a.entry(k).or_insert(0) += v;
                }
                a
            }
        );

    for (lang, count) in &language_counts {
        println!("{}: {}", lang, count);
    }

    Ok(())
}
```

## Thread Pool Configuration

```rust
use rayon::ThreadPoolBuilder;

fn main() -> mrrc::Result<()> {
    // Configure thread pool
    ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();

    // Rayon operations now use 4 threads
    // ...

    Ok(())
}
```

## Complete Example

```rust
use mrrc::MarcReader;
use rayon::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() -> mrrc::Result<()> {
    let files: Vec<PathBuf> = std::fs::read_dir("data")?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "mrc"))
        .collect();

    let total_records = AtomicUsize::new(0);
    let total_with_isbn = AtomicUsize::new(0);

    files.par_iter().for_each(|path| {
        if let Ok(file) = File::open(path) {
            let mut reader = MarcReader::new(file);

            while let Ok(Some(record)) = reader.read_record() {
                total_records.fetch_add(1, Ordering::Relaxed);

                if record.isbn().is_some() {
                    total_with_isbn.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    });

    let records = total_records.load(Ordering::Relaxed);
    let with_isbn = total_with_isbn.load(Ordering::Relaxed);

    println!("Total records: {}", records);
    println!("With ISBN: {} ({:.1}%)",
             with_isbn,
             100.0 * with_isbn as f64 / records as f64);

    Ok(())
}
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
