//! Portable SIMD string kernel on `wide` vectors (pattern from bufjson 1.3).
//!
//! * x86 + `simd-dyn`: AVX2 vs SSE2 chosen once via `is_x86_feature_detected!`
//!   and cached in a `OnceLock` fn pointer.
//! * Otherwise the widest vector known at compile time (`target_feature`).
//!
//! Agent TODO: add the `#[target_feature(enable = "avx2")]` dispatch shim
//! (the only `unsafe` allowed in the crate), with a test that forces each
//! kernel and compares against the scalar LUT.

use wide::u8x16;

trait ByteVec: Copy {
    const N: usize;
    fn load(chunk: &[u8]) -> Option<Self>;
    fn interesting(self) -> u32;
}

impl ByteVec for u8x16 {
    const N: usize = 16;

    #[inline(always)]
    fn load(chunk: &[u8]) -> Option<Self> {
        Some(Self::new(chunk.get(..16)?.try_into().ok()?))
    }

    #[inline(always)]
    fn interesting(self) -> u32 {
        let q = self.simd_eq(Self::splat(b'"'));
        let b = self.simd_eq(Self::splat(b'\\'));
        let c = self.simd_lt(Self::splat(0x20));
        let h = self.simd_gt(Self::splat(0x7f));
        (q | b | c | h).to_bitmask()
    }
}

#[inline(always)]
fn run_end<V: ByteVec>(buf: &[u8], mut i: usize) -> usize {
    while let Some(v) = buf.get(i..).and_then(V::load) {
        let bits = v.interesting();
        if bits != 0 {
            return i + bits.trailing_zeros() as usize;
        }
        i += V::N;
    }
    i
}

/// First interesting byte at/after `i` (vector part only; caller finishes the tail).
#[inline]
pub(crate) fn string_run_end(buf: &[u8], i: usize) -> usize {
    // TODO(agent): AVX2 u8x32 path + runtime dispatch under `simd-dyn`.
    run_end::<u8x16>(buf, i)
}
