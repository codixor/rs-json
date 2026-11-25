use serde_json::Map;

use crate::JsonValue;

const DEFAULT_OBJECT_CAPACITY: usize = 8;
const DEFAULT_ARRAY_CAPACITY: usize = 4;

/// Owned JSON-LD object builder backed by `serde_json::Map<String, Value>`.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonLdObject {
    inner: Map<String, JsonValue>,
}

/// Owned JSON-LD array builder backed by `Vec<Value>`.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonLdArray {
    inner: Vec<JsonValue>,
}

impl JsonLdObject {
    /// Creates an empty JSON-LD object.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_OBJECT_CAPACITY)
    }

    /// Creates an empty JSON-LD object with reserved field capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Map::with_capacity(capacity),
        }
    }

    /// Inserts a field by mutating the object in place.
    pub fn insert<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.inner.insert(key.into(), value.into());
        self
    }

    /// Returns a builder-updated object with a new field.
    #[must_use]
    pub fn field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.insert(key, value);
        self
    }

    /// Sets `@context`.
    #[must_use]
    pub fn context<V>(self, value: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.field("@context", value)
    }

    /// Sets `@type`.
    #[must_use]
    pub fn ty<V>(self, value: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.field("@type", value)
    }

    /// Sets `@id`.
    #[must_use]
    pub fn id<V>(self, value: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.field("@id", value)
    }

    /// Sets `@graph`.
    #[must_use]
    pub fn graph(self, graph: JsonLdArray) -> Self {
        self.field("@graph", graph)
    }

    /// Returns the underlying object map.
    #[must_use]
    pub fn as_map(&self) -> &Map<String, JsonValue> {
        &self.inner
    }

    /// Converts the builder into the underlying object map.
    #[must_use]
    pub fn into_map(self) -> Map<String, JsonValue> {
        self.inner
    }

    /// Clones the object as a generic JSON value.
    #[must_use]
    pub fn to_value(&self) -> JsonValue {
        JsonValue::Object(self.inner.clone())
    }
}

impl JsonLdArray {
    /// Creates an empty JSON-LD array.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_ARRAY_CAPACITY)
    }

    /// Creates an empty JSON-LD array with reserved item capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Pushes a value by mutating the array in place.
    pub fn push<V>(&mut self, value: V) -> &mut Self
    where
        V: Into<JsonValue>,
    {
        self.inner.push(value.into());
        self
    }

    /// Returns a builder-updated array with a new item.
    #[must_use]
    pub fn item<V>(mut self, value: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.push(value);
        self
    }

    /// Returns the underlying array items.
    #[must_use]
    pub fn as_items(&self) -> &[JsonValue] {
        &self.inner
    }

    /// Converts the builder into the underlying array items.
    #[must_use]
    pub fn into_items(self) -> Vec<JsonValue> {
        self.inner
    }

    /// Clones the array as a generic JSON value.
    #[must_use]
    pub fn to_value(&self) -> JsonValue {
        JsonValue::Array(self.inner.clone())
    }
}

impl From<JsonLdObject> for JsonValue {
    fn from(value: JsonLdObject) -> Self {
        JsonValue::Object(value.inner)
    }
}

impl From<JsonLdArray> for JsonValue {
    fn from(value: JsonLdArray) -> Self {
        JsonValue::Array(value.inner)
    }
}

impl Default for JsonLdObject {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for JsonLdArray {
    fn default() -> Self {
        Self::new()
    }
}
