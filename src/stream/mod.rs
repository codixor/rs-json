//! Pull events over any [`crate::lex::Lexer`]. The tape builder uses the same
//! grammar. Storage for open containers grows with the configured nesting limit.

#[cfg(feature = "pointer")]
pub mod pointer;

use crate::config::Limits;
use crate::lex::{Lexer, Token};
use crate::stack::{Frame, NestStack};
use crate::{Error, ErrorKind};

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum State {
    #[default]
    Value,
    FirstElement,
    FirstKey,
    Key,
    Colon,
    AfterValue,
    End,
    AfterRoot,
}

/// The DOM builder and pull parser advance this same grammar one event at a time.
#[derive(Debug, Default)]
pub(crate) struct Grammar {
    stack: NestStack,
    state: State,
}

impl Grammar {
    pub(crate) fn clear(&mut self) {
        self.stack.clear();
        self.state = State::Value;
    }

    fn finished_value(&self) -> State {
        if self.stack.depth() == 0 {
            State::End
        } else {
            State::AfterValue
        }
    }

    fn close(&mut self, frame: Frame) -> Event {
        self.stack.pop();
        self.state = self.finished_value();
        match frame {
            Frame::Array => Event::ArrayEnd,
            Frame::Object => Event::ObjectEnd,
        }
    }

    fn value(&mut self, token: Token, max_depth: u32) -> Result<Event, ErrorKind> {
        let (frame, state, start) = match token {
            Token::ObjBegin => (Frame::Object, State::FirstKey, Event::ObjectStart),
            Token::ArrBegin => (Frame::Array, State::FirstElement, Event::ArrayStart),
            _ => {
                let event = match token {
                    Token::Str { escaped } => Event::Str { escaped },
                    Token::Num => Event::Num,
                    Token::True | Token::False => Event::Bool(token == Token::True),
                    Token::Null => Event::Null,
                    _ => return Err(ErrorKind::InvalidToken),
                };
                self.state = self.finished_value();
                return Ok(event);
            }
        };
        self.stack
            .push(frame, max_depth)
            .map_err(|()| ErrorKind::DepthLimit)?;
        self.state = state;
        Ok(start)
    }

    fn step(
        &mut self,
        token: Token,
        max_depth: u32,
        multi: bool,
    ) -> Result<Option<Event>, ErrorKind> {
        if self.state == State::AfterRoot {
            if token == Token::Eof {
                return Ok(Some(Event::Eof));
            }
            if !multi {
                return Err(ErrorKind::TrailingCharacters);
            }
            self.state = State::Value;
        }
        if token == Token::Eof {
            if multi && self.state == State::Value && self.stack.depth() == 0 {
                return Ok(Some(Event::Eof));
            }
            return Err(ErrorKind::UnexpectedEof);
        }
        match (self.state, token) {
            (State::FirstElement, Token::ArrEnd) => Ok(Some(self.close(Frame::Array))),
            (State::FirstKey, Token::ObjEnd) => Ok(Some(self.close(Frame::Object))),
            (State::FirstKey | State::Key, Token::Str { escaped }) => {
                self.state = State::Colon;
                Ok(Some(Event::Key { escaped }))
            }
            (State::Colon, Token::Colon) => {
                self.state = State::Value;
                Ok(None)
            }
            (State::FirstKey | State::Key | State::Colon, _) => Err(ErrorKind::InvalidObject),
            (State::Value | State::FirstElement, _) => self.value(token, max_depth).map(Some),
            (State::AfterValue, _) => match (self.stack.top(), token) {
                (Some(Frame::Array), Token::ArrEnd) => Ok(Some(self.close(Frame::Array))),
                (Some(Frame::Object), Token::ObjEnd) => Ok(Some(self.close(Frame::Object))),
                (Some(Frame::Array), Token::Comma) => {
                    self.state = State::Value;
                    Ok(None)
                }
                (Some(Frame::Object), Token::Comma) => {
                    self.state = State::Key;
                    Ok(None)
                }
                (Some(Frame::Array), _) => Err(ErrorKind::InvalidArray),
                _ => Err(ErrorKind::InvalidObject),
            },
            _ => unreachable!("root transitions are handled before reading a value"),
        }
    }

    pub(crate) fn next<L: Lexer>(
        &mut self,
        lexer: &mut L,
        max_depth: u32,
        multi: bool,
    ) -> Result<Event, Error> {
        if self.state == State::End {
            self.state = State::AfterRoot;
            return Ok(Event::End);
        }
        loop {
            let token = lexer.next_token()?;
            match self.step(token, max_depth, multi) {
                Ok(Some(event)) => return Ok(event),
                Ok(None) => {}
                Err(kind) => {
                    let offset = lexer
                        .span()
                        .start
                        .saturating_sub(u64::from(matches!(token, Token::Str { .. })));
                    return Err(Error {
                        kind,
                        pos: lexer.position(offset),
                    });
                }
            }
        }
    }
}

/// Streaming pull parser.
#[derive(Debug)]
pub struct PullParser<L: Lexer> {
    lexer: L,
    grammar: Grammar,
    limits: Limits,
    multi_document: bool,
    error: Option<Error>,
}

impl<L: Lexer> PullParser<L> {
    /// Wrap a lexer.
    pub fn new(lexer: L, limits: Limits) -> Self {
        Self {
            lexer,
            grammar: Grammar::default(),
            limits,
            multi_document: false,
            error: None,
        }
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
    /// Grammar, lexical, depth or I/O errors. A failed parser retains its error.
    pub fn next_event(&mut self) -> Result<Event, Error> {
        if let Some(error) = self.error {
            return Err(error);
        }
        self.grammar
            .next(&mut self.lexer, self.limits.max_depth, self.multi_document)
            .inspect_err(|&error| self.error = Some(error))
    }

    /// Recover the lexer, including any buffered input it owns.
    pub fn into_inner(self) -> L {
        self.lexer
    }

    /// Raw bytes of the last key, string or number event.
    pub fn content(&self) -> &[u8] {
        self.lexer.content()
    }

    /// Consume the next value and its descendants, applying the same nesting limit.
    ///
    /// # Errors
    /// As [`Self::next_event`], or an invalid token if the next event is not a value.
    pub fn skip_value(&mut self) -> Result<(), Error> {
        match self.next_event()? {
            Event::ObjectStart | Event::ArrayStart => {
                let level = self.grammar.stack.depth();
                while self.grammar.stack.depth() >= level {
                    self.next_event()?;
                }
                Ok(())
            }
            Event::Str { .. } | Event::Num | Event::Bool(_) | Event::Null => Ok(()),
            _ => Err(Error {
                kind: ErrorKind::InvalidToken,
                pos: self.lexer.position(self.lexer.span().start),
            }),
        }
    }
}

#[cfg(feature = "read")]
impl<R: std::io::Read> PullParser<crate::lex::read::ReadLexer<R>> {
    /// Parse a synchronous reader. The lexer and grammar share the supplied limits.
    ///
    /// ```
    /// use rs_json::{Limits, config::BufConfig, stream::{Event, PullParser}};
    /// let mut p = PullParser::from_reader(&b"[null]"[..], BufConfig::default(), Limits::default());
    /// p.skip_value()?;
    /// assert_eq!(p.next_event()?, Event::End);
    /// assert_eq!(p.next_event()?, Event::Eof);
    /// # Ok::<(), rs_json::Error>(())
    /// ```
    pub fn from_reader(reader: R, cfg: crate::config::BufConfig, limits: Limits) -> Self {
        Self::new(
            crate::lex::read::ReadLexer::new(reader, cfg, limits),
            limits,
        )
    }
}
