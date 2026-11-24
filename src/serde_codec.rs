use serde::Serialize;
use serde::de::DeserializeOwned;
use std::io::{Read, Write};
use thiserror::Error;

/// Result type for typed JSON encoding and decoding.
pub type JsonSerdeResult<T> = Result<T, JsonSerdeError>;

/// Workspace-owned JSON value type.
pub type JsonValue = serde_json::Value;

/// Strongly typed error for serde-backed JSON encoding and decoding.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum JsonSerdeError {
    #[error("failed to serialize JSON: {message}")]
    Serialize { message: String },
    #[error("failed to deserialize JSON: {message}")]
    Deserialize { message: String },
}

/// Serialize a value to UTF-8 JSON bytes.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_vec<T: Serialize>(value: &T) -> JsonSerdeResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| JsonSerdeError::Serialize {
        message: error.to_string(),
    })
}

/// Serialize a value to pretty UTF-8 JSON bytes.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_vec_pretty<T: Serialize>(value: &T) -> JsonSerdeResult<Vec<u8>> {
    serde_json::to_vec_pretty(value).map_err(|error| JsonSerdeError::Serialize {
        message: error.to_string(),
    })
}

/// Serialize a value to a JSON string.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_string<T: Serialize>(value: &T) -> JsonSerdeResult<String> {
    serde_json::to_string(value).map_err(|error| JsonSerdeError::Serialize {
        message: error.to_string(),
    })
}

/// Serialize a value to a pretty JSON string.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_string_pretty<T: Serialize>(value: &T) -> JsonSerdeResult<String> {
    serde_json::to_string_pretty(value).map_err(|error| JsonSerdeError::Serialize {
        message: error.to_string(),
    })
}

/// Serialize a value to a writer.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_writer<T: Serialize, W: Write>(writer: W, value: &T) -> JsonSerdeResult<()> {
    serde_json::to_writer(writer, value).map_err(|error| JsonSerdeError::Serialize {
        message: error.to_string(),
    })
}

/// Serialize a value to a pretty JSON writer.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_writer_pretty<T: Serialize, W: Write>(writer: W, value: &T) -> JsonSerdeResult<()> {
    serde_json::to_writer_pretty(writer, value).map_err(|error| JsonSerdeError::Serialize {
        message: error.to_string(),
    })
}

/// Convert a typed value into a workspace-owned [`JsonValue`].
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when serialization fails.
pub fn to_value<T: Serialize>(value: &T) -> JsonSerdeResult<JsonValue> {
    serde_json::to_value(value).map_err(|error| JsonSerdeError::Serialize {
        message: error.to_string(),
    })
}

/// Deserialize a value from UTF-8 JSON bytes.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when decoding fails.
pub fn from_slice<T: DeserializeOwned>(slice: &[u8]) -> JsonSerdeResult<T> {
    serde_json::from_slice(slice).map_err(|error| JsonSerdeError::Deserialize {
        message: error.to_string(),
    })
}

/// Deserialize a value from a JSON string slice.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when decoding fails.
pub fn from_str<T: DeserializeOwned>(source: &str) -> JsonSerdeResult<T> {
    serde_json::from_str(source).map_err(|error| JsonSerdeError::Deserialize {
        message: error.to_string(),
    })
}

/// Deserialize a value from a reader.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when decoding fails.
pub fn from_reader<T: DeserializeOwned, R: Read>(reader: R) -> JsonSerdeResult<T> {
    serde_json::from_reader(reader).map_err(|error| JsonSerdeError::Deserialize {
        message: error.to_string(),
    })
}

/// Convert a workspace-owned [`JsonValue`] into a typed value.
///
/// # Errors
///
/// Returns [`JsonSerdeError`] when decoding fails.
pub fn from_value<T: DeserializeOwned>(value: JsonValue) -> JsonSerdeResult<T> {
    serde_json::from_value(value).map_err(|error| JsonSerdeError::Deserialize {
        message: error.to_string(),
    })
}
