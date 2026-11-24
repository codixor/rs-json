use thiserror::Error;

const EXTRACT_PADDING: usize = 8;

/// Stable parser error categories for the crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum JsonErrorKind {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("invalid token")]
    InvalidToken,
    #[error("invalid string")]
    InvalidString,
    #[error("invalid number")]
    InvalidNumber,
    #[error("invalid object structure")]
    InvalidObject,
    #[error("invalid array structure")]
    InvalidArray,
    #[error("trailing characters after root value")]
    TrailingCharacters,
}

/// Parser error with a stable kind, byte index, and nearby source extract.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{kind} at index {index} in JSON data")]
pub struct JsonError {
    kind: JsonErrorKind,
    index: usize,
    extract: Option<String>,
}

impl JsonError {
    pub(crate) fn new(source: &str, index: usize, kind: JsonErrorKind) -> Self {
        let start = index.saturating_sub(EXTRACT_PADDING);
        let end = usize::min(source.len(), index.saturating_add(EXTRACT_PADDING));
        let extract = source.get(start..end).map(ToOwned::to_owned);

        Self {
            kind,
            index,
            extract,
        }
    }

    /// Stable parser error category.
    #[must_use]
    pub const fn kind(&self) -> JsonErrorKind {
        self.kind
    }

    /// Byte index where parsing failed.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Optional nearby source extract for diagnostics.
    #[must_use]
    pub const fn extract(&self) -> &Option<String> {
        &self.extract
    }

    /// Legacy compatibility accessor.
    #[must_use]
    pub const fn get_index(&self) -> usize {
        self.index()
    }

    /// Legacy compatibility accessor.
    #[must_use]
    pub const fn get_extract(&self) -> &Option<String> {
        self.extract()
    }
}
