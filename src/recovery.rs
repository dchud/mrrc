//! Recovery policy types for malformed and truncated MARC records.
//!
//! This module defines the knobs the ISO 2709 readers expose for handling
//! records that are truncated, malformed, or otherwise incomplete:
//! [`RecoveryMode`] (what to do when an error fires), [`ValidationLevel`]
//! (what counts as an error), and [`RecoveryCap`] (how many recovered
//! errors one stream tolerates). The salvage logic itself — the clamped
//! directory walk that extracts whatever fields a short buffer still
//! covers — lives in [`crate::iso2709_skeleton`].

use crate::error::{MarcError, Result};
use crate::iso2709::ParseContext;

/// Default cap on the number of recovered errors tolerated in one stream
/// before a reader raises [`MarcError::FatalReaderError`] and halts.
pub const DEFAULT_MAX_ERRORS: usize = 10_000;

/// Per-stream recovered-error cap shared by the three ISO 2709 readers.
///
/// In [`RecoveryMode::Lenient`] / [`RecoveryMode::Permissive`], each
/// recovered parse failure allocates a diagnostic object; without a cap these
/// would accumulate without bound on a pathological stream. [`RecoveryCap`]
/// holds the count and trip state. Each reader owns one and calls
/// [`RecoveryCap::note`] at every recovery site.
///
/// The cap is intentionally a struct rather than a trait — the call site is
/// in the per-byte hot path and `dyn`-dispatch would defeat inlining.
#[derive(Debug, Clone)]
pub struct RecoveryCap {
    max_errors: usize,
    error_count: usize,
    exceeded: bool,
}

impl Default for RecoveryCap {
    fn default() -> Self {
        Self::new()
    }
}

impl RecoveryCap {
    /// Construct a cap with the [`DEFAULT_MAX_ERRORS`] limit.
    #[must_use]
    pub fn new() -> Self {
        RecoveryCap {
            max_errors: DEFAULT_MAX_ERRORS,
            error_count: 0,
            exceeded: false,
        }
    }

    /// Set the cap value. `0` disables the cap (unbounded accumulation).
    pub fn set_max(&mut self, n: usize) {
        self.max_errors = n;
    }

    /// Return true once a [`RecoveryCap::note`] call has tripped the cap.
    /// After this is true the owning reader should treat the stream as
    /// exhausted and return `Ok(None)` from `read_record`.
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.exceeded
    }

    /// Record a recovered parse failure against the cap.
    ///
    /// Returns `Err(MarcError::FatalReaderError)` the moment the configured
    /// cap is exceeded, and flags the cap exhausted for future calls.
    /// Returns `Ok(())` when the cap is `0` (disabled) or the count is
    /// still under the limit.
    ///
    /// # Errors
    ///
    /// Returns `MarcError::FatalReaderError` when the cap is exceeded.
    pub fn note(&mut self, ctx: &ParseContext) -> Result<()> {
        if self.max_errors == 0 {
            return Ok(());
        }
        self.error_count += 1;
        if self.error_count > self.max_errors {
            self.exceeded = true;
            let idx = ctx.record_index;
            return Err(MarcError::FatalReaderError {
                cap: self.max_errors,
                errors_seen: self.error_count,
                record_index: if idx == 0 { None } else { Some(idx) },
                source_name: ctx.source_name.clone(),
            });
        }
        Ok(())
    }
}

/// Strategy for handling malformed or truncated records.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecoveryMode {
    /// Strict mode: return errors for any malformation (default)
    #[default]
    Strict,
    /// Lenient mode: attempt to recover and salvage valid data
    Lenient,
    /// Permissive mode: be very lenient with recovery, accepting partial data
    Permissive,
}

/// What counts as an error during parsing — orthogonal to [`RecoveryMode`],
/// which controls what to *do* when one fires.
///
/// Single rule across all readers (bibliographic, authority, holdings):
/// [`Structural`](Self::Structural) is lossy everywhere;
/// [`StrictMarc`](Self::StrictMarc) is strict everywhere.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidationLevel {
    /// Only ISO 2709 structural errors fire (leader, directory, EOR,
    /// base address, truncation). UTF-8 decode is lossy
    /// (`U+FFFD` substitution); indicator and subfield-code byte
    /// validation are skipped. The default — closest to historical
    /// reader behavior and to pymarc 5.3.1.
    #[default]
    Structural,
    /// Adds universal byte-level MARC 21 checks: indicator bytes
    /// (`E201` `InvalidIndicator`), subfield-code bytes
    /// (`E202` `BadSubfieldCode`), and strict UTF-8 decoding
    /// (`E301` `EncodingError`). Applied uniformly across every
    /// reader.
    StrictMarc,
}
