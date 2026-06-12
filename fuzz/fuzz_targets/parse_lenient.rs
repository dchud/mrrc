#![no_main]

use libfuzzer_sys::fuzz_target;
use mrrc::{MarcReader, RecoveryMode};
use std::io::Cursor;

// Drive the full ISO 2709 reader over arbitrary bytes in Lenient
// recovery mode. parse_record drains the same reader in its default
// (Strict) mode, which stops at the first malformed record;
// recovery_mode_consistency reads only the first record per mode. This
// target exercises the salvage path across a whole stream: per-record
// error accumulation, skip-ahead to the next record boundary after a
// malformed record, and the error-cap bookkeeping. An `Err(MarcError)`
// (for example the cap tripping on pathological input) is correct
// behavior, so the Result is discarded — libfuzzer only flags panics,
// OOMs, and timeouts.
fuzz_target!(|data: &[u8]| {
    let mut reader = MarcReader::new(Cursor::new(data)).with_recovery_mode(RecoveryMode::Lenient);
    loop {
        match reader.read_record() {
            Ok(Some(_)) => continue,
            Ok(None) | Err(_) => break,
        }
    }
});
