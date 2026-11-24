use crate::{JsonLdArray, JsonLdDocument, JsonLdObject, JsonValue};

use super::SCHEMA_ORG_CONTEXT;

/// A Google-friendly `Organization` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Organization {
    schema_type: String,
    name: String,
    url: Option<String>,
    logo: Option<JsonValue>,
    same_as: Vec<String>,
    description: Option<String>,
    email: Option<String>,
    telephone: Option<String>,
    merchant_return_policy: Option<JsonValue>,
    extra: JsonLdObject,
}

impl Organization {
    /// Creates an `Organization` builder with its required `name`.
    #[must_use]
    pub fn new<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Self {
            schema_type: String::from("Organization"),
            name: name.into(),
            url: None,
            logo: None,
            same_as: Vec::new(),
            description: None,
            email: None,
            telephone: None,
            merchant_return_policy: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Overrides the emitted schema.org type, for example `OnlineStore`.
    #[must_use]
    pub fn schema_type<T>(mut self, schema_type: T) -> Self
    where
        T: Into<String>,
    {
        self.schema_type = schema_type.into();
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

    /// Sets `logo` from a raw JSON-LD value.
    #[must_use]
    pub fn logo<V>(mut self, value: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.logo = Some(value.into());
        self
    }

    /// Sets `logo` from a URL string.
    #[must_use]
    pub fn logo_url<U>(self, url: U) -> Self
    where
        U: Into<String>,
    {
        self.logo(url.into())
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

    /// Sets `description`.
    #[must_use]
    pub fn description<D>(mut self, description: D) -> Self
    where
        D: Into<String>,
    {
        self.description = Some(description.into());
        self
    }

    /// Sets `email`.
    #[must_use]
    pub fn email<E>(mut self, email: E) -> Self
    where
        E: Into<String>,
    {
        self.email = Some(email.into());
        self
    }

    /// Sets `telephone`.
    #[must_use]
    pub fn telephone<T>(mut self, telephone: T) -> Self
    where
        T: Into<String>,
    {
        self.telephone = Some(telephone.into());
        self
    }

    /// Sets `hasMerchantReturnPolicy` from a raw JSON-LD value.
    #[must_use]
    pub fn merchant_return_policy<V>(mut self, policy: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.merchant_return_policy = Some(policy.into());
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

    /// Returns the organization as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty(self.schema_type)
            .field("name", self.name);

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        if let Some(logo) = self.logo {
            object = object.field("logo", logo);
        }

        if !self.same_as.is_empty() {
            let mut same_as = JsonLdArray::new();
            for url in self.same_as {
                same_as = same_as.item(url);
            }
            object = object.field("sameAs", same_as);
        }

        if let Some(description) = self.description {
            object = object.field("description", description);
        }

        if let Some(email) = self.email {
            object = object.field("email", email);
        }

        if let Some(telephone) = self.telephone {
            object = object.field("telephone", telephone);
        }

        if let Some(policy) = self.merchant_return_policy {
            object = object.field("hasMerchantReturnPolicy", policy);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the organization as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl From<Organization> for JsonLdObject {
    fn from(value: Organization) -> Self {
        value.into_object()
    }
}

impl From<Organization> for JsonLdDocument {
    fn from(value: Organization) -> Self {
        value.into_document()
    }
}

impl From<Organization> for JsonValue {
    fn from(value: Organization) -> Self {
        value.into_object().into()
    }
}
