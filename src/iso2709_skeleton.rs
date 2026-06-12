//! Shared parse skeleton for the three ISO 2709 reader types.
//!
//! [`MarcReader`], [`AuthorityMarcReader`], and [`HoldingsMarcReader`] all
//! consume the same wire format: a 24-byte leader, a directory of 12-byte
//! entries, and the data area. The variation between them is per-type:
//! which `record_type` byte the leader is allowed to carry, which
//! [`DataFieldParseConfig`] governs subfield parsing, how a parsed [`Field`]
//! is filed into the per-type record output, and (for the bibliographic
//! reader) which error shape the truncated-record directory walk records.
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

use crate::error::{MarcError, Result};
use crate::iso2709::{
    self, DataFieldParseConfig, FIELD_TERMINATOR, LEADER_LEN, ParseContext, is_control_field_tag,
    parse_4digits, parse_5digits, parse_data_field, read_leader_bytes, read_record_data,
};
use crate::leader::Leader;
use crate::record::Field;
use crate::recovery::{RecoveryCap, RecoveryMode, ValidationLevel};
use std::io::Read;

/// Per-type policy + per-record builder for the shared ISO 2709 parse
/// skeleton. Implemented by a small adapter type inside each public reader
/// module; the public reader owns nothing more than `RecoveryCap`,
/// `ParseContext`, and the underlying `Read` source — the per-record output
/// is constructed by the builder under the skeleton's control.
///
/// Default-method bodies match the bibliographic reader's permissive shape:
/// lossy UTF-8 for tags and control-field values, no minimum data-field
/// length. Authority and holdings override only the methods where their
/// behavior actually differs.
pub trait Iso2709Builder: Sized {
    /// The fully-parsed per-record output type (bib `Record`,
    /// authority `AuthorityRecord`, holdings `HoldingsRecord`).
    type Output;

    /// The [`DataFieldParseConfig`] this reader uses for subfield
    /// parsing at the given validation level.
    fn parse_config(level: ValidationLevel) -> DataFieldParseConfig;

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

    /// MARC 21 semantic leader validation invoked by the skeleton at
    /// `validation_level=strict_marc`. The default targets the bibliographic
    /// allowed-value sets via [`crate::RecordStructureValidator::validate_leader`].
    /// Authority and holdings override this to dispatch to the
    /// [`crate::RecordStructureValidator::validate_leader_authority`] and
    /// [`crate::RecordStructureValidator::validate_leader_holdings`] variants
    /// — their allowed sets differ at positions 5, 6, 17, 18, and treat
    /// positions 7, 8, 19 as undefined.
    ///
    /// # Errors
    ///
    /// Returns `MarcError::InvalidLeader` when the leader violates this
    /// reader type's MARC 21 allowed-value sets.
    fn validate_leader_strict_marc(leader: &Leader) -> Result<()> {
        crate::RecordStructureValidator::validate_leader(leader)
    }

    /// Construct a fresh builder around the leader-validated record.
    fn new_for(leader: Leader) -> Self;

    /// File a parsed control field (00X tag) into the in-progress output.
    fn add_control_field(&mut self, tag: String, value: String);

    /// File a parsed data field (non-00X tag) into the in-progress output.
    /// Authority and holdings dispatch by tag here to organize fields by
    /// their functional role (heading, tracings, locations, captions, etc.).
    fn add_data_field(&mut self, tag: String, field: Field);

    /// Decode a control field's bytes into its string value. The
    /// default strips the trailing `FIELD_TERMINATOR` byte and dispatches
    /// on `level`: lossy under [`ValidationLevel::Structural`], strict
    /// (raising [`crate::MarcError::EncodingError`]) under
    /// [`ValidationLevel::StrictMarc`]. Authority overrides to also
    /// strip a trailing `SUBFIELD_DELIMITER`; holdings overrides for its
    /// stricter byte-count guard but uses the same level dispatch.
    ///
    /// # Errors
    ///
    /// Returns `MarcError::EncodingError` when `level` is
    /// [`ValidationLevel::StrictMarc`] and the bytes aren't valid UTF-8.
    /// The lossy path never errors.
    #[inline]
    fn decode_control_field_value(
        field_bytes: &[u8],
        tag: &str,
        ctx: &ParseContext,
        level: ValidationLevel,
    ) -> Result<String> {
        let raw = &field_bytes[..field_bytes.len().saturating_sub(1)];
        match level {
            ValidationLevel::Structural => Ok(String::from_utf8_lossy(raw).to_string()),
            ValidationLevel::StrictMarc => {
                std::str::from_utf8(raw).map(str::to_string).map_err(|e| {
                    ctx.err_encoding(format!("Invalid UTF-8 in control field {tag}: {e}"))
                })
            },
        }
    }

