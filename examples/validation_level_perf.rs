//! One-shot timing comparison: `Structural` (lossy UTF-8) vs
//! `StrictMarc` (strict UTF-8 + indicator + subfield-code checks)
//! across the 10k-record fixture. Run with:
//!
//! ```sh
//! cargo run --release --example validation_level_perf
//! ```
//!
//! This is a quick sanity check, not a calibrated benchmark. Use
//! `cargo bench --bench marc_benchmarks` for the steady-state numbers.

use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::time::Instant;

use mrrc::{MarcReader, ValidationLevel};

const ITERATIONS: u32 = 20;

fn drain(bytes: &[u8], level: ValidationLevel) -> usize {
    let mut reader = MarcReader::new(Cursor::new(bytes.to_vec())).with_validation_level(level);
    let mut count = 0usize;
    while let Ok(Some(_)) = reader.read_record() {
        count += 1;
    }
    count
}

fn time_level(bytes: &[u8], level: ValidationLevel, label: &str) -> f64 {
    // Warmup
    let count = drain(bytes, level);
    let mut total_ns: u128 = 0;
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        let _ = drain(bytes, level);
        total_ns += t0.elapsed().as_nanos();
    }
    #[allow(clippy::cast_precision_loss)]
    let avg_ms = (total_ns / u128::from(ITERATIONS)) as f64 / 1_000_000.0;
    println!("{label}: {count} records, avg {avg_ms:.2} ms/run over {ITERATIONS} iterations");
    avg_ms
}

fn main() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/fixtures/10k_records.mrc");
    let bytes = fs::read(&path).expect("read 10k fixture");
    println!("Corpus: {} ({} bytes)\n", path.display(), bytes.len());

    let structural = time_level(&bytes, ValidationLevel::Structural, "Structural ");
    let strict = time_level(&bytes, ValidationLevel::StrictMarc, "StrictMarc ");

    let overhead_pct = ((strict - structural) / structural) * 100.0;
    println!(
        "\nStrictMarc overhead vs Structural: {overhead_pct:+.1}% ({:+.2} ms)",
        strict - structural
    );
}
