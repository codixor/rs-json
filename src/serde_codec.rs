mod bounds;
mod detail;

pub use detail::{Detail, MAX_DETAIL_CHARS};

use crate::{Category, FormatError, Limits, Pos, Position};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::{Read, Write};

/// The underlying JSON codec's I/O, syntax, data and end-of-input categories.
pub use serde_json::error::Category as JsonSerdeCategory;

/// Result type for typed JSON encoding and decoding.
pub type JsonSerdeResult<T> = Result<T, JsonSerdeError>;

/// Workspace-owned JSON value type.
pub type JsonValue = serde_json::Value;

/// A Serde codec failure. `Display` and `Debug` show only the category and
/// source location. [`JsonSerdeError::detail`] adds sanitized text that never
/// quotes an input value: field names, the expected type, and messages raised
/// by the target type's own impls (see [`Detail`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonSerdeError {
    /// Encoding failed.
    Serialize {
        /// I/O for a failed writer, or Data for a value that cannot be encoded.
        category: Category,
        /// Sanitized message from the value's `Serialize` impl or the codec.
        detail: Detail,
    },
    /// Decoding failed.
    Deserialize {
        /// Decode category, including Data for a number outside the requested representation.
        category: JsonSerdeCategory,
        /// Source line, starting at one. Zero indicates an unknown location.
        line: usize,
        /// Source column as reported by the codec. It can be zero at a line boundary.
        column: usize,
        /// Sanitized message: never an input value (see [`Detail`]).
        detail: Detail,
    },
    /// Encoded input exceeded a configured parser bound.
    Limit {
        /// Location of the first excess byte or container.
        pos: Pos,
    },
}

impl JsonSerdeError {
    fn encode(error: &serde_json::Error) -> Self {
        Self::Serialize {
            category: if error.is_io() {
                Category::Io
            } else {
                Category::Data
            },
            detail: Detail::encode(error.is_io(), &error.to_string()),
        }
    }

    fn decode(error: &serde_json::Error) -> Self {
        // serde_json groups NumberOutOfRange with grammar errors and exposes
        // only its formatted diagnostic. Custom Serde messages are already Data.
        let category = if error.is_syntax()
            && error.to_string().split(" at line ").next() == Some("number out of range")
        {
            JsonSerdeCategory::Data
        } else {
            error.classify()
        };
        Self::Deserialize {
            category,
            line: error.line(),
            column: error.column(),
            detail: Detail::decode(category, &error.to_string()),
        }
    }

    /// Sanitized diagnostic text, for messages a person acts on: the field at
    /// fault (``unknown field `tokn` ``), the expected type, or a message from
    /// the target type's own `Deserialize`/`Serialize` impl. Values taken from
    /// the input are reduced to their kind; see [`Detail`] for the rules.
    /// Limit and I/O errors have none.
    pub fn detail(&self) -> Option<&str> {
        match self {
            Self::Serialize { detail, .. } | Self::Deserialize { detail, .. } => detail.as_str(),
            Self::Limit { .. } => None,
        }
    }

    /// The category of a decode failure. Encoding and limit errors return `None`.
    pub fn category(&self) -> Option<JsonSerdeCategory> {
        match self {
            Self::Deserialize { category, .. } => Some(*category),
            Self::Serialize { .. } | Self::Limit { .. } => None,
        }
    }

    /// The reported line and column when a decode failure has a source location.
    pub fn position(&self) -> Option<(usize, usize)> {
        match self {
            Self::Deserialize { line, column, .. } if *line != 0 => Some((*line, *column)),
            Self::Limit { pos } => Some((pos.line as usize, pos.col as usize)),
            _ => None,
        }
    }
}

