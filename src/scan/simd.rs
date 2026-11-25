//! Portable 16-byte string kernel. The caller finishes a partial vector with scalar loads.

use wide::u8x16;

trait ByteVec: Copy {
    const N: usize;
    fn load(chunk: &[u8]) -> Option<Self>;
    fn interesting(self) -> u32;
}

impl ByteVec for u8x16 {
    const N: usize = 16;

    #[inline]
    fn load(chunk: &[u8]) -> Option<Self> {
        Some(Self::new(chunk.get(..16)?.try_into().ok()?))
    }

    #[inline]
    fn interesting(self) -> u32 {
        let q = self.simd_eq(Self::splat(b'"'));
        let b = self.simd_eq(Self::splat(b'\\'));
        let c = self.simd_lt(Self::splat(0x20));
        let h = self.simd_gt(Self::splat(0x7f));
        (q | b | c | h).to_bitmask()
    }
}

#[inline]
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
    run_end::<u8x16>(buf, i)
}
