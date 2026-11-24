//! Borrowed views over the tape.

use super::{Document, Node, NodeTag};
use alloc::borrow::Cow;

/// Public value kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
    /// `null`
    Null,
    /// `true`/`false`
    Bool,
    /// Number
    Number,
    /// String
    String,
    /// Array
    Array,
    /// Object
    Object,
}

/// Cheap `Copy` view of one tape node. Lifetimes: `'d` document, `'a` source.
#[derive(Debug, Clone, Copy)]
pub struct Value<'d, 'a> {
    doc: &'d Document<'a>,
    idx: u32,
}

impl<'d, 'a> Value<'d, 'a> {
    pub(crate) fn new(doc: &'d Document<'a>, idx: u32) -> Self {
        Self { doc, idx }
    }

    fn node(&self) -> Node {
        self.doc
            .tape
            .get(self.idx as usize)
            .copied()
            .expect("value indexes a parsed tape node")
    }

    /// Kind of this value.
    pub fn kind(&self) -> ValueKind {
        match self.node().tag {
            NodeTag::Null => ValueKind::Null,
            NodeTag::True | NodeTag::False => ValueKind::Bool,
            NodeTag::Num => ValueKind::Number,
            NodeTag::Str => ValueKind::String,
            NodeTag::Arr => ValueKind::Array,
            NodeTag::Obj => ValueKind::Object,
        }
    }

    /// Raw source text (escapes intact, quotes excluded for strings).
    ///
    /// # Panics
    /// Only if an internal tape invariant is violated. Public constructors validate the tape.
    pub fn raw(&self) -> &'a str {
        let n = self.node();
        let start = n.a as usize;
        let tail = self
            .doc
            .src
            .get(start..)
            .expect("node starts in the source");
        let len = match n.tag {
            NodeTag::Obj | NodeTag::Arr => {
                // Container b stores its child count. Locate the closing delimiter
                // only on demand, without an extra offset on every tape node.
                use crate::lex::{Lexer, Token, slice::SliceLexer};
                let mut lexer = SliceLexer::new(tail, false);
                let mut depth = 0usize;
                loop {
                    match lexer.next_token().expect("the document was validated") {
                        Token::ObjBegin | Token::ArrBegin => depth += 1,
                        Token::ObjEnd | Token::ArrEnd => {
                            depth -= 1;
                            if depth == 0 {
                                break lexer.offset();
                            }
                        }
                        Token::Eof => unreachable!("validated container has a closer"),
                        _ => {}
                    }
                }
            }
            _ => n.b as usize,
        };
        core::str::from_utf8(tail.get(..len).expect("node ends in the source"))
            .expect("token text was validated")
    }

    /// String value: borrowed when unescaped, owned only when it had escapes.
    pub fn as_str(&self) -> Option<Cow<'a, str>> {
        let n = self.node();
        (n.tag == NodeTag::Str).then(|| {
            if n.flags & Node::ESCAPED == 0 {
                Cow::Borrowed(self.raw())
            } else {
                Cow::Owned(crate::unescape::unescape(self.raw()))
            }
        })
    }

    /// Boolean value.
    pub fn as_bool(&self) -> Option<bool> {
        match self.node().tag {
            NodeTag::True => Some(true),
            NodeTag::False => Some(false),
            _ => None,
        }
    }

    /// Integer value (fast path, no float fallback).
    pub fn as_i64(&self) -> Option<i64> {
        (self.node().tag == NodeTag::Num)
            .then(|| crate::num::parse_i64(self.raw().as_bytes()))
            .flatten()
    }

    /// Float value (core `f64::from_str`, already Eisel–Lemire).
    pub fn as_f64(&self) -> Option<f64> {
        (self.node().tag == NodeTag::Num)
            .then(|| self.raw().parse().ok())
            .flatten()
    }

    /// `true` for `null`.
    pub fn is_null(&self) -> bool {
        self.node().tag == NodeTag::Null
    }

    /// Object member lookup. Compares raw key bytes first; falls back to
    /// escape-aware comparison only for keys flagged `ESCAPED` (fixes the old
    /// bug where `{"ab":1}["ab"]` was not found).
    pub fn get(&self, key: &str) -> Option<Self> {
        self.entries()?.find(|(k, _)| *k == key).map(|(_, v)| v)
    }

    /// Array element by index (O(n) skip using subtree end pointers).
    pub fn at(&self, i: usize) -> Option<Self> {
        self.elements()?.nth(i)
    }

    /// Array iterator.
    pub fn elements(&self) -> Option<ArrayIter<'d, 'a>> {
        let n = self.node();
        (n.tag == NodeTag::Arr).then(|| ArrayIter {
            doc: self.doc,
            next: self.idx + 1,
            end: n.c,
        })
    }

    /// Object iterator yielding `(key, value)`.
    pub fn entries(&self) -> Option<ObjectIter<'d, 'a>> {
        let n = self.node();
        (n.tag == NodeTag::Obj).then(|| ObjectIter {
            doc: self.doc,
            next: self.idx + 1,
            end: n.c,
        })
    }

    fn skip(doc: &Document<'_>, idx: u32) -> u32 {
        let n = doc
            .tape
            .get(idx as usize)
            .copied()
            .expect("iterator visits a parsed tape node");
        match n.tag {
            NodeTag::Arr | NodeTag::Obj => n.c,
            _ => idx + 1,
        }
    }
}

/// Array element iterator.
#[derive(Debug, Clone)]
pub struct ArrayIter<'d, 'a> {
    doc: &'d Document<'a>,
    next: u32,
    end: u32,
}

impl<'d, 'a> Iterator for ArrayIter<'d, 'a> {
    type Item = Value<'d, 'a>;
    fn next(&mut self) -> Option<Self::Item> {
        (self.next < self.end).then(|| {
            let v = Value::new(self.doc, self.next);
            self.next = Value::skip(self.doc, self.next);
            v
        })
    }
}

/// Object entry iterator.
#[derive(Debug, Clone)]
pub struct ObjectIter<'d, 'a> {
    doc: &'d Document<'a>,
    next: u32,
    end: u32,
}

/// Key view that compares escape-aware against `&str` without allocating
/// on the unescaped path.
#[derive(Debug, Clone, Copy)]
pub struct Key<'d, 'a>(Value<'d, 'a>);

impl PartialEq<&str> for Key<'_, '_> {
    fn eq(&self, other: &&str) -> bool {
        let n = self.0.node();
        if n.flags & Node::ESCAPED == 0 {
            self.0.raw() == *other
        } else {
            crate::unescape::unescaped_eq(self.0.raw(), other)
        }
    }
}

impl<'d, 'a> Iterator for ObjectIter<'d, 'a> {
    type Item = (Key<'d, 'a>, Value<'d, 'a>);
    fn next(&mut self) -> Option<Self::Item> {
        (self.next < self.end).then(|| {
            let k = Value::new(self.doc, self.next);
            let vi = self.next + 1;
            self.next = Value::skip(self.doc, vi);
            (Key(k), Value::new(self.doc, vi))
        })
    }
}
