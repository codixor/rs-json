//! Lightweight JSON parsing for the workspace.
//!
//! The crate stays intentionally small:
//! - zero-copy views into the source input
//! - simple DOM traversal helpers
//! - parser-first API with typed errors
//! - serde-backed encode/decode helpers behind the same workspace facade

mod error;
mod generics;
mod item;
mod key;
mod kind;
pub mod ld;
mod parser;
mod serde_codec;
mod slice;

#[doc(inline)]
pub use crate::error::{JsonError, JsonError as Error, JsonErrorKind};
#[doc(inline)]
pub use crate::generics::{ArrayIterator, MapIterator};
#[doc(inline)]
pub use crate::item::JsonItem;
#[doc(inline)]
pub use crate::key::Key;
#[doc(inline)]
pub use crate::kind::JsonType;
#[doc(inline)]
pub use crate::ld::{JsonLdArray, JsonLdDocument, JsonLdObject};
#[doc(inline)]
pub use crate::serde_codec::{
    JsonSerdeError, JsonSerdeResult, JsonValue, from_reader, from_slice, from_str, from_value,
    to_string, to_string_pretty, to_value, to_vec, to_vec_pretty, to_writer, to_writer_pretty,
};

/// Parse JSON text into a zero-copy DOM.
///
/// The returned [`JsonItem`] borrows from `source`, so the source text must
/// outlive the parsed tree.
///
/// # Errors
///
/// Returns [`JsonError`] when the input is not valid JSON.
pub fn parse(source: &str) -> Result<JsonItem<'_>, JsonError> {
    parser::parse(source)
}
