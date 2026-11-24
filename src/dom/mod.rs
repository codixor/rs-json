//! Flat tape DOM.
//!
//! The old crate built one `JsonItem` (56 bytes) per value plus one heap
//! `Vec` per container, recursively. The tape stores every value as a
//! 16-byte [`Node`] in a single `Vec`, in document order, and containers
//! record the index one past their last descendant, so skipping a subtree
//! is O(1) and the whole parse does one growable allocation (zero when a
//! [`Parser`] is reused).

mod tape;
mod value;

pub use tape::{Node, NodeTag};
pub use value::{ArrayIter, Key, ObjectIter, Value, ValueKind};

use crate::config::Limits;
use crate::stack::NestStack;
use crate::Error;
use alloc::vec::Vec;

/// Parsed document borrowing the source text.
#[derive(Debug)]
pub struct Document<'a> {
    pub(crate) src: &'a [u8],
    pub(crate) tape: Vec<Node>,
}

impl<'a> Document<'a> {
    /// Parse with explicit limits. `trusted_utf8` = input came from `&str`.
    ///
    /// # Errors
    /// Invalid JSON or exceeded limits.
    pub fn parse_with(src: &'a [u8], limits: Limits, trusted_utf8: bool) -> Result<Self, Error> {
        let mut tape = Vec::new();
        let mut stack = NestStack::new();
        let mut open = Vec::new();
        build(src, limits, trusted_utf8, &mut tape, &mut stack, &mut open)?;
        Ok(Self { src, tape })
    }

    /// Root value view.
    pub fn root(&self) -> Value<'_, 'a> {
        Value::new(self, 0)
    }

    /// Number of tape nodes (for capacity tuning and tests).
    pub fn node_count(&self) -> usize {
        self.tape.len()
    }
}

/// Reusable parser: keeps tape, nesting stack and open-container index
/// allocations across documents. Use for many small documents (API
/// responses, NDJSON lines, websocket frames).
#[derive(Debug, Default)]
pub struct Parser {
    tape: Vec<Node>,
    stack: NestStack,
    open: Vec<u32>,
    /// Limits applied to every parse.
    pub limits: Limits,
}

impl Parser {
    /// New parser with default limits.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse `src`, reusing internal buffers. The returned document borrows
    /// both `src` and `self`.
    ///
    /// # Errors
    /// Invalid JSON or exceeded limits.
    pub fn parse<'p, 'a>(&'p mut self, src: &'a str) -> Result<DocRef<'p, 'a>, Error> {
        self.tape.clear();
        self.stack.clear();
        self.open.clear();
        build(src.as_bytes(), self.limits, true, &mut self.tape, &mut self.stack, &mut self.open)?;
        Ok(DocRef { src: src.as_bytes(), tape: &self.tape })
    }
}

/// Document view whose tape is owned by a [`Parser`].
#[derive(Debug, Clone, Copy)]
pub struct DocRef<'p, 'a> {
    /// Source bytes.
    pub src: &'a [u8],
    /// Borrowed tape.
    pub tape: &'p [Node],
}

/// Iterative tape builder shared by `Document` and `Parser`.
///
/// Algorithm:
/// 1. `tape.reserve(estimate(src.len()))` — start with `len / 8`, tune.
/// 2. Drive `SliceLexer`; on `{`/`[` push `Node::open` and its tape index on
///    `open`, push frame on `stack` (→ `DepthLimit`).
/// 3. Scalars push one node (`start`, `len`, escaped flag).
/// 4. On `}`/`]` pop, patch the open node's `count` and `end` (index past
///    subtree).
/// 5. A tiny state enum (ExpectValue / ExpectKey / ExpectColon /
///    ExpectCommaOrEnd) enforces grammar without recursion.
/// 6. After root: only whitespace allowed (`TrailingCharacters`).
fn build(
    src: &[u8],
    limits: Limits,
    trusted_utf8: bool,
    tape: &mut Vec<Node>,
    stack: &mut NestStack,
    open: &mut Vec<u32>,
) -> Result<(), Error> {
    if src.len() as u64 > limits.max_document_bytes {
        return Err(Error::at(src, 0, crate::ErrorKind::SizeLimit));
    }
    let _ = (trusted_utf8, tape, stack, open);
    todo!("agent: iterative tape builder")
}
