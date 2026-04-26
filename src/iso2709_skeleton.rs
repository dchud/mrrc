//! Shared parse skeleton for the three ISO 2709 reader types.
//!
//! [`MarcReader`], [`AuthorityMarcReader`], and [`HoldingsMarcReader`] all
//! consume the same wire format: a 24-byte leader, a directory of 12-byte
//! entries, and the data area. The variation between them is per-type:
//! which `record_type` byte the leader is allowed to carry, which
//! [`DataFieldParseConfig`] governs subfield parsing, how a parsed [`Field`]
//! is filed into the per-type record output, and (for the bibliographic
//! reader) what to do when a record is truncated mid-stream.
//!
//! [`Iso2709Builder`] captures those per-type policies as a single trait;
//! [`parse_iso2709_record`] is the generic skeleton that drives one record's
//! parse against any builder. Each reader's `read_record` collapses to a
//! one-line dispatch through the skeleton.
//!
//! ## Performance note
//!
//! The skeleton is parameterized `<R: Read, B: Iso2709Builder>` and
//! intentionally avoids `dyn` dispatch — every call site monomorphizes the
//! function for its concrete builder, so trait method calls inline at
//! generated-code level. This preserves the bibliographic read path's
//! hot-loop characteristics; switching to a trait object dispatch here
//! has been measured to regress parallel reader benchmarks
//! significantly. Re-verify with `cargo bench --bench parallel_benchmarks`
//! before changing dispatch shape.
//!
//! [`MarcReader`]: crate::MarcReader
//! [`AuthorityMarcReader`]: crate::AuthorityMarcReader
//! [`HoldingsMarcReader`]: crate::HoldingsMarcReader

use crate::error::Result;
use crate::iso2709::{
    self, is_control_field_tag, parse_4digits, parse_5digits, parse_data_field, read_leader_bytes,
    read_record_data, DataFieldParseConfig, ParseContext, FIELD_TERMINATOR, LEADER_LEN,
};
use crate::leader::Leader;
use crate::record::Field;
use crate::recovery::{RecoveryCap, RecoveryMode};
use std::io::Read;

/// Per-type policy + per-record builder for the shared ISO 2709 parse
/// skeleton. Implemented by a small adapter type inside each public reader
/// module; the public reader owns nothing more than `RecoveryCap`,
/// `ParseContext`, and the underlying `Read` source — the per-record output
/// is constructed by the builder under the skeleton's control.
///
/// Default-method bodies match the bibliographic reader's permissive shape:
/// lossy UTF-8 for tags and control-field values, no minimum data-field
/// length, no truncated-record salvage. Authority and holdings override
/// only the methods where their behavior actually differs.
pub trait Iso2709Builder: Sized {
    /// The fully-parsed per-record output type (bib `Record`,
    /// authority `AuthorityRecord`, holdings `HoldingsRecord`).
    type Output;

    /// The [`DataFieldParseConfig`] this reader uses for subfield parsing.
    fn parse_config() -> DataFieldParseConfig;

    /// Validate the parsed leader's `record_type` byte for this reader.
    /// Returns `Err` to abort the record (e.g., authority requires `'z'`).
    /// The bibliographic reader accepts any leader and leaves this as the
    /// trait default of `Ok(())`.
    ///
    /// # Errors
    ///
    /// Returns `MarcError` (typically `InvalidField`) when `leader.record_type`
    /// is not allowed for this reader.
    fn validate_record_type(leader: &Leader, ctx: &ParseContext) -> Result<()> {
        let _ = (leader, ctx);
        Ok(())
    }

    /// Construct a fresh builder around the leader-validated record.
    fn new_for(leader: Leader) -> Self;

    /// File a parsed control field (00X tag) into the in-progress output.
    fn add_control_field(&mut self, tag: String, value: String);

    /// File a parsed data field (non-00X tag) into the in-progress output.
    /// Authority and holdings dispatch by tag here to organize fields by
    /// their functional role (heading, tracings, locations, captions, etc.).
    fn add_data_field(&mut self, tag: String, field: Field);