    /// Per-reader minimum data-field byte count guard. Returning `Err`
    /// signals "field too short to parse"; the skeleton handles the
    /// strict-Err / lenient-skip dispatch (with cap accounting) in the
    /// usual way. Default: any non-zero length is fine.
    ///
    /// Authority returns `Err` when `field_bytes.len() < 2` (can't read
    /// indicators); holdings returns `Err` when `< 3`.
    ///
    /// The skeleton invokes this guard only for **data fields**. Control
    /// fields (001-009) decode on a separate path
    /// ([`Iso2709Builder::decode_control_field_value`]) and never reach
    /// it — a control field carries no indicators, so the "too short for
    /// indicators" minimum does not apply. A too-short control field is
    /// not an error here; it decodes to a (possibly empty) value.
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

    /// Error shape for non-digit directory length/start bytes while
    /// walking a truncated record's directory in lenient/permissive
    /// mode. `false` (the default) keeps the walker's usual
    /// [`MarcError::DirectoryInvalid`] (E101) recharacterization on
    /// every walk; the bibliographic reader sets `true` so the
    /// truncated-record walk records the numeric parser's
    /// [`MarcError::InvalidField`] (E106) error instead — the shape
    /// bibliographic salvage diagnostics carry. Non-truncated records
    /// use E101 regardless of this setting.
    const TRUNCATED_WALK_DIGIT_ERRORS_AS_INVALID_FIELD: bool = false;

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
    validation_level: ValidationLevel,
    errors: &mut Vec<MarcError>,
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
    let leader = parse_and_validate_leader::<B>(
        &leader_bytes,
        ctx,
        cap,
        recovery_mode,
        validation_level,
        errors,
    )?;

    let record_length = leader.record_length as usize;

    ctx.advance(LEADER_LEN);

    // Read the full record data. In non-Strict modes a short read returns
    // a zero-padded buffer of `expected_len` plus the actual bytes_read;
    // strict mode has already errored out via `?`. The truncated-record
    // dispatch in `parse_record_body` is the lenient/permissive recovery
    // point. Wrapping in Arc moves the Vec (no byte copy).
    let (record_data, bytes_read) = read_record_data(reader, record_length, recovery_mode, ctx)?;
    let record_data = std::sync::Arc::new(record_data);
    let body_range = 0..record_data.len();
    let buffer_base_offset = ctx.stream_byte_offset;

    parse_record_body::<B>(
        leader,
        &record_data,
        body_range,
        buffer_base_offset,
        bytes_read,
        ctx,
        cap,
        recovery_mode,
        validation_level,
        errors,
    )
}

/// Parse and validate the 24 leader bytes: structural parse, read-readiness
/// validation, optional `StrictMarc` semantic validation (dispatched through
/// the builder), and the per-reader record-type guard.
fn parse_and_validate_leader<B: Iso2709Builder>(
    leader_bytes: &[u8; LEADER_LEN],
    ctx: &mut ParseContext,
    cap: &mut RecoveryCap,
    recovery_mode: RecoveryMode,
    validation_level: ValidationLevel,
    errors: &mut Vec<MarcError>,
) -> Result<Leader> {
    // Leader errors bypass ParseContext (Leader::from_bytes builds MarcError
    // directly via leader_msg); enrich any raised error with positional
    // context from the live ParseContext plus a byte window around the
    // leader bytes for hex-dump rendering. Without with_position, callers
    // see InvalidLeader (E002) without record_index / byte_offset /
    // record_byte_offset / source_name — the structured-positional-context
    // promise of the v0.8 error work.
    let leader_offset = ctx.stream_byte_offset;
    let leader = Leader::from_bytes(leader_bytes).map_err(|e| {
        e.with_position(ctx)
            .with_bytes_near(leader_bytes, leader_offset)
    })?;
    leader.validate_for_reading().map_err(|e| {
        e.with_position(ctx)
            .with_bytes_near(leader_bytes, leader_offset)
    })?;

    // At StrictMarc, run MARC 21 semantic leader validation: record_status,
    // record_type, bibliographic_level, encoding_level, and friends. The
    // allowed-value sets differ per reader type, so dispatch through the
    // builder's `validate_leader_strict_marc` — bibliographic uses the
    // default (`RecordStructureValidator::validate_leader`); authority and
    // holdings override to use their format's allowed-value sets. Surfaces
    // as InvalidLeader (E002) with the same positional enrichment as
    // structural leader errors. In lenient/permissive the violation is
    // recorded against `errors` + `cap` and parsing continues with the
    // (semantically dubious but structurally parseable) leader.
    if validation_level == ValidationLevel::StrictMarc
        && let Err(e) = B::validate_leader_strict_marc(&leader)
    {
        let enriched = e
            .with_position(ctx)
            .with_bytes_near(leader_bytes, leader_offset);
        if recovery_mode == RecoveryMode::Strict {
            return Err(enriched);
        }
        errors.push(enriched);
        cap.note(ctx)?;
    }

    B::validate_record_type(&leader, ctx)?;
    Ok(leader)
}

