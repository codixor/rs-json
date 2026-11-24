//! Revalidate a previous tape against a fresh UTF-8 input of the same length.
//! The plan stores byte classes for token payloads and grammar characters for
//! syntax. It never keeps an input reference, a hash, or a copy of a string value.

use super::{Node, NodeTag};
use alloc::vec::Vec;

const MAX_LEN: usize = 512;
const PLAIN: u8 = u8::MAX - b' ';

#[derive(Debug, Default)]
pub(super) struct Layout {
    low: Vec<u8>,
    width: Vec<u8>,
    previous_len: usize,
    depth_limit: u32,
    active: bool,
    cooldown: u8,
}

impl Layout {
    pub(super) fn accepts(&self, src: &str, depth_limit: u32) -> bool {
        self.active
            && depth_limit >= self.depth_limit
            && self.low.len() == src.len()
            && matches_bytes(src.as_bytes(), &self.low, &self.width)
    }

    pub(super) fn reset(&mut self) {
        if self.active {
            // A poorly matching stream should spend its time in the regular parser.
            self.cooldown = 8;
        }
        self.active = false;
    }

    /// Wait for a repeated length before compiling. Reserve during warmup so
    /// the second successful parse needs no additional heap storage.
    pub(super) fn remember(&mut self, src: &str, tape: &[Node], depth_limit: u32) {
        let len = src.len();
        if self.cooldown != 0 {
            self.cooldown -= 1;
            self.previous_len = 0;
            return;
        }
        if len > MAX_LEN || len == 0 {
            self.previous_len = 0;
            return;
        }
        self.low.resize(len, 0);
        self.width.resize(len, 0);
        if core::mem::replace(&mut self.previous_len, len) != len {
            return;
        }
        let bytes = src.as_bytes();
        let mut ranges = self.low.iter_mut().zip(&mut self.width);
        let mut pos = 0;
        // Only the gaps between token bodies contain fixed grammar spelling.
        for n in tape {
            if !matches!(n.tag, NodeTag::Str | NodeTag::Num) {
                continue;
            }
            let start = n.a as usize;
            let end = start + n.b as usize;
            for &b in bytes.get(pos..start).expect("ordered tape spans") {
                set(&mut ranges, b, b);
            }
            let body = bytes.get(start..end).expect("token span in input");
            if n.tag == NodeTag::Num {
                number(body, &mut ranges);
            } else if n.flags & Node::ESCAPED == 0 {
                for _ in body {
                    set(&mut ranges, b' ', u8::MAX);
                }
            } else {
                escaped(body, &mut ranges);
            }
            pos = end;
        }
        for &b in bytes.get(pos..).expect("end of final token") {
            set(&mut ranges, b, b);
        }
        self.depth_limit = depth_limit;
        self.active = true;
    }
}

type Ranges<'a> = core::iter::Zip<core::slice::IterMut<'a, u8>, core::slice::IterMut<'a, u8>>;

#[inline]
fn set(ranges: &mut Ranges<'_>, low: u8, high: u8) {
    let (l, w) = ranges
        .next()
        .expect("one predicate per validated input byte");
    *l = low;
    *w = high - low;
}

fn number(bytes: &[u8], ranges: &mut Ranges<'_>) {
    let first = usize::from(bytes.first() == Some(&b'-'));
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'0' if i == first => set(ranges, b'0', b'0'),
            b'1'..=b'9' if i == first => set(ranges, b'1', b'9'),
            b'0'..=b'9' => set(ranges, b'0', b'9'),
            _ => set(ranges, b, b),
        }
    }
}

fn escaped(bytes: &[u8], ranges: &mut Ranges<'_>) {
    let mut iter = bytes.iter().copied();
    while let Some(b) = iter.next() {
        if b != b'\\' {
            set(ranges, b' ', u8::MAX);
            continue;
        }
        set(ranges, b'\\', b'\\');
        let code = iter.next().expect("validated escape");
        set(ranges, code, code);
        if code == b'u' {
            // Preserve the UTF-16 surrogate class by constraining the first
            // two nibbles. The remaining nibbles may be any digit in the
            // same hex alphabet. A validated pair thus stays a valid pair.
            for i in 0..4 {
                let digit = iter.next().expect("validated hex quad");
                let (lo, hi) = hex_class(i, digit);
                set(ranges, lo, hi);
            }
        }
    }
}

fn hex_class(i: usize, b: u8) -> (u8, u8) {
    match (i, b) {
        (0, b'a'..=b'c') => (b'a', b'c'),
        (0, b'A'..=b'C') => (b'A', b'C'),
        (0, b'd' | b'D') => (b, b),
        (0, b'e'..=b'f') => (b'e', b'f'),
        (0, b'E'..=b'F') => (b'E', b'F'),
        (1, b'0'..=b'7') => (b'0', b'7'),
        (1, b'8'..=b'9') => (b'8', b'9'),
        (1, b'a'..=b'b') => (b'a', b'b'),
        (1, b'A'..=b'B') => (b'A', b'B'),
        (1, b'c'..=b'f') => (b'c', b'f'),
        (1, b'C'..=b'F') => (b'C', b'F'),
        (_, b'0'..=b'9') => (b'0', b'9'),
        (_, b'a'..=b'f') => (b'a', b'f'),
        (_, b'A'..=b'F') => (b'A', b'F'),
        _ => unreachable!("validated hexadecimal digit"),
    }
}

#[inline]
fn scalar(b: u8, lo: u8, width: u8) -> bool {
    b.wrapping_sub(lo) <= width && !(width == PLAIN && matches!(b, b'"' | b'\\'))
}

fn matches_bytes(src: &[u8], low: &[u8], width: &[u8]) -> bool {
    #[cfg(feature = "simd")]
    {
        use wide::u8x16;
        let (input, tail) = src.as_chunks::<16>();
        let (lower, lt) = low.as_chunks::<16>();
        let (widths, wt) = width.as_chunks::<16>();
        let mut bad = u8x16::ZERO;
        for ((chunk, lo), w) in input.iter().zip(lower).zip(widths) {
            let v = u8x16::new(*chunk);
            let w = u8x16::new(*w);
            let special = v.simd_eq(u8x16::splat(b'"')) | v.simd_eq(u8x16::splat(b'\\'));
            bad |= (v - u8x16::new(*lo)).simd_gt(w) | (special & w.simd_eq(u8x16::splat(PLAIN)));
        }
        bad.to_bitmask() == 0
            && tail
                .iter()
                .zip(lt)
                .zip(wt)
                .all(|((&b, &l), &w)| scalar(b, l, w))
    }
    #[cfg(not(feature = "simd"))]
    src.iter()
        .zip(low)
        .zip(width)
        .all(|((&b, &l), &w)| scalar(b, l, w))
}
