use crate::{JsonLdDocument, JsonLdObject, JsonValue};

use super::{SCHEMA_ORG_CONTEXT, named_thing};

/// A Google-friendly `AggregateRating` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct AggregateRating {
    rating_value: JsonValue,
    rating_count: JsonValue,
    review_count: Option<JsonValue>,
    best_rating: Option<JsonValue>,
    worst_rating: Option<JsonValue>,
    extra: JsonLdObject,
}

/// A Google-friendly `Review` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Review {
    author: JsonValue,
    body: Option<String>,
    date_published: Option<String>,
    rating: Option<JsonValue>,
    item_reviewed: Option<JsonValue>,
    extra: JsonLdObject,
}

impl AggregateRating {
    /// Creates an `AggregateRating` builder with the core numeric fields.
    #[must_use]
    pub fn new<V, C>(rating_value: V, rating_count: C) -> Self
    where
        V: Into<JsonValue>,
        C: Into<JsonValue>,
    {
        Self {
            rating_value: rating_value.into(),
            rating_count: rating_count.into(),
            review_count: None,
            best_rating: None,
            worst_rating: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Sets `reviewCount`.
    #[must_use]
    pub fn review_count<V>(mut self, review_count: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.review_count = Some(review_count.into());
        self
    }

    /// Sets `bestRating`.
    #[must_use]
    pub fn best_rating<V>(mut self, best_rating: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.best_rating = Some(best_rating.into());
        self
    }

    /// Sets `worstRating`.
    #[must_use]
    pub fn worst_rating<V>(mut self, worst_rating: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.worst_rating = Some(worst_rating.into());
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

    /// Returns the aggregate rating as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("AggregateRating")
            .field("ratingValue", self.rating_value)
            .field("ratingCount", self.rating_count);

        if let Some(review_count) = self.review_count {
            object = object.field("reviewCount", review_count);
        }

        if let Some(best_rating) = self.best_rating {
            object = object.field("bestRating", best_rating);
        }

        if let Some(worst_rating) = self.worst_rating {
            object = object.field("worstRating", worst_rating);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl Review {
    /// Creates a `Review` builder with a simple person author.
    #[must_use]
    pub fn new<A>(author_name: A) -> Self
    where
        A: Into<String>,
    {
        Self {
            author: JsonValue::from(named_thing("Person", author_name.into())),
            body: None,
            date_published: None,
            rating: None,
            item_reviewed: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Overrides the review `author` with a raw JSON-LD value.
    #[must_use]
    pub fn author<V>(mut self, author: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.author = author.into();
        self
    }

    /// Sets `reviewBody`.
    #[must_use]
    pub fn review_body<B>(mut self, review_body: B) -> Self
    where
        B: Into<String>,
    {
        self.body = Some(review_body.into());
        self
    }

    /// Sets `datePublished`.
    #[must_use]
    pub fn date_published<D>(mut self, date_published: D) -> Self
    where
        D: Into<String>,
    {
        self.date_published = Some(date_published.into());
        self
    }

    /// Sets `reviewRating`.
    #[must_use]
    pub fn review_rating<V>(mut self, review_rating: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.rating = Some(review_rating.into());
        self
    }

    /// Sets `itemReviewed`.
    #[must_use]
    pub fn item_reviewed<V>(mut self, item_reviewed: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.item_reviewed = Some(item_reviewed.into());
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

    /// Returns the review as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("Review")
            .field("author", self.author);

        if let Some(review_body) = self.body {
            object = object.field("reviewBody", review_body);
        }

        if let Some(date_published) = self.date_published {
            object = object.field("datePublished", date_published);
        }

        if let Some(review_rating) = self.rating {
            object = object.field("reviewRating", review_rating);
        }

        if let Some(item_reviewed) = self.item_reviewed {
            object = object.field("itemReviewed", item_reviewed);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the review as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl From<AggregateRating> for JsonLdObject {
    fn from(value: AggregateRating) -> Self {
        value.into_object()
    }
}

impl From<AggregateRating> for JsonValue {
    fn from(value: AggregateRating) -> Self {
        value.into_object().into()
    }
}

impl From<Review> for JsonLdObject {
    fn from(value: Review) -> Self {
        value.into_object()
    }
}

impl From<Review> for JsonLdDocument {
    fn from(value: Review) -> Self {
        value.into_document()
    }
}

impl From<Review> for JsonValue {
    fn from(value: Review) -> Self {
        value.into_object().into()
    }
}
