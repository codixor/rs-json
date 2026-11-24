//! Buffered lexer over a synchronous reader. Tokens contained in one refill
//! borrow the buffer. A token crossing a refill uses reusable spill storage.

mod fragment;

use super::slice::SliceLexer;
use super::{Lexer, Span, Token};
use crate::config::{BufConfig, Limits};
use crate::{Error, ErrorKind, Pos};
use alloc::{boxed::Box, vec::Vec};
use core::ops::Range;
use std::io::{self, Read};

/// A streaming lexer with a fixed read buffer and bounded token storage.
#[derive(Debug)]
pub struct ReadLexer<R: Read> {
    reader: R,
    buf: Box<[u8]>,
    head: usize,
    tail: usize,
    eof: bool,
    pos: Pos,
    token_start: Pos,
    span: Span,
    max_token_bytes: u64,
    // While capturing, the current token is spill + buf[segment..head].
    segment: Option<usize>,
    limit_end: u64,
    spill: Vec<u8>,
    local_content: Range<usize>,
}

impl<R: Read> ReadLexer<R> {
    /// Use the requested buffer size. Zero selects the minimum of one byte.
    pub fn new(reader: R, cfg: BufConfig, limits: Limits) -> Self {
        let pos = Pos {
            offset: 0,
            line: 1,
            col: 1,
        };
        Self {
            reader,
            buf: alloc::vec![0; cfg.read_buf.max(1)].into_boxed_slice(),
            head: 0,
            tail: 0,
            eof: false,
            pos,
            token_start: pos,
            max_token_bytes: (limits.max_token_bytes as u64).min(u64::from(u32::MAX)),
            span: Span::default(),
            segment: None,
            limit_end: u64::MAX,
            spill: Vec::new(),
            local_content: 0..0,
        }
    }

    /// Recover the reader and any unread bytes already fetched from it.
    /// Consume these bytes before continuing with the returned reader.
    pub fn into_inner(self) -> (R, Vec<u8>) {
        let rest = self
            .buf
            .get(self.head..self.tail)
            .unwrap_or_default()
            .to_vec();
        (self.reader, rest)
    }

