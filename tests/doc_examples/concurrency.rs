//! Code for `docs/tutorials/rust/concurrency.md`.

mod process_records_in_parallel {
    // --8<-- [start:process-records-in-parallel]
    use mrrc::{MarcReader, RecordHelpers};
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
        let titles: Vec<_> = records.par_iter().filter_map(|r| r.title()).collect();

        println!("Found {} titles", titles.len());
        Ok(())
    }
    // --8<-- [end:process-records-in-parallel]
}

mod batch_parsing {
    // --8<-- [start:batch-parsing]
    use mrrc::RecordHelpers;
    use mrrc::boundary_scanner::RecordBoundaryScanner;
    use mrrc::rayon_parser_pool::parse_batch_parallel;
    use rayon::prelude::*;

    fn main() -> mrrc::Result<()> {
        // Read the whole file into one buffer
        let data = std::fs::read("records.mrc")?;

        // Locate each record as an (offset, length) pair
        let mut scanner = RecordBoundaryScanner::new();
        let boundaries = scanner.scan(&data)?;

        // Parse every record in parallel against the shared buffer
        let records = parse_batch_parallel(&boundaries, &data)?;

        // Process results in parallel
        let isbn_count: usize = records.par_iter().filter(|r| r.isbn().is_some()).count();

        println!("Records with ISBN: {isbn_count}");
        Ok(())
    }
    // --8<-- [end:batch-parsing]
}

mod processing_multiple_files {
    // --8<-- [start:processing-multiple-files]
    use mrrc::MarcReader;
    use rayon::prelude::*;
    use std::fs::File;
    use std::path::{Path, PathBuf};

    fn process_file(path: &Path) -> mrrc::Result<usize> {
        let file = File::open(path)?;
        let mut reader = MarcReader::new(file);
        let mut count = 0;

        while let Some(_record) = reader.read_record()? {
            count += 1;
        }

        Ok(count)
    }

    fn main() -> mrrc::Result<()> {
        let files: Vec<PathBuf> = std::fs::read_dir("data")?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "mrc"))
            .collect();

        let results: Vec<_> = files
            .par_iter()
            .map(|path| (path, process_file(path)))
            .collect();

        for (path, result) in results {
            match result {
                Ok(count) => println!("{}: {} records", path.display(), count),
                Err(e) => eprintln!("{}: error - {}", path.display(), e),
            }
        }

        Ok(())
    }
    // --8<-- [end:processing-multiple-files]

    #[test]
    fn counts_fixture_records() {
        let path = Path::new(crate::fixtures::RECORDS_PATH);
        assert_eq!(process_file(path).unwrap(), 105);
    }
}

mod parallel_aggregation {
    // --8<-- [start:parallel-aggregation]
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
                r.get_control_field("008")
                    .and_then(|f| f.get(35..38).map(str::to_string))
            })
            .fold(HashMap::new, |mut acc, lang| {
                *acc.entry(lang).or_insert(0) += 1;
                acc
            })
            .reduce(HashMap::new, |mut a, b| {
                for (k, v) in b {
                    *a.entry(k).or_insert(0) += v;
                }
                a
            });

        for (lang, count) in &language_counts {
            println!("{lang}: {count}");
        }

        Ok(())
    }
    // --8<-- [end:parallel-aggregation]
}

mod thread_pool_configuration {
    // --8<-- [start:thread-pool-configuration]
    use rayon::ThreadPoolBuilder;

    fn main() {
        // Configure the global thread pool. build_global() errors only if a
        // global pool already exists.
        ThreadPoolBuilder::new()
            .num_threads(4)
            .build_global()
            .expect("global rayon thread pool already initialized");

        // Rayon operations now use 4 threads
        // ...
    }
    // --8<-- [end:thread-pool-configuration]
}

mod complete_example {
    #![allow(clippy::cast_precision_loss)]

    // --8<-- [start:complete-example]
    use mrrc::{MarcReader, RecordHelpers};
    use rayon::prelude::*;
    use std::fs::File;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn main() -> mrrc::Result<()> {
        let files: Vec<PathBuf> = std::fs::read_dir("data")?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "mrc"))
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

        println!("Total records: {records}");
        println!(
            "With ISBN: {} ({:.1}%)",
            with_isbn,
            100.0 * with_isbn as f64 / records as f64
        );

        Ok(())
    }
    // --8<-- [end:complete-example]
}
