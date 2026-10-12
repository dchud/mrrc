//! Code for `docs/reference/error-codes.md`.

use mrrc::MarcError;

fn handle_indicator_error(e: MarcError) {}

fn match_on_code(err: MarcError) -> mrrc::Result<()> {
    // --8<-- [start:match-on-code]
    match err {
        e if e.code() == "E201" => handle_indicator_error(e),
        e => return Err(e),
    }
    // --8<-- [end:match-on-code]
    Ok(())
}
