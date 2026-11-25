//! JSON-LD builders for owned schema documents.
//!
//! This module is additive to the parser-first `rs-json` surface. It is meant
//! for building JSON-LD documents explicitly, without requiring caller-side
//! derive models.

mod builder;
mod document;
pub mod google;

pub use self::builder::{JsonLdArray, JsonLdObject};
pub use self::document::JsonLdDocument;
