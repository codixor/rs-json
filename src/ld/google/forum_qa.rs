use crate::{JsonLdDocument, JsonLdObject, JsonValue};

use super::{SCHEMA_ORG_CONTEXT, named_thing, values_to_json};

/// A Google-friendly `Answer` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Answer {
    text: String,
    author: Option<JsonValue>,
    date_published: Option<String>,
    upvote_count: Option<JsonValue>,
    url: Option<String>,
    extra: JsonLdObject,
}

/// A Google-friendly `Question` builder for `QAPage`.
#[derive(Debug, Clone, PartialEq)]
pub struct Question {
    name: String,
    text: String,
    author: Option<JsonValue>,
    date_created: Option<String>,
    upvote_count: Option<JsonValue>,
    answer_count: Option<JsonValue>,
    accepted_answer: Option<JsonValue>,
    suggested_answers: Vec<JsonValue>,
    url: Option<String>,
    extra: JsonLdObject,
}

/// A Google-friendly `QAPage` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct QAPage {
    main_entity: JsonValue,
    extra: JsonLdObject,
}

/// A Google-friendly `Comment` builder for forum threads.
#[derive(Debug, Clone, PartialEq)]
pub struct ForumComment {
    text: String,
    author: JsonValue,
    date_published: String,
    upvote_count: Option<JsonValue>,
    url: Option<String>,
    extra: JsonLdObject,
}

/// A Google-friendly `DiscussionForumPosting` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscussionForumPosting {
    headline: String,
    article_body: String,
    author: JsonValue,
    date_published: String,
    comments: Vec<JsonValue>,
    shared_contents: Vec<JsonValue>,
    url: Option<String>,
    interaction_statistics: Vec<JsonValue>,
    extra: JsonLdObject,
}

impl Answer {
    /// Creates an `Answer` builder with its required `text`.
    #[must_use]
    pub fn new<T>(text: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            text: text.into(),
            author: None,
            date_published: None,
            upvote_count: None,
            url: None,
            extra: JsonLdObject::new(),
        }
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

