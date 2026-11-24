//! Lazy string unescaping (only runs for strings flagged `ESCAPED`).

use alloc::string::String;

/// Expand escapes in a validated raw string body.
///
/// Agent TODO: copy plain runs found with `scan::string_run_end` in bulk,
/// decode `\uXXXX` including surrogate pairs (input already validated by the
/// lexer, so this is infallible), `String::with_capacity(raw.len())`.
pub fn unescape(raw: &str) -> String {
    let _ = raw;
    todo!("agent: unescape")
}

/// Compare an escaped raw body with plain text without allocating
/// (streaming decode, early exit on first mismatch).
pub fn unescaped_eq(raw: &str, other: &str) -> bool {
    let _ = (raw, other);
    todo!("agent: allocation-free escaped comparison")
}
