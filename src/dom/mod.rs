//! Flat tape DOM.
//!
//! The old crate built one `JsonItem` (56 bytes) per value plus one heap
//! `Vec` per container, recursively. The tape stores every value as a
//! 16-byte [`Node`] in document order on the main tape, and containers
//! record the index one past their last descendant, so skipping a subtree
//! is O(1). A small owned document can carry its nodes inline. A reusable
//! [`Parser`] retains the tape and its layout between calls.

mod layout;
mod small;
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

#[derive(Debug)]
enum OwnedTape {
    Inline(small::Inline),
    Heap(Vec<Node>),
}
impl OwnedTape {
    fn len(&self) -> usize {
        match self {
            Self::Inline(t) => t.len(),
            Self::Heap(t) => t.len(),
        }
    }
    fn get(&self, i: usize) -> Option<Node> {
        match self {
            Self::Inline(t) => t.get(i),
            Self::Heap(t) => t.get(i).copied(),
        }
    }
}

/// Parsed document borrowing the source text.
#[derive(Debug)]
pub struct Document<'a> {
    pub(crate) src: &'a [u8],
    tape: OwnedTape,
}

impl<'a> Document<'a> {
    pub(crate) fn from_str(src: &'a str) -> Result<Self, Error> {
        if src.len() <= small::INLINE_BYTES {
            #[cfg(all(feature = "simd", target_pointer_width = "64"))]
            if let Some(tape) = small::Inline::string_object(src.as_bytes()) {
                return Ok(Self {
                    src: src.as_bytes(),
                    tape: OwnedTape::Inline(tape),
                });
            }
            let mut tape = small::Inline::new();
            if small::build(src, Limits::default().max_depth, &mut tape).is_some() {
                return Ok(Self {
                    src: src.as_bytes(),
                    tape: OwnedTape::Inline(tape),
                });
            }
        }
        if src.len() <= small::MAX_BYTES {
            let mut tape = Vec::with_capacity((src.len() / 4 + 1).min(64));
            if small::build(src, Limits::default().max_depth, &mut tape).is_some() {
                return Ok(Self {
                    src: src.as_bytes(),
                    tape: OwnedTape::Heap(tape),
                });
            }
        }
        Self::from_lexer(src.as_bytes(), Limits::default(), SliceLexer::from_str(src))
    }

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
        Ok(Self {
            src,
            tape: OwnedTape::Heap(tape),
        })
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
    layout: layout::Layout,
    open: Vec<u32>,
    /// Limits applied to every parse.
    pub limits: Limits,
}

