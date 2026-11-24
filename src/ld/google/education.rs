use crate::{JsonLdDocument, JsonLdObject, JsonValue};

use super::{SCHEMA_ORG_CONTEXT, values_to_json};

/// A Google-friendly education-Q&A flashcard `Question`.
#[derive(Debug, Clone, PartialEq)]
pub struct FlashcardQuestion {
    text: String,
    accepted_answer_text: String,
    extra: JsonLdObject,
}

/// A Google-friendly `Quiz` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Quiz {
    name: Option<String>,
    about: Option<String>,
    educational_level: Option<String>,
    has_part: Vec<JsonValue>,
    extra: JsonLdObject,
}

/// A Google-friendly `Course` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Course {
    name: String,
    description: String,
    provider: Option<JsonValue>,
    url: Option<String>,
    image: Option<JsonValue>,
    extra: JsonLdObject,
}

/// A Google-friendly course-list `ItemList` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct CourseList {
    items: Vec<(String, JsonValue)>,
    extra: JsonLdObject,
}

impl FlashcardQuestion {
    /// Creates a flashcard `Question` with the Education Q&A required fields.
    #[must_use]
    pub fn new<Q, A>(text: Q, accepted_answer_text: A) -> Self
    where
        Q: Into<String>,
        A: Into<String>,
    {
        Self {
            text: text.into(),
            accepted_answer_text: accepted_answer_text.into(),
            extra: JsonLdObject::new(),
        }
    }

    /// Adds an extra field for unsupported-but-needed properties.
    #[must_use]
    pub fn field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.extra = self.extra.field(key, value);
        self
    }

    /// Returns the flashcard question as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("Question")
            .field("eduQuestionType", "Flashcard")
            .field("text", self.text)
            .field(
                "acceptedAnswer",
                JsonLdObject::new()
                    .ty("Answer")
                    .field("text", self.accepted_answer_text),
            );

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl Quiz {
    /// Creates an empty `Quiz` builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            about: None,
            educational_level: None,
            has_part: Vec::new(),
            extra: JsonLdObject::new(),
        }
    }

    /// Sets `name`.
    #[must_use]
    pub fn name<N>(mut self, name: N) -> Self
    where
        N: Into<String>,
    {
        self.name = Some(name.into());
        self
    }

    /// Sets `about`.
    #[must_use]
    pub fn about<A>(mut self, about: A) -> Self
    where
        A: Into<String>,
    {
        self.about = Some(about.into());
        self
    }

    /// Sets `educationalLevel`.
    #[must_use]
    pub fn educational_level<L>(mut self, educational_level: L) -> Self
    where
        L: Into<String>,
    {
        self.educational_level = Some(educational_level.into());
        self
    }

    /// Adds one `hasPart` flashcard question.
    #[must_use]
    pub fn question<V>(mut self, question: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.has_part.push(question.into());
        self
    }

    /// Adds one simple flashcard question.
    #[must_use]
    pub fn flashcard<Q, A>(self, text: Q, accepted_answer_text: A) -> Self
    where
        Q: Into<String>,
        A: Into<String>,
    {
        self.question(FlashcardQuestion::new(text, accepted_answer_text))
    }

    /// Adds an extra field for unsupported-but-needed properties.
    #[must_use]
    pub fn field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.extra = self.extra.field(key, value);
        self
    }

    /// Returns the quiz as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new().ty("Quiz");

        if let Some(name) = self.name {
            object = object.field("name", name);
        }

        if let Some(about) = self.about {
            object = object.field("about", about);
        }

        if let Some(educational_level) = self.educational_level {
            object = object.field("educationalLevel", educational_level);
        }

        if !self.has_part.is_empty() {
            object = object.field("hasPart", values_to_json(self.has_part));
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the quiz as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl Default for Quiz {
    fn default() -> Self {
        Self::new()
    }
}

impl Course {
    /// Creates a `Course` builder with the Google-required core fields.
    #[must_use]
    pub fn new<N, D>(name: N, description: D) -> Self
    where
        N: Into<String>,
        D: Into<String>,
    {
        Self {
            name: name.into(),
            description: description.into(),
            provider: None,
            url: None,
            image: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Sets `provider`.
    #[must_use]
    pub fn provider<V>(mut self, provider: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.provider = Some(provider.into());
        self
    }

    /// Sets a simple organization `provider`.
    #[must_use]
    pub fn provider_name<N>(mut self, provider_name: N) -> Self
    where
        N: Into<String>,
    {
        self.provider = Some(
            JsonLdObject::new()
                .ty("Organization")
                .field("name", provider_name.into())
                .into(),
        );
        self
    }

    /// Sets `url`.
    #[must_use]
    pub fn url<U>(mut self, url: U) -> Self
    where
        U: Into<String>,
    {
        self.url = Some(url.into());
        self
    }

    /// Sets `image`.
    #[must_use]
    pub fn image<V>(mut self, image: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.image = Some(image.into());
        self
    }

    /// Adds an extra field for unsupported-but-needed properties.
    #[must_use]
    pub fn field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.extra = self.extra.field(key, value);
        self
    }

    /// Returns the course as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("Course")
            .field("name", self.name)
            .field("description", self.description);

        if let Some(provider) = self.provider {
            object = object.field("provider", provider);
        }

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        if let Some(image) = self.image {
            object = object.field("image", image);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl CourseList {
    /// Creates an empty course-list `ItemList` builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            extra: JsonLdObject::new(),
        }
    }

    /// Adds one `Course` item using the next ordinal position.
    #[must_use]
    pub fn course<U>(mut self, url: U, course: Course) -> Self
    where
        U: Into<String>,
    {
        self.items.push((url.into(), course.into()));
        self
    }

    /// Adds an extra field for unsupported-but-needed properties.
    #[must_use]
    pub fn field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.extra = self.extra.field(key, value);
        self
    }

    /// Returns the course list as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let list_items = self
            .items
            .into_iter()
            .enumerate()
            .map(|(index, (url, course))| {
                JsonLdObject::new()
                    .ty("ListItem")
                    .field("position", index + 1)
                    .field("url", url)
                    .field("item", course)
            })
            .map(JsonValue::from)
            .collect();

        let mut object = JsonLdObject::new()
            .ty("ItemList")
            .field("itemListElement", values_to_json(list_items));

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the course list as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl Default for CourseList {
    fn default() -> Self {
        Self::new()
    }
}

impl From<FlashcardQuestion> for JsonLdObject {
    fn from(value: FlashcardQuestion) -> Self {
        value.into_object()
    }
}

impl From<FlashcardQuestion> for JsonValue {
    fn from(value: FlashcardQuestion) -> Self {
        value.into_object().into()
    }
}

impl From<Quiz> for JsonLdObject {
    fn from(value: Quiz) -> Self {
        value.into_object()
    }
}

impl From<Quiz> for JsonLdDocument {
    fn from(value: Quiz) -> Self {
        value.into_document()
    }
}

impl From<Quiz> for JsonValue {
    fn from(value: Quiz) -> Self {
        value.into_object().into()
    }
}

impl From<Course> for JsonLdObject {
    fn from(value: Course) -> Self {
        value.into_object()
    }
}

impl From<Course> for JsonValue {
    fn from(value: Course) -> Self {
        value.into_object().into()
    }
}

impl From<CourseList> for JsonLdObject {
    fn from(value: CourseList) -> Self {
        value.into_object()
    }
}

impl From<CourseList> for JsonLdDocument {
    fn from(value: CourseList) -> Self {
        value.into_document()
    }
}

impl From<CourseList> for JsonValue {
    fn from(value: CourseList) -> Self {
        value.into_object().into()
    }
}
