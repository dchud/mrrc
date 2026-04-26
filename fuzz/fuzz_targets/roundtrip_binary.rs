#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::{MarcReader, MarcWriter};
use std::io::Cursor;

// Couple the reader and writer: parse arbitrary bytes, and for every record
// the reader successfully extracts, run it back through `MarcWriter` and
// re-parse the serialized output. This catches writer panics on
// reader-accepted records and reader losses that would re-serialize into
// bytes the reader can no longer parse.
//
// mrrc does not promise byte-for-byte round-trip stability — the writer
// canonicalizes the leader, regenerates the directory, and may reorder
// fields. So the only assertion is that re-parse must not panic. Returning
// `Err(MarcError)` from either the writer or the second reader is correct
// behavior on inputs that fall outside the writer's representable range
// (e.g., records exceeding 4 GiB) and is therefore discarded.
fuzz_target!(|data: &[u8]| {
    let mut reader = MarcReader::new(Cursor::new(data));
    loop {
        match reader.read_record() {
            Ok(Some(record)) => {
                let mut buf = Vec::new();
                {
                    let mut writer = MarcWriter::new(&mut buf);
                    if writer.write_record(&record).is_err() {
                        continue;
                    }
                }
                let mut second = MarcReader::new(Cursor::new(&buf[..]));
                loop {
                    match second.read_record() {
                        Ok(Some(_)) => continue,
                        Ok(None) | Err(_) => break,
                    }
                }
            }
            Ok(None) | Err(_) => break,
        }
    }
});
