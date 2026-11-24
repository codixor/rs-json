use crate::{JsonLdDocument, JsonLdObject, JsonValue};

use super::SCHEMA_ORG_CONTEXT;

/// A Google-friendly `SoftwareApplication` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct SoftwareApplication {
    name: String,
    price: JsonValue,
    price_currency: Option<String>,
    operating_system: Option<String>,
    application_category: Option<String>,
    aggregate_rating: Option<JsonValue>,
    review: Option<JsonValue>,
    url: Option<String>,
    offer_extra: JsonLdObject,
    extra: JsonLdObject,
}

impl SoftwareApplication {
    /// Creates a `SoftwareApplication` builder with the Google-required `name` and `offers.price`.
    #[must_use]
    pub fn new<N, P>(name: N, price: P) -> Self
    where
        N: Into<String>,
        P: Into<JsonValue>,
    {
        Self {
            name: name.into(),
            price: price.into(),
            price_currency: None,
            operating_system: None,
            application_category: None,
            aggregate_rating: None,
            review: None,
            url: None,
            offer_extra: JsonLdObject::new(),
            extra: JsonLdObject::new(),
        }
    }

    /// Creates a free `SoftwareApplication`.
    #[must_use]
    pub fn free<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Self::new(name, 0)
    }

    /// Sets `offers.priceCurrency`.
    #[must_use]
    pub fn price_currency<C>(mut self, price_currency: C) -> Self
    where
        C: Into<String>,
    {
        self.price_currency = Some(price_currency.into());
        self
    }

    /// Sets `operatingSystem`.
    #[must_use]
    pub fn operating_system<O>(mut self, operating_system: O) -> Self
    where
        O: Into<String>,
    {
        self.operating_system = Some(operating_system.into());
        self
    }

    /// Sets `applicationCategory`.
    #[must_use]
    pub fn application_category<C>(mut self, application_category: C) -> Self
    where
        C: Into<String>,
    {
        self.application_category = Some(application_category.into());
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

    /// Sets `review`.
    #[must_use]
    pub fn review<V>(mut self, review: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.review = Some(review.into());
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

    /// Adds an extra field to the nested `Offer`.
    #[must_use]
    pub fn offer_field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.offer_extra = self.offer_extra.field(key, value);
        self
    }

    /// Adds an extra top-level field for unsupported-but-needed properties.
    #[must_use]
    pub fn field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.extra = self.extra.field(key, value);
        self
    }

    /// Returns the software application as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut offer = JsonLdObject::new().ty("Offer").field("price", self.price);

        if let Some(price_currency) = self.price_currency {
            offer = offer.field("priceCurrency", price_currency);
        }

        for (key, value) in self.offer_extra.into_map() {
            offer = offer.field(key, value);
        }

        let mut object = JsonLdObject::new()
            .ty("SoftwareApplication")
            .field("name", self.name)
            .field("offers", offer);

        if let Some(operating_system) = self.operating_system {
            object = object.field("operatingSystem", operating_system);
        }

        if let Some(application_category) = self.application_category {
            object = object.field("applicationCategory", application_category);
        }

        if let Some(aggregate_rating) = self.aggregate_rating {
            object = object.field("aggregateRating", aggregate_rating);
        }

        if let Some(review) = self.review {
            object = object.field("review", review);
        }

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the software application as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl From<SoftwareApplication> for JsonLdObject {
    fn from(value: SoftwareApplication) -> Self {
        value.into_object()
    }
}

impl From<SoftwareApplication> for JsonLdDocument {
    fn from(value: SoftwareApplication) -> Self {
        value.into_document()
    }
}

impl From<SoftwareApplication> for JsonValue {
    fn from(value: SoftwareApplication) -> Self {
        value.into_object().into()
    }
}
