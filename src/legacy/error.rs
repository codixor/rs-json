use super::JsonErrorKind;
use thiserror::Error;
const EXTRACT_PADDING: usize = 8;

/// Parser error with a stable kind, byte index, and nearby source extract.
#[derive(Clone, PartialEq, Eq, Error)]
#[error("{kind} at index {index} in JSON data")]
pub struct JsonError {
    kind: JsonErrorKind,
    index: usize,
    extract: Option<String>,
}

impl JsonError {
    pub(super) fn from_parse(source: &str, error: crate::Error) -> Self {
        let (kind, index) = Self::location(source, error);
        let start = index.saturating_sub(EXTRACT_PADDING);
        let end = usize::min(source.len(), index.saturating_add(EXTRACT_PADDING));
        let extract = source.get(start..end).map(ToOwned::to_owned);

        Self {
            kind,
            index,
            extract,
        }
    }

    fn location(source: &str, error: crate::Error) -> (JsonErrorKind, usize) {
        use crate::{
            lex::slice::SliceLexer,
            stream::{Event, PullParser},
        };
        // 0.1 classifies any nonspace tail after a valid root as trailing data,
        // even when the next token itself is lexically invalid.
        if error.kind == JsonErrorKind::InvalidToken {
            let mut parser =
                PullParser::new(SliceLexer::from_str(source), crate::Limits::default());
            while let Ok(event) = parser.next_event() {
                if event == Event::End {
                    let end = parser.into_inner().offset();
                    return (
                        JsonErrorKind::TrailingCharacters,
                        crate::scan::skip_ws(source.as_bytes(), end),
                    );
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

impl core::fmt::Debug for JsonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("JsonError")
            .field("kind", &self.kind)
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}
