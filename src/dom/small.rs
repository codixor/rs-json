//! Write compact JSON from a validated UTF-8 slice directly into its tape.
//! While a container is open, its end-index field points to its parent.
//! Closing it replaces that link with the final subtree end.

use super::{Node, NodeTag};
use alloc::vec::Vec;

pub(super) const MAX_BYTES: usize = 4096;
const ROOT: u32 = u32::MAX;

// Leave room for the initialized length beside the packed nodes.
const INLINE_NODES: usize =
    (core::mem::size_of::<Vec<Node>>() - 1) / core::mem::size_of::<PackedNode>();

#[derive(Clone, Copy, Debug)]
struct PackedNode {
    tag: NodeTag,
    start: u8,
    len: u8,
    // Scalar flags or a container link, matching the full node.
    aux: u8,
}
const EMPTY: PackedNode = PackedNode {
    tag: NodeTag::Null,
    start: 0,
    len: 0,
    aux: 0,
};

#[derive(Debug)]
pub(super) struct Inline {
    nodes: [PackedNode; INLINE_NODES],
    used: u8,
}

impl Inline {
    pub(super) fn new() -> Self {
        Self {
            nodes: [EMPTY; INLINE_NODES],
            used: 0,
        }
    }
    pub(super) fn len(&self) -> usize {
        usize::from(self.used)
    }
    pub(super) fn get(&self, i: usize) -> Option<Node> {
        if i >= self.len() {
            return None;
        }
        let p = self.nodes.get(i)?;
        let container = matches!(p.tag, NodeTag::Obj | NodeTag::Arr);
        let mut n = Node::scalar(
            p.tag,
            u32::from(p.start),
            u32::from(p.len),
            if container { 0 } else { p.aux },
        );
        if container {
            n.c = u32::from(p.aux);
        }
        Some(n)
    }
}

pub(super) trait Output {
    fn append(&mut self, node: Node) -> Option<()>;
    fn child(&mut self, i: usize) -> Option<NodeTag>;
    fn close(&mut self, i: usize, end: u32) -> Option<u32>;
    fn count(&self) -> usize;
}

impl Output for Inline {
    #[inline]
    #[allow(
        clippy::cast_possible_truncation,
        reason = "inline input offsets fit in a byte; the low byte of ROOT is the parent sentinel"
    )]
    fn append(&mut self, n: Node) -> Option<()> {
        let i = self.len();
        *self.nodes.get_mut(i)? = PackedNode {
            tag: n.tag,
            start: n.a as u8,
            len: n.b as u8,
            aux: if matches!(n.tag, NodeTag::Obj | NodeTag::Arr) {
                n.c as u8
            } else {
                n.flags
            },
        };
        self.used += 1;
        Some(())
    }
    #[inline]
    fn child(&mut self, i: usize) -> Option<NodeTag> {
        let n = self.nodes.get_mut(i)?;
        n.len += 1;
        Some(n.tag)
    }
    #[inline]
    fn close(&mut self, i: usize, end: u32) -> Option<u32> {
        let n = self.nodes.get_mut(i)?;
        let parent = core::mem::replace(&mut n.aux, u8::try_from(end).ok()?);
        Some(if parent == u8::MAX {
            ROOT
        } else {
            u32::from(parent)
        })
    }
    #[inline]
    fn count(&self) -> usize {
        self.len()
    }
}

impl Output for Vec<Node> {
    #[inline]
    fn append(&mut self, n: Node) -> Option<()> {
        self.push(n);
        Some(())
    }
    #[inline]
    fn child(&mut self, i: usize) -> Option<NodeTag> {
        let n = self.get_mut(i)?;
        n.b += 1;
        Some(n.tag)
    }
    #[inline]
    fn close(&mut self, i: usize, end: u32) -> Option<u32> {
        Some(core::mem::replace(&mut self.get_mut(i)?.c, end))
    }
    #[inline]
    fn count(&self) -> usize {
        self.len()
    }
}

pub(super) const INLINE_BYTES: usize = INLINE_NODES * core::mem::size_of::<Node>();

#[allow(
    clippy::inline_always,
    reason = "keep the first quote scan in the caller on short inputs"
)]
#[inline(always)]
fn stop(src: &[u8], mut pos: usize) -> Option<usize> {
    #[cfg(feature = "simd")]
    if src.len() >= 16 {
        use wide::u8x16;
        while pos < src.len() {
            let base = pos.min(src.len() - 16);
            let v = u8x16::new(*src.get(base..)?.first_chunk::<16>()?);
            let bits = (v.simd_eq(u8x16::splat(b'"'))
                | v.simd_eq(u8x16::splat(b'\\'))
                | v.simd_lt(u8x16::splat(32)))
            .to_bitmask()
                >> (pos - base);
            if bits != 0 {
                return Some(pos + bits.trailing_zeros() as usize);
            }
            pos = base + 16;
        }
        return None;
    }
    while let Some(&b) = src.get(pos) {
        if b < 32 || b == b'"' || b == b'\\' {
            return Some(pos);
        }
        pos += 1;
    }
    None
}

