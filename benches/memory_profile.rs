//! Heap use per record when reading each benchmark fixture.
//!
//! Not a criterion bench: a counting global allocator measures, for each
//! fixture, the heap a parsed record owns, the number of live allocations
//! behind it, and the allocation calls made while reading it. Run it before
//! and after a change to record storage and compare the tables. Codspeed
//! does not run it.
//!
//! ```text
//! cargo bench --bench memory_profile
//! ```

#![allow(missing_docs)]
// The counting allocator forwards to `System`; it is the only way to count
// heap use without a profiler dependency. Benchmark-only: the library itself
// has no `unsafe` code.
#![allow(unsafe_code)]
// Allocation sizes fit in `isize`, and the per-record averages are table
// output, where `f64` precision is ample.
#![allow(clippy::cast_possible_wrap, clippy::cast_precision_loss)]

use mrrc::{MarcReader, Record};
use std::alloc::{GlobalAlloc, Layout, System};
use std::io::Cursor;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering::Relaxed};

struct Counting;

static LIVE_BYTES: AtomicIsize = AtomicIsize::new(0);
static LIVE_BLOCKS: AtomicIsize = AtomicIsize::new(0);
static ALLOC_CALLS: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every method forwards to `System` with the caller's arguments
// unchanged; the counters are bookkeeping only.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LIVE_BYTES.fetch_add(layout.size() as isize, Relaxed);
        LIVE_BLOCKS.fetch_add(1, Relaxed);
        ALLOC_CALLS.fetch_add(1, Relaxed);
        // SAFETY: forwarded unchanged; the caller upholds `alloc`'s contract.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE_BYTES.fetch_sub(layout.size() as isize, Relaxed);
        LIVE_BLOCKS.fetch_sub(1, Relaxed);
        // SAFETY: forwarded unchanged; the caller upholds `dealloc`'s contract.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        LIVE_BYTES.fetch_add(new_size as isize - layout.size() as isize, Relaxed);
        ALLOC_CALLS.fetch_add(1, Relaxed);
        // SAFETY: forwarded unchanged; the caller upholds `realloc`'s contract.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

const FIXTURES: &[&str] = &[
    "10k_records.mrc",
    "realistic.mrc",
    "realistic_marc8.mrc",
    "large_records.mrc",
    "very_large_records.mrc",
];

fn read_records(bytes: &[u8], capacity: usize) -> Vec<Record> {
    let mut reader = MarcReader::new(Cursor::new(bytes));
    let mut records = Vec::with_capacity(capacity);
    while let Some(record) = reader.read_record().expect("fixture parses") {
        records.push(record);
    }
    records
}

fn main() {
    println!(
        "size_of::<Record>() = {} B (inline, not counted below)\n",
        size_of::<Record>()
    );
    println!(
        "{:<24}{:>9}{:>12}{:>15}{:>15}{:>17}",
        "fixture", "records", "fields/rec", "heap B/record", "blocks/record", "alloc calls/rec"
    );
    for name in FIXTURES {
        let path = format!("tests/data/fixtures/{name}");
        let bytes =
            std::fs::read(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {path}"));
        let n = read_records(&bytes, 0).len();

        let (bytes0, blocks0, calls0) = (
            LIVE_BYTES.load(Relaxed),
            LIVE_BLOCKS.load(Relaxed),
            ALLOC_CALLS.load(Relaxed),
        );
        let records = read_records(&bytes, n);
        let calls = ALLOC_CALLS.load(Relaxed) - calls0;
        // Exclude the Vec<Record> buffer, sized exactly to `n` records.
        let held_bytes =
            (LIVE_BYTES.load(Relaxed) - bytes0) as f64 - (n * size_of::<Record>()) as f64;
        let held_blocks = (LIVE_BLOCKS.load(Relaxed) - blocks0 - 1) as f64;
        let fields: usize = records
            .iter()
            .map(|r| r.control_fields_iter().count() + r.fields().count())
            .sum();
        drop(records);

        let per = |v: f64| v / n as f64;
        println!(
            "{:<24}{:>9}{:>12.1}{:>15.0}{:>15.1}{:>17.1}",
            name,
            n,
            per(fields as f64),
            per(held_bytes),
            per(held_blocks),
            per(calls as f64)
        );
    }
}
