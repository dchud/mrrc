//! Pure-Rust wall-clock throughput harness, the native-ceiling column behind
//! the three-way comparison in `docs/benchmarks/results.md`.
//!
//! It mirrors, operation for operation, what `scripts/benchmark_comparison.py`
//! measures for the mrrc Python wrapper and pymarc — over the same fixture —
//! so the three numbers sit in one table. The difference between this column
//! and the Python wrapper is the cost of crossing into Python and building
//! Python objects: this harness parses to Rust `Record`s and stops; the
//! wrapper additionally materializes Python `Record`/`Field` objects.
//!
//! `scripts/benchmark_comparison.py` invokes this with `--json` and merges the
//! result. Run it directly with:
//!
//! ```text
//! cargo run --release --example benchmark_native -- \
//!     tests/data/fixtures/realistic.mrc --repeat 9
//! ```
//!
//! Wall-clock numbers are only worth citing from a quiet machine on AC power.

use std::path::Path;
use std::time::Instant;

use mrrc::boundary_scanner::RecordBoundaryScanner;
use mrrc::rayon_parser_pool::parse_batch_parallel;
use mrrc::{MarcReader, MarcWriter, RecordHelpers};

/// Each op does one full pass over the file and returns
/// (`record_count`, `elapsed_seconds`), matching its Python counterpart.
type Op = fn(&Path) -> (usize, f64);

/// `read` — parse every record, no field access. Mirrors `for r in reader`.
fn op_read(path: &Path) -> (usize, f64) {
    let start = Instant::now();
    let mut reader = MarcReader::from_path(path).expect("open fixture");
    let mut count = 0usize;
    while let Some(_record) = reader.read_record().expect("read record") {
        count += 1;
    }
    (count, start.elapsed().as_secs_f64())
}

/// `read_bulk` — scan boundaries and parse the whole file in one parallel
/// rayon call, mrrc's fastest read path. Mirrors `parse_batch_parallel`.
fn op_read_bulk(path: &Path) -> (usize, f64) {
    let start = Instant::now();
    let buffer = std::fs::read(path).expect("read fixture");
    let mut scanner = RecordBoundaryScanner::new();
    let boundaries = scanner.scan(&buffer).expect("scan boundaries");
    let records = parse_batch_parallel(&boundaries, &buffer).expect("parse batch");
    (records.len(), start.elapsed().as_secs_f64())
}

/// `extract` — parse, then touch fields the way the Python `extract` op does:
/// `record.title` plus `field.value()` for every field (control and data).
fn op_extract(path: &Path) -> (usize, f64) {
    let start = Instant::now();
    let mut reader = MarcReader::from_path(path).expect("open fixture");
    let mut count = 0usize;
    let mut acc = 0usize;
    while let Some(record) = reader.read_record().expect("read record") {
        count += 1;
        if let Some(title) = record.title() {
            acc += title.len();
        }
        // Control fields: their value is the data string itself (the wrapper's
        // `field.value()` returns `_data` for control fields).
        for values in record.control_fields.values() {
            for value in values {
                acc += value.len();
            }
        }
        // Data fields: value is the space-joined subfield values.
        for field in record.fields() {
            acc += field.value().len();
        }
    }
    std::hint::black_box(acc);
    (count, start.elapsed().as_secs_f64())
}

/// `roundtrip` — parse, then re-encode each record to ISO 2709 bytes.
/// Mirrors `record.as_marc()` per record.
fn op_roundtrip(path: &Path) -> (usize, f64) {
    let start = Instant::now();
    let mut reader = MarcReader::from_path(path).expect("open fixture");
    let mut count = 0usize;
    let mut acc = 0usize;
    while let Some(record) = reader.read_record().expect("read record") {
        count += 1;
        let mut buffer = Vec::new();
        {
            let mut writer = MarcWriter::new(&mut buffer);
            writer.write_record(&record).expect("serialize record");
        }
        acc += buffer.len();
    }
    std::hint::black_box(acc);
    (count, start.elapsed().as_secs_f64())
}

fn op_by_name(name: &str) -> Option<Op> {
    match name {
        "read" => Some(op_read),
        "read_bulk" => Some(op_read_bulk),
        "extract" => Some(op_extract),
        "roundtrip" => Some(op_roundtrip),
        _ => None,
    }
}

/// Median of `repeat` measured repetitions after discarding one cache-warming
/// run — the same protocol as the Python harness. For an even count, average
/// the two middle values (matching Python's `statistics.median`).
// Record counts are small; the usize -> f64 cast for rec/s is exact in practice.
#[allow(clippy::cast_precision_loss)]
fn measure(op: Op, path: &Path, repeat: usize) -> (usize, f64) {
    let (warm_count, _) = op(path); // discard the cache-warming repetition
    let mut count = warm_count;
    let mut rates: Vec<f64> = Vec::with_capacity(repeat);
    for _ in 0..repeat {
        let (n, elapsed) = op(path);
        if n != count {
            eprintln!("record count changed between runs ({count} vs {n}); aborting");
            std::process::exit(1);
        }
        count = n;
        rates.push(if elapsed > 0.0 {
            n as f64 / elapsed
        } else {
            0.0
        });
    }
    rates.sort_by(|a, b| a.partial_cmp(b).expect("no NaN rates"));
    let mid = rates.len() / 2;
    let median = if rates.len() % 2 == 1 {
        rates[mid]
    } else {
        f64::midpoint(rates[mid - 1], rates[mid])
    };
    (count, median)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut path: Option<String> = None;
    let mut repeat = 9usize;
    let mut ops_arg = "read,read_bulk,extract,roundtrip".to_string();
    let mut json = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repeat" => {
                i += 1;
                repeat = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(repeat);
            },
            "--ops" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    ops_arg.clone_from(v);
                }
            },
            "--json" => json = true,
            other if !other.starts_with("--") => path = Some(other.to_string()),
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            },
        }
        i += 1;
    }

    let path = path.unwrap_or_else(|| {
        eprintln!(
            "usage: benchmark_native <fixture.mrc> [--repeat N] \
             [--ops read,read_bulk,extract,roundtrip] [--json]"
        );
        std::process::exit(2);
    });
    let path = Path::new(&path);
    if !path.exists() {
        eprintln!("no such file: {}", path.display());
        std::process::exit(2);
    }

    let op_names: Vec<&str> = ops_arg
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    let mut results: Vec<(String, usize, f64)> = Vec::new();
    for name in &op_names {
        let op = op_by_name(name).unwrap_or_else(|| {
            eprintln!("unknown operation: {name}");
            std::process::exit(2);
        });
        let (count, rate) = measure(op, path, repeat);
        eprintln!("  {name:<10} mrrc (Rust) {rate:>14.0} rec/s  ({count} records)");
        results.push((name.to_string(), count, rate));
    }

    if json {
        let body: Vec<String> = results
            .iter()
            .map(|(name, count, rate)| {
                format!("\"{name}\":{{\"count\":{count},\"rec_s\":{rate:.1}}}")
            })
            .collect();
        println!("{{{}}}", body.join(","));
    }
}
