#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::{MarcReader, MarcWriter};
use std::io::Cursor;

// Reader → writer → reader coupling. Only panics fail; `Err` from either
// is correct behavior on inputs outside the writer's representable range.
// See docs/contributing/fuzzing.md for the asserted property.
fuzz_target!(|data: &[u8]| {
    let mut reader = MarcReader::new(Cursor::new(data));
    while let Ok(Some(record)) = reader.read_record() {
        let mut buf = Vec::new();
        if MarcWriter::new(&mut buf).write_record(&record).is_err() {
            continue;
        }
        let mut second = MarcReader::new(Cursor::new(&buf[..]));
        while let Ok(Some(_)) = second.read_record() {}
    }
});
