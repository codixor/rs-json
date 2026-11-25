//! Tokenizers. Both lexers emit the same [`Token`] stream; only where token
//! bytes live differs.
//!
//! * [`slice::SliceLexer`] — whole input in memory; token content is always a
//!   borrowed sub-slice. Used by the tape DOM and the small-file path.
//! * `read::ReadLexer` (feature `read`) — refills a fixed buffer from
//!   `io::Read`; a token that straddles a refill is copied into a reusable
//!   spill buffer (bounded by `Limits::max_token_bytes`).

#[cfg(feature = "read")]
pub mod read;
pub mod slice;

/// Lexical token kinds. Content of `Str`/`Num` is fetched from the lexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    /// `{`
    ObjBegin,
    /// `}`
    ObjEnd,
    /// `[`
    ArrBegin,
    /// `]`
    ArrEnd,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// String; `escaped` = contains at least one `\` escape.
    Str {
        /// Whether unescaping is needed.
        escaped: bool,
    },
    /// Number literal (validated grammar, not converted).
    Num,
    /// `true`
    True,
    /// `false`
    False,
    /// `null`
    Null,
    /// End of input.
    Eof,
}

/// Byte span of the current token's content (quotes excluded for strings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Absolute byte offset of the first content byte.
    pub start: u64,
    /// Content length.
    pub len: u32,
}

/// Common lexer interface used by the pull parser.
pub trait Lexer {
    /// Advance to the next non-whitespace token.
    ///
    /// # Errors
    /// Lexical errors (bad literal, bad escape, bad number, EOF in token).
    fn next_token(&mut self) -> Result<Token, crate::Error>;
    /// Read a token within an absolute document end and a raw token-body limit.
    /// Built-in lexers check before spilling or advancing beyond a bound.
    /// Custom lexers that buffer tokens should override this method to bound
    /// their storage before returning a token.
    ///
    /// # Errors
    /// Returns a lexical error or a size limit at the first excess byte.
    fn next_token_bounded(
        &mut self,
        document_end: u64,
        max_token: usize,
    ) -> Result<Token, crate::Error> {
        let token = self.next_token()?;
        let span = self.span();
        let end = span
            .start
            .saturating_add(u64::from(span.len))
            .saturating_add(u64::from(matches!(token, Token::Str { .. })));
        let body = matches!(
            token,
            Token::Str { .. } | Token::Num | Token::True | Token::False | Token::Null
        );
        let limit = if body && span.len as usize > max_token {
            document_end.min(span.start.saturating_add(max_token as u64))
        } else {
            document_end
        };
        if end > limit {
            return Err(crate::Error {
                kind: crate::ErrorKind::SizeLimit,
                pos: self.position(limit),
            });
        }
        Ok(token)
    }

    /// Span of the token last returned.
    fn span(&self) -> Span;
    /// Raw content bytes of the last `Str`/`Num` token (escapes intact).
    fn content(&self) -> &[u8];
    /// Resolve a byte offset on the error path. Stream lexers can supply line and column.
    fn position(&self, offset: u64) -> crate::Pos {
        crate::Pos {
            offset,
            ..crate::Pos::default()
        }
    }
}
