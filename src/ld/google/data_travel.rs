use crate::{JsonLdDocument, JsonLdObject, JsonValue};

use super::{SCHEMA_ORG_CONTEXT, strings_to_json, values_to_json};

/// A Google-friendly `DataCatalog` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct DataCatalog {
    name: String,
    url: Option<String>,
    extra: JsonLdObject,
}

/// A Google-friendly `DataDownload` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct DataDownload {
    content_url: String,
    encoding_format: Option<String>,
    extra: JsonLdObject,
}

/// A Google-friendly `Dataset` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Dataset {
    name: String,
    description: String,
    creator: JsonValue,
    url: Option<String>,
    same_as: Vec<String>,
    license: Option<JsonValue>,
    distributions: Vec<JsonValue>,
    data_catalogs: Vec<JsonValue>,
    keywords: Vec<String>,
    is_accessible_for_free: Option<bool>,
    extra: JsonLdObject,
}

/// A Google-friendly `Accommodation` builder for vacation rentals.
#[derive(Debug, Clone, PartialEq)]
pub struct Accommodation {
    occupancy_value: u64,
    number_of_rooms: Option<JsonValue>,
    number_of_bathrooms_total: Option<JsonValue>,
    beds: Vec<JsonValue>,
    amenity_features: Vec<JsonValue>,
    floor_size: Option<JsonValue>,
    extra: JsonLdObject,
}

/// A Google-friendly `VacationRental` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct VacationRental {
    name: String,
    identifier: String,
    latitude: f64,
    longitude: f64,
    accommodation: JsonValue,
    images: Vec<JsonValue>,
    description: Option<String>,
    address: Option<JsonValue>,
    url: Option<String>,
    aggregate_rating: Option<JsonValue>,
    extra: JsonLdObject,
}

