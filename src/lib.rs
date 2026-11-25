//! `rs-json` — zero-copy JSON for every input size.
//!
//! Parsing entry points:
//!
//! | Layer | Entry point | Use when |
//! |-------|-------------|----------|
//! | Tape DOM | [`parse`], [`Parser::parse`] | whole document in memory, random access |
//! | Pull stream | [`stream::PullParser`] | huge or unbounded input, bounded memory |
//! | File helper | `input::parse_file` | owns a snapshot for DOM parsing |
//!
//! Work is deferred: strings are not unescaped and numbers are not converted
//! until a caller asks, and every string token carries an `escaped` flag so
//! the common no-escape path stays a borrowed `&str`.
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate alloc;

pub mod config;
pub mod dom;
pub mod error;
pub mod input;
pub mod lex;
pub mod num;
pub mod scan;
pub mod stack;
pub mod stream;
pub mod unescape;

#[cfg(feature = "ndjson")]
pub mod ndjson;

#[cfg(feature = "serde")]
/// Typed serialization and deserialization delegated to `serde_json`.
pub mod serde_codec;

#[cfg(feature = "ld")]
pub mod ld;

#[cfg(feature = "legacy")]
#[deprecated(since = "0.2.0", note = "use Document and Value for new code")]
pub mod legacy;
#[cfg(feature = "legacy")]
#[allow(deprecated)]
pub use legacy::{ArrayIterator, JsonError, JsonErrorKind, JsonItem, JsonType, Key, MapIterator};

#[cfg(feature = "ld")]
#[doc(inline)]
pub use crate::ld::{JsonLdArray, JsonLdDocument, JsonLdObject};
#[cfg(feature = "serde")]
#[doc(inline)]
pub use crate::serde_codec::{
    JsonSerdeCategory, JsonSerdeError, JsonSerdeResult, JsonValue, from_reader, from_slice,
    from_str, from_value, to_string, to_string_pretty, to_value, to_vec, to_vec_pretty, to_writer,
    to_writer_pretty,
};

pub use crate::config::Limits;
pub use crate::dom::{Document, Parser, Value, ValueKind};
pub use crate::error::{Error, ErrorKind, Pos};

/// Parse a complete JSON text into a borrowed tape [`Document`].
///
/// The returned document owns its tape. Short tapes can fit inline. For repeated small
/// documents keep a [`Parser`] and call [`Parser::parse`] so the tape and
/// stack allocations are reused (zero allocations after warm-up).
///
/// # Errors
/// Returns [`Error`] for invalid JSON or when a [`Limits`] bound is exceeded.
pub fn parse(src: &str) -> Result<Document<'_>, Error> {
    Document::from_str(src)
}

/// Parse UTF-8 bytes. String contents are UTF-8 validated with `simdutf8`
/// (structural bytes are ASCII, so only string payloads need checking).
///
/// # Errors
/// Returns [`Error`] for invalid JSON, invalid UTF-8, or exceeded limits.
pub fn parse_bytes(src: &[u8]) -> Result<Document<'_>, Error> {
    Document::parse_with(src, Limits::default(), false)
}