    fn refill(&mut self) -> Result<(), Error> {
        if self.head < self.tail || self.eof {
            return Ok(());
        }
        if let Some(start) = self.segment {
            let part = self
                .buf
                .get(start..self.head)
                .expect("consumed token segment");
            self.spill.extend_from_slice(part);
            self.segment = Some(0);
        }
        self.head = 0;
        self.tail = 0;
        loop {
            match self.reader.read(&mut self.buf) {
                Ok(n) if n <= self.buf.len() => {
                    self.tail = n;
                    self.eof = n == 0;
                    return Ok(());
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                _ => {
                    return Err(Error {
                        kind: ErrorKind::Io,
                        pos: self.pos,
                    });
                }
            }
        }
    }

    fn skip_whitespace(&mut self) -> Result<(), Error> {
        loop {
            let buf = self.chunk()?;
            let len = crate::scan::skip_ws(buf, 0);
            if len == 0 {
                return Ok(());
            }
            let whitespace = buf.get(..len).expect("whitespace prefix");
            let last_lf = whitespace.iter().rposition(|&b| b == b'\n');
            if let Some(last) = last_lf {
                let lines = whitespace.split(|&b| b == b'\n').count() - 1;
                self.pos.line = self.pos.line.saturating_add(saturate_u32(lines));
                self.pos.col = saturate_u32(len - last);
            } else {
                self.pos.col = self.pos.col.saturating_add(saturate_u32(len));
            }
            self.head += len;
            self.pos.offset += len as u64;
        }
    }

    fn peek(&mut self) -> Result<Option<u8>, Error> {
        Ok(self.chunk()?.first().copied())
    }

    fn error(&self, offset: u64, kind: ErrorKind) -> Error {
        Error {
            kind,
            pos: self.position(offset),
        }
    }

    fn chunk(&mut self) -> Result<&[u8], Error> {
        self.refill()?;
        let buf = self.buf.get(self.head..self.tail).expect("unread buffer");
        // At the cap, look at one byte to distinguish a delimiter from an overlong token.
        let budget = usize::try_from(self.limit_end - self.offset())
            .unwrap_or(usize::MAX)
            .max(1);
        Ok(buf.get(..budget).unwrap_or(buf))
    }

    fn offset(&self) -> u64 {
        self.pos.offset
    }

    fn advance(&mut self, len: usize) -> Result<(), Error> {
        if len as u64 > self.limit_end - self.pos.offset {
            return Err(self.error(self.limit_end, ErrorKind::SizeLimit));
        }
        self.head += len;
        self.pos.offset += len as u64;
        self.pos.col = self.pos.col.saturating_add(saturate_u32(len));
        Ok(())
    }

    fn begin_content(&mut self, bounded: bool) {
        self.segment = Some(self.head);
        self.limit_end = if bounded {
            self.pos.offset.saturating_add(self.max_token_bytes)
        } else {
            u64::MAX
        };
    }

    fn end_content(&mut self) {
        let start = self.segment.take().expect("token capture is active");
        self.local_content = start..self.head;
        if !self.spill.is_empty() {
            self.spill.extend_from_slice(
                self.buf
                    .get(self.local_content.clone())
                    .expect("last segment"),
            );
        }
        self.limit_end = u64::MAX;
    }

    fn fragment(&mut self, first: Option<u8>) -> Result<Token, Error> {
        let string = first == Some(b'"');
        if string {
            self.advance(1)?;
        }
        let start = self.offset();
        self.begin_content(string || matches!(first, Some(b'-' | b'0'..=b'9')));
        let kind = match first {
            Some(b'"') => Token::Str {
                escaped: fragment::string_body(self)?,
            },
            Some(b'-' | b'0'..=b'9') => {
                fragment::number(self)?;
                Token::Num
            }
            Some(b't') => {
                fragment::literal(self, b"true")?;
                Token::True
            }
            Some(b'f') => {
                fragment::literal(self, b"false")?;
                Token::False
            }
            Some(b'n') => {
                fragment::literal(self, b"null")?;
                Token::Null
            }
            _ => return Err(self.error(start, ErrorKind::InvalidToken)),
        };
        let len = u32::try_from(self.offset() - start)
            .map_err(|_| self.error(start, ErrorKind::SizeLimit))?;
        self.end_content();
        self.span = Span { start, len };
        if string {
            self.advance(1)?;
        }
        Ok(kind)
    }
}

fn saturate_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

impl<R: Read> Lexer for ReadLexer<R> {
    fn next_token(&mut self) -> Result<Token, Error> {
        self.spill.clear();
        self.segment = None;
        self.limit_end = u64::MAX;
        self.skip_whitespace()?;
        self.token_start = self.pos;
        // Most tokens fit in the current refill. Reuse the slice lexer directly.
        let base = self.pos.offset;
        let cap = self.max_token_bytes;
        let bytes = self.buf.get(self.head..self.tail).expect("unread buffer");
        let first = bytes.first().copied();
        let string = first == Some(b'"');
        let number = matches!(first, Some(b'-' | b'0'..=b'9'));
        // Include the quotes or the number delimiter, but avoid scanning an
        // arbitrarily large refill when the caller selected a small token cap.
        let window = if string || number {
            let width = usize::try_from(cap + 2).unwrap_or(usize::MAX);
            bytes.get(..width).unwrap_or(bytes)
        } else {
            bytes
        };
        let mut slice = SliceLexer::new(window, false);
        if let Ok(kind) = slice.next_token()
            && (kind != Token::Num || slice.offset() < bytes.len())
        {
            let span = slice.span();
            if (string || number) && u64::from(span.len) > cap {
                return Err(self.error(base + span.start + cap, ErrorKind::SizeLimit));
            }
            let start = self.head + usize::try_from(span.start).expect("offset inside buffer");
            self.local_content = start..start + span.len as usize;
            self.span = Span {
                start: base + span.start,
                len: span.len,
            };
            self.advance(slice.offset())?;
            return Ok(kind);
        }

        // The current buffer may end inside a token. A number ending exactly
        // at its edge also needs lookahead for more digits, a fraction or an exponent.
        self.fragment(first)
    }

    fn span(&self) -> Span {
        self.span
    }

    fn content(&self) -> &[u8] {
        if self.spill.is_empty() {
            self.buf.get(self.local_content.clone()).unwrap_or_default()
        } else {
            &self.spill
        }
    }

    fn position(&self, offset: u64) -> Pos {
        // A valid JSON token occupies one line. Even after a refill we can
        // resolve the opening delimiter or an offset within the current token.
        if let Some(delta) = offset.checked_sub(self.token_start.offset) {
            Pos {
                offset,
                line: self.token_start.line,
                col: self
                    .token_start
                    .col
                    .saturating_add(u32::try_from(delta).unwrap_or(u32::MAX)),
            }
        } else {
            Pos {
                offset,
                ..Pos::default()
            }
        }
    }
}
