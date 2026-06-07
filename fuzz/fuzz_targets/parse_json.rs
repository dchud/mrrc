#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::json::json_to_record;

// Drive the plain-JSON reader over arbitrary bytes. The reader takes a
// parsed `serde_json::Value`, so the harness first runs `serde_json` over
// the raw bytes and only reaches mrrc's structural validation when the
// bytes are well-formed JSON. An `Err` from either the JSON parse or the
// record mapping is correct behavior on malformed input, so both Results
// are discarded — libfuzzer only flags panics, OOMs, and timeouts.
fuzz_target!(|data: &[u8]| {
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(data) {
        let _ = json_to_record(&value);
    }
});
