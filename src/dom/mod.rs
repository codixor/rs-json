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

use crate::Error;
use crate::ErrorKind;
use crate::config::Limits;
use crate::lex::{Lexer, slice::SliceLexer};
use crate::stream::{Event, Grammar};
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
        Self::from_lexer(src, limits, SliceLexer::new(src, trusted_utf8))
    }

    pub(crate) fn from_lexer(
        src: &'a [u8],
        limits: Limits,
        lexer: SliceLexer<'a>,
    ) -> Result<Self, Error> {
        let mut tape = Vec::new();
        build(
            src,
            limits,
            lexer,
            &mut tape,
            &mut Grammar::default(),
            &mut Vec::new(),
        )?;
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
    grammar: Grammar,
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
        self.grammar.clear();
        self.open.clear();
        build(
            src.as_bytes(),
            self.limits,
            SliceLexer::from_str(src),
            &mut self.tape,
            &mut self.grammar,
            &mut self.open,
        )?;
        Ok(DocRef {
            src: src.as_bytes(),
            tape: &self.tape,
        })
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

/// Store the shared grammar's events as a flat tape. Only accepted containers
/// enter the open-index stack; a rejected opener cannot allocate a tape node.
fn build(
    src: &[u8],
    limits: Limits,
    mut lexer: SliceLexer<'_>,
    tape: &mut Vec<Node>,
    grammar: &mut Grammar,
    open: &mut Vec<u32>,
) -> Result<(), Error> {
    if src.len() as u64 > limits.max_document_bytes.min(u64::from(u32::MAX)) {
        return Err(Error::at(src, 0, ErrorKind::SizeLimit));
    }
    loop {
        let event = grammar.next(&mut lexer, limits.max_depth, false)?;
        let span = lexer.span();
        // At most one node per input byte, and the document fits u32 offsets.
        let index = u32::try_from(tape.len())
            .map_err(|_| Error::at(src, lexer.offset(), ErrorKind::SizeLimit))?;
        let start = u32::try_from(span.start)
            .map_err(|_| Error::at(src, lexer.offset(), ErrorKind::SizeLimit))?;
        let node = match event {
            Event::End => continue,
            Event::Eof => return Ok(()),
            Event::ArrayEnd | Event::ObjectEnd => {
                let id = open.pop().expect("grammar closed an open container");
                tape.get_mut(id as usize)
                    .expect("open index names a tape node")
                    .c = index;
                continue;
            }
            Event::ArrayStart => Node::open(NodeTag::Arr, start),
            Event::ObjectStart => Node::open(NodeTag::Obj, start),
            Event::Key { escaped } | Event::Str { escaped } => Node::scalar(
                NodeTag::Str,
                start,
                span.len,
                if escaped { Node::ESCAPED } else { 0 },
            ),
            Event::Num => Node::scalar(
                NodeTag::Num,
                start,
                span.len,
                if lexer.number_is_integer() {
                    Node::NUM_INT
                } else {
                    0
                },
            ),
            Event::Bool(true) => Node::scalar(NodeTag::True, start, span.len, 0),
            Event::Bool(false) => Node::scalar(NodeTag::False, start, span.len, 0),
            Event::Null => Node::scalar(NodeTag::Null, start, span.len, 0),
        };
        if !matches!(event, Event::Key { .. })
            && let Some(&id) = open.last()
        {
            tape.get_mut(id as usize).expect("parent is on the tape").b += 1;
        }
        if matches!(event, Event::ArrayStart | Event::ObjectStart) {
            open.push(index);
        }
        tape.push(node);
    }
}
