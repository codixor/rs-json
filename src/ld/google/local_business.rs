use crate::{JsonLdDocument, JsonLdObject, JsonValue};

use super::{SCHEMA_ORG_CONTEXT, strings_to_json, values_to_json};

/// A Google-friendly `LocalBusiness` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalBusiness {
    schema_type: String,
    name: String,
    address: JsonValue,
    url: Option<String>,
    telephone: Option<String>,
    images: Vec<JsonValue>,
    price_range: Option<String>,
    geo: Option<JsonValue>,
    opening_hours: Vec<JsonValue>,
    same_as: Vec<String>,
    aggregate_rating: Option<JsonValue>,
    serves_cuisine: Vec<String>,
    extra: JsonLdObject,
}

impl LocalBusiness {
    /// Creates a `LocalBusiness` builder with the Google-required `name` and `address`.
    #[must_use]
    pub fn new<N, A>(name: N, address: A) -> Self
    where
        N: Into<String>,
        A: Into<JsonValue>,
    {
        Self {
            schema_type: String::from("LocalBusiness"),
            name: name.into(),
            address: address.into(),
            url: None,
            telephone: None,
            images: Vec::new(),
            price_range: None,
            geo: None,
            opening_hours: Vec::new(),
            same_as: Vec::new(),
            aggregate_rating: None,
            serves_cuisine: Vec::new(),
            extra: JsonLdObject::new(),
        }
    }

    /// Overrides the emitted schema.org type, for example `Restaurant` or `Store`.
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

    /// Sets `telephone`.
    #[must_use]
    pub fn telephone<T>(mut self, telephone: T) -> Self
    where
        T: Into<String>,
    {
        self.telephone = Some(telephone.into());
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

    /// Sets `priceRange`.
    #[must_use]
    pub fn price_range<P>(mut self, price_range: P) -> Self
    where
        P: Into<String>,
    {
        self.price_range = Some(price_range.into());
        self
    }

    /// Sets `geo` from a raw JSON-LD value.
    #[must_use]
    pub fn geo<V>(mut self, geo: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.geo = Some(geo.into());
        self
    }

    /// Sets `geo` from numeric latitude and longitude.
    #[must_use]
    pub fn geo_coordinates(mut self, latitude: f64, longitude: f64) -> Self {
        self.geo = Some(
            JsonLdObject::new()
                .ty("GeoCoordinates")
                .field("latitude", latitude)
                .field("longitude", longitude)
                .into(),
        );
        self
    }

    /// Adds one `openingHoursSpecification`.
    #[must_use]
    pub fn opening_hours_specification<V>(mut self, specification: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.opening_hours.push(specification.into());
        self
    }

    /// Adds one simple `openingHoursSpecification` entry.
    #[must_use]
    pub fn opening_hours<D, O, C>(mut self, day_of_week: D, opens: O, closes: C) -> Self
    where
        D: Into<String>,
        O: Into<String>,
        C: Into<String>,
    {
        self.opening_hours.push(
            JsonLdObject::new()
                .ty("OpeningHoursSpecification")
                .field("dayOfWeek", day_of_week.into())
                .field("opens", opens.into())
                .field("closes", closes.into())
                .into(),
        );
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

    /// Sets `aggregateRating`.
    #[must_use]
    pub fn aggregate_rating<V>(mut self, aggregate_rating: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.aggregate_rating = Some(aggregate_rating.into());
        self
    }

    /// Adds one `servesCuisine` value.
    #[must_use]
    pub fn serves_cuisine<C>(mut self, cuisine: C) -> Self
    where
        C: Into<String>,
    {
        self.serves_cuisine.push(cuisine.into());
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

    /// Returns the local business as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty(self.schema_type)
            .field("name", self.name)
            .field("address", self.address);

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        if let Some(telephone) = self.telephone {
            object = object.field("telephone", telephone);
        }

        if !self.images.is_empty() {
            object = object.field("image", values_to_json(self.images));
        }

        if let Some(price_range) = self.price_range {
            object = object.field("priceRange", price_range);
        }

        if let Some(geo) = self.geo {
            object = object.field("geo", geo);
        }

        if !self.opening_hours.is_empty() {
            object = object.field(
                "openingHoursSpecification",
                values_to_json(self.opening_hours),
            );
        }

        if !self.same_as.is_empty() {
            object = object.field("sameAs", strings_to_json(self.same_as));
        }

        if let Some(aggregate_rating) = self.aggregate_rating {
            object = object.field("aggregateRating", aggregate_rating);
        }

        if !self.serves_cuisine.is_empty() {
            object = object.field("servesCuisine", strings_to_json(self.serves_cuisine));
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the local business as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl From<LocalBusiness> for JsonLdObject {
    fn from(value: LocalBusiness) -> Self {
        value.into_object()
    }
}

impl From<LocalBusiness> for JsonLdDocument {
    fn from(value: LocalBusiness) -> Self {
        value.into_document()
    }
}

impl From<LocalBusiness> for JsonValue {
    fn from(value: LocalBusiness) -> Self {
        value.into_object().into()
    }
}