impl DataCatalog {
    /// Creates a `DataCatalog` builder with its required `name`.
    #[must_use]
    pub fn new<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            url: None,
            extra: JsonLdObject::new(),
        }
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

    /// Returns the catalog as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("DataCatalog")
            .field("name", self.name);

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl DataDownload {
    /// Creates a `DataDownload` builder with its required `contentUrl`.
    #[must_use]
    pub fn new<U>(content_url: U) -> Self
    where
        U: Into<String>,
    {
        Self {
            content_url: content_url.into(),
            encoding_format: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Sets `encodingFormat`.
    #[must_use]
    pub fn encoding_format<F>(mut self, encoding_format: F) -> Self
    where
        F: Into<String>,
    {
        self.encoding_format = Some(encoding_format.into());
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

    /// Returns the download as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("DataDownload")
            .field("contentUrl", self.content_url);

        if let Some(encoding_format) = self.encoding_format {
            object = object.field("encodingFormat", encoding_format);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl Dataset {
    /// Creates a `Dataset` builder with the Google-required core fields.
    #[must_use]
    pub fn new<N, D, C>(name: N, description: D, creator: C) -> Self
    where
        N: Into<String>,
        D: Into<String>,
        C: Into<JsonValue>,
    {
        Self {
            name: name.into(),
            description: description.into(),
            creator: creator.into(),
            url: None,
            same_as: Vec::new(),
            license: None,
            distributions: Vec::new(),
            data_catalogs: Vec::new(),
            keywords: Vec::new(),
            is_accessible_for_free: None,
            extra: JsonLdObject::new(),
        }
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

    /// Adds one `sameAs` URL.
    #[must_use]
    pub fn same_as<U>(mut self, url: U) -> Self
    where
        U: Into<String>,
    {
        self.same_as.push(url.into());
        self
    }

    /// Sets `license`.
    #[must_use]
    pub fn license<V>(mut self, license: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.license = Some(license.into());
        self
    }

    /// Adds one `distribution`.
    #[must_use]
    pub fn distribution<V>(mut self, distribution: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.distributions.push(distribution.into());
        self
    }

    /// Adds one `includedInDataCatalog`.
    #[must_use]
    pub fn included_in_data_catalog<V>(mut self, data_catalog: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.data_catalogs.push(data_catalog.into());
        self
    }

    /// Adds one `keywords` value.
    #[must_use]
    pub fn keyword<K>(mut self, keyword: K) -> Self
    where
        K: Into<String>,
    {
        self.keywords.push(keyword.into());
        self
    }

    /// Sets `isAccessibleForFree`.
    #[must_use]
    pub fn accessible_for_free(mut self, is_accessible_for_free: bool) -> Self {
        self.is_accessible_for_free = Some(is_accessible_for_free);
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

    /// Returns the dataset as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("Dataset")
            .field("name", self.name)
            .field("description", self.description)
            .field("creator", self.creator);

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        if !self.same_as.is_empty() {
            object = object.field("sameAs", strings_to_json(self.same_as));
        }

        if let Some(license) = self.license {
            object = object.field("license", license);
        }

        if !self.distributions.is_empty() {
            object = object.field("distribution", values_to_json(self.distributions));
        }

        if !self.data_catalogs.is_empty() {
            object = object.field("includedInDataCatalog", values_to_json(self.data_catalogs));
        }

        if !self.keywords.is_empty() {
            object = object.field("keywords", strings_to_json(self.keywords));
        }

        if let Some(is_accessible_for_free) = self.is_accessible_for_free {
            object = object.field("isAccessibleForFree", is_accessible_for_free);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the dataset as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl Accommodation {
    /// Creates an `Accommodation` builder with the Google-required occupancy count.
    #[must_use]
    pub fn new(occupancy_value: u64) -> Self {
        Self {
            occupancy_value,
            number_of_rooms: None,
            number_of_bathrooms_total: None,
            beds: Vec::new(),
            amenity_features: Vec::new(),
            floor_size: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Sets `numberOfRooms`.
    #[must_use]
    pub fn number_of_rooms<V>(mut self, number_of_rooms: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.number_of_rooms = Some(number_of_rooms.into());
        self
    }

    /// Sets `numberOfBathroomsTotal`.
    #[must_use]
    pub fn number_of_bathrooms_total<V>(mut self, number_of_bathrooms_total: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.number_of_bathrooms_total = Some(number_of_bathrooms_total.into());
        self
    }

    /// Adds one `bed`.
    #[must_use]
    pub fn bed<T>(mut self, type_of_bed: T, number_of_beds: u64) -> Self
    where
        T: Into<String>,
    {
        self.beds.push(
            JsonLdObject::new()
                .ty("BedDetails")
                .field("typeOfBed", type_of_bed.into())
                .field("numberOfBeds", number_of_beds)
                .into(),
        );
        self
    }

    /// Adds one `amenityFeature`.
    #[must_use]
    pub fn amenity<N>(mut self, name: N, value: bool) -> Self
    where
        N: Into<String>,
    {
        self.amenity_features.push(
            JsonLdObject::new()
                .ty("LocationFeatureSpecification")
                .field("name", name.into())
                .field("value", value)
                .into(),
        );
        self
    }

    /// Sets `floorSize`.
    #[must_use]
    pub fn floor_size<U>(mut self, value: u64, unit_code: U) -> Self
    where
        U: Into<String>,
    {
        self.floor_size = Some(
            JsonLdObject::new()
                .ty("QuantitativeValue")
                .field("value", value)
                .field("unitCode", unit_code.into())
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

    /// Returns the accommodation as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new().ty("Accommodation").field(
            "occupancy",
            JsonLdObject::new()
                .ty("QuantitativeValue")
                .field("value", self.occupancy_value),
        );

        if let Some(number_of_rooms) = self.number_of_rooms {
            object = object.field("numberOfRooms", number_of_rooms);
        }

        if let Some(number_of_bathrooms_total) = self.number_of_bathrooms_total {
            object = object.field("numberOfBathroomsTotal", number_of_bathrooms_total);
        }

        if !self.beds.is_empty() {
            object = object.field("bed", values_to_json(self.beds));
        }

        if !self.amenity_features.is_empty() {
            object = object.field("amenityFeature", values_to_json(self.amenity_features));
        }

        if let Some(floor_size) = self.floor_size {
            object = object.field("floorSize", floor_size);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl VacationRental {
    /// Creates a `VacationRental` builder with the Google-required core fields.
    #[must_use]
    pub fn new<N, I, A>(
        name: N,
        identifier: I,
        latitude: f64,
        longitude: f64,
        accommodation: A,
    ) -> Self
    where
        N: Into<String>,
        I: Into<String>,
        A: Into<JsonValue>,
    {
        Self {
            name: name.into(),
            identifier: identifier.into(),
            latitude,
            longitude,
            accommodation: accommodation.into(),
            images: Vec::new(),
            description: None,
            address: None,
            url: None,
            aggregate_rating: None,
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

    /// Sets `description`.
    #[must_use]
    pub fn description<D>(mut self, description: D) -> Self
    where
        D: Into<String>,
    {
        self.description = Some(description.into());
        self
    }

    /// Sets `address`.
    #[must_use]
    pub fn address<V>(mut self, address: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.address = Some(address.into());
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

    /// Sets `aggregateRating`.
    #[must_use]
    pub fn aggregate_rating<V>(mut self, aggregate_rating: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.aggregate_rating = Some(aggregate_rating.into());
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

    /// Returns the vacation rental as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("VacationRental")
            .field("name", self.name)
            .field("identifier", self.identifier)
            .field("latitude", self.latitude)
            .field("longitude", self.longitude)
            .field("containsPlace", self.accommodation);

        if !self.images.is_empty() {
            object = object.field("image", values_to_json(self.images));
        }

        if let Some(description) = self.description {
            object = object.field("description", description);
        }

        if let Some(address) = self.address {
            object = object.field("address", address);
        }

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        if let Some(aggregate_rating) = self.aggregate_rating {
            object = object.field("aggregateRating", aggregate_rating);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the vacation rental as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl From<DataCatalog> for JsonLdObject {
    fn from(value: DataCatalog) -> Self {
        value.into_object()
    }
}

impl From<DataCatalog> for JsonValue {
    fn from(value: DataCatalog) -> Self {
        value.into_object().into()
    }
}

impl From<DataDownload> for JsonLdObject {
    fn from(value: DataDownload) -> Self {
        value.into_object()
    }
}

impl From<DataDownload> for JsonValue {
    fn from(value: DataDownload) -> Self {
        value.into_object().into()
    }
}

impl From<Dataset> for JsonLdObject {
    fn from(value: Dataset) -> Self {
        value.into_object()
    }
}

impl From<Dataset> for JsonLdDocument {
    fn from(value: Dataset) -> Self {
        value.into_document()
    }
}

impl From<Dataset> for JsonValue {
    fn from(value: Dataset) -> Self {
        value.into_object().into()
    }
}

impl From<Accommodation> for JsonLdObject {
    fn from(value: Accommodation) -> Self {
        value.into_object()
    }
}

impl From<Accommodation> for JsonValue {
    fn from(value: Accommodation) -> Self {
        value.into_object().into()
    }
}

impl From<VacationRental> for JsonLdObject {
    fn from(value: VacationRental) -> Self {
        value.into_object()
    }
}

impl From<VacationRental> for JsonLdDocument {
    fn from(value: VacationRental) -> Self {
        value.into_document()
    }
}

impl From<VacationRental> for JsonValue {
    fn from(value: VacationRental) -> Self {
        value.into_object().into()
    }
}
