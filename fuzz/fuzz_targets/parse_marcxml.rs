#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::marcxml::marcxml_to_record;

// Drive the MARCXML reader over arbitrary bytes. The reader takes a
// `&str`, so the harness first checks the bytes are valid UTF-8 and only
// reaches the XML parse and mrrc's structural validation when they are.
// An `Err` on malformed XML — XML-spec parse errors as well as
// MARCXML-spec structural errors — is correct behavior, so the Result is
// discarded. libfuzzer only flags panics, OOMs, and timeouts.
fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = marcxml_to_record(text);
    }
});