impl Parser {
    /// New parser with default limits.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse `src`, reusing internal buffers. For small messages with stable token
    /// boundaries, every input byte is checked against the previous tape’s layout.
    /// The returned document borrows
    /// both `src` and `self`.
    ///
    /// # Errors
    /// Invalid JSON or exceeded limits.
    pub fn parse<'p, 'a>(&'p mut self, src: &'a str) -> Result<DocRef<'p, 'a>, Error> {
        if src.len() as u64 <= self.limits.max_document_bytes
            && self.layout.accepts(src, self.limits.max_depth)
        {
            return Ok(DocRef {
                src: src.as_bytes(),
                tape: &self.tape,
            });
        }
        self.layout.reset();
        self.tape.clear();
        if src.len() <= small::MAX_BYTES
            && src.len() as u64 <= self.limits.max_document_bytes
            && small::build(src, self.limits.max_depth, &mut self.tape).is_some()
        {
            self.layout.remember(src, &self.tape, self.limits.max_depth);
            return Ok(DocRef {
                src: src.as_bytes(),
                tape: &self.tape,
            });
        }
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
        self.layout.remember(src, &self.tape, self.limits.max_depth);
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

#[cfg(test)]
mod tests {
    use super::{Document, Parser};
    use alloc::format;
    use alloc::string::String;
    use proptest::prelude::*;

    fn assert_tape(nodes: &[super::Node], doc: &Document<'_>) {
        assert_eq!(nodes.len(), doc.node_count());
        for (i, &node) in nodes.iter().enumerate() {
            assert_eq!(doc.tape.get(i), Some(node));
        }
    }
    fn compare(parser: &mut Parser, input: &str) {
        let reference = Document::parse_with(input.as_bytes(), parser.limits, false);
        if parser.limits == crate::Limits::default() {
            match (crate::parse(input), &reference) {
                (Ok(doc), Ok(expected)) => {
                    assert_eq!(doc.node_count(), expected.node_count());
                    for i in 0..doc.node_count() {
                        assert_eq!(doc.tape.get(i), expected.tape.get(i));
                    }
                }
                (Err(err), Err(expected)) => assert_eq!(&err, expected),
                _ => panic!("owned and byte parsers disagree on {input:?}"),
            }
        }
        match reference {
            Ok(doc) => assert_tape(parser.parse(input).unwrap().tape, &doc),
            Err(err) => assert_eq!(parser.parse(input).unwrap_err(), err),
        }
    }

    #[test]
    fn inline_string_objects_match_the_byte_parser() {
        let mut parser = Parser::new();
        let seeds = [
            r#"{"k":"\uD800\uDC00\nZ","":"a\\b"}"#,
            r#"{"\\":"日\u00e9","a\"b":"\/\b\f\n\r\t"}"#,
            r#"{"long":"abcdefghijklmnop\\","two":""}"#,
            r#"{"":"a relatively long, plain: string"}"#,
        ];
        for seed in seeds {
            #[cfg(all(feature = "simd", target_pointer_width = "64"))]
            assert!(
                super::small::Inline::string_object(seed.as_bytes()).is_some(),
                "{seed}"
            );
            compare(&mut parser, seed);
            // Includes damaged delimiters, escapes, hex digits, surrogates,
            // controls and a closing quote immediately after a backslash.
            for offset in 0..seed.len() {
                let mut input = seed.as_bytes().to_vec();
                for byte in 0..=127 {
                    *input.get_mut(offset).expect("offset within seed") = byte;
                    if let Ok(src) = core::str::from_utf8(&input) {
                        compare(&mut parser, src);
                    }
                }
            }
        }
        #[cfg(all(feature = "simd", target_pointer_width = "64"))]
        for body_len in [15, 16, 17, 31, 32, 33, 47, 48, 49, 63, 64, 65] {
            let text = "x".repeat(body_len - 1);
            let src = format!(r#"{{"key":"{text}"}}"#);
            assert_eq!(
                super::small::Inline::string_object(src.as_bytes()).is_some(),
                (16..=64).contains(&body_len)
            );
            compare(&mut parser, &src);
        }
        for pad in 0..70 {
            let prefix = "x".repeat(pad);
            for tail in [
                r"\\",
                r#"\\\""#,
                r"\uDBFF\uDFFF",
                r"\udfff",
                r"\u123",
                "\\",
                "\x1f",
            ] {
                compare(&mut parser, &format!(r#"{{"x":"{prefix}{tail}","y":""}}"#));
                compare(&mut parser, &format!(r#"{{"x":"","y":"{prefix}{tail}"}}"#));
            }
        }
    }

    #[test]
    fn varying_boundaries_and_escapes_match_the_reference_tape() {
        let mut parser = Parser::new();
        for len in (0..66).chain(4080..4110) {
            let pad = "x".repeat(len);
            for suffix in [r#"\""#, r"\\", r"\ud83d\ude00", "日", "\n", r"\udc00"] {
                let input = format!(r#"{{"k":"{pad}{suffix}","z":[-0.1e-2,true,{{}}]}}"#);
                compare(&mut parser, &input);
            }
        }
    }

    proptest! {
        #[test]
        fn edited_documents_match_the_reference(
            key in ".{0,12}", text in ".{0,48}", n in any::<i32>(),
            edit in 0..256usize, byte in 0u8..128,
        ) {
            let value = serde_json::json!({key: [text, n, false, null, {"v": []}]});
            let mut parser = Parser::new();
            for input in [serde_json::to_string(&value).unwrap(), serde_json::to_string_pretty(&value).unwrap()] {
                compare(&mut parser, &input);
                let mut damaged = input.into_bytes();
                let offset = edit % damaged.len();
                *damaged.get_mut(offset).unwrap() = byte;
                if let Ok(changed) = String::from_utf8(damaged) {
                    compare(&mut parser, &changed);
                }
            }
        }
    }

    #[test]
    fn a_reused_tape_matches_fresh_parsing_for_single_byte_edits() {
        // Include each UTF-16 surrogate boundary, leading zero, fractions,
        // signed exponents, syntax whitespace, enum literals and escaped keys.
        let seeds = [
            r#"{"a\u0062":["a\"b\\c","x\uD7FF","\ud800\udfff","\uE000",-0.12E+3,19]}"#,
            " [ true, false, null, \"str\", 0 ] ",
            r#"{"token":"\udbff\udc00","quote":"\/\t\r\f\n\b"}"#,
        ];
        for seed in seeds {
            let mut parser = Parser::new();
            parser.parse(seed).unwrap();
            parser.parse(seed).unwrap();
            assert!(parser.layout.accepts(seed, parser.limits.max_depth));
            for i in 0..seed.len() {
                let mut bytes = seed.as_bytes().to_vec();
                for b in 0..=127 {
                    *bytes.get_mut(i).unwrap() = b;
                    let edit = core::str::from_utf8(&bytes).unwrap();
                    if parser.layout.accepts(edit, parser.limits.max_depth) {
                        let doc = Document::parse_with(&bytes, parser.limits, false).unwrap();
                        assert_tape(&parser.tape, &doc);
                    }
                }
            }
        }
    }

    #[test]
    fn fresh_values_errors_and_limit_changes_invalidate_stale_tapes() {
        let mut parser = Parser::new();
        let first = r#"{"abc":[12,"\u00e9"]}"#;
        let changed = r#"{"日本":[34,"\u00e4"]}"#;
        let same_width = r#"{"日":[34,"\u00e4"]}"#;
        for s in [
            first,
            first,
            same_width,
            changed,
            r#"{"bad":[00,"\u00e4"]}"#,
            first,
        ] {
            compare(&mut parser, s);
        }
        parser.parse(first).unwrap();
        parser.limits.max_depth = 1;
        compare(&mut parser, first);
        parser.limits.max_depth = 10;
        parser.limits.max_document_bytes = 1;
        compare(&mut parser, first);
    }
}
