//! Migration API for callers that use 0.1 indexing and raw string bodies.
//! The document lives in a shared tape. Indexing materializes handles for the
//! immediate children of each visited container and caches them for later lookups.

use crate::{Document, Value, ValueKind};
use std::{
    ops::Index,
    sync::{Arc, OnceLock},
};

/// Error categories shared with the tape parser.
pub use crate::ErrorKind as JsonErrorKind;
mod error;
pub use error::JsonError;
/// Original short name for a compatibility parser error.
pub type Error = JsonError;
/// Borrowed array elements in source order.
pub type ArrayIterator<'a, V> = core::slice::Iter<'a, V>;
/// Borrowed object entries in source order.
pub struct MapIterator<'a, K, V>(core::slice::Iter<'a, (K, V)>);
impl<'a, K, V> Iterator for MapIterator<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(key, value)| (key, value))
    }
}

/// JSON value kind returned by the compatibility API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonType {
    /// Null.
    JsonNull,
    /// True.
    JsonTrue,
    /// False.
    JsonFalse,
    /// String.
    JsonString,
    /// Number.
    JsonNumber,
    /// Object.
    JsonMap,
    /// Array.
    JsonArray,
    /// Missing key or index.
    Empty,
}

/// Raw object key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key<'a>(&'a str);
impl<'a> Key<'a> {
    /// Construct a key from a raw string body.
    pub const fn new(source: &'a str) -> Self {
        Self(source)
    }
    /// View the raw string body.
    pub const fn as_str(&self) -> &'a str {
        self.0
    }
}

enum Children<'a> {
    Array(Vec<JsonItem<'a>>),
    Object(Vec<(Key<'a>, JsonItem<'a>)>),
    Scalar,
}

/// Handle supporting 0.1 indexing, iteration and scalar access.
pub struct JsonItem<'a> {
    doc: Option<Arc<Document<'a>>>,
    idx: u32,
    children: OnceLock<Children<'a>>,
    missing: OnceLock<Box<Self>>,
}

impl core::fmt::Debug for JsonItem<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("JsonItem")
            .field("kind", &self.kind())
            .finish_non_exhaustive()
    }
}

impl Clone for JsonItem<'_> {
    fn clone(&self) -> Self {
        Self::new(self.doc.clone(), self.idx)
    }
}

impl<'a> JsonItem<'a> {
    fn new(doc: Option<Arc<Document<'a>>>, idx: u32) -> Self {
        Self {
            doc,
            idx,
            children: OnceLock::new(),
            missing: OnceLock::new(),
        }
    }
    fn view(&self) -> Option<Value<'_, 'a>> {
        self.doc.as_deref().map(|d| Value::new(d, self.idx))
    }
    fn missing(&self) -> &Self {
        if self.doc.is_none() {
            self
        } else {
            self.missing.get_or_init(|| Box::new(Self::new(None, 0)))
        }
    }
    fn children(&self) -> &Children<'a> {
        self.children.get_or_init(|| {
            let Some(v) = self.view() else {
                return Children::Scalar;
            };
            let child = |v: Value<'_, 'a>| Self::new(self.doc.clone(), v.idx);
            if let Some(values) = v.elements() {
                Children::Array(values.map(child).collect())
            } else if let Some(entries) = v.entries() {
                Children::Object(entries.map(|(k, v)| (Key(k.raw()), child(v))).collect())
            } else {
                Children::Scalar
            }
        })
    }
    /// Original source text, excluding quotes for strings.
    pub fn as_str(&self) -> Option<&'a str> {
        Some(self.view()?.raw())
    }
    /// Parse a numeric item as a float.
    pub fn as_f64(&self) -> Option<f64> {
        self.view()?.as_f64()
    }
    /// Parse an integer item as a signed 128-bit value.
    pub fn as_i128(&self) -> Option<i128> {
        let v = self.view()?;
        (v.kind() == ValueKind::Number)
            .then(|| v.raw().parse().ok())
            .flatten()
    }
    /// Boolean value.
    pub fn as_bool(&self) -> Option<bool> {
        self.view()?.as_bool()
    }
    /// Whether this item is JSON null.
    pub fn is_null(&self) -> bool {
        self.view().is_some_and(|v| v.is_null())
    }
    /// Whether this item was present in the source.
    pub fn exists(&self) -> bool {
        self.doc.is_some()
    }
    /// Original 0.1 value kind.
    pub fn kind(&self) -> JsonType {
        let Some(v) = self.view() else {
            return JsonType::Empty;
        };
        match v.kind() {
            ValueKind::Null => JsonType::JsonNull,
            ValueKind::Bool if v.as_bool() == Some(true) => JsonType::JsonTrue,
            ValueKind::Bool => JsonType::JsonFalse,
            ValueKind::String => JsonType::JsonString,
            ValueKind::Number => JsonType::JsonNumber,
            ValueKind::Object => JsonType::JsonMap,
            ValueKind::Array => JsonType::JsonArray,
        }
    }
    /// Alias for [`Self::kind`].
    pub fn get_type(&self) -> JsonType {
        self.kind()
    }
    /// Iterate the immediate elements in source order.
    pub fn elements(&self) -> Option<ArrayIterator<'_, Self>> {
        if let Children::Array(a) = self.children() {
            Some(a.iter())
        } else {
            None
        }
    }
    /// Iterate the immediate object entries in source order.
    pub fn entries(&self) -> Option<MapIterator<'_, Key<'a>, Self>> {
        if let Children::Object(a) = self.children() {
            Some(MapIterator(a.iter()))
        } else {
            None
        }
    }
}

// Drain materialized descendants iteratively even on threads with small stacks.
impl Drop for JsonItem<'_> {
    fn drop(&mut self) {
        let Some(children) = self.children.take() else {
            return;
        };
        let mut work = vec![children];
        while let Some(children) = work.pop() {
            match children {
                Children::Array(items) => {
                    work.extend(items.into_iter().filter_map(|mut v| v.children.take()));
                }
                Children::Object(items) => {
                    work.extend(items.into_iter().filter_map(|(_, mut v)| v.children.take()));
                }
                Children::Scalar => {}
            }
        }
    }
}

impl Index<usize> for JsonItem<'_> {
    type Output = Self;
    fn index(&self, i: usize) -> &Self {
        self.elements()
            .and_then(|a| a.as_slice().get(i))
            .unwrap_or_else(|| self.missing())
    }
}
impl Index<&str> for JsonItem<'_> {
    type Output = Self;
    fn index(&self, k: &str) -> &Self {
        self.entries()
            .and_then(|mut entries| {
                entries.find_map(|(key, v)| crate::unescape::unescaped_eq(key.0, k).then_some(v))
            })
            .unwrap_or_else(|| self.missing())
    }
}

/// Parse an input using 0.2 validation and return a compatibility handle.
///
/// # Errors
/// Reports syntax and resource limits. Errors retain only the category and position.
pub fn parse(src: &str) -> Result<JsonItem<'_>, JsonError> {
    crate::parse(src)
        .map(|doc| JsonItem::new(Some(Arc::new(doc)), 0))
        .map_err(|err| JsonError::from_parse(src, err))
}
