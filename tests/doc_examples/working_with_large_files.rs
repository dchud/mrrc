//! Code for `docs/guides/working-with-large-files.md`.

use mrrc::Record;

fn process(record: &Record) {}

fn iterator_processing() -> mrrc::Result<()> {
    // --8<-- [start:iterator-processing]
    use mrrc::MarcReader;
    use std::fs::File;

    let file = File::open("large_file.mrc")?;
    let mut reader = MarcReader::new(file);

    while let Some(record) = reader.read_record()? {
        process(&record);
    }
    // --8<-- [end:iterator-processing]
    Ok(())
}

fn load_all_records() -> mrrc::Result<()> {
    // --8<-- [start:load-all-records]
    use mrrc::MarcReader;
    use std::fs::File;

    let file = File::open("file.mrc")?;
    let mut reader = MarcReader::new(file);

    let mut records = Vec::new();
    while let Some(record) = reader.read_record()? {
        records.push(record);
    }
    // --8<-- [end:load-all-records]
    Ok(())
}

mod batch_processing {
    // --8<-- [start:batch-processing]
    use mrrc::{MarcReader, Record};
    use std::fs::File;

    fn process_batches<F>(path: &str, batch_size: usize, mut processor: F) -> mrrc::Result<()>
    where
        F: FnMut(&[Record]),
    {
        let file = File::open(path)?;
        let mut reader = MarcReader::new(file);
        let mut batch = Vec::with_capacity(batch_size);

        while let Some(record) = reader.read_record()? {
            batch.push(record);
            if batch.len() >= batch_size {
                processor(&batch);
                batch.clear();
            }
        }

        if !batch.is_empty() {
            processor(&batch);
        }

        Ok(())
    }
    // --8<-- [end:batch-processing]

    #[test]
    fn batches_fixture_records() {
        let mut sizes = Vec::new();
        process_batches(crate::fixtures::RECORDS_PATH, 50, |batch| {
            sizes.push(batch.len());
        })
        .unwrap();
        assert_eq!(sizes, [50, 50, 5]);
    }
}

mod parallel_file_processing {
    // --8<-- [start:parallel-file-processing]
    use mrrc::MarcReader;
    use rayon::prelude::*;
    use std::fs::File;

    fn process_files_parallel(paths: &[&str]) -> mrrc::Result<usize> {
        let counts: Vec<_> = paths
            .par_iter()
            .map(|path| {
                let file = File::open(path)?;
                let mut reader = MarcReader::new(file);
                let mut count = 0;
                while reader.read_record()?.is_some() {
                    count += 1;
                }
                Ok(count)
            })
            .collect::<mrrc::Result<Vec<_>>>()?;

        Ok(counts.iter().sum())
    }
    // --8<-- [end:parallel-file-processing]

    #[test]
    fn counts_fixture_records() {
        let paths = [
            crate::fixtures::RECORDS_PATH,
            "tests/data/multi_records.mrc",
        ];
        assert_eq!(process_files_parallel(&paths).unwrap(), 108);
    }
}

mod record_boundary_scanner {
    // --8<-- [start:record-boundary-scanner]
    use mrrc::Record;
    use mrrc::boundary_scanner::RecordBoundaryScanner;
    use mrrc::rayon_parser_pool::parse_batch_parallel;

    fn parallel_single_file(path: &str) -> mrrc::Result<Vec<Record>> {
        // Load file into memory
        let data = std::fs::read(path)?;

        // Find record boundaries: one (offset, length) pair per record
        let mut scanner = RecordBoundaryScanner::new();
        let boundaries = scanner.scan(&data)?;

        // Parse the records in parallel against the shared buffer
        parse_batch_parallel(&boundaries, &data)
    }
    // --8<-- [end:record-boundary-scanner]

    #[test]
    fn parses_fixture_records() {
        let records = parallel_single_file(crate::fixtures::RECORDS_PATH).unwrap();
        assert_eq!(records.len(), 105);
    }
}