#[allow(
    clippy::inline_always,
    reason = "avoid spilling the input cursor and node for each object key"
)]
#[inline(always)]
fn string(src: &[u8], cursor: &mut usize) -> Option<Node> {
    let start = *cursor + 1;
    let end = match src.get(start..)? {
        [b'"', ..] => start,
        [b, b'"', ..] if *b >= 32 && *b != b'\\' => start + 1,
        _ => stop(src, start)?,
    };
    let (end, flags) = match *src.get(end)? {
        b'"' => (end, 0),
        b'\\' => (string_tail(src, end)?, Node::ESCAPED),
        _ => return None,
    };
    *cursor = end + 1;
    Some(Node::scalar(
        NodeTag::Str,
        small_index(start),
        small_index(end - start),
        flags,
    ))
}

fn string_tail(src: &[u8], pos: usize) -> Option<usize> {
    let mut tail = src.get(pos..)?;
    while let Some((&b, rest)) = tail.split_first() {
        match b {
            b'"' => return Some(src.len() - tail.len()),
            b'\\' => {
                let (&kind, after) = rest.split_first()?;
                tail = match kind {
                    b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => after,
                    b'u' => after.get(crate::unescape::validated_unicode_len(after)?..)?,
                    _ => return None,
                };
            }
            0..0x20 => return None,
            _ => tail = rest,
        }
    }
    None
}

#[inline]
fn number(src: &[u8], cursor: &mut usize) -> Option<Node> {
    let start = *cursor;
    let mut p = start + usize::from(src.get(start) == Some(&b'-'));
    match src.get(p)? {
        b'0' => p += 1,
        b'1'..=b'9' => {
            p += 1;
            while src.get(p).is_some_and(u8::is_ascii_digit) {
                p += 1;
            }
        }
        _ => return None,
    }
    let mut flags = Node::NUM_INT;
    if src.get(p) == Some(&b'.') {
        p += 1;
        if !src.get(p)?.is_ascii_digit() {
            return None;
        }
        p += 1;
        while src.get(p).is_some_and(u8::is_ascii_digit) {
            p += 1;
        }
        flags = 0;
    }
    if matches!(src.get(p), Some(b'e' | b'E')) {
        p += 1;
        if matches!(src.get(p), Some(b'+' | b'-')) {
            p += 1;
        }
        if !src.get(p)?.is_ascii_digit() {
            return None;
        }
        p += 1;
        while src.get(p).is_some_and(u8::is_ascii_digit) {
            p += 1;
        }
        flags = 0;
    }
    *cursor = p;
    Some(Node::scalar(
        NodeTag::Num,
        small_index(start),
        small_index(p - start),
        flags,
    ))
}

#[inline]
#[allow(
    clippy::cast_possible_truncation,
    reason = "small documents and their tapes are bounded by MAX_BYTES"
)]
fn small_index(index: usize) -> u32 {
    index as u32
}

#[inline]
fn key(src: &[u8], cursor: &mut usize, tape: &mut impl Output) -> Option<()> {
    if *src.get(*cursor)? != b'"' {
        return None;
    }
    tape.append(string(src, cursor)?)?;
    let colon = *cursor;
    if *src.get(colon)? != b':' {
        return None;
    }
    *cursor = colon + 1;
    Some(())
}

/// A miss is replayed by the full lexer to preserve its exact error and offset.
pub(super) fn build(src: &str, max_depth: u32, tape: &mut impl Output) -> Option<()> {
    debug_assert!(src.len() <= MAX_BYTES);
    let src = src.as_bytes();
    let mut pos = 0;
    let mut parent = ROOT;
    let mut depth = 0;
    'value: loop {
        let start = pos;
        let b = *src.get(pos)?;
        match b {
            b'{' | b'[' => {
                if depth == max_depth {
                    return None;
                }
                depth += 1;
                let (tag, end) = if b == b'{' {
                    (NodeTag::Obj, b'}')
                } else {
                    (NodeTag::Arr, b']')
                };
                let mut node = Node::open(tag, small_index(start));
                node.c = parent;
                parent = small_index(tape.count());
                tape.append(node)?;
                pos += 1;
                if src.get(pos) != Some(&end) {
                    if tag == NodeTag::Obj {
                        key(src, &mut pos, tape)?;
                    }
                    continue;
                }
                let end = small_index(tape.count());
                parent = tape.close(parent as usize, end)?;
                depth -= 1;
                pos += 1;
            }
            _ => {
                let node = match b {
                    b'"' => string(src, &mut pos)?,
                    b'-' | b'0'..=b'9' => number(src, &mut pos)?,
                    _ => {
                        let tail = src.get(pos..)?;
                        let (len, tag) = match b {
                            b't' if tail.starts_with(b"true") => (4, NodeTag::True),
                            b'f' if tail.starts_with(b"false") => (5, NodeTag::False),
                            b'n' if tail.starts_with(b"null") => (4, NodeTag::Null),
                            _ => return None,
                        };
                        pos += len;
                        Node::scalar(tag, small_index(start), small_index(len), 0)
                    }
                };
                tape.append(node)?;
            }
        }
        loop {
            if parent == ROOT {
                return (pos == src.len()).then_some(());
            }
            let end = small_index(tape.count());
            let object = tape.child(parent as usize)? == NodeTag::Obj;
            let delimiter = *src.get(pos)?;
            pos += 1;
            if delimiter == b',' {
                if object {
                    key(src, &mut pos, tape)?;
                }
                continue 'value;
            }
            if delimiter != if object { b'}' } else { b']' } {
                return None;
            }
            parent = tape.close(parent as usize, end)?;
            depth -= 1;
        }
    }
}
