//! Byte-classification kernels shared by every lexer.
//!
//! Strategy (from bufjson + the old rs-json `memchr` experiment, which
//! regressed short strings):
//! 1. **Scalar LUT** — one cache-aligned 256-entry table; used for the first
//!    bytes of every string, where most JSON strings already end.
//! 2. **SWAR** — 8 bytes per step in a `u64`, no dependencies, wins for
//!    medium strings and on targets without SIMD.
//! 3. **SIMD** (`simd` feature, `wide`) — 16/32 bytes per step for long runs.
//!    On x86 with `simd-dyn` the AVX2-vs-SSE2 choice is made once at runtime.
//!
//! No input padding is required: every kernel finishes the tail scalar, so
//! the DOM stays zero-copy (simd-json needs a padded mutable copy).

mod scalar;
#[cfg(feature = "simd")]
mod simd;
mod swar;

pub use scalar::{is_string_boring, skip_ws_scalar};

/// Bytes checked with the scalar LUT before switching to wide kernels.
/// Tune in `benches/parse_small.rs` (candidates: 8, 16, 32).
pub const SCALAR_PREFIX: usize = 16;

/// Return the index of the first byte at or after `i` that ends a plain
/// string run: `"`, `\`, a control byte `< 0x20`, or a non-ASCII byte
/// (non-ASCII is "interesting" only so UTF-8 validation can be batched by the
/// caller). Returns `buf.len()` if none.
#[inline]
pub fn string_run_end(buf: &[u8], i: usize) -> usize {
    let end = buf.len().min(i.saturating_add(SCALAR_PREFIX));
    let mut j = i;
    while j < end {
        // SAFETY-free: `j < end <= buf.len()`.
        if !is_string_boring(buf.get(j).copied().unwrap_or(b'"')) {
            return j;
        }
        j += 1;
    }
    if j >= buf.len() {
        return buf.len();
    }
    wide_run_end(buf, j)
}

#[inline]
fn wide_run_end(buf: &[u8], i: usize) -> usize {
    #[cfg(feature = "simd")]
    {
        let j = simd::string_run_end(buf, i);
        scalar::finish(buf, j)
    }
    #[cfg(not(feature = "simd"))]
    {
        let j = swar::string_run_end(buf, i);
        scalar::finish(buf, j)
    }
}

/// Skip JSON whitespace starting at `i`. Compact JSON has 0 bytes of
/// whitespace between tokens and pretty JSON has one newline plus an indent
/// run, so the first byte is checked before any wide kernel is considered.
#[inline]
pub fn skip_ws(buf: &[u8], i: usize) -> usize {
    match buf.get(i) {
        Some(b' ' | b'\n' | b'\r' | b'\t') => {}
        _ => return i,
    }
    let j = i + 1;
    match buf.get(j) {
        Some(b' ' | b'\t') => swar::skip_ws(buf, j),
        _ => skip_ws_scalar(buf, j),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_end_matches_naive() {
        let samples: &[&[u8]] = &[
            b"",
            b"abc\"",
            b"0123456789abcdef0123456789abcdef0123456789\"x",
            b"long run with escape \\n here and more more more more more",
            b"ctrl\x01",
            "unicodé after a while ...................................".as_bytes(),
        ];
        for s in samples {
            for start in 0..=s.len() {
                let naive = s
                    .iter()
                    .skip(start)
                    .position(|&b| !is_string_boring(b))
                    .map_or(s.len(), |p| p + start);
                assert_eq!(string_run_end(s, start), naive, "{s:?} @ {start}");
            }
        }
    }

    #[test]
    fn ws_skip() {
        assert_eq!(skip_ws(b"x", 0), 0);
        assert_eq!(skip_ws(b"\n        \"a\"", 0), 9);
        assert_eq!(skip_ws(b"   ", 0), 3);
    }
}
