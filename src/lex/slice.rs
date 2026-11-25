//! Lexer over a complete in-memory buffer.

use super::{Lexer, Span, Token};
use crate::{Error, ErrorKind, Pos};

/// Zero-copy lexer over a complete byte slice.
#[derive(Debug)]
pub struct SliceLexer<'a> {
    src: &'a [u8],
    pos: usize,
    span: Span,
    trusted_utf8: bool,
    integral: bool,
}

impl<'a> SliceLexer<'a> {
    /// New lexer at offset 0. Byte input is validated as it is consumed, even with a UTF-8 hint.
    pub fn new(src: &'a [u8], _trusted_utf8: bool) -> Self {
        Self {
            src,
            pos: 0,
            span: Span::default(),
            integral: false,
            trusted_utf8: false,
        }
    }

    pub(crate) fn from_str(src: &'a str) -> Self {
        Self {
            src: src.as_bytes(),
            pos: 0,
            span: Span::default(),
            trusted_utf8: true,
            integral: false,
        }
    }

    /// Content of the last token with the input lifetime.
    pub fn content_a(&self) -> &'a [u8] {
        let start = usize::try_from(self.span.start).unwrap_or(self.src.len());
        self.src
            .get(start..start.saturating_add(self.span.len as usize))
            .unwrap_or_default()
    }

    /// Current byte offset.
    pub fn offset(&self) -> usize {
        self.pos
    }

    pub(crate) fn number_is_integer(&self) -> bool {
        self.integral
    }

    fn err(&self, kind: ErrorKind) -> Error {
        Error::at(self.src, self.pos, kind)
    }

    fn byte(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn token(&mut self, start: usize, end: usize, token: Token) -> Result<Token, Error> {
        self.span = Span {
            start: start as u64,
            len: u32::try_from(end - start).map_err(|_| self.err(ErrorKind::SizeLimit))?,
        };
        Ok(token)
    }

    fn lex_string(&mut self, end: usize, document_end: usize) -> Result<Token, Error> {
        self.pos += 1;
        let start = self.pos;
        let mut escaped = false;
        let body = self.src.get(..end).unwrap_or(self.src);
        loop {
            self.pos = crate::scan::string_run_end(body, self.pos);
            if self.pos >= end
                && self.byte().is_some()
                && (self.byte() != Some(b'"') || self.pos >= document_end)
            {
                return Err(Error::at(self.src, end, ErrorKind::SizeLimit));
            }
            match self.byte() {
                None => return Err(self.err(ErrorKind::UnexpectedEof)),
                Some(b'"') => {
                    let end = self.pos;
                    self.pos += 1;
                    return self.token(start, end, Token::Str { escaped });
                }
                Some(b'\\') => {
                    let tail = self.src.get(self.pos + 1..).unwrap_or_default();
                    let (_, used) = crate::unescape::escape(tail).map_err(|kind| self.err(kind))?;
                    escaped = true;
                    self.pos += 1 + used;
                    if self.pos > end {
                        return Err(Error::at(self.src, end, ErrorKind::SizeLimit));
                    }
                }
                Some(0x80..=0xff) => {
                    let tail = body.get(self.pos..).unwrap_or_default();
                    let len = tail.iter().position(u8::is_ascii).unwrap_or(tail.len());
                    let run = tail.get(..len).unwrap_or_default();
                    if !self.trusted_utf8 && simdutf8::basic::from_utf8(run).is_err() {
                        if self.pos + len == end
                            && self.src.get(end).is_some_and(|b| !b.is_ascii())
                            && core::str::from_utf8(run)
                                .is_err_and(|error| error.error_len().is_none())
                        {
                            return Err(Error::at(self.src, end, ErrorKind::SizeLimit));
                        }
                        return Err(self.err(ErrorKind::InvalidString));
                    }
                    self.pos += len;
                }
                _ => return Err(self.err(ErrorKind::InvalidString)),
            }
        }
    }

    fn digits(&mut self) -> Result<(), Error> {
        let start = self.pos;
        while matches!(self.byte(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if start == self.pos {
            Err(self.err(ErrorKind::InvalidNumber))
        } else {
            Ok(())
        }
    }

    fn lex_number(&mut self) -> Result<Token, Error> {
        let start = self.pos;
        if self.byte() == Some(b'-') {
            self.pos += 1;
        }
        if self.byte() == Some(b'0') {
            self.pos += 1;
            if matches!(self.byte(), Some(b'0'..=b'9')) {
                return Err(self.err(ErrorKind::InvalidNumber));
            }
        } else {
            self.digits()?;
        }
        self.integral = !matches!(self.byte(), Some(b'.' | b'e' | b'E'));
        if self.byte() == Some(b'.') {
            self.pos += 1;
            self.digits()?;
        }
        if matches!(self.byte(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.byte(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            self.digits()?;
        }
        self.token(start, self.pos, Token::Num)
    }
}

impl Lexer for SliceLexer<'_> {
    fn next_token(&mut self) -> Result<Token, Error> {
        self.next_token_bounded(u64::MAX, usize::MAX)
    }

    fn next_token_bounded(&mut self, document_end: u64, max_token: usize) -> Result<Token, Error> {
        let document_end = usize::try_from(document_end).unwrap_or(usize::MAX);
        if self.pos > document_end {
            return Err(Error::at(self.src, document_end, ErrorKind::SizeLimit));
        }
        let doc = self.src.get(..document_end).unwrap_or(self.src);
        self.pos = crate::scan::skip_ws(doc, self.pos);
        let start = self.pos;
        if start >= document_end && self.byte().is_some() {
            return Err(self.err(ErrorKind::SizeLimit));
        }
        let (token, len) = match self.byte() {
            None => (Token::Eof, 0),
            Some(b'{') => (Token::ObjBegin, 1),
            Some(b'}') => (Token::ObjEnd, 1),
            Some(b'[') => (Token::ArrBegin, 1),
            Some(b']') => (Token::ArrEnd, 1),
            Some(b':') => (Token::Colon, 1),
            Some(b',') => (Token::Comma, 1),
            Some(b'"') => {
                let end = document_end.min((start + 1).saturating_add(max_token));
                return self.lex_string(end, document_end);
            }
            Some(b'-' | b'0'..=b'9') => {
                let end = document_end.min(start.saturating_add(max_token));
                // One lookahead distinguishes a delimiter from more number bytes.
                let full = self.src;
                self.src = full.get(..end.saturating_add(1)).unwrap_or(full);
                let result = self.lex_number();
                self.src = full;
                if self.pos > end {
                    return Err(Error::at(full, end, ErrorKind::SizeLimit));
                }
                return result;
            }
            Some(b) => {
                let (token, word): (_, &[u8]) = match b {
                    b't' => (Token::True, b"true"),
                    b'f' => (Token::False, b"false"),
                    b'n' => (Token::Null, b"null"),
                    _ => return Err(self.err(ErrorKind::InvalidToken)),
                };
                if !self
                    .src
                    .get(start..)
                    .is_some_and(|tail| tail.starts_with(word))
                {
                    return Err(self.err(ErrorKind::InvalidToken));
                }
                let end = document_end.min(start.saturating_add(max_token));
                if word.len() > end - start {
                    return Err(Error::at(self.src, end, ErrorKind::SizeLimit));
                }
                (token, word.len())
            }
        };
        self.pos += len;
        self.token(start, self.pos, token)
    }

    fn span(&self) -> Span {
        self.span
    }
    fn content(&self) -> &[u8] {
        self.content_a()
    }
    fn position(&self, offset: u64) -> Pos {
        Pos::resolve(self.src, offset)
    }
}
