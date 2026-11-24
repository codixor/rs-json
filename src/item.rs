use std::ops::Index;

use crate::generics::Container::{Array, Map};
use crate::generics::IterArray;
use crate::generics::IterMap;
use crate::generics::{ArrayIterator, Container, MapIterator};
use crate::key::Key;
use crate::kind::JsonType;
use crate::kind::JsonType::{Empty, JsonArray, JsonFalse, JsonMap, JsonNull, JsonNumber, JsonTrue};
use crate::slice::Slice;

static EMPTY_ITEM: JsonItem<'static> = JsonItem::empty();

/// Zero-copy JSON DOM item.
#[derive(Debug, Clone)]
pub struct JsonItem<'a> {
    pub(crate) slice: Slice<'a>,
    pub(crate) json_type: JsonType,
    container: Option<Container<Key<'a>, JsonItem<'a>>>,
}

impl<'a> JsonItem<'a> {
    pub(crate) fn new(slice: Slice<'a>, json_type: JsonType) -> Self {
        Self {
            slice,
            json_type,
            container: None,
        }
    }

    pub(crate) fn new_array(slice: Slice<'a>, items: Vec<JsonItem<'a>>) -> Self {
        if items.is_empty() {
            Self::new(slice, JsonArray)
        } else {
            Self {
                slice,
                json_type: JsonArray,
                container: Some(Array(items)),
            }
        }
    }

    pub(crate) fn new_map(slice: Slice<'a>, items: Vec<(Key<'a>, JsonItem<'a>)>) -> Self {
        if items.is_empty() {
            Self::new(slice, JsonMap)
        } else {
            Self {
                slice,
                json_type: JsonMap,
                container: Some(Map(items)),
            }
        }
    }

    pub(crate) const fn empty() -> Self {
        Self {
            slice: Slice::empty(),
            json_type: Empty,
            container: None,
        }
    }

    /// Returns the raw source view for this item.
    ///
    /// For strings, the returned slice excludes the surrounding quotes and
    /// keeps escapes exactly as they appeared in the input.
    #[must_use]
    pub fn as_str(&self) -> Option<&'a str> {
        if self.json_type == Empty {
            None
        } else {
            Some(self.slice.as_str())
        }
    }

    /// Tries to convert the item to `f64`.
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        (self.json_type == JsonNumber)
            .then(|| self.slice.as_str().parse::<f64>().ok())
            .flatten()
    }

    /// Tries to convert the item to `i128`.
    #[must_use]
    pub fn as_i128(&self) -> Option<i128> {
        (self.json_type == JsonNumber)
            .then(|| self.slice.as_str().parse::<i128>().ok())
            .flatten()
    }

    /// Tries to convert the item to `bool`.
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self.json_type {
            JsonTrue => Some(true),
            JsonFalse => Some(false),
            _ => None,
        }
    }

    /// Checks whether the item is JSON `null`.
    #[must_use]
    pub fn is_null(&self) -> bool {
        self.json_type == JsonNull
    }

    /// Checks whether the item exists.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.json_type != Empty
    }

    /// Returns the item kind.
    #[must_use]
    pub const fn kind(&self) -> JsonType {
        self.json_type
    }

    /// Legacy compatibility accessor for the item kind.
    #[must_use]
    pub const fn get_type(&self) -> JsonType {
        self.kind()
    }

    /// Iterates over array elements.
    #[must_use]
    pub fn elements(&self) -> Option<ArrayIterator<'_, JsonItem<'a>>> {
        match &self.container {
            Some(Array(items)) => Some(ArrayIterator {
                iter: IterArray::Vec(items.iter()),
            }),
            None if self.json_type == JsonArray => Some(ArrayIterator {
                iter: IterArray::Empty,
            }),
            Some(Map(_)) | None => None,
        }
    }

    /// Iterates over object entries.
    #[must_use]
    pub fn entries(&self) -> Option<MapIterator<'_, Key<'a>, JsonItem<'a>>> {
        match &self.container {
            Some(Map(items)) => Some(MapIterator {
                iter: IterMap::Vec(items.iter()),
            }),
            None if self.json_type == JsonMap => Some(MapIterator {
                iter: IterMap::Empty,
            }),
            Some(Array(_)) | None => None,
        }
    }
}

impl<'a> Index<usize> for JsonItem<'a> {
    type Output = JsonItem<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        match &self.container {
            Some(Array(items)) => items.get(index).unwrap_or(&EMPTY_ITEM),
            _ => &EMPTY_ITEM,
        }
    }
}

impl<'a> Index<&str> for JsonItem<'a> {
    type Output = JsonItem<'a>;

    fn index(&self, key: &str) -> &Self::Output {
        match &self.container {
            Some(Map(items)) => items
                .iter()
                .find_map(|(candidate, value)| (candidate.as_str() == key).then_some(value))
                .unwrap_or(&EMPTY_ITEM),
            _ => &EMPTY_ITEM,
        }
    }
}
