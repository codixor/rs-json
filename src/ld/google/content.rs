use crate::{JsonLdDocument, JsonLdObject, JsonValue};

use super::{SCHEMA_ORG_CONTEXT, named_thing};

/// A Google-friendly `Article` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Article {
    schema_type: String,
    headline: String,
    images: Vec<JsonValue>,
    date_published: Option<String>,
    date_modified: Option<String>,
    description: Option<String>,
    author: Option<JsonValue>,
    publisher: Option<JsonValue>,
    main_entity_of_page: Option<String>,
    extra: JsonLdObject,
}

/// A Google-friendly `Recipe` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
    name: String,
    images: Vec<JsonValue>,
    author: Option<JsonValue>,
    date_published: Option<String>,
    description: Option<String>,
    prep_time: Option<String>,
    cook_time: Option<String>,
    total_time: Option<String>,
    yield_value: Option<JsonValue>,
    ingredients: Vec<String>,
    instructions: Vec<JsonValue>,
    aggregate_rating: Option<JsonValue>,
    reviews: Vec<JsonValue>,
    extra: JsonLdObject,
}

/// A Google-friendly `NewsArticle` builder layered on top of [`Article`].
#[derive(Debug, Clone, PartialEq)]
pub struct NewsArticle {
    inner: Article,
}

/// A Google-friendly `BlogPosting` builder layered on top of [`Article`].
#[derive(Debug, Clone, PartialEq)]
pub struct BlogPosting {
    inner: Article,
}

