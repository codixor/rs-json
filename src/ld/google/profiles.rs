use crate::{JsonLdDocument, JsonLdObject, JsonValue};

use super::{SCHEMA_ORG_CONTEXT, strings_to_json, values_to_json};

/// A Google-friendly profile `Person` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfilePerson {
    name: String,
    alternate_name: Option<String>,
    description: Option<String>,
    identifier: Option<String>,
    image: Option<JsonValue>,
    same_as: Vec<String>,
    interaction_statistics: Vec<JsonValue>,
    extra: JsonLdObject,
}

/// A Google-friendly `ProfilePage` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfilePage {
    main_entity: JsonValue,
    date_created: Option<String>,
    date_modified: Option<String>,
    has_parts: Vec<JsonValue>,
    extra: JsonLdObject,
}

impl ProfilePerson {
    /// Creates a profile `Person` with the Google-required `name`.
    #[must_use]
    pub fn new<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            alternate_name: None,
            description: None,
            identifier: None,
            image: None,
            same_as: Vec::new(),
            interaction_statistics: Vec::new(),
            extra: JsonLdObject::new(),
        }
    }

    /// Sets `alternateName`.
    #[must_use]
    pub fn alternate_name<N>(mut self, alternate_name: N) -> Self
    where
        N: Into<String>,
    {
        self.alternate_name = Some(alternate_name.into());
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

    /// Sets `identifier`.
    #[must_use]
    pub fn identifier<I>(mut self, identifier: I) -> Self
    where
        I: Into<String>,
    {
        self.identifier = Some(identifier.into());
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

    /// Adds one `sameAs` URL.
    #[must_use]
    pub fn same_as<U>(mut self, url: U) -> Self
    where
        U: Into<String>,
    {
        self.same_as.push(url.into());
        self
    }

    /// Adds one `agentInteractionStatistic`.
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

    /// Returns the profile person as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new().ty("Person").field("name", self.name);

        if let Some(alternate_name) = self.alternate_name {
            object = object.field("alternateName", alternate_name);
        }

        if let Some(description) = self.description {
            object = object.field("description", description);
        }

        if let Some(identifier) = self.identifier {
            object = object.field("identifier", identifier);
        }

        if let Some(image) = self.image {
            object = object.field("image", image);
        }

        if !self.same_as.is_empty() {
            object = object.field("sameAs", strings_to_json(self.same_as));
        }

        if !self.interaction_statistics.is_empty() {
            object = object.field(
                "agentInteractionStatistic",
                values_to_json(self.interaction_statistics),
            );
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl ProfilePage {
    /// Creates a `ProfilePage` builder with its required `mainEntity`.
    #[must_use]
    pub fn new<V>(main_entity: V) -> Self
    where
        V: Into<JsonValue>,
    {
        Self {
            main_entity: main_entity.into(),
            date_created: None,
            date_modified: None,
            has_parts: Vec::new(),
            extra: JsonLdObject::new(),
        }
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

    /// Sets `dateModified`.
    #[must_use]
    pub fn date_modified<D>(mut self, date_modified: D) -> Self
    where
        D: Into<String>,
    {
        self.date_modified = Some(date_modified.into());
        self
    }

    /// Adds one `hasPart`.
    #[must_use]
    pub fn has_part<V>(mut self, part: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.has_parts.push(part.into());
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

    /// Returns the profile page as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("ProfilePage")
            .field("mainEntity", self.main_entity);

        if let Some(date_created) = self.date_created {
            object = object.field("dateCreated", date_created);
        }

        if let Some(date_modified) = self.date_modified {
            object = object.field("dateModified", date_modified);
        }

        if !self.has_parts.is_empty() {
            object = object.field("hasPart", values_to_json(self.has_parts));
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the profile page as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl From<ProfilePerson> for JsonLdObject {
    fn from(value: ProfilePerson) -> Self {
        value.into_object()
    }
}

impl From<ProfilePerson> for JsonValue {
    fn from(value: ProfilePerson) -> Self {
        value.into_object().into()
    }
}

impl From<ProfilePage> for JsonLdObject {
    fn from(value: ProfilePage) -> Self {
        value.into_object()
    }
}

impl From<ProfilePage> for JsonLdDocument {
    fn from(value: ProfilePage) -> Self {
        value.into_document()
    }
}

impl From<ProfilePage> for JsonValue {
    fn from(value: ProfilePage) -> Self {
        value.into_object().into()
    }
}
