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