    /// Decode a control field's bytes into its string value. The default
    /// strips the trailing `FIELD_TERMINATOR` byte and decodes lossily.
    /// Authority overrides to also strip a trailing `SUBFIELD_DELIMITER`;
    /// holdings overrides to use strict UTF-8 (raising
    /// [`crate::MarcError::EncodingError`] on bad bytes). The hook lets
    /// each reader keep its historical strict-vs-lossy policy.
    ///
    /// # Errors
    ///
    /// Returns `MarcError` when the implementing reader applies a strict
    /// decode policy and the bytes are not valid UTF-8. The default
    /// (lossy) implementation never errors.
    #[inline]
    fn decode_control_field_value(
        field_bytes: &[u8],
        tag: &str,
        ctx: &ParseContext,
    ) -> Result<String> {
        let _ = (tag, ctx);
        Ok(
            String::from_utf8_lossy(&field_bytes[..field_bytes.len().saturating_sub(1)])
                .to_string(),
        )
    }

    /// Per-reader minimum data-field byte count guard. Returning `Err`
    /// signals "field too short to parse"; the skeleton handles the
    /// strict-Err / lenient-skip dispatch (with cap accounting) in the
    /// usual way. Default: any non-zero length is fine.
    ///
    /// Authority returns `Err` when `field_bytes.len() < 2` (can't read
    /// indicators); holdings returns `Err` when `< 3`.
    ///
    /// # Errors
    ///
    /// Returns `MarcError::InvalidField` when the byte count is below
    /// the implementing reader's minimum.
    #[inline]
    fn validate_data_field_bytes(field_bytes: &[u8], tag: &str, ctx: &ParseContext) -> Result<()> {
        let _ = (field_bytes, tag, ctx);
        Ok(())
    }

    /// Per-reader truncated-record salvage. The skeleton calls this only
    /// in lenient/permissive mode after recording the truncation against
    /// the cap. `None` (the default) means "no per-type salvage path —
    /// fall through to best-effort directory parsing". The bibliographic
    /// reader overrides this to invoke
    /// [`crate::recovery::try_recover_record`].
    #[must_use]
    fn try_recover_truncated(
        leader: Leader,
        partial_data: &[u8],
        base_address: usize,
        mode: RecoveryMode,
        ctx: &ParseContext,
    ) -> Option<Result<Self::Output>> {
        let _ = (leader, partial_data, base_address, mode, ctx);
        None
    }

    /// Finalize the in-progress builder into the per-reader output.
    fn finalize(self) -> Self::Output;
}

