//! Errors carry only a kind and a byte offset (no allocation on the error
//! path). Line/column are computed lazily from the source when displayed,
//! or tracked per refill chunk by the streaming lexer.

/// Stable error categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Input ended inside a value.
    #[error("unexpected end of input")]
    UnexpectedEof,
    /// Byte that cannot start or continue a token.
    #[error("invalid token")]
    InvalidToken,
    /// Bad escape, control character, or invalid UTF-8 inside a string.
    #[error("invalid string")]
    InvalidString,
    /// `\uD800`-style surrogate without its pair.
    #[error("unpaired UTF-16 surrogate in escape")]
    LoneSurrogate,
    /// Number grammar violation.
    #[error("invalid number")]
    InvalidNumber,
    /// Missing `:`/`,`/`}` or non-string key.
    #[error("invalid object structure")]
    InvalidObject,
    /// Missing `,`/`]`.
    #[error("invalid array structure")]
    InvalidArray,
    /// Non-whitespace after the root value.
    #[error("trailing characters after root value")]
    TrailingCharacters,
    /// Nesting exceeded [`crate::Limits::max_depth`].
    #[error("nesting too deep")]
    DepthLimit,
    /// Input or token exceeded a size limit.
    #[error("size limit exceeded")]
    SizeLimit,
    /// Underlying reader failed (streaming layer).
    #[error("i/o error")]
    Io,
}

/// Location of an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pos {
    /// Byte offset from the start of the input (u64: streams can exceed 4 GiB).
    pub offset: u64,
    /// 1-based line, 0 when not yet resolved.
    pub line: u32,
    /// 1-based column in bytes, 0 when not yet resolved.
    pub col: u32,
}

impl Pos {
    /// Resolve line/column for `offset` by counting newlines in `src`.
    /// Only called on the error path.
    pub fn resolve(src: &[u8], offset: u64) -> Self {
        let end = usize::try_from(offset).unwrap_or(usize::MAX).min(src.len());
        let head = src.get(..end).unwrap_or(src);
        let mut line = 1u32;
        let mut line_start = 0;
        for (i, &byte) in head.iter().enumerate() {
            if byte == b'\n' {
                line = line.saturating_add(1);
                line_start = i + 1;
            }
        }
        Self {
            offset,
            line,
            col: u32::try_from(end - line_start)
                .unwrap_or(u32::MAX)
                .saturating_add(1),
        }
    }
}

/// Parse error: kind + position. 24 bytes, `Copy`, never allocates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{kind} at line {}, column {} (byte {})", pos.line, pos.col, pos.offset)]
pub struct Error {
    /// What went wrong.
    pub kind: ErrorKind,
    /// Where it went wrong.
    pub pos: Pos,
}

impl Error {
    /// Build an error at a byte offset; line/col filled from `src`.
    pub fn at(src: &[u8], offset: usize, kind: ErrorKind) -> Self {
        Self {
            kind,
            pos: Pos::resolve(src, offset as u64),
        }
    }

    /// Byte offset accessor (compat with old `JsonError::index`).
    pub fn offset(&self) -> u64 {
        self.pos.offset
    }
}
