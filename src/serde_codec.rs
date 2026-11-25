use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;
use std::io::{Read, Write};

/// The underlying JSON codec's I/O, syntax, data and end-of-input categories.
pub use serde_json::error::Category as JsonSerdeCategory;

/// Result type for typed JSON encoding and decoding.
pub type JsonSerdeResult<T> = Result<T, JsonSerdeError>;

/// Workspace-owned JSON value type.
pub type JsonValue = serde_json::Value;

/// A Serde codec failure. Errors returned by this crate retain only a fixed
/// encoding diagnostic or the decode category and source location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonSerdeError {
    /// Encoding failed.
    Serialize {
        /// The crate supplies a fixed diagnostic. Manually constructed errors
        /// display the message supplied by their caller.
        message: String,
    },
    /// Decoding failed.
    Deserialize {
        /// How the underlying codec classified the failure.
        category: JsonSerdeCategory,
        /// Source line, starting at one. Zero indicates an unknown location.
        line: usize,
        /// Source column as reported by the codec. It can be zero at a line boundary.
        column: usize,
    },
}

impl JsonSerdeError {
    fn encode(error: &serde_json::Error) -> Self {
        let message = if error.is_io() {
            "I/O error while writing JSON"
        } else {
            "value could not be encoded"
        };
        Self::Serialize {
            message: message.to_owned(),
        }
    }

    fn decode(error: &serde_json::Error) -> Self {
        Self::Deserialize {
            category: error.classify(),
            line: error.line(),
            column: error.column(),
        }
    }

    /// The category of a decode failure. Encoding errors return `None`.
    pub fn category(&self) -> Option<JsonSerdeCategory> {
        match self {
            Self::Deserialize { category, .. } => Some(*category),
            Self::Serialize { .. } => None,
        }
    }

    /// The reported line and column when a decode failure has a source location.
    pub fn position(&self) -> Option<(usize, usize)> {
        match self {
            Self::Deserialize { line, column, .. } if *line != 0 => Some((*line, *column)),
            _ => None,
        }
    }
}

impl fmt::Display for JsonSerdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize { message } => write!(f, "failed to serialize JSON: {message}"),
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

/// Serialize a value to UTF-8 JSON bytes.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_vec<T: Serialize>(value: &T) -> JsonSerdeResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Serialize a value to pretty UTF-8 JSON bytes.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_vec_pretty<T: Serialize>(value: &T) -> JsonSerdeResult<Vec<u8>> {
    serde_json::to_vec_pretty(value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Serialize a value to a JSON string.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_string<T: Serialize>(value: &T) -> JsonSerdeResult<String> {
    serde_json::to_string(value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Serialize a value to a pretty JSON string.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_string_pretty<T: Serialize>(value: &T) -> JsonSerdeResult<String> {
    serde_json::to_string_pretty(value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Serialize a value to a writer.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_writer<T: Serialize, W: Write>(writer: W, value: &T) -> JsonSerdeResult<()> {
    serde_json::to_writer(writer, value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Serialize a value to a pretty JSON writer.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_writer_pretty<T: Serialize, W: Write>(writer: W, value: &T) -> JsonSerdeResult<()> {
    serde_json::to_writer_pretty(writer, value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Convert a typed value into a workspace-owned [`JsonValue`].
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_value<T: Serialize>(value: &T) -> JsonSerdeResult<JsonValue> {
    serde_json::to_value(value).map_err(|error| JsonSerdeError::encode(&error))
}

/// Deserialize a value from UTF-8 JSON bytes.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when decoding fails.
pub fn from_slice<T: DeserializeOwned>(slice: &[u8]) -> JsonSerdeResult<T> {
    serde_json::from_slice(slice).map_err(|error| JsonSerdeError::decode(&error))
}

/// Deserialize a value from a JSON string slice.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when decoding fails.
pub fn from_str<T: DeserializeOwned>(source: &str) -> JsonSerdeResult<T> {
    serde_json::from_str(source).map_err(|error| JsonSerdeError::decode(&error))
}

/// Deserialize a value from a reader.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when decoding fails.
pub fn from_reader<T: DeserializeOwned, R: Read>(reader: R) -> JsonSerdeResult<T> {
    serde_json::from_reader(reader).map_err(|error| JsonSerdeError::decode(&error))
}

/// Convert a workspace-owned [`JsonValue`] into a typed value.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when decoding fails.
pub fn from_value<T: DeserializeOwned>(value: JsonValue) -> JsonSerdeResult<T> {
    serde_json::from_value(value).map_err(|error| JsonSerdeError::decode(&error))
}
