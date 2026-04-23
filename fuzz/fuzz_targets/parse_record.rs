#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::MarcReader;
use std::io::Cursor;

// Drive the full ISO 2709 reader over arbitrary bytes. An `Err(MarcError)`
// on malformed input is correct behavior, not a bug, so the Result is
// discarded — libfuzzer only flags panics, OOMs, and timeouts.
fuzz_target!(|data: &[u8]| {
    let mut reader = MarcReader::new(Cursor::new(data));
    loop {
        match reader.read_record() {
            Ok(Some(_)) => continue,
            Ok(None) | Err(_) => break,
        }
    }
});
