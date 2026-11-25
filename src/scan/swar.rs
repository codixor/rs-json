//! SIMD-within-a-register kernels on `u64` words (no deps, any target).

const LO: u64 = 0x0101_0101_0101_0101;
const HI: u64 = 0x8080_8080_8080_8080;

#[inline]
fn load(buf: &[u8], i: usize) -> Option<u64> {
    let chunk: [u8; 8] = buf.get(i..i.checked_add(8)?)?.try_into().ok()?;
    Some(u64::from_le_bytes(chunk))
}

/// High bit set in each byte lane that equals `b`.
#[inline]
fn eq_mask(w: u64, b: u8) -> u64 {
    let x = w ^ (LO * u64::from(b));
    x.wrapping_sub(LO) & !x & HI
}

/// Lanes holding `"`, `\`, `< 0x20` or `>= 0x80`.
#[inline]
fn interesting(w: u64) -> u64 {
    let ctrl = w.wrapping_sub(LO * 0x20) & !w & HI; // < 0x20 (exact when no lane >= 0x80)
    eq_mask(w, b'"') | eq_mask(w, b'\\') | ctrl | (w & HI)
}

/// First interesting byte at/after `i`, word at a time; scalar tail is the caller's.
#[allow(dead_code)]
#[inline]
pub(crate) fn string_run_end(buf: &[u8], mut i: usize) -> usize {
    while let Some(w) = load(buf, i) {
        let m = interesting(w);
        if m != 0 {
            // Borrow-propagation false positives only occur *above* a true
            // positive lane, so the lowest set lane is exact.
            return i + (m.trailing_zeros() / 8) as usize;
        }
        i += 8;
    }
    i
}

/// Skip runs of spaces/tabs 8 bytes at a time (indentation), then scalar.
#[inline]
pub(crate) fn skip_ws(buf: &[u8], mut i: usize) -> usize {
    while let Some(w) = load(buf, i) {
        // The subtraction-based equality mask can borrow across byte lanes.
        // Only skip a whole word when every byte is JSON whitespace.
        if !w
            .to_le_bytes()
            .iter()
            .all(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        {
            break;
        }
        i += 8;
    }
    super::skip_ws_scalar(buf, i)
}
