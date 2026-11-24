//! Lexer over a complete in-memory buffer.

use super::{Lexer, Span, Token};
use crate::Error;

/// Zero-copy lexer over `&'a [u8]`.
#[derive(Debug)]
pub struct SliceLexer<'a> {
    src: &'a [u8],
    pos: usize,
    span: Span,
    /// When true the input came from `&str` and UTF-8 checks are skipped.
    trusted_utf8: bool,
}

impl<'a> SliceLexer<'a> {
    /// New lexer at offset 0.
    pub fn new(src: &'a [u8], trusted_utf8: bool) -> Self {
        Self { src, pos: 0, span: Span::default(), trusted_utf8 }
    }

    /// Content of the last token with the input lifetime (DOM needs `'a`).
    pub fn content_a(&self) -> &'a [u8] {
        let s = usize::try_from(self.span.start).unwrap_or(usize::MAX);
        self.src.get(s..s + self.span.len as usize).unwrap_or_default()
    }

    /// Current byte offset.
    pub fn offset(&self) -> usize {
        self.pos
    }

    /// Scan a string body starting after the opening quote.
    ///
    /// Algorithm: loop { `scan::string_run_end`; match the stop byte:
    /// `"` → done; `\` → validate escape (incl. surrogate pairs → `LoneSurrogate`),
    /// set escaped; `< 0x20` → `InvalidString`; `>= 0x80` → if !trusted_utf8,
    /// validate the non-ASCII run with `simdutf8::basic::from_utf8` on the
    /// run up to the next ASCII byte }.
    fn lex_string(&mut self) -> Result<Token, Error> {
        let _ = (self.trusted_utf8, crate::scan::string_run_end(self.src, self.pos));
        todo!("agent: implement per doc comment; record self.span")
    }

    /// Validate number grammar (RFC 8259 §6) without converting; port the
    /// old rs-json `parse_number` state machine, tightened to a digit-run
    /// helper that `num::parse_u64` can reuse.
    fn lex_number(&mut self) -> Result<Token, Error> {
        todo!("agent: number grammar")
    }
}

impl Lexer for SliceLexer<'_> {
    fn next_token(&mut self) -> Result<Token, Error> {
        self.pos = crate::scan::skip_ws(self.src, self.pos);
        let Some(&b) = self.src.get(self.pos) else {
            return Ok(Token::Eof);
        };
        // Single-byte dispatch; literals compared as 4/5-byte words.
        match b {
            b'{' => { self.pos += 1; Ok(Token::ObjBegin) }
            b'}' => { self.pos += 1; Ok(Token::ObjEnd) }
            b'[' => { self.pos += 1; Ok(Token::ArrBegin) }
            b']' => { self.pos += 1; Ok(Token::ArrEnd) }
            b':' => { self.pos += 1; Ok(Token::Colon) }
            b',' => { self.pos += 1; Ok(Token::Comma) }
            b'"' => { self.pos += 1; self.lex_string() }
            b'-' | b'0'..=b'9' => self.lex_number(),
            b't' | b'f' | b'n' => todo!("agent: literal match via u32 word compare"),
            _ => Err(Error::at(self.src, self.pos, crate::ErrorKind::InvalidToken)),
        }
    }

    fn span(&self) -> Span {
        self.span
    }

    fn content(&self) -> &[u8] {
        self.content_a()
    }
}