impl fmt::Display for JsonSerdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize { category, .. } => {
                write!(f, "failed to serialize JSON ({category:?})")
            }
            Self::Limit { pos } => write!(
                f,
                "JSON limit exceeded at line {} column {} (byte {})",
                pos.line, pos.col, pos.offset
            ),
            Self::Deserialize { category, .. } => {
                write!(f, "failed to deserialize JSON ({category:?})")?;
                if let Some((line, column)) = self.position() {
                    write!(f, " at line {line} column {column}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for JsonSerdeError {}

impl FormatError for JsonSerdeError {
    fn category(&self) -> Category {
        match self {
            Self::Serialize { category, .. } => *category,
            Self::Deserialize { category, .. } => match category {
                JsonSerdeCategory::Io => Category::Io,
                JsonSerdeCategory::Syntax => Category::Syntax,
                JsonSerdeCategory::Data => Category::Data,
                JsonSerdeCategory::Eof => Category::Eof,
            },
            Self::Limit { .. } => Category::Limit,
        }
    }

    fn position(&self) -> Option<Position> {
        if let Self::Limit { pos } = self {
            return Some((*pos).into());
        }
        let (line, column) = self.position()?;
        let position = Position {
            offset: None,
            line: u32::try_from(line)
                .ok()
                .and_then(core::num::NonZeroU32::new),
            column: u32::try_from(column)
                .ok()
                .and_then(core::num::NonZeroU32::new),
        };
        (position.line.is_some() || position.column.is_some()).then_some(position)
    }
}

// Every caller prechecks the source or wraps the reader with the same inclusive
// depth ceiling. The backend's internal counter would reject the 128th opener.
fn decode_bounded<'de, T: Deserialize<'de>, R: serde_json::de::Read<'de>>(
    mut decoder: serde_json::Deserializer<R>,
) -> Result<T, serde_json::Error> {
    decoder.disable_recursion_limit();
    let value = T::deserialize(&mut decoder)?;
    decoder.end()?;
    Ok(value)
}

/// Serialize a value to UTF-8 JSON bytes.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_vec<T: ?Sized + Serialize>(value: &T) -> JsonSerdeResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Serialize a value to pretty UTF-8 JSON bytes.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_vec_pretty<T: ?Sized + Serialize>(value: &T) -> JsonSerdeResult<Vec<u8>> {
    serde_json::to_vec_pretty(value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Serialize a value to a JSON string.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_string<T: ?Sized + Serialize>(value: &T) -> JsonSerdeResult<String> {
    serde_json::to_string(value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Serialize a value to a pretty JSON string.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_string_pretty<T: ?Sized + Serialize>(value: &T) -> JsonSerdeResult<String> {
    serde_json::to_string_pretty(value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Serialize a value to a writer.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_writer<T: ?Sized + Serialize, W: Write>(writer: W, value: &T) -> JsonSerdeResult<()> {
    serde_json::to_writer(writer, value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Serialize a value to a pretty JSON writer.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_writer_pretty<T: ?Sized + Serialize, W: Write>(
    writer: W,
    value: &T,
) -> JsonSerdeResult<()> {
    serde_json::to_writer_pretty(writer, value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Convert a typed value into a workspace-owned [`JsonValue`].
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_value<T: ?Sized + Serialize>(value: &T) -> JsonSerdeResult<JsonValue> {
    serde_json::to_value(value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Deserialize a value from UTF-8 JSON bytes.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when decoding fails.
pub fn from_slice<'de, T: Deserialize<'de>>(slice: &'de [u8]) -> JsonSerdeResult<T> {
    from_slice_with_limits(slice, Limits::default())
}

/// Decode a UTF-8 byte slice using inclusive shared bounds. Values can borrow from the slice.
///
/// # Errors
/// Returns a limit or codec diagnostic.
pub fn from_slice_with_limits<'de, T: Deserialize<'de>>(
    slice: &'de [u8],
    limits: Limits,
) -> JsonSerdeResult<T> {
    bounds::check_slice(slice, limits)?;
    decode_bounded(serde_json::Deserializer::from_slice(slice))
        .map_err(|error| JsonSerdeError::decode(&error))
}

/// Deserialize a value from a JSON string slice.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when decoding fails.
pub fn from_str<'de, T: Deserialize<'de>>(source: &'de str) -> JsonSerdeResult<T> {
    from_str_with_limits(source, Limits::default())
}

/// Decode a string with inclusive shared bounds.
///
/// # Errors
/// Returns a limit or codec diagnostic.
pub fn from_str_with_limits<'de, T: Deserialize<'de>>(
    source: &'de str,
    limits: Limits,
) -> JsonSerdeResult<T> {
    bounds::check_slice(source.as_bytes(), limits)?;
    decode_bounded(serde_json::Deserializer::from_str(source))
        .map_err(|error| JsonSerdeError::decode(&error))
}

/// Deserialize a value from a reader.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when decoding fails.
pub fn from_reader<T: DeserializeOwned, R: Read>(reader: R) -> JsonSerdeResult<T> {
    from_reader_with_limits(reader, Limits::default())
}

/// Decode a reader incrementally with inclusive shared bounds.
///
/// # Errors
/// Returns a limit, I/O or codec diagnostic.
pub fn from_reader_with_limits<T: DeserializeOwned, R: Read>(
    reader: R,
    limits: Limits,
) -> JsonSerdeResult<T> {
    let mut input = bounds::BoundedRead::new(reader, limits);
    let decoded = decode_bounded(serde_json::Deserializer::from_reader(&mut input));
    if let Some(pos) = input.limit_position() {
        return Err(JsonSerdeError::Limit { pos });
    }
    decoded.map_err(|error| JsonSerdeError::decode(&error))
}

/// Convert a workspace-owned [`JsonValue`] into a typed value.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when decoding fails.
pub fn from_value<T: DeserializeOwned>(value: JsonValue) -> JsonSerdeResult<T> {
    serde_json::from_value(value).map_err(|error| JsonSerdeError::decode(&error))
}
