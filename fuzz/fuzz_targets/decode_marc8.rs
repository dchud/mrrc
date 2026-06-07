#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::encoding::{decode_bytes, MarcEncoding};

// Drive the MARC-8 decoder over arbitrary bytes. MARC-8 is a stateful,
// escape-driven multi-byte encoding (ISO 2022 family): shift-in/shift-out
// sequences, ESC-based charset designations, undefined codepoints, and
// truncated escapes are all input-shaped paths a generic ISO 2709 fuzzer
// reaches only incidentally. `decode_bytes` with `MarcEncoding::Marc8` is
// the public entry to the decoder state machine. An `Err` on bytes the
// decoder cannot map is correct behavior, so the Result is discarded —
// libfuzzer only flags panics, OOMs, and timeouts (the last guards
// against a state machine that never advances its cursor).
fuzz_target!(|data: &[u8]| {
    let _ = decode_bytes(data, MarcEncoding::Marc8);
});