/// Drive one record's parse from a byte source through a per-type builder.
///
/// Returns `Ok(None)` when the source is at EOF before the leader, or when
/// the cap was previously exhausted by an earlier call. Returns
/// `Ok(Some(output))` for a successfully parsed record.
///
/// All recovery-branch sites (incomplete directory entry, bad field-length
/// digits, bad start-position digits, field-extends-beyond-data,
/// `parse_data_field` failure, per-type minimum-byte-count violations)
/// follow the same shape: in [`RecoveryMode::Strict`] the first error
/// propagates; in [`RecoveryMode::Lenient`] / [`RecoveryMode::Permissive`]
/// the offending entry is skipped and the recovery is recorded against
/// the per-stream `cap`. Once the cap is exhausted, this and all subsequent
/// calls return `Ok(None)`.
///
/// # Errors
///
/// Returns `MarcError` on the first unrecovered parse failure: malformed
/// leader, structural directory error in strict mode, I/O error from the
/// underlying reader, or `MarcError::FatalReaderError` when the cap is
/// exceeded.
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub fn parse_iso2709_record<R, B>(
    reader: &mut R,
    ctx: &mut ParseContext,
    cap: &mut RecoveryCap,
    recovery_mode: RecoveryMode,
) -> Result<Option<B::Output>>
where
    R: Read,
    B: Iso2709Builder,
{
    if cap.is_exhausted() {
        return Ok(None);
    }

    let Some(leader_bytes) = read_leader_bytes(reader)? else {
        return Ok(None);
    };

    ctx.begin_record();

    // Leader errors bypass ParseContext (Leader::from_bytes builds MarcError
    // directly); enrich any raised error with a byte window around the leader
    // bytes for hex-dump rendering.
    let leader_offset = ctx.stream_byte_offset;
    let leader = Leader::from_bytes(&leader_bytes)
        .map_err(|e| e.with_bytes_near(&leader_bytes, leader_offset))?;
    leader
        .validate_for_reading()
        .map_err(|e| e.with_bytes_near(&leader_bytes, leader_offset))?;

    B::validate_record_type(&leader, ctx)?;

    let record_length = leader.record_length as usize;
    let base_address = leader.data_base_address as usize;
    let directory_size = base_address - 24;

    ctx.advance(LEADER_LEN);

    // Read the full record data. In non-Strict modes a short read returns
    // `(buffer, true)`; salvage from a partial buffer is not implemented in
    // the read primitive so the truncated-record dispatch below is the
    // recovery point.
    let (record_data, _was_truncated) =
        read_record_data(reader, record_length, recovery_mode, ctx)?;

    // Hand the loaded buffer to the context so `err_*` helpers raised during
    // directory/field parsing capture a bytes_near window for hex-dump
    // rendering.
    let record_data_offset = ctx.stream_byte_offset;
    ctx.set_parse_buffer(&record_data, record_data_offset);

    if record_data.len() < (record_length - 24) {
        if recovery_mode == RecoveryMode::Strict {
            return Err(ctx.err_truncated_record(
                Some(record_length.saturating_sub(LEADER_LEN)),
                Some(record_data.len()),
            ));
        }
        cap.note(ctx)?;
        // Per-type recovery hook (bibliographic uses try_recover_record;
        // authority + holdings fall through to best-effort directory parsing).
        if let Some(result) = B::try_recover_truncated(
            leader.clone(),
            &record_data,
            base_address,
            recovery_mode,
            ctx,
        ) {
            return result.map(Some);
        }
    }

    // Clamp directory + data slices at the actual buffer length so a short
    // read in lenient mode does not panic.
    let directory_end = std::cmp::min(directory_size, record_data.len());
    let directory: &[u8] = if directory_end > 0 {
        &record_data[..directory_end]
    } else {
        &[]
    };
    let data_start = std::cmp::min(base_address - 24, record_data.len());
    let data: &[u8] = if data_start < record_data.len() {
        &record_data[data_start..]
    } else {
        &[]
    };

    let mut builder = B::new_for(leader);

    // Walk directory entries (12 bytes each: tag(3) + length(4) + start(5)),
    // terminated by `FIELD_TERMINATOR`.
    let mut pos = 0;
    while pos < directory.len() {
        // Keep stream_byte_offset pointed at the current directory byte so
        // errors raised below carry a precise byte_offset and the bytes_near
        // hex-dump caret lands at the actual offending byte.
        ctx.stream_byte_offset = record_data_offset + pos;
        if directory[pos] == FIELD_TERMINATOR {
            break;
        }

        if pos + 12 > directory.len() {
            if recovery_mode == RecoveryMode::Strict {
                return Err(ctx.err_directory_invalid(
                    Some(&directory[pos..]),
                    "complete 12-byte directory entry",
                ));
            }
            cap.note(ctx)?;
            break;
        }

        let entry_chunk = &directory[pos..pos + 12];
        let tag = String::from_utf8_lossy(&entry_chunk[0..3]).to_string();
        let field_length = match parse_4digits(&entry_chunk[3..7]) {
            Ok(len) => len,
            Err(e) => {
                if recovery_mode == RecoveryMode::Strict {
                    return Err(e);
                }
                cap.note(ctx)?;
                pos += 12;
                continue;
            },
        };
        let start_position = match parse_5digits(&entry_chunk[7..12]) {
            Ok(s) => s,
            Err(e) => {
                if recovery_mode == RecoveryMode::Strict {
                    return Err(e);
                }
                cap.note(ctx)?;
                pos += 12;
                continue;
            },
        };
        pos += 12;

        let end_position = start_position + field_length;
        if end_position > data.len() {
            if recovery_mode == RecoveryMode::Strict {
                ctx.current_field_tag = tag.as_bytes().try_into().ok();
                return Err(ctx.err_invalid_field(format!(
                    "Field {tag} exceeds data area (end {end_position} > {})",
                    data.len()
                )));
            }
            cap.note(ctx)?;
            // Salvage what bytes we have — extract a clamped slice and try
            // to parse. If the parse fails, silently skip; we already counted
            // the recovery via the field-exceeds-data branch above.
            let available_end = std::cmp::min(end_position, data.len());
            if available_end > start_position {
                let field_data = &data[start_position..available_end];
                if tag != "LDR" {
                    if is_control_field_tag(&tag) {
                        if let Ok(value) = B::decode_control_field_value(field_data, &tag, ctx) {
                            if tag == "001" {
                                ctx.record_control_number = Some(value.clone());
                            }
                            builder.add_control_field(tag.clone(), value);
                        }
                    } else if B::validate_data_field_bytes(field_data, &tag, ctx).is_ok() {
                        ctx.current_field_tag = tag.as_bytes().try_into().ok();
                        ctx.stream_byte_offset = record_data_offset + data_start + start_position;
                        if let Ok(field) =
                            parse_data_field(field_data, &tag, B::parse_config(), ctx)
                        {
                            builder.add_data_field(tag, field);
                        }
                        ctx.current_field_tag = None;
                    }
                }
            }
            continue;
        }

        let field_data = &data[start_position..end_position];

        if tag == "LDR" {
            continue;
        }

        if is_control_field_tag(&tag) {
            let value = match B::decode_control_field_value(field_data, &tag, ctx) {
                Ok(v) => v,
                Err(e) => {
                    if recovery_mode == RecoveryMode::Strict {
                        return Err(e);
                    }
                    cap.note(ctx)?;
                    continue;
                },
            };
            if tag == "001" {
                ctx.record_control_number = Some(value.clone());
            }
            builder.add_control_field(tag, value);
            continue;
        }

        // Per-reader minimum-bytes guard (authority's `< 2`, holdings' `< 3`).
        if let Err(e) = B::validate_data_field_bytes(field_data, &tag, ctx) {
            if recovery_mode == RecoveryMode::Strict {
                return Err(e);
            }
            cap.note(ctx)?;
            continue;
        }

        // Data field. Point stream_byte_offset at the field's absolute start
        // so any error raised inside parse_data_field carries a precise
        // byte_offset for hex-dump caret alignment.
        ctx.current_field_tag = tag.as_bytes().try_into().ok();
        ctx.stream_byte_offset = record_data_offset + data_start + start_position;
        let parsed = parse_data_field(field_data, &tag, B::parse_config(), ctx);
        ctx.current_field_tag = None;
        match parsed {
            Ok(field) => builder.add_data_field(tag, field),
            Err(e) => {
                if recovery_mode == RecoveryMode::Strict {
                    return Err(e);
                }
                cap.note(ctx)?;
            },
        }
    }

    // Restore stream_byte_offset to the end of the current record. The
    // directory/field loop above moved it mid-record for precise error-
    // offset alignment; this restores the invariant that stream_byte_offset
    // equals bytes consumed from the stream.
    ctx.stream_byte_offset = record_data_offset + record_length.saturating_sub(LEADER_LEN);
    Ok(Some(builder.finalize()))
}

// Re-export a couple of names callers commonly want alongside the trait
// without forcing them to also `use crate::iso2709::...`.
pub use iso2709::{DataFieldParseConfig as ParseConfig, FIELD_TERMINATOR as DIRECTORY_TERMINATOR};
