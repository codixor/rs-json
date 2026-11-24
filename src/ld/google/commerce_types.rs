use crate::{JsonLdArray, JsonLdDocument, JsonLdObject, JsonValue};

use super::SCHEMA_ORG_CONTEXT;

/// A Google-friendly `MerchantReturnPolicy` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct MerchantReturnPolicy {
    return_policy_category: String,
    merchant_return_days: Option<JsonValue>,
    applicable_country: Vec<String>,
    return_fees: Option<String>,
    return_method: Option<String>,
    refund_type: Option<String>,
    return_shipping_fees_amount: Option<JsonValue>,
    extra: JsonLdObject,
}

/// A Google-friendly `OfferShippingDetails` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct OfferShippingDetails {
    shipping_destinations: Vec<String>,
    shipping_rate: JsonValue,
    delivery_time: Option<JsonValue>,
    shipping_label: Option<String>,
    extra: JsonLdObject,
}

/// A Google-friendly `AggregateOffer` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct AggregateOffer {
    low_price: JsonValue,
    high_price: JsonValue,
    price_currency: String,
    offer_count: Option<JsonValue>,
    offers: Vec<JsonValue>,
    availability: Option<String>,
    extra: JsonLdObject,
}

impl MerchantReturnPolicy {
    /// Creates a `MerchantReturnPolicy` builder with its required category.
    #[must_use]
    pub fn new<C>(return_policy_category: C) -> Self
    where
        C: Into<String>,
    {
        Self {
            return_policy_category: return_policy_category.into(),
            merchant_return_days: None,
            applicable_country: Vec::new(),
            return_fees: None,
            return_method: None,
            refund_type: None,
            return_shipping_fees_amount: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Sets `merchantReturnDays`.
    #[must_use]
    pub fn merchant_return_days<V>(mut self, merchant_return_days: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.merchant_return_days = Some(merchant_return_days.into());
        self
    }

    /// Adds one `applicableCountry` code.
    #[must_use]
    pub fn applicable_country<C>(mut self, country_code: C) -> Self
    where
        C: Into<String>,
    {
        self.applicable_country.push(country_code.into());
        self
    }

    /// Sets `returnFees`.
    #[must_use]
    pub fn return_fees<F>(mut self, return_fees: F) -> Self
    where
        F: Into<String>,
    {
        self.return_fees = Some(return_fees.into());
        self
    }

    /// Sets `returnMethod`.
    #[must_use]
    pub fn return_method<M>(mut self, return_method: M) -> Self
    where
        M: Into<String>,
    {
        self.return_method = Some(return_method.into());
        self
    }

    /// Sets `refundType`.
    #[must_use]
    pub fn refund_type<R>(mut self, refund_type: R) -> Self
    where
        R: Into<String>,
    {
        self.refund_type = Some(refund_type.into());
        self
    }

    /// Sets `returnShippingFeesAmount`.
    #[must_use]
    pub fn return_shipping_fees_amount<V>(mut self, amount: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.return_shipping_fees_amount = Some(amount.into());
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

    /// Returns the merchant return policy as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("MerchantReturnPolicy")
            .field("returnPolicyCategory", self.return_policy_category);

        if let Some(merchant_return_days) = self.merchant_return_days {
            object = object.field("merchantReturnDays", merchant_return_days);
        }

        if !self.applicable_country.is_empty() {
            object = object.field(
                "applicableCountry",
                strings_to_json(self.applicable_country),
            );
        }

        if let Some(return_fees) = self.return_fees {
            object = object.field("returnFees", return_fees);
        }

        if let Some(return_method) = self.return_method {
            object = object.field("returnMethod", return_method);
        }

        if let Some(refund_type) = self.refund_type {
            object = object.field("refundType", refund_type);
        }

        if let Some(return_shipping_fees_amount) = self.return_shipping_fees_amount {
            object = object.field("returnShippingFeesAmount", return_shipping_fees_amount);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl OfferShippingDetails {
    /// Creates shipping details with one destination country and a shipping rate.
    #[must_use]
    pub fn new<C, R>(destination_country: C, shipping_rate: R) -> Self
    where
        C: Into<String>,
        R: Into<JsonValue>,
    {
        Self {
            shipping_destinations: vec![destination_country.into()],
            shipping_rate: shipping_rate.into(),
            delivery_time: None,
            shipping_label: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Adds another destination country.
    #[must_use]
    pub fn destination_country<C>(mut self, destination_country: C) -> Self
    where
        C: Into<String>,
    {
        self.shipping_destinations.push(destination_country.into());
        self
    }

    /// Sets `deliveryTime`.
    #[must_use]
    pub fn delivery_time<V>(mut self, delivery_time: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.delivery_time = Some(delivery_time.into());
        self
    }

    /// Sets `shippingLabel`.
    #[must_use]
    pub fn shipping_label<L>(mut self, shipping_label: L) -> Self
    where
        L: Into<String>,
    {
        self.shipping_label = Some(shipping_label.into());
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

    /// Returns the shipping details as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let destination = if let [country] = self.shipping_destinations.as_slice() {
            JsonValue::from(
                JsonLdObject::new()
                    .ty("DefinedRegion")
                    .field("addressCountry", country.clone()),
            )
        } else {
            let mut array = JsonLdArray::new();
            for country in self.shipping_destinations {
                array = array.item(
                    JsonLdObject::new()
                        .ty("DefinedRegion")
                        .field("addressCountry", country),
                );
            }
            array.into()
        };

        let mut object = JsonLdObject::new()
            .ty("OfferShippingDetails")
            .field("shippingDestination", destination)
            .field("shippingRate", self.shipping_rate);

        if let Some(delivery_time) = self.delivery_time {
            object = object.field("deliveryTime", delivery_time);
        }

        if let Some(shipping_label) = self.shipping_label {
            object = object.field("shippingLabel", shipping_label);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl AggregateOffer {
    /// Creates an `AggregateOffer` builder with its core price fields.
    #[must_use]
    pub fn new<L, H, C>(low_price: L, high_price: H, price_currency: C) -> Self
    where
        L: Into<JsonValue>,
        H: Into<JsonValue>,
        C: Into<String>,
    {
        Self {
            low_price: low_price.into(),
            high_price: high_price.into(),
            price_currency: price_currency.into(),
            offer_count: None,
            offers: Vec::new(),
            availability: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Sets `offerCount`.
    #[must_use]
    pub fn offer_count<V>(mut self, offer_count: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.offer_count = Some(offer_count.into());
        self
    }

    /// Adds one nested offer.
    #[must_use]
    pub fn offer<V>(mut self, offer: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.offers.push(offer.into());
        self
    }

    /// Sets `availability`.
    #[must_use]
    pub fn availability<A>(mut self, availability: A) -> Self
    where
        A: Into<String>,
    {
        self.availability = Some(availability.into());
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

    /// Returns the aggregate offer as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("AggregateOffer")
            .field("lowPrice", self.low_price)
            .field("highPrice", self.high_price)
            .field("priceCurrency", self.price_currency);

        if let Some(offer_count) = self.offer_count {
            object = object.field("offerCount", offer_count);
        }

        if !self.offers.is_empty() {
            object = object.field("offers", values_to_json(self.offers));
        }

        if let Some(availability) = self.availability {
            object = object.field("availability", availability);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the aggregate offer as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl From<MerchantReturnPolicy> for JsonLdObject {
    fn from(value: MerchantReturnPolicy) -> Self {
        value.into_object()
    }
}

impl From<MerchantReturnPolicy> for JsonValue {
    fn from(value: MerchantReturnPolicy) -> Self {
        value.into_object().into()
    }
}

impl From<OfferShippingDetails> for JsonLdObject {
    fn from(value: OfferShippingDetails) -> Self {
        value.into_object()
    }
}

impl From<OfferShippingDetails> for JsonValue {
    fn from(value: OfferShippingDetails) -> Self {
        value.into_object().into()
    }
}

impl From<AggregateOffer> for JsonLdObject {
    fn from(value: AggregateOffer) -> Self {
        value.into_object()
    }
}

impl From<AggregateOffer> for JsonLdDocument {
    fn from(value: AggregateOffer) -> Self {
        value.into_document()
    }
}

impl From<AggregateOffer> for JsonValue {
    fn from(value: AggregateOffer) -> Self {
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
