//! Pull (event) parser over any [`crate::lex::Lexer`]: constant memory,
//! arbitrarily large input. Grammar enforcement mirrors the tape builder
//! but emits events instead of nodes.

#[cfg(feature = "pointer")]
pub mod pointer;

use crate::config::Limits;
use crate::lex::Lexer;
use crate::stack::NestStack;
use crate::Error;

/// Parse events. Content for `Key`/`Str`/`Num` is read via
/// [`PullParser::content`] before the next call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// `{`
    ObjectStart,
    /// `}`
    ObjectEnd,
    /// `[`
    ArrayStart,
    /// `]`
    ArrayEnd,
    /// Object key (`escaped` flag as in `Token::Str`).
    Key {
        /// Needs unescaping.
        escaped: bool,
    },
    /// String value.
    Str {
        /// Needs unescaping.
        escaped: bool,
    },
    /// Number literal.
    Num,
    /// Boolean.
    Bool(bool),
    /// `null`
    Null,
    /// Root value finished; next call returns `Eof` or, in multi-document
    /// mode (NDJSON / concatenated JSON), the next root.
    End,
    /// Input exhausted.
    Eof,
}

/// Streaming pull parser.
#[derive(Debug)]
pub struct PullParser<L: Lexer> {
    lexer: L,
    stack: NestStack,
    limits: Limits,
    multi_document: bool,
}

impl<L: Lexer> PullParser<L> {
    /// Wrap a lexer.
    pub fn new(lexer: L, limits: Limits) -> Self {
        Self { lexer, stack: NestStack::new(), limits, multi_document: false }
    }

    /// Accept a sequence of root values (NDJSON, JSON-seq, concatenated).
    #[must_use]
    pub fn multi_document(mut self, on: bool) -> Self {
        self.multi_document = on;
        self
    }

    /// Next event.
    ///
    /// # Errors
    /// Grammar, lexical, depth or I/O errors.
    pub fn next_event(&mut self) -> Result<Event, Error> {
        let _ = (&mut self.lexer, &mut self.stack, self.limits, self.multi_document);
        todo!("agent: state machine identical to dom::build, emitting events")
    }

    /// Raw bytes of the last `Key`/`Str`/`Num` event.
    pub fn content(&self) -> &[u8] {
        self.lexer.content()
    }

    /// Skip the value that starts at the current position without
    /// materialising it (depth counter only) — the main win for selective
    /// extraction from huge files.
    ///
    /// # Errors
    /// As [`Self::next_event`].
    pub fn skip_value(&mut self) -> Result<(), Error> {
        todo!("agent: fast skip")
    }
}
