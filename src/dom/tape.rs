//! 16-byte tape node.

/// Node type tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NodeTag {
    /// `null`
    Null,
    /// `true`
    True,
    /// `false`
    False,
    /// Number literal; `a` = start, `b` = len.
    Num,
    /// String; `a` = start (after quote), `b` = len; flag `ESCAPED`.
    Str,
    /// Array; `a` = start offset, `b` = element count, `c` = end index.
    Arr,
    /// Object; `a` = start offset, `b` = entry count, `c` = end index.
    /// Children alternate key `Str` node, value node.
    Obj,
}

/// One value on the tape. Exactly 16 bytes (asserted below).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Node {
    /// Kind.
    pub tag: NodeTag,
    /// Bit flags (`ESCAPED`, future `NUM_INT` / `NUM_NEG` hints).
    pub flags: u8,
    _pad: u16,
    /// Start byte offset.
    pub a: u32,
    /// Length (scalars) or child count (containers).
    pub b: u32,
    /// Containers: tape index one past the last descendant.
    pub c: u32,
}

const _: () = assert!(core::mem::size_of::<Node>() == 16);

impl Node {
    /// String contains escapes.
    pub const ESCAPED: u8 = 1;
    /// Number has no `.`/`e` (lexer already knows; saves a rescan).
    pub const NUM_INT: u8 = 2;

    /// Scalar node.
    pub const fn scalar(tag: NodeTag, start: u32, len: u32, flags: u8) -> Self {
        Self {
            tag,
            flags,
            _pad: 0,
            a: start,
            b: len,
            c: 0,
        }
    }

    /// Container node to be patched on close.
    pub const fn open(tag: NodeTag, start: u32) -> Self {
        Self {
            tag,
            flags: 0,
            _pad: 0,
            a: start,
            b: 0,
            c: 0,
        }
    }
}
