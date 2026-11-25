use crate::{JsonLdArray, JsonLdDocument, JsonLdObject, JsonValue};

use super::{AggregateOffer, SCHEMA_ORG_CONTEXT, schema_property_url};

/// A Google-friendly `Offer` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Offer {
    price: JsonValue,
    price_currency: String,
    availability: Option<String>,
    url: Option<String>,
    item_condition: Option<String>,
    shipping_details: Option<JsonValue>,
    merchant_return_policy: Option<JsonValue>,
    extra: JsonLdObject,
}

/// A Google-friendly `Product` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    name: String,
    url: Option<String>,
    sku: Option<String>,
    description: Option<String>,
    images: Vec<JsonValue>,
    brand_name: Option<String>,
    in_product_group_with_id: Option<String>,
    offers: Vec<JsonValue>,
    extra: JsonLdObject,
}

/// A Google-friendly `ProductGroup` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductGroup {
    name: String,
    group_id: String,
    url: Option<String>,
    description: Option<String>,
    brand_name: Option<String>,
    varies_by: Vec<String>,
    variants: Vec<Product>,
    extra: JsonLdObject,
}

impl Offer {
    /// Creates an `Offer` builder with its core price fields.
    #[must_use]
    pub fn new<P, C>(price: P, price_currency: C) -> Self
    where
        P: Into<JsonValue>,
        C: Into<String>,
    {
        Self {
            price: price.into(),
            price_currency: price_currency.into(),
            availability: None,
            url: None,
            item_condition: None,
            shipping_details: None,
            merchant_return_policy: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Sets `availability`.
    #[must_use]
    pub fn availability<U>(mut self, availability: U) -> Self
    where
        U: Into<String>,
    {
        self.availability = Some(availability.into());
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

    /// Sets `itemCondition`.
    #[must_use]
    pub fn item_condition<U>(mut self, item_condition: U) -> Self
    where
        U: Into<String>,
    {
        self.item_condition = Some(item_condition.into());
        self
    }

    /// Sets `shippingDetails` from a raw JSON-LD value.
    #[must_use]
    pub fn shipping_details<V>(mut self, shipping_details: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.shipping_details = Some(shipping_details.into());
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

    /// Returns the offer as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("Offer")
            .field("price", self.price)
            .field("priceCurrency", self.price_currency);

        if let Some(availability) = self.availability {
            object = object.field("availability", availability);
        }

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        if let Some(item_condition) = self.item_condition {
            object = object.field("itemCondition", item_condition);
        }

        if let Some(shipping_details) = self.shipping_details {
            object = object.field("shippingDetails", shipping_details);
        }

        if let Some(policy) = self.merchant_return_policy {
            object = object.field("hasMerchantReturnPolicy", policy);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl Product {
    /// Creates a `Product` builder with its required `name`.
    #[must_use]
    pub fn new<N>(name: N) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            url: None,
            sku: None,
            description: None,
            images: Vec::new(),
            brand_name: None,
            in_product_group_with_id: None,
            offers: Vec::new(),
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

    /// Sets `sku`.
    #[must_use]
    pub fn sku<S>(mut self, sku: S) -> Self
    where
        S: Into<String>,
    {
        self.sku = Some(sku.into());
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

    /// Adds one `image` entry.
    #[must_use]
    pub fn image<V>(mut self, image: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.images.push(image.into());
        self
    }

    /// Sets `brand` from a simple brand name.
    #[must_use]
    pub fn brand_name<B>(mut self, brand_name: B) -> Self
    where
        B: Into<String>,
    {
        self.brand_name = Some(brand_name.into());
        self
    }

    /// Sets `inProductGroupWithID`.
    #[must_use]
    pub fn in_product_group_with_id<I>(mut self, product_group_id: I) -> Self
    where
        I: Into<String>,
    {
        self.in_product_group_with_id = Some(product_group_id.into());
        self
    }

    /// Adds one `offers` entry.
    #[must_use]
    pub fn offer<V>(mut self, offer: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.offers.push(offer.into());
        self
    }

    /// Adds an `AggregateOffer` as the `offers` payload.
    #[must_use]
    pub fn aggregate_offer(self, aggregate_offer: AggregateOffer) -> Self {
        self.offer(aggregate_offer)
    }

    /// Adds one `review`.
    #[must_use]
    pub fn review<V>(mut self, review: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.extra = self.extra.field("review", review);
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

    /// Returns the product as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new().ty("Product").field("name", self.name);

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        if let Some(sku) = self.sku {
            object = object.field("sku", sku);
        }

        if let Some(description) = self.description {
            object = object.field("description", description);
        }

        if !self.images.is_empty() {
            object = object.field("image", values_to_json(self.images));
        }

        if let Some(brand_name) = self.brand_name {
            object = object.field(
                "brand",
                JsonLdObject::new().ty("Brand").field("name", brand_name),
            );
        }

        if let Some(product_group_id) = self.in_product_group_with_id {
            object = object.field("inProductGroupWithID", product_group_id);
        }

        if !self.offers.is_empty() {
            object = object.field("offers", values_to_json_iter(self.offers));
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the product as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }

    fn with_product_group_id(mut self, group_id: String) -> Self {
        if self.in_product_group_with_id.is_none() {
            self.in_product_group_with_id = Some(group_id);
        }
        self
    }
}

impl ProductGroup {
    /// Creates a `ProductGroup` builder with `name` and `productGroupID`.
    #[must_use]
    pub fn new<N, I>(name: N, product_group_id: I) -> Self
    where
        N: Into<String>,
        I: Into<String>,
    {
        Self {
            name: name.into(),
            group_id: product_group_id.into(),
            url: None,
            description: None,
            brand_name: None,
            varies_by: Vec::new(),
            variants: Vec::new(),
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

    /// Sets `description`.
    #[must_use]
    pub fn description<D>(mut self, description: D) -> Self
    where
        D: Into<String>,
    {
        self.description = Some(description.into());
        self
    }

    /// Sets `brand` from a simple brand name.
    #[must_use]
    pub fn brand_name<B>(mut self, brand_name: B) -> Self
    where
        B: Into<String>,
    {
        self.brand_name = Some(brand_name.into());
        self
    }

    /// Adds one schema.org property URL to `variesBy`.
    #[must_use]
    pub fn varies_by_schema_property<P>(mut self, property_name: P) -> Self
    where
        P: AsRef<str>,
    {
        self.varies_by
            .push(schema_property_url(property_name.as_ref()));
        self
    }

    /// Adds one raw URL or identifier to `variesBy`.
    #[must_use]
    pub fn varies_by<U>(mut self, property_url: U) -> Self
    where
        U: Into<String>,
    {
        self.varies_by.push(property_url.into());
        self
    }

    /// Adds one variant product to `hasVariant`.
    #[must_use]
    pub fn variant(mut self, product: Product) -> Self {
        self.variants.push(product);
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

    /// Returns the product group as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("ProductGroup")
            .field("name", self.name)
            .field("productGroupID", self.group_id.clone());

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        if let Some(description) = self.description {
            object = object.field("description", description);
        }

        if let Some(brand_name) = self.brand_name {
            object = object.field(
                "brand",
                JsonLdObject::new().ty("Brand").field("name", brand_name),
            );
        }

        if !self.varies_by.is_empty() {
            let values = self.varies_by.into_iter().map(JsonValue::from);
            object = object.field("variesBy", values_to_array_json(values));
        }

        if !self.variants.is_empty() {
            let variants = self
                .variants
                .into_iter()
                .map(|variant| variant.with_product_group_id(self.group_id.clone()))
                .map(JsonLdObject::from)
                .map(JsonValue::from);
            object = object.field("hasVariant", values_to_array_json(variants));
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the product group as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl From<Offer> for JsonLdObject {
    fn from(value: Offer) -> Self {
        value.into_object()
    }
}

impl From<Offer> for JsonValue {
    fn from(value: Offer) -> Self {
        value.into_object().into()
    }
}

impl From<Product> for JsonLdObject {
    fn from(value: Product) -> Self {
        value.into_object()
    }
}

impl From<Product> for JsonLdDocument {
    fn from(value: Product) -> Self {
        value.into_document()
    }
}

impl From<Product> for JsonValue {
    fn from(value: Product) -> Self {
        value.into_object().into()
    }
}

impl From<ProductGroup> for JsonLdObject {
    fn from(value: ProductGroup) -> Self {
        value.into_object()
    }
}

impl From<ProductGroup> for JsonLdDocument {
    fn from(value: ProductGroup) -> Self {
        value.into_document()
    }
}

impl From<ProductGroup> for JsonValue {
    fn from(value: ProductGroup) -> Self {
        value.into_object().into()
    }
}

fn values_to_json(values: Vec<JsonValue>) -> JsonValue {
    values_to_json_iter(values)
}

fn values_to_array_json<I>(values: I) -> JsonValue
where
    I: IntoIterator<Item = JsonValue>,
{
    let mut array = JsonLdArray::new();
    for value in values {
        array = array.item(value);
    }
    array.into()
}

fn values_to_json_iter<I>(values: I) -> JsonValue
where
    I: IntoIterator<Item = JsonValue>,
{
    let mut iter = values.into_iter();
    match (iter.next(), iter.next()) {
        (Some(first), None) => first,
        (Some(first), Some(second)) => {
            let mut array = JsonLdArray::new().item(first).item(second);
            for value in iter {
                array = array.item(value);
            }
            array.into()
        }
        (None, _) => JsonValue::Array(Vec::new()),
    }
}
