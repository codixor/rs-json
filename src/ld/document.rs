use crate::{JsonLdArray, JsonLdObject, JsonSerdeError, JsonSerdeResult, JsonValue};

/// Top-level JSON-LD document builder.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct JsonLdDocument {
    root: JsonLdObject,
}

impl JsonLdDocument {
    /// Creates an empty top-level JSON-LD document.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a document from a prebuilt JSON-LD object.
    #[must_use]
    pub fn from_object(root: JsonLdObject) -> Self {
        Self { root }
    }

    /// Adds or replaces a top-level field.
    #[must_use]
    pub fn field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.root.insert(key, value);
        self
    }

    /// Sets `@context` on the top-level document.
    #[must_use]
    pub fn context<V>(mut self, value: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.root.insert("@context", value);
        self
    }

    /// Sets `@type` on the top-level document.
    #[must_use]
    pub fn ty<V>(mut self, value: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.root.insert("@type", value);
        self
    }

    /// Sets `@id` on the top-level document.
    #[must_use]
    pub fn id<V>(mut self, value: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.root.insert("@id", value);
        self
    }

    /// Sets `@graph` on the top-level document.
    #[must_use]
    pub fn graph(mut self, graph: JsonLdArray) -> Self {
        self.root.insert("@graph", graph);
        self
    }

    /// Returns the top-level object.
    #[must_use]
    pub fn root(&self) -> &JsonLdObject {
        &self.root
    }

    /// Converts the document into a generic JSON value.
    #[must_use]
    pub fn into_value(self) -> JsonValue {
        self.root.into()
    }

    /// Clones the document as a generic JSON value.
    #[must_use]
    pub fn to_value(&self) -> JsonValue {
        self.root.to_value()
    }

    fn to_canonical_value(&self) -> JsonValue {
        canonical_json_value(&self.to_value())
    }

    /// Serializes the document to compact JSON.
    ///
    /// # Errors
    ///
    /// Returns a serialization error if the inner JSON value cannot be
    /// rendered.
    pub fn to_json_string(&self) -> JsonSerdeResult<String> {
        serde_json::to_string(&self.to_canonical_value()).map_err(|error| {
            JsonSerdeError::Serialize {
                message: error.to_string(),
            }
        })
    }

    /// Serializes the document to pretty JSON.
    ///
    /// # Errors
    ///
    /// Returns a serialization error if the inner JSON value cannot be
    /// rendered.
    pub fn to_json_string_pretty(&self) -> JsonSerdeResult<String> {
        serde_json::to_string_pretty(&self.to_canonical_value()).map_err(|error| {
            JsonSerdeError::Serialize {
                message: error.to_string(),
            }
        })
    }

    /// Wraps the compact JSON-LD payload in a `<script>` tag.
    ///
    /// # Errors
    ///
    /// Returns a serialization error if the inner JSON value cannot be
    /// rendered.
    pub fn to_script_tag(&self) -> JsonSerdeResult<String> {
        const PREFIX: &str = r#"<script type="application/ld+json">"#;
        const SUFFIX: &str = "</script>";

        let json = self.to_json_string()?;
        let mut out = String::with_capacity(PREFIX.len() + json.len() + SUFFIX.len());
        out.push_str(PREFIX);
        out.push_str(&json);
        out.push_str(SUFFIX);
        Ok(out)
    }

    /// Wraps the pretty JSON-LD payload in a `<script>` tag.
    ///
    /// # Errors
    ///
    /// Returns a serialization error if the inner JSON value cannot be
    /// rendered.
    pub fn to_script_tag_pretty(&self) -> JsonSerdeResult<String> {
        const PREFIX: &str = "<script type=\"application/ld+json\">\n";
        const SUFFIX: &str = "\n</script>";

        let json = self.to_json_string_pretty()?;
        let mut out = String::with_capacity(PREFIX.len() + json.len() + SUFFIX.len());
        out.push_str(PREFIX);
        out.push_str(&json);
        out.push_str(SUFFIX);
        Ok(out)
    }
}

fn canonical_json_value(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Array(items) => {
            JsonValue::Array(items.iter().map(canonical_json_value).collect())
        }
        JsonValue::Object(map) => {
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(key, _)| *key);

            let mut sorted = serde_json::Map::with_capacity(entries.len());
            for (key, value) in entries {
                sorted.insert(key.clone(), canonical_json_value(value));
            }

            JsonValue::Object(sorted)
        }
        _ => value.clone(),
    }
}
