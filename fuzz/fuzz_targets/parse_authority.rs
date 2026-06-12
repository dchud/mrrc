#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::AuthorityMarcReader;
use std::io::Cursor;

// Drive the authority reader over arbitrary bytes. The authority reader
// shares the ISO 2709 framing logic with the bibliographic reader
// (covered by parse_record) but installs its own record builder and
// type-specific hooks: leader/06 must be 'z', and fields land in the
// AuthorityRecord field map rather than the bibliographic structure.
// An `Err(MarcError)` on malformed input is correct behavior, not a
// bug, so the Result is discarded — libfuzzer only flags panics, OOMs,
// and timeouts.
fuzz_target!(|data: &[u8]| {
    let mut reader = AuthorityMarcReader::new(Cursor::new(data));
    loop {
        match reader.read_record() {
            Ok(Some(_)) => continue,
            Ok(None) | Err(_) => break,
        }
    }
});