    /// Sets `upvoteCount`.
    #[must_use]
    pub fn upvote_count<V>(mut self, upvote_count: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.upvote_count = Some(upvote_count.into());
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

    /// Returns the answer as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new().ty("Answer").field("text", self.text);

        if let Some(author) = self.author {
            object = object.field("author", author);
        }

        if let Some(date_published) = self.date_published {
            object = object.field("datePublished", date_published);
        }

        if let Some(upvote_count) = self.upvote_count {
            object = object.field("upvoteCount", upvote_count);
        }

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl Question {
    /// Creates a `Question` builder with the Google-visible core question text.
    #[must_use]
    pub fn new<N, T>(name: N, text: T) -> Self
    where
        N: Into<String>,
        T: Into<String>,
    {
        Self {
            name: name.into(),
            text: text.into(),
            author: None,
            date_created: None,
            upvote_count: None,
            answer_count: None,
            accepted_answer: None,
            suggested_answers: Vec::new(),
            url: None,
            extra: JsonLdObject::new(),
        }
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

    /// Sets `dateCreated`.
    #[must_use]
    pub fn date_created<D>(mut self, date_created: D) -> Self
    where
        D: Into<String>,
    {
        self.date_created = Some(date_created.into());
        self
    }

    /// Sets `upvoteCount`.
    #[must_use]
    pub fn upvote_count<V>(mut self, upvote_count: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.upvote_count = Some(upvote_count.into());
        self
    }

    /// Sets `answerCount`.
    #[must_use]
    pub fn answer_count<V>(mut self, answer_count: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.answer_count = Some(answer_count.into());
        self
    }

    /// Sets `acceptedAnswer`.
    #[must_use]
    pub fn accepted_answer<V>(mut self, accepted_answer: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.accepted_answer = Some(accepted_answer.into());
        self
    }

    /// Adds one `suggestedAnswer`.
    #[must_use]
    pub fn suggested_answer<V>(mut self, suggested_answer: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.suggested_answers.push(suggested_answer.into());
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

    /// Returns the question as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("Question")
            .field("name", self.name)
            .field("text", self.text);

        if let Some(author) = self.author {
            object = object.field("author", author);
        }

        if let Some(date_created) = self.date_created {
            object = object.field("dateCreated", date_created);
        }

        if let Some(upvote_count) = self.upvote_count {
            object = object.field("upvoteCount", upvote_count);
        }

        if let Some(answer_count) = self.answer_count {
            object = object.field("answerCount", answer_count);
        }

        if let Some(accepted_answer) = self.accepted_answer {
            object = object.field("acceptedAnswer", accepted_answer);
        }

        if !self.suggested_answers.is_empty() {
            object = object.field("suggestedAnswer", values_to_json(self.suggested_answers));
        }

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl QAPage {
    /// Creates a `QAPage` builder with its required `mainEntity`.
    #[must_use]
    pub fn new<V>(question: V) -> Self
    where
        V: Into<JsonValue>,
    {
        Self {
            main_entity: question.into(),
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

    /// Returns the page as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("QAPage")
            .field("mainEntity", self.main_entity);

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the page as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl ForumComment {
    /// Creates a `Comment` builder with the Google-required core fields.
    #[must_use]
    pub fn new<T, A, D>(text: T, author_name: A, date_published: D) -> Self
    where
        T: Into<String>,
        A: Into<String>,
        D: Into<String>,
    {
        Self {
            text: text.into(),
            author: JsonValue::from(named_thing("Person", author_name.into())),
            date_published: date_published.into(),
            upvote_count: None,
            url: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Overrides the emitted `author`.
    #[must_use]
    pub fn author<V>(mut self, author: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.author = author.into();
        self
    }

    /// Sets `upvoteCount`.
    #[must_use]
    pub fn upvote_count<V>(mut self, upvote_count: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.upvote_count = Some(upvote_count.into());
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

    /// Returns the comment as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("Comment")
            .field("text", self.text)
            .field("author", self.author)
            .field("datePublished", self.date_published);

        if let Some(upvote_count) = self.upvote_count {
            object = object.field("upvoteCount", upvote_count);
        }

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl DiscussionForumPosting {
    /// Creates a `DiscussionForumPosting` builder with Google's core post fields.
    #[must_use]
    pub fn new<H, T, A, D>(headline: H, article_body: T, author_name: A, date_published: D) -> Self
    where
        H: Into<String>,
        T: Into<String>,
        A: Into<String>,
        D: Into<String>,
    {
        Self {
            headline: headline.into(),
            article_body: article_body.into(),
            author: JsonValue::from(named_thing("Person", author_name.into())),
            date_published: date_published.into(),
            comments: Vec::new(),
            shared_contents: Vec::new(),
            url: None,
            interaction_statistics: Vec::new(),
            extra: JsonLdObject::new(),
        }
    }

    /// Overrides the emitted `author`.
    #[must_use]
    pub fn author<V>(mut self, author: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.author = author.into();
        self
    }

    /// Adds one nested `Comment`.
    #[must_use]
    pub fn comment<V>(mut self, comment: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.comments.push(comment.into());
        self
    }

    /// Adds one nested shared content item.
    #[must_use]
    pub fn shared_content<V>(mut self, shared_content: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.shared_contents.push(shared_content.into());
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

    /// Adds one `interactionStatistic`.
    #[must_use]
    pub fn interaction_statistic<A>(
        mut self,
        action_type_url: A,
        user_interaction_count: u64,
    ) -> Self
    where
        A: Into<String>,
    {
        self.interaction_statistics.push(
            JsonLdObject::new()
                .ty("InteractionCounter")
                .field(
                    "interactionType",
                    JsonLdObject::new()
                        .ty("Action")
                        .field("@id", action_type_url.into()),
                )
                .field("userInteractionCount", user_interaction_count)
                .into(),
        );
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

    /// Returns the posting as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("DiscussionForumPosting")
            .field("headline", self.headline)
            .field("articleBody", self.article_body)
            .field("author", self.author)
            .field("datePublished", self.date_published);

        if !self.comments.is_empty() {
            object = object.field("comment", values_to_json(self.comments));
        }

        if !self.shared_contents.is_empty() {
            object = object.field("sharedContent", values_to_json(self.shared_contents));
        }

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        if !self.interaction_statistics.is_empty() {
            object = object.field(
                "interactionStatistic",
                values_to_json(self.interaction_statistics),
            );
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the posting as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl From<Answer> for JsonLdObject {
    fn from(value: Answer) -> Self {
        value.into_object()
    }
}

impl From<Answer> for JsonValue {
    fn from(value: Answer) -> Self {
        value.into_object().into()
    }
}

impl From<Question> for JsonLdObject {
    fn from(value: Question) -> Self {
        value.into_object()
    }
}

impl From<Question> for JsonValue {
    fn from(value: Question) -> Self {
        value.into_object().into()
    }
}

impl From<QAPage> for JsonLdObject {
    fn from(value: QAPage) -> Self {
        value.into_object()
    }
}

impl From<QAPage> for JsonLdDocument {
    fn from(value: QAPage) -> Self {
        value.into_document()
    }
}

impl From<QAPage> for JsonValue {
    fn from(value: QAPage) -> Self {
        value.into_object().into()
    }
}

impl From<ForumComment> for JsonLdObject {
    fn from(value: ForumComment) -> Self {
        value.into_object()
    }
}

impl From<ForumComment> for JsonValue {
    fn from(value: ForumComment) -> Self {
        value.into_object().into()
    }
}

impl From<DiscussionForumPosting> for JsonLdObject {
    fn from(value: DiscussionForumPosting) -> Self {
        value.into_object()
    }
}

impl From<DiscussionForumPosting> for JsonLdDocument {
    fn from(value: DiscussionForumPosting) -> Self {
        value.into_document()
    }
}

impl From<DiscussionForumPosting> for JsonValue {
    fn from(value: DiscussionForumPosting) -> Self {
        value.into_object().into()
    }
}
