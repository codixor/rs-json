use super::JsonErrorKind;
use thiserror::Error;

/// Parser error with a stable kind, byte index and source position.
#[derive(Clone, PartialEq, Eq, Error)]
#[error("{kind} at index {index} in JSON data")]
pub struct JsonError {
    kind: JsonErrorKind,
    category: crate::Category,
    index: usize,
    pos: crate::Pos,
}

impl JsonError {
    pub(super) fn from_parse(source: &str, error: crate::Error) -> Self {
        let (kind, index) = Self::location(source, error);
        let trailing = kind == JsonErrorKind::TrailingCharacters;
        Self {
            kind,
            category: if trailing {
                kind.category()
            } else {
                error.kind.category()
            },
            index,
            pos: if trailing {
                crate::Pos::resolve(source.as_bytes(), index as u64)
            } else {
                error.pos
            },
        }
    }

    fn location(source: &str, error: crate::Error) -> (JsonErrorKind, usize) {
        use crate::{
            lex::slice::SliceLexer,
            stream::{Event, Grammar},
        };
        // 0.1 classifies any nonspace tail after a valid root as trailing data,
        // even when the next token itself is lexically invalid.
        if matches!(
            error.kind,
            JsonErrorKind::InvalidToken | JsonErrorKind::UnexpectedEof
        ) {
            let bytes = source.as_bytes();
            let mut lexer = SliceLexer::from_str(source);
            let mut grammar = Grammar::default();
            loop {
                let previous = lexer.offset();
                match grammar.next(&mut lexer, crate::Limits::default(), false) {
                    Ok(Event::End) => {
                        return (
                            JsonErrorKind::TrailingCharacters,
                            crate::scan::skip_ws(bytes, previous),
                        );
                    }
                    Ok(Event::Eof) => break,
                    Ok(_) => {}
                    Err(_) => {
                        if error.kind == JsonErrorKind::UnexpectedEof {
                            // A grammar step can consume a comma or colon before reading its value.
                            let mut start = crate::scan::skip_ws(bytes, previous);
                            if matches!(bytes.get(start), Some(b',' | b':')) {
                                start = crate::scan::skip_ws(bytes, start + 1);
                            }
                            match bytes.get(start) {
                                Some(b'-' | b'0'..=b'9') => {
                                    return (JsonErrorKind::InvalidNumber, source.len());
                                }
                                Some(b't' | b'f' | b'n') => {
                                    return (JsonErrorKind::InvalidToken, start);
                                }
                                _ => {}
                            }
                        }
                        break;
                    }
                }
            }
        }
        (
            error.kind,
            usize::try_from(error.offset()).unwrap_or(source.len()),
        )
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

    /// Compatibility accessor. Since 0.4 errors retain no source extract.
    #[must_use]
    pub const fn extract(&self) -> &Option<String> {
        &None
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

impl core::fmt::Debug for JsonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("JsonError")
            .field("kind", &self.kind)
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}

impl crate::FormatError for JsonError {
    fn category(&self) -> crate::Category {
        self.category
    }

    fn position(&self) -> Option<crate::Position> {
        Some(self.pos.into())
    }
}