/// Parse one complete ISO 2709 record (leader + body) from in-memory bytes,
/// with no reader I/O and no per-record byte copies: the caller's buffer is
/// shared with the parse context by refcount, and the directory/field
/// parsers work on slices of it. `record_bytes` is the full record — the
/// 24-byte leader followed by the body — exactly as a length-delimited
/// reader assembles it.
///
/// Returns `Ok(None)` for an empty buffer (EOF parity with the reader
/// entry point) or when the recovery cap is exhausted.
///
/// # Errors
///
/// Same error surface as [`parse_iso2709_record`]: structural and
/// validation failures abort in `RecoveryMode::Strict` and accumulate into
/// `errors` otherwise.
pub fn parse_iso2709_record_from_bytes<B: Iso2709Builder>(
    record_bytes: &std::sync::Arc<Vec<u8>>,
    ctx: &mut ParseContext,
    cap: &mut RecoveryCap,
    recovery_mode: RecoveryMode,
    validation_level: ValidationLevel,
    errors: &mut Vec<MarcError>,
) -> Result<Option<B::Output>> {
    if cap.is_exhausted() || record_bytes.is_empty() {
        return Ok(None);
    }

    ctx.begin_record();

    if record_bytes.len() < LEADER_LEN {
        return Err(ctx.err_truncated_record(Some(LEADER_LEN), Some(record_bytes.len())));
    }
    let leader_bytes: [u8; LEADER_LEN] = record_bytes[..LEADER_LEN]
        .try_into()
        .expect("length checked above");

    let leader = parse_and_validate_leader::<B>(
        &leader_bytes,
        ctx,
        cap,
        recovery_mode,
        validation_level,
        errors,
    )?;

    let record_length = leader.record_length as usize;
    let expected_data_len = record_length.saturating_sub(LEADER_LEN);
    let buffer_base_offset = ctx.stream_byte_offset;

    ctx.advance(LEADER_LEN);

    let body_len = record_bytes.len() - LEADER_LEN;
    if body_len < expected_data_len {
        // Mirror read_record_data's short-read behavior: strict aborts with
        // E005; lenient/permissive parse a zero-padded body so downstream
        // recovery sees the same input shape as the reader path. The pad
        // copy happens only on this truncated-record path.
        if recovery_mode == RecoveryMode::Strict {
            return Err(ctx.err_truncated_record(Some(expected_data_len), Some(body_len)));
        }
        let mut padded = record_bytes[LEADER_LEN..].to_vec();
        padded.resize(expected_data_len, 0);
        let padded = std::sync::Arc::new(padded);
        let range = 0..expected_data_len;
        return parse_record_body::<B>(
            leader,
            &padded,
            range,
            ctx.stream_byte_offset,
            body_len,
            ctx,
            cap,
            recovery_mode,
            validation_level,
            errors,
        );
    }

    // Clamp to the leader's claimed length for parity with the reader path,
    // which reads exactly expected_data_len bytes.
    let body_range = LEADER_LEN..LEADER_LEN + expected_data_len;
    parse_record_body::<B>(
        leader,
        record_bytes,
        body_range,
        buffer_base_offset,
        expected_data_len,
        ctx,
        cap,
        recovery_mode,
        validation_level,
        errors,
    )
}

