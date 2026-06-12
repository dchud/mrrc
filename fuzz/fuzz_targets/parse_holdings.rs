#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::HoldingsMarcReader;
use std::io::Cursor;

// Drive the holdings reader over arbitrary bytes. Like the authority
// reader (parse_authority), the holdings reader shares ISO 2709 framing
// with the bibliographic reader but installs its own record builder and
// type-specific hooks: leader/06 must be one of x/y/v/u, and fields
// land in the HoldingsRecord field map. An `Err(MarcError)` on
// malformed input is correct behavior, not a bug, so the Result is
// discarded — libfuzzer only flags panics, OOMs, and timeouts.
fuzz_target!(|data: &[u8]| {
    let mut reader = HoldingsMarcReader::new(Cursor::new(data));
    loop {
        match reader.read_record() {
            Ok(Some(_)) => continue,
            Ok(None) | Err(_) => break,
        }
    }
});
