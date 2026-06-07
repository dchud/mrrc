#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::marcjson::marcjson_to_record;

// Drive the marcjson reader over arbitrary bytes. marcjson is the
// community JSON-for-MARC convention and is a separate module from the
// plain-JSON reader (parse_json), with its own structural shape (a
// top-level object with `leader` and `fields`) and validation. Like the
// plain-JSON reader it takes a parsed `serde_json::Value`, so the harness
// first runs `serde_json` over the raw bytes. An `Err` from either the
// JSON parse or the record mapping is correct behavior on malformed
// input, so both Results are discarded — libfuzzer only flags panics,
// OOMs, and timeouts.
fuzz_target!(|data: &[u8]| {
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(data) {
        let _ = marcjson_to_record(&value);
    }
});