/// Shared back half of record parsing: directory walk, field decode, and
/// recovery dispatch, operating on `ctx_buffer[body_range]` — the record
/// body. `ctx_buffer` may be the body alone (reader path) or the full
/// record including its leader (slice path); `ctx_buffer_base_offset` is
/// the absolute stream offset of `ctx_buffer[0]` so error hex dumps align
/// either way. `bytes_read` is the count of real (non-padding) body bytes.
#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]
fn parse_record_body<B: Iso2709Builder>(
    leader: Leader,
    ctx_buffer: &std::sync::Arc<Vec<u8>>,
    body_range: std::ops::Range<usize>,
    ctx_buffer_base_offset: usize,
    bytes_read: usize,
    ctx: &mut ParseContext,
    cap: &mut RecoveryCap,
    recovery_mode: RecoveryMode,
    validation_level: ValidationLevel,
    errors: &mut Vec<MarcError>,
) -> Result<Option<B::Output>> {
    let record_length = leader.record_length as usize;
    let base_address = leader.data_base_address as usize;
    let directory_size = base_address - 24;
    let expected_data_len = record_length.saturating_sub(LEADER_LEN);

    // Hand the buffer to the context so `err_*` helpers raised during
    // directory/field parsing capture a bytes_near window for hex-dump
    // rendering; sharing is a refcount bump, not a copy.
    let record_data_offset = ctx.stream_byte_offset;
    ctx.set_parse_buffer(std::sync::Arc::clone(ctx_buffer), ctx_buffer_base_offset);
    let record_data: &[u8] = &ctx_buffer[body_range];

    let truncated = bytes_read < expected_data_len;
    if truncated {
        // recovery_mode is Lenient or Permissive (Strict already returned).
        // Record the truncation and fall through to the clamped directory
        // walk below, which salvages whatever fields the buffer still
        // covers.
        let err = ctx.err_truncated_record(Some(expected_data_len), Some(bytes_read));
        errors.push(err);
        cap.note(ctx)?;
    }

    // The byte at the leader's claimed end-of-record position must be
    // RECORD_TERMINATOR (0x1D); a different byte means the leader's record
    // length disagrees with the data — the record either runs past or stops
    // short of where the leader said. Strict mode surfaces this as E006;
    // lenient/permissive let directory parsing proceed and absorb the
    // disagreement via the existing recovery cap.
    if recovery_mode == RecoveryMode::Strict
        && record_data.len() == record_length - LEADER_LEN
        && record_data.last() != Some(&iso2709::RECORD_TERMINATOR)
    {
        ctx.stream_byte_offset = record_data_offset + record_data.len() - 1;
        return Err(ctx.err_end_of_record_not_found());
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
            let err = ctx
                .err_directory_invalid(Some(&directory[pos..]), "complete 12-byte directory entry");
            if recovery_mode == RecoveryMode::Strict {
                return Err(err);
            }
            errors.push(err);
            cap.note(ctx)?;
            break;
        }

        let entry_chunk = &directory[pos..pos + 12];
        // Tag bytes must be 3 ASCII characters per the codec. Lossy UTF-8
        // conversion would silently replace non-ASCII bytes with U+FFFD,
        // producing a Field whose tag re-encodes to more than 3 bytes —
        // the writer cannot then fit that tag back into the 3-byte
        // directory tag field, breaking round-trip.
        let tag_bytes: &[u8; 3] = entry_chunk[0..3]
            .try_into()
            .expect("entry_chunk guaranteed >= 12 bytes by the slice above");
        if !tag_bytes.iter().all(u8::is_ascii) {
            let err =
                ctx.err_directory_invalid(Some(tag_bytes), "3 ASCII bytes (directory entry tag)");
            if recovery_mode == RecoveryMode::Strict {
                return Err(err);
            }
            errors.push(err);
            cap.note(ctx)?;
            pos += 12;
            continue;
        }
        // SAFETY: every byte is ASCII, hence valid UTF-8.
        let tag = std::str::from_utf8(tag_bytes)
            .expect("ASCII bytes are valid UTF-8")
            .to_string();
        // parse_4digits / parse_5digits build MarcError::InvalidField (E106)
        // for any non-digit byte. In the directory-walker context the
        // offending bytes describe a structurally invalid directory entry,
        // not a malformed data field — recharacterize as DirectoryInvalid
        // (E101) so the variant matches the docs. The InvalidField shape is
        // preserved for the data-field call sites (parse_directory_entry)
        // that share these helpers, and — per the builder's
        // `TRUNCATED_WALK_DIGIT_ERRORS_AS_INVALID_FIELD` — for the
        // bibliographic reader's truncated-record salvage walk.
        let keep_invalid_field = truncated && B::TRUNCATED_WALK_DIGIT_ERRORS_AS_INVALID_FIELD;
        let field_length = match parse_4digits(&entry_chunk[3..7]) {
            Ok(n) => n,
            Err(parse_err) => {
                ctx.current_field_tag = tag.as_bytes().try_into().ok();
                ctx.stream_byte_offset = record_data_offset + pos + 3;
                let err = if keep_invalid_field {
                    parse_err
                } else {
                    ctx.err_directory_invalid(
                        Some(&entry_chunk[3..7]),
                        "4 ASCII digits (directory entry length)",
                    )
                };
                ctx.current_field_tag = None;
                if recovery_mode == RecoveryMode::Strict {
                    return Err(err);
                }
                errors.push(err);
                cap.note(ctx)?;
                pos += 12;
                continue;
            },
        };
        let start_position = match parse_5digits(&entry_chunk[7..12]) {
            Ok(n) => n,
            Err(parse_err) => {
                ctx.current_field_tag = tag.as_bytes().try_into().ok();
                ctx.stream_byte_offset = record_data_offset + pos + 7;
                let err = if keep_invalid_field {
                    parse_err
                } else {
                    ctx.err_directory_invalid(
                        Some(&entry_chunk[7..12]),
                        "5 ASCII digits (directory entry start position)",
                    )
                };
                ctx.current_field_tag = None;
                if recovery_mode == RecoveryMode::Strict {
                    return Err(err);
                }
                errors.push(err);
                cap.note(ctx)?;
                pos += 12;
                continue;
            },
        };
        pos += 12;

        let end_position = start_position + field_length;
        if end_position > data.len() {
            ctx.current_field_tag = tag.as_bytes().try_into().ok();
            let err = ctx.err_invalid_field(format!(
                "Field {tag} exceeds data area (end {end_position} > {})",
                data.len()
            ));
            if recovery_mode == RecoveryMode::Strict {
                return Err(err);
            }
            errors.push(err);
            cap.note(ctx)?;
            // Salvage what bytes we have — extract a clamped slice and try
            // to parse. If the parse fails, silently skip; we already counted
            // the recovery via the field-exceeds-data branch above.
            let available_end = std::cmp::min(end_position, data.len());
            if available_end > start_position {
                let field_data = &data[start_position..available_end];
                if tag != "LDR" {
                    if is_control_field_tag(&tag) {
                        if let Ok(value) =
                            B::decode_control_field_value(field_data, &tag, ctx, validation_level)
                        {
                            if tag == "001" {
                                ctx.record_control_number = Some(value.clone());
                            }
                            builder.add_control_field(tag.clone(), value);
                        }
                    } else if B::validate_data_field_bytes(field_data, &tag, ctx).is_ok() {
                        ctx.current_field_tag = tag.as_bytes().try_into().ok();
                        ctx.stream_byte_offset = record_data_offset + data_start + start_position;
                        if let Ok(field) = parse_data_field(
                            field_data,
                            &tag,
                            B::parse_config(validation_level),
                            ctx,
                        ) {
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
            let value = match B::decode_control_field_value(field_data, &tag, ctx, validation_level)
            {
                Ok(v) => v,
                Err(e) => {
                    if recovery_mode == RecoveryMode::Strict {
                        return Err(e);
                    }
                    errors.push(e);
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

        // Point stream_byte_offset at the field's absolute start and record
        // the field's tag before the per-reader guard runs, so any error the
        // guard raises carries a precise byte_offset (the field's data-area
        // position, not the directory entry) and the correct field_tag. The
        // same state also feeds parse_data_field below for hex-dump caret
        // alignment.
        ctx.current_field_tag = tag.as_bytes().try_into().ok();
        ctx.stream_byte_offset = record_data_offset + data_start + start_position;

        // Per-reader minimum-bytes guard (authority's `< 2`, holdings' `< 3`).
        if let Err(e) = B::validate_data_field_bytes(field_data, &tag, ctx) {
            // Clear field context so it doesn't leak into the next iteration
            // on the lenient skip-continue path (mirrors the reset below).
            ctx.current_field_tag = None;
            if recovery_mode == RecoveryMode::Strict {
                return Err(e);
            }
            errors.push(e);
            cap.note(ctx)?;
            continue;
        }

        // Data field.
        let parsed = parse_data_field(field_data, &tag, B::parse_config(validation_level), ctx);
        ctx.current_field_tag = None;
        match parsed {
            Ok(field) => builder.add_data_field(tag, field),
            Err(e) => {
                if recovery_mode == RecoveryMode::Strict {
                    return Err(e);
                }
                errors.push(e);
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