impl Article {
    /// Creates an `Article` builder with its required headline.
    #[must_use]
    pub fn new<H>(headline: H) -> Self
    where
        H: Into<String>,
    {
        Self {
            schema_type: String::from("Article"),
            headline: headline.into(),
            images: Vec::new(),
            date_published: None,
            date_modified: None,
            description: None,
            author: None,
            publisher: None,
            main_entity_of_page: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Overrides the emitted schema.org type, for example `NewsArticle`.
    #[must_use]
    pub fn schema_type<T>(mut self, schema_type: T) -> Self
    where
        T: Into<String>,
    {
        self.schema_type = schema_type.into();
        self
    }

    /// Adds one `image`.
    #[must_use]
    pub fn image<V>(mut self, image: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.images.push(image.into());
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

    /// Sets `dateModified`.
    #[must_use]
    pub fn date_modified<D>(mut self, date_modified: D) -> Self
    where
        D: Into<String>,
    {
        self.date_modified = Some(date_modified.into());
        self
    }

    /// Sets `description`.
    #[must_use]
    pub fn description<D>(mut self, description: D) -> Self
    where
        D: Into<String>,
    {
        self.description = Some(description.into());
        self
    }

    /// Sets a simple person `author`.
    #[must_use]
    pub fn author_name<A>(mut self, author_name: A) -> Self
    where
        A: Into<String>,
    {
        self.author = Some(JsonValue::from(named_thing("Person", author_name.into())));
        self
    }

    /// Overrides the emitted `author`.
    #[must_use]
    pub fn author<V>(mut self, author: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.author = Some(author.into());
        self
    }

    /// Overrides the emitted `publisher`.
    #[must_use]
    pub fn publisher<V>(mut self, publisher: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.publisher = Some(publisher.into());
        self
    }

    /// Sets `mainEntityOfPage`.
    #[must_use]
    pub fn main_entity_of_page<U>(mut self, url: U) -> Self
    where
        U: Into<String>,
    {
        self.main_entity_of_page = Some(url.into());
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

    /// Returns the article as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty(self.schema_type)
            .field("headline", self.headline);

        if !self.images.is_empty() {
            object = object.field("image", values_to_json(self.images));
        }

        if let Some(date_published) = self.date_published {
            object = object.field("datePublished", date_published);
        }

        if let Some(date_modified) = self.date_modified {
            object = object.field("dateModified", date_modified);
        }

        if let Some(description) = self.description {
            object = object.field("description", description);
        }

        if let Some(author) = self.author {
            object = object.field("author", author);
        }

        if let Some(publisher) = self.publisher {
            object = object.field("publisher", publisher);
        }

        if let Some(main_entity_of_page) = self.main_entity_of_page {
            object = object.field("mainEntityOfPage", main_entity_of_page);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the article as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl Recipe {
    /// Creates a `Recipe` builder with its required name.
    #[must_use]
    pub fn new<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            images: Vec::new(),
            author: None,
            date_published: None,
            description: None,
            prep_time: None,
            cook_time: None,
            total_time: None,
            yield_value: None,
            ingredients: Vec::new(),
            instructions: Vec::new(),
            aggregate_rating: None,
            reviews: Vec::new(),
            extra: JsonLdObject::new(),
        }
    }

    /// Adds one `image`.
    #[must_use]
    pub fn image<V>(mut self, image: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.images.push(image.into());
        self
    }

    /// Sets a simple person `author`.
    #[must_use]
    pub fn author_name<A>(mut self, author_name: A) -> Self
    where
        A: Into<String>,
    {
        self.author = Some(JsonValue::from(named_thing("Person", author_name.into())));
        self
    }

    /// Overrides the emitted `author`.
    #[must_use]
    pub fn author<V>(mut self, author: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.author = Some(author.into());
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

    /// Sets `description`.
    #[must_use]
    pub fn description<D>(mut self, description: D) -> Self
    where
        D: Into<String>,
    {
        self.description = Some(description.into());
        self
    }

    /// Sets `prepTime`.
    #[must_use]
    pub fn prep_time<T>(mut self, prep_time: T) -> Self
    where
        T: Into<String>,
    {
        self.prep_time = Some(prep_time.into());
        self
    }

    /// Sets `cookTime`.
    #[must_use]
    pub fn cook_time<T>(mut self, cook_time: T) -> Self
    where
        T: Into<String>,
    {
        self.cook_time = Some(cook_time.into());
        self
    }

    /// Sets `totalTime`.
    #[must_use]
    pub fn total_time<T>(mut self, total_time: T) -> Self
    where
        T: Into<String>,
    {
        self.total_time = Some(total_time.into());
        self
    }

    /// Sets `recipeYield`.
    #[must_use]
    pub fn recipe_yield<V>(mut self, recipe_yield: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.yield_value = Some(recipe_yield.into());
        self
    }

    /// Adds one `recipeIngredient`.
    #[must_use]
    pub fn ingredient<I>(mut self, ingredient: I) -> Self
    where
        I: Into<String>,
    {
        self.ingredients.push(ingredient.into());
        self
    }

    /// Adds a plain-text instruction step.
    #[must_use]
    pub fn instruction_text<I>(mut self, instruction: I) -> Self
    where
        I: Into<String>,
    {
        self.instructions.push(JsonValue::from(instruction.into()));
        self
    }

    /// Adds a raw instruction object.
    #[must_use]
    pub fn instruction<V>(mut self, instruction: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.instructions.push(instruction.into());
        self
    }

    /// Sets `aggregateRating`.
    #[must_use]
    pub fn aggregate_rating<V>(mut self, aggregate_rating: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.aggregate_rating = Some(aggregate_rating.into());
        self
    }

    /// Adds one `review`.
    #[must_use]
    pub fn review<V>(mut self, review: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.reviews.push(review.into());
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

    /// Returns the recipe as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new().ty("Recipe").field("name", self.name);

        if !self.images.is_empty() {
            object = object.field("image", values_to_json(self.images));
        }

        if let Some(author) = self.author {
            object = object.field("author", author);
        }

        if let Some(date_published) = self.date_published {
            object = object.field("datePublished", date_published);
        }

        if let Some(description) = self.description {
            object = object.field("description", description);
        }

        if let Some(prep_time) = self.prep_time {
            object = object.field("prepTime", prep_time);
        }

        if let Some(cook_time) = self.cook_time {
            object = object.field("cookTime", cook_time);
        }

        if let Some(total_time) = self.total_time {
            object = object.field("totalTime", total_time);
        }

        if let Some(recipe_yield) = self.yield_value {
            object = object.field("recipeYield", recipe_yield);
        }

        if !self.ingredients.is_empty() {
            object = object.field("recipeIngredient", strings_to_json(self.ingredients));
        }

        if !self.instructions.is_empty() {
            object = object.field("recipeInstructions", values_to_json(self.instructions));
        }

        if let Some(aggregate_rating) = self.aggregate_rating {
            object = object.field("aggregateRating", aggregate_rating);
        }

        if !self.reviews.is_empty() {
            object = object.field("review", values_to_json(self.reviews));
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the recipe as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl NewsArticle {
    /// Creates a `NewsArticle` builder with its required headline.
    #[must_use]
    pub fn new<H>(headline: H) -> Self
    where
        H: Into<String>,
    {
        Self {
            inner: Article::new(headline).schema_type("NewsArticle"),
        }
    }

    /// Adds one `image`.
    #[must_use]
    pub fn image<V>(mut self, image: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.inner = self.inner.image(image);
        self
    }

    /// Sets `datePublished`.
    #[must_use]
    pub fn date_published<D>(mut self, date_published: D) -> Self
    where
        D: Into<String>,
    {
        self.inner = self.inner.date_published(date_published);
        self
    }

    /// Sets `dateModified`.
    #[must_use]
    pub fn date_modified<D>(mut self, date_modified: D) -> Self
    where
        D: Into<String>,
    {
        self.inner = self.inner.date_modified(date_modified);
        self
    }

    /// Sets `description`.
    #[must_use]
    pub fn description<D>(mut self, description: D) -> Self
    where
        D: Into<String>,
    {
        self.inner = self.inner.description(description);
        self
    }

    /// Sets a simple person `author`.
    #[must_use]
    pub fn author_name<A>(mut self, author_name: A) -> Self
    where
        A: Into<String>,
    {
        self.inner = self.inner.author_name(author_name);
        self
    }

    /// Overrides the emitted `author`.
    #[must_use]
    pub fn author<V>(mut self, author: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.inner = self.inner.author(author);
        self
    }

    /// Overrides the emitted `publisher`.
    #[must_use]
    pub fn publisher<V>(mut self, publisher: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.inner = self.inner.publisher(publisher);
        self
    }

    /// Sets `mainEntityOfPage`.
    #[must_use]
    pub fn main_entity_of_page<U>(mut self, url: U) -> Self
    where
        U: Into<String>,
    {
        self.inner = self.inner.main_entity_of_page(url);
        self
    }

    /// Adds an extra field for unsupported-but-needed properties.
    #[must_use]
    pub fn field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.inner = self.inner.field(key, value);
        self
    }

    /// Returns the news article as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        self.inner.into_object()
    }

    /// Returns the news article as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        self.inner.into_document()
    }
}

impl BlogPosting {
    /// Creates a `BlogPosting` builder with its required headline.
    #[must_use]
    pub fn new<H>(headline: H) -> Self
    where
        H: Into<String>,
    {
        Self {
            inner: Article::new(headline).schema_type("BlogPosting"),
        }
    }

    /// Adds one `image`.
    #[must_use]
    pub fn image<V>(mut self, image: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.inner = self.inner.image(image);
        self
    }

    /// Sets `datePublished`.
    #[must_use]
    pub fn date_published<D>(mut self, date_published: D) -> Self
    where
        D: Into<String>,
    {
        self.inner = self.inner.date_published(date_published);
        self
    }

    /// Sets `dateModified`.
    #[must_use]
    pub fn date_modified<D>(mut self, date_modified: D) -> Self
    where
        D: Into<String>,
    {
        self.inner = self.inner.date_modified(date_modified);
        self
    }

    /// Sets `description`.
    #[must_use]
    pub fn description<D>(mut self, description: D) -> Self
    where
        D: Into<String>,
    {
        self.inner = self.inner.description(description);
        self
    }

    /// Sets a simple person `author`.
    #[must_use]
    pub fn author_name<A>(mut self, author_name: A) -> Self
    where
        A: Into<String>,
    {
        self.inner = self.inner.author_name(author_name);
        self
    }

    /// Overrides the emitted `author`.
    #[must_use]
    pub fn author<V>(mut self, author: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.inner = self.inner.author(author);
        self
    }

    /// Overrides the emitted `publisher`.
    #[must_use]
    pub fn publisher<V>(mut self, publisher: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.inner = self.inner.publisher(publisher);
        self
    }

    /// Sets `mainEntityOfPage`.
    #[must_use]
    pub fn main_entity_of_page<U>(mut self, url: U) -> Self
    where
        U: Into<String>,
    {
        self.inner = self.inner.main_entity_of_page(url);
        self
    }

    /// Adds an extra field for unsupported-but-needed properties.
    #[must_use]
    pub fn field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.inner = self.inner.field(key, value);
        self
    }

    /// Returns the blog posting as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        self.inner.into_object()
    }

    /// Returns the blog posting as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        self.inner.into_document()
    }
}

impl From<Article> for JsonLdObject {
    fn from(value: Article) -> Self {
        value.into_object()
    }
}

impl From<Article> for JsonLdDocument {
    fn from(value: Article) -> Self {
        value.into_document()
    }
}

impl From<Article> for JsonValue {
    fn from(value: Article) -> Self {
        value.into_object().into()
    }
}

impl From<NewsArticle> for JsonLdObject {
    fn from(value: NewsArticle) -> Self {
        value.into_object()
    }
}

impl From<NewsArticle> for JsonLdDocument {
    fn from(value: NewsArticle) -> Self {
        value.into_document()
    }
}

impl From<NewsArticle> for JsonValue {
    fn from(value: NewsArticle) -> Self {
        value.into_object().into()
    }
}

impl From<BlogPosting> for JsonLdObject {
    fn from(value: BlogPosting) -> Self {
        value.into_object()
    }
}

impl From<BlogPosting> for JsonLdDocument {
    fn from(value: BlogPosting) -> Self {
        value.into_document()
    }
}

impl From<BlogPosting> for JsonValue {
    fn from(value: BlogPosting) -> Self {
        value.into_object().into()
    }
}

impl From<Recipe> for JsonLdObject {
    fn from(value: Recipe) -> Self {
        value.into_object()
    }
}

impl From<Recipe> for JsonLdDocument {
    fn from(value: Recipe) -> Self {
        value.into_document()
    }
}

impl From<Recipe> for JsonValue {
    fn from(value: Recipe) -> Self {
        value.into_object().into()
    }
}

fn strings_to_json(values: Vec<String>) -> JsonValue {
    values_to_json(values.into_iter().map(JsonValue::from).collect())
}

fn values_to_json(values: Vec<JsonValue>) -> JsonValue {
    match values.as_slice() {
        [single] => single.clone(),
        _ => JsonValue::Array(values),
    }
}
