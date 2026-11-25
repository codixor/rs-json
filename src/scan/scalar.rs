#[repr(align(64))]
struct CacheAligned<T>(T);

/// `true` for bytes that may appear inside a string without special
/// handling: printable ASCII except `"` and `\`.
#[expect(
    clippy::indexing_slicing,
    reason = "the loop condition bounds the table index"
)]
static BORING: CacheAligned<[bool; 256]> = CacheAligned({
    let mut t = [false; 256];
    let mut i = 0usize;
    while i < 256 {
        #[allow(clippy::cast_possible_truncation)]
        let b = i as u8;
        t[i] = b >= 0x20 && b < 0x80 && b != b'"' && b != b'\\';
        i += 1;
    }
    t
});

/// LUT lookup for a single byte.
#[inline]
#[expect(
    clippy::indexing_slicing,
    reason = "all byte values index the 256-entry table"
)]
pub fn is_string_boring(b: u8) -> bool {
    BORING.0[b as usize]
}

/// Scalar tail: advance from `i` while bytes are boring.
#[inline]
pub(crate) fn finish(buf: &[u8], mut i: usize) -> usize {
    while let Some(&b) = buf.get(i) {
        if !is_string_boring(b) {
            break;
        }
        i += 1;
    }
    i
}

/// Scalar whitespace skip.
#[inline]
pub fn skip_ws_scalar(buf: &[u8], mut i: usize) -> usize {
    while let Some(b' ' | b'\n' | b'\r' | b'\t') = buf.get(i) {
        i += 1;
    }
    i
}
