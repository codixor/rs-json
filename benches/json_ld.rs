use criterion::{Criterion, criterion_group, criterion_main};
use hashbrown::HashMap as HashBrownMap;
use indexmap::IndexMap;
use serde::Serialize;
use serde_json::Map as SerdeMap;
use std::hint::black_box;

use rs_json::{
    JsonLdArray, JsonLdDocument, JsonLdObject,
    ld::google::{
        AggregateRating, BroadcastEvent, Clip, LocalBusiness, Offer, PostalAddress, Product,
        VideoObject,
    },
};

#[derive(Serialize)]
struct TypedBrand {
    #[serde(rename = "@type")]
    schema_type: &'static str,
    name: &'static str,
}

#[derive(Serialize)]
struct TypedOffer {
    #[serde(rename = "@type")]
    schema_type: &'static str,
    price: &'static str,
    #[serde(rename = "priceCurrency")]
    price_currency: &'static str,
    availability: &'static str,
}

#[derive(Serialize)]
struct TypedAggregateRating {
    #[serde(rename = "@type")]
    schema_type: &'static str,
    #[serde(rename = "ratingValue")]
    rating_value: &'static str,
    #[serde(rename = "ratingCount")]
    rating_count: u64,
}

#[derive(Serialize)]
struct TypedProductDocument {
    #[serde(rename = "@context")]
    context: &'static str,
    #[serde(rename = "@type")]
    schema_type: &'static str,
    name: &'static str,
    sku: &'static str,
    description: &'static str,
    image: &'static str,
    brand: TypedBrand,
    offers: TypedOffer,
    #[serde(rename = "aggregateRating")]
    aggregate_rating: TypedAggregateRating,
}

#[derive(Serialize)]
struct TypedGeoCoordinates {
    #[serde(rename = "@type")]
    schema_type: &'static str,
    latitude: f64,
    longitude: f64,
}

#[derive(Serialize)]
struct TypedOpeningHoursSpecification {
    #[serde(rename = "@type")]
    schema_type: &'static str,
    #[serde(rename = "dayOfWeek")]
    day_of_week: &'static str,
    opens: &'static str,
    closes: &'static str,
}

#[derive(Serialize)]
struct TypedPostalAddress {
    #[serde(rename = "@type")]
    schema_type: &'static str,
    #[serde(rename = "streetAddress")]
    street_address: &'static str,
    #[serde(rename = "addressLocality")]
    address_locality: &'static str,
    #[serde(rename = "addressRegion")]
    address_region: &'static str,
    #[serde(rename = "postalCode")]
    postal_code: &'static str,
    #[serde(rename = "addressCountry")]
    address_country: &'static str,
}

#[derive(Serialize)]
struct TypedLocalBusinessDocument {
    #[serde(rename = "@context")]
    context: &'static str,
    #[serde(rename = "@type")]
    schema_type: &'static str,
    name: &'static str,
    address: TypedPostalAddress,
    url: &'static str,
    telephone: &'static str,
    image: &'static str,
    #[serde(rename = "priceRange")]
    price_range: &'static str,
    geo: TypedGeoCoordinates,
    #[serde(rename = "openingHoursSpecification")]
    opening_hours_specification: TypedOpeningHoursSpecification,
    #[serde(rename = "sameAs")]
    same_as: &'static str,
    #[serde(rename = "aggregateRating")]
    aggregate_rating: TypedAggregateRating,
    #[serde(rename = "servesCuisine")]
    serves_cuisine: [&'static str; 2],
}

#[derive(Serialize)]
struct TypedClip {
    #[serde(rename = "@type")]
    schema_type: &'static str,
    name: &'static str,
    #[serde(rename = "startOffset")]
    start_offset: u64,
    #[serde(rename = "endOffset")]
    end_offset: u64,
    url: &'static str,
}

#[derive(Serialize)]
struct TypedBroadcastEvent {
    #[serde(rename = "@type")]
    schema_type: &'static str,
    #[serde(rename = "isLiveBroadcast")]
    is_live_broadcast: bool,
    #[serde(rename = "startDate")]
    start_date: &'static str,
    #[serde(rename = "endDate")]
    end_date: &'static str,
}

#[derive(Serialize)]
struct TypedVideoDocument {
    #[serde(rename = "@context")]
    context: &'static str,
    #[serde(rename = "@type")]
    schema_type: &'static str,
    name: &'static str,
    description: &'static str,
    #[serde(rename = "uploadDate")]
    upload_date: &'static str,
    #[serde(rename = "thumbnailUrl")]
    thumbnail_url: &'static str,
    duration: &'static str,
    #[serde(rename = "embedUrl")]
    embed_url: &'static str,
    #[serde(rename = "contentUrl")]
    content_url: &'static str,
    #[serde(rename = "hasPart")]
    has_part: TypedClip,
    publication: TypedBroadcastEvent,
    #[serde(rename = "regionsAllowed")]
    regions_allowed: [&'static str; 2],
    #[serde(rename = "isFamilyFriendly")]
    is_family_friendly: bool,
}

trait ObjectStoreBackend {
    type Map;

    fn with_capacity(capacity: usize) -> Self::Map;
    fn insert(map: &mut Self::Map, key: &'static str, value: rs_json::JsonValue);
    fn into_json_object(map: Self::Map) -> rs_json::JsonValue;
}

struct SerdeMapBackend;
struct IndexMapBackend;
struct HashBrownBackend;
struct HashBrownAHashBackend;

impl ObjectStoreBackend for SerdeMapBackend {
    type Map = SerdeMap<String, rs_json::JsonValue>;

    fn with_capacity(capacity: usize) -> Self::Map {
        SerdeMap::with_capacity(capacity)
    }

    fn insert(map: &mut Self::Map, key: &'static str, value: rs_json::JsonValue) {
        map.insert(String::from(key), value);
    }

    fn into_json_object(map: Self::Map) -> rs_json::JsonValue {
        rs_json::JsonValue::Object(map)
    }
}

impl ObjectStoreBackend for IndexMapBackend {
    type Map = IndexMap<String, rs_json::JsonValue>;

    fn with_capacity(capacity: usize) -> Self::Map {
        IndexMap::with_capacity(capacity)
    }

    fn insert(map: &mut Self::Map, key: &'static str, value: rs_json::JsonValue) {
        map.insert(String::from(key), value);
    }

    fn into_json_object(map: Self::Map) -> rs_json::JsonValue {
        let mut out = SerdeMap::with_capacity(map.len());
        for (key, value) in map {
            out.insert(key, value);
        }
        rs_json::JsonValue::Object(out)
    }
}

impl ObjectStoreBackend for HashBrownBackend {
    type Map = HashBrownMap<String, rs_json::JsonValue>;

    fn with_capacity(capacity: usize) -> Self::Map {
        HashBrownMap::with_capacity(capacity)
    }

    fn insert(map: &mut Self::Map, key: &'static str, value: rs_json::JsonValue) {
        map.insert(String::from(key), value);
    }

    fn into_json_object(map: Self::Map) -> rs_json::JsonValue {
        let mut out = SerdeMap::with_capacity(map.len());
        for (key, value) in map {
            out.insert(key, value);
        }
        rs_json::JsonValue::Object(out)
    }
}

impl ObjectStoreBackend for HashBrownAHashBackend {
    type Map = HashBrownMap<String, rs_json::JsonValue, ahash::RandomState>;

    fn with_capacity(capacity: usize) -> Self::Map {
        HashBrownMap::with_capacity_and_hasher(capacity, ahash::RandomState::new())
    }

    fn insert(map: &mut Self::Map, key: &'static str, value: rs_json::JsonValue) {
        map.insert(String::from(key), value);
    }

    fn into_json_object(map: Self::Map) -> rs_json::JsonValue {
        let mut out = SerdeMap::with_capacity(map.len());
        for (key, value) in map {
            out.insert(key, value);
        }
        rs_json::JsonValue::Object(out)
    }
}

fn build_product_with<B: ObjectStoreBackend>() -> rs_json::JsonValue {
    let mut brand = B::with_capacity(2);
    B::insert(&mut brand, "@type", "Brand".into());
    B::insert(&mut brand, "name", "Example Apparel".into());

    let mut offer = B::with_capacity(5);
    B::insert(&mut offer, "@type", "Offer".into());
    B::insert(&mut offer, "price", "149.00".into());
    B::insert(&mut offer, "priceCurrency", "USD".into());
    B::insert(
        &mut offer,
        "availability",
        "https://schema.org/InStock".into(),
    );
    B::insert(
        &mut offer,
        "url",
        "https://example.com/products/coat-green-s".into(),
    );

    let mut rating = B::with_capacity(3);
    B::insert(&mut rating, "@type", "AggregateRating".into());
    B::insert(&mut rating, "ratingValue", "4.8".into());
    B::insert(&mut rating, "ratingCount", 27.into());

    let mut object = B::with_capacity(8);
    B::insert(&mut object, "@context", "https://schema.org".into());
    B::insert(&mut object, "@type", "Product".into());
    B::insert(
        &mut object,
        "name",
        "Wool winter coat - green, size small".into(),
    );
    B::insert(&mut object, "sku", "COAT-GREEN-S".into());
    B::insert(
        &mut object,
        "description",
        "Soft wool winter coat in green, size small.".into(),
    );
    B::insert(
        &mut object,
        "image",
        "https://example.com/images/coat-green-s.png".into(),
    );
    B::insert(&mut object, "brand", B::into_json_object(brand));
    B::insert(&mut object, "offers", B::into_json_object(offer));
    B::insert(&mut object, "aggregateRating", B::into_json_object(rating));
    B::into_json_object(object)
}

fn build_local_business_with<B: ObjectStoreBackend>() -> rs_json::JsonValue {
    let mut address = B::with_capacity(6);
    B::insert(&mut address, "@type", "PostalAddress".into());
    B::insert(&mut address, "streetAddress", "14 Coffee Lane".into());
    B::insert(&mut address, "addressLocality", "Dubai".into());
    B::insert(&mut address, "addressRegion", "Dubai".into());
    B::insert(&mut address, "postalCode", "00000".into());
    B::insert(&mut address, "addressCountry", "AE".into());

    let mut geo = B::with_capacity(3);
    B::insert(&mut geo, "@type", "GeoCoordinates".into());
    B::insert(&mut geo, "latitude", 25.2048.into());
    B::insert(&mut geo, "longitude", 55.2708.into());

    let mut hours = B::with_capacity(4);
    B::insert(&mut hours, "@type", "OpeningHoursSpecification".into());
    B::insert(&mut hours, "dayOfWeek", "https://schema.org/Monday".into());
    B::insert(&mut hours, "opens", "08:00".into());
    B::insert(&mut hours, "closes", "17:00".into());

    let mut rating = B::with_capacity(3);
    B::insert(&mut rating, "@type", "AggregateRating".into());
    B::insert(&mut rating, "ratingValue", "4.8".into());
    B::insert(&mut rating, "ratingCount", 120.into());

    let mut object = B::with_capacity(12);
    B::insert(&mut object, "@context", "https://schema.org".into());
    B::insert(&mut object, "@type", "CafeOrCoffeeShop".into());
    B::insert(&mut object, "name", "Example Cafe".into());
    B::insert(&mut object, "address", B::into_json_object(address));
    B::insert(&mut object, "url", "https://example.com/cafe".into());
    B::insert(&mut object, "telephone", "+971555000000".into());
    B::insert(&mut object, "image", "https://example.com/cafe.png".into());
    B::insert(&mut object, "priceRange", "$$".into());
    B::insert(&mut object, "geo", B::into_json_object(geo));
    B::insert(
        &mut object,
        "openingHoursSpecification",
        B::into_json_object(hours),
    );
    B::insert(
        &mut object,
        "sameAs",
        "https://www.instagram.com/example-cafe".into(),
    );
    B::insert(&mut object, "aggregateRating", B::into_json_object(rating));
    B::insert(
        &mut object,
        "servesCuisine",
        rs_json::JsonValue::Array(vec!["Coffee".into(), "Pastries".into()]),
    );
    B::into_json_object(object)
}

fn build_media_video_with<B: ObjectStoreBackend>() -> rs_json::JsonValue {
    let mut clip = B::with_capacity(5);
    B::insert(&mut clip, "@type", "Clip".into());
    B::insert(&mut clip, "name", "Builder overview".into());
    B::insert(&mut clip, "startOffset", 0.into());
    B::insert(&mut clip, "endOffset", 120.into());
    B::insert(&mut clip, "url", "https://example.com/video?t=0".into());

    let mut publication = B::with_capacity(4);
    B::insert(&mut publication, "@type", "BroadcastEvent".into());
    B::insert(&mut publication, "isLiveBroadcast", true.into());
    B::insert(&mut publication, "startDate", "2026-04-08T10:00:00Z".into());
    B::insert(&mut publication, "endDate", "2026-04-08T10:09:00Z".into());

    let mut object = B::with_capacity(10);
    B::insert(&mut object, "@context", "https://schema.org".into());
    B::insert(&mut object, "@type", "VideoObject".into());
    B::insert(
        &mut object,
        "name",
        "Rust JSON-LD Builder Walkthrough".into(),
    );
    B::insert(
        &mut object,
        "description",
        "A short walkthrough of the rs-json Google builder families.".into(),
    );
    B::insert(&mut object, "uploadDate", "2026-04-08T10:00:00Z".into());
    B::insert(
        &mut object,
        "thumbnailUrl",
        "https://example.com/video-thumb.png".into(),
    );
    B::insert(&mut object, "duration", "PT8M42S".into());
    B::insert(
        &mut object,
        "embedUrl",
        "https://example.com/embed/video".into(),
    );
    B::insert(
        &mut object,
        "contentUrl",
        "https://cdn.example.com/video.mp4".into(),
    );
    B::insert(&mut object, "hasPart", B::into_json_object(clip));
    B::insert(&mut object, "publication", B::into_json_object(publication));
    B::insert(
        &mut object,
        "regionsAllowed",
        rs_json::JsonValue::Array(vec!["AE".into(), "US".into()]),
    );
    B::insert(&mut object, "isFamilyFriendly", true.into());
    B::into_json_object(object)
}

fn sample_google_typed_document() -> JsonLdDocument {
    Product::new("Wool winter coat - green, size small")
        .sku("COAT-GREEN-S")
        .description("Soft wool winter coat in green, size small.")
        .image("https://example.com/images/coat-green-s.png")
        .brand_name("Example Apparel")
        .offer(
            Offer::new("149.00", "USD")
                .availability("https://schema.org/InStock")
                .url("https://example.com/products/coat-green-s"),
        )
        .field("aggregateRating", AggregateRating::new("4.8", 27))
        .into_document()
}

fn sample_explicit_document() -> JsonLdDocument {
    JsonLdDocument::new()
        .context("https://schema.org")
        .ty("Product")
        .field("name", "Wool winter coat - green, size small")
        .field("sku", "COAT-GREEN-S")
        .field("description", "Soft wool winter coat in green, size small.")
        .field("image", "https://example.com/images/coat-green-s.png")
        .field(
            "brand",
            JsonLdObject::new()
                .ty("Brand")
                .field("name", "Example Apparel"),
        )
        .field(
            "offers",
            JsonLdObject::new()
                .ty("Offer")
                .field("price", "149.00")
                .field("priceCurrency", "USD")
                .field("availability", "https://schema.org/InStock")
                .field("url", "https://example.com/products/coat-green-s"),
        )
        .field(
            "aggregateRating",
            JsonLdObject::new()
                .ty("AggregateRating")
                .field("ratingValue", "4.8")
                .field("ratingCount", 27),
        )
}

fn sample_json_macro_value() -> serde_json::Value {
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Product",
        "name": "Wool winter coat - green, size small",
        "sku": "COAT-GREEN-S",
        "description": "Soft wool winter coat in green, size small.",
        "image": "https://example.com/images/coat-green-s.png",
        "brand": {
            "@type": "Brand",
            "name": "Example Apparel"
        },
        "offers": {
            "@type": "Offer",
            "price": "149.00",
            "priceCurrency": "USD",
            "availability": "https://schema.org/InStock",
            "url": "https://example.com/products/coat-green-s"
        },
        "aggregateRating": {
            "@type": "AggregateRating",
            "ratingValue": "4.8",
            "ratingCount": 27
        }
    })
}

fn sample_typed_document() -> TypedProductDocument {
    TypedProductDocument {
        context: "https://schema.org",
        schema_type: "Product",
        name: "Wool winter coat - green, size small",
        sku: "COAT-GREEN-S",
        description: "Soft wool winter coat in green, size small.",
        image: "https://example.com/images/coat-green-s.png",
        brand: TypedBrand {
            schema_type: "Brand",
            name: "Example Apparel",
        },
        offers: TypedOffer {
            schema_type: "Offer",
            price: "149.00",
            price_currency: "USD",
            availability: "https://schema.org/InStock",
        },
        aggregate_rating: TypedAggregateRating {
            schema_type: "AggregateRating",
            rating_value: "4.8",
            rating_count: 27,
        },
    }
}

fn sample_local_business_typed_document() -> JsonLdDocument {
    LocalBusiness::new(
        "Example Cafe",
        PostalAddress::new("AE")
            .street_address("14 Coffee Lane")
            .locality("Dubai")
            .region("Dubai")
            .postal_code("00000"),
    )
    .schema_type("CafeOrCoffeeShop")
    .url("https://example.com/cafe")
    .telephone("+971555000000")
    .image("https://example.com/cafe.png")
    .price_range("$$")
    .geo_coordinates(25.2048, 55.2708)
    .opening_hours("https://schema.org/Monday", "08:00", "17:00")
    .same_as("https://www.instagram.com/example-cafe")
    .aggregate_rating(AggregateRating::new("4.8", 120))
    .serves_cuisine("Coffee")
    .serves_cuisine("Pastries")
    .into_document()
}

fn sample_local_business_explicit_document() -> JsonLdDocument {
    JsonLdDocument::new()
        .context("https://schema.org")
        .ty("CafeOrCoffeeShop")
        .field("name", "Example Cafe")
        .field(
            "address",
            JsonLdObject::new()
                .ty("PostalAddress")
                .field("streetAddress", "14 Coffee Lane")
                .field("addressLocality", "Dubai")
                .field("addressRegion", "Dubai")
                .field("postalCode", "00000")
                .field("addressCountry", "AE"),
        )
        .field("url", "https://example.com/cafe")
        .field("telephone", "+971555000000")
        .field("image", "https://example.com/cafe.png")
        .field("priceRange", "$$")
        .field(
            "geo",
            JsonLdObject::new()
                .ty("GeoCoordinates")
                .field("latitude", 25.2048)
                .field("longitude", 55.2708),
        )
        .field(
            "openingHoursSpecification",
            JsonLdObject::new()
                .ty("OpeningHoursSpecification")
                .field("dayOfWeek", "https://schema.org/Monday")
                .field("opens", "08:00")
                .field("closes", "17:00"),
        )
        .field("sameAs", "https://www.instagram.com/example-cafe")
        .field(
            "aggregateRating",
            JsonLdObject::new()
                .ty("AggregateRating")
                .field("ratingValue", "4.8")
                .field("ratingCount", 120),
        )
        .field(
            "servesCuisine",
            JsonLdArray::new().item("Coffee").item("Pastries"),
        )
}

fn sample_local_business_json_macro_value() -> serde_json::Value {
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "CafeOrCoffeeShop",
        "name": "Example Cafe",
        "address": {
            "@type": "PostalAddress",
            "streetAddress": "14 Coffee Lane",
            "addressLocality": "Dubai",
            "addressRegion": "Dubai",
            "postalCode": "00000",
            "addressCountry": "AE"
        },
        "url": "https://example.com/cafe",
        "telephone": "+971555000000",
        "image": "https://example.com/cafe.png",
        "priceRange": "$$",
        "geo": {
            "@type": "GeoCoordinates",
            "latitude": 25.2048,
            "longitude": 55.2708
        },
        "openingHoursSpecification": {
            "@type": "OpeningHoursSpecification",
            "dayOfWeek": "https://schema.org/Monday",
            "opens": "08:00",
            "closes": "17:00"
        },
        "sameAs": "https://www.instagram.com/example-cafe",
        "aggregateRating": {
            "@type": "AggregateRating",
            "ratingValue": "4.8",
            "ratingCount": 120
        },
        "servesCuisine": ["Coffee", "Pastries"]
    })
}

fn sample_local_business_serde_document() -> TypedLocalBusinessDocument {
    TypedLocalBusinessDocument {
        context: "https://schema.org",
        schema_type: "CafeOrCoffeeShop",
        name: "Example Cafe",
        address: TypedPostalAddress {
            schema_type: "PostalAddress",
            street_address: "14 Coffee Lane",
            address_locality: "Dubai",
            address_region: "Dubai",
            postal_code: "00000",
            address_country: "AE",
        },
        url: "https://example.com/cafe",
        telephone: "+971555000000",
        image: "https://example.com/cafe.png",
        price_range: "$$",
        geo: TypedGeoCoordinates {
            schema_type: "GeoCoordinates",
            latitude: 25.2048,
            longitude: 55.2708,
        },
        opening_hours_specification: TypedOpeningHoursSpecification {
            schema_type: "OpeningHoursSpecification",
            day_of_week: "https://schema.org/Monday",
            opens: "08:00",
            closes: "17:00",
        },
        same_as: "https://www.instagram.com/example-cafe",
        aggregate_rating: TypedAggregateRating {
            schema_type: "AggregateRating",
            rating_value: "4.8",
            rating_count: 120,
        },
        serves_cuisine: ["Coffee", "Pastries"],
    }
}

fn sample_video_typed_document() -> JsonLdDocument {
    VideoObject::new(
        "Rust JSON-LD Builder Walkthrough",
        "A short walkthrough of the rs-json Google builder families.",
        "2026-04-08T10:00:00Z",
        "https://example.com/video-thumb.png",
    )
    .duration("PT8M42S")
    .embed_url("https://example.com/embed/video")
    .content_url("https://cdn.example.com/video.mp4")
    .clip(Clip::new(
        "Builder overview",
        0,
        120,
        "https://example.com/video?t=0",
    ))
    .broadcast_event(BroadcastEvent::new("2026-04-08T10:00:00Z").end_date("2026-04-08T10:09:00Z"))
    .region_allowed("AE")
    .region_allowed("US")
    .family_friendly(true)
    .into_document()
}

fn sample_video_explicit_document() -> JsonLdDocument {
    JsonLdDocument::new()
        .context("https://schema.org")
        .ty("VideoObject")
        .field("name", "Rust JSON-LD Builder Walkthrough")
        .field(
            "description",
            "A short walkthrough of the rs-json Google builder families.",
        )
        .field("uploadDate", "2026-04-08T10:00:00Z")
        .field("thumbnailUrl", "https://example.com/video-thumb.png")
        .field("duration", "PT8M42S")
        .field("embedUrl", "https://example.com/embed/video")
        .field("contentUrl", "https://cdn.example.com/video.mp4")
        .field(
            "hasPart",
            JsonLdObject::new()
                .ty("Clip")
                .field("name", "Builder overview")
                .field("startOffset", 0)
                .field("endOffset", 120)
                .field("url", "https://example.com/video?t=0"),
        )
        .field(
            "publication",
            JsonLdObject::new()
                .ty("BroadcastEvent")
                .field("isLiveBroadcast", true)
                .field("startDate", "2026-04-08T10:00:00Z")
                .field("endDate", "2026-04-08T10:09:00Z"),
        )
        .field("regionsAllowed", JsonLdArray::new().item("AE").item("US"))
        .field("isFamilyFriendly", true)
}

fn sample_video_json_macro_value() -> serde_json::Value {
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "VideoObject",
        "name": "Rust JSON-LD Builder Walkthrough",
        "description": "A short walkthrough of the rs-json Google builder families.",
        "uploadDate": "2026-04-08T10:00:00Z",
        "thumbnailUrl": "https://example.com/video-thumb.png",
        "duration": "PT8M42S",
        "embedUrl": "https://example.com/embed/video",
        "contentUrl": "https://cdn.example.com/video.mp4",
        "hasPart": {
            "@type": "Clip",
            "name": "Builder overview",
            "startOffset": 0,
            "endOffset": 120,
            "url": "https://example.com/video?t=0"
        },
        "publication": {
            "@type": "BroadcastEvent",
            "isLiveBroadcast": true,
            "startDate": "2026-04-08T10:00:00Z",
            "endDate": "2026-04-08T10:09:00Z"
        },
        "regionsAllowed": ["AE", "US"],
        "isFamilyFriendly": true
    })
}

fn sample_video_serde_document() -> TypedVideoDocument {
    TypedVideoDocument {
        context: "https://schema.org",
        schema_type: "VideoObject",
        name: "Rust JSON-LD Builder Walkthrough",
        description: "A short walkthrough of the rs-json Google builder families.",
        upload_date: "2026-04-08T10:00:00Z",
        thumbnail_url: "https://example.com/video-thumb.png",
        duration: "PT8M42S",
        embed_url: "https://example.com/embed/video",
        content_url: "https://cdn.example.com/video.mp4",
        has_part: TypedClip {
            schema_type: "Clip",
            name: "Builder overview",
            start_offset: 0,
            end_offset: 120,
            url: "https://example.com/video?t=0",
        },
        publication: TypedBroadcastEvent {
            schema_type: "BroadcastEvent",
            is_live_broadcast: true,
            start_date: "2026-04-08T10:00:00Z",
            end_date: "2026-04-08T10:09:00Z",
        },
        regions_allowed: ["AE", "US"],
        is_family_friendly: true,
    }
}

fn bench_json_ld_to_value(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/ld_to_value");

    group.bench_function("google_typed", |b| {
        b.iter(|| black_box(sample_google_typed_document().to_value()));
    });
    group.bench_function("explicit_builder", |b| {
        b.iter(|| black_box(sample_explicit_document().to_value()));
    });
    group.bench_function("json_macro", |b| {
        b.iter(|| black_box(sample_json_macro_value()));
    });
    group.bench_function("typed_serialize", |b| {
        b.iter(|| black_box(serde_json::to_value(sample_typed_document()).unwrap()));
    });

    group.finish();
}

fn bench_json_ld_to_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("json/ld_to_string");

    group.bench_function("google_typed", |b| {
        b.iter(|| black_box(sample_google_typed_document().to_json_string().unwrap()));
    });
    group.bench_function("explicit_builder", |b| {
        b.iter(|| black_box(sample_explicit_document().to_json_string().unwrap()));
    });
    group.bench_function("json_macro", |b| {
        b.iter(|| black_box(serde_json::to_string(&sample_json_macro_value()).unwrap()));
    });
    group.bench_function("typed_serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(&sample_typed_document()).unwrap()));
    });

    group.finish();
}

fn bench_json_ld_local_business(c: &mut Criterion) {
    let mut to_value = c.benchmark_group("json/ld_family/local_business/to_value");

    to_value.bench_function("google_typed", |b| {
        b.iter(|| black_box(sample_local_business_typed_document().to_value()));
    });
    to_value.bench_function("explicit_builder", |b| {
        b.iter(|| black_box(sample_local_business_explicit_document().to_value()));
    });
    to_value.bench_function("json_macro", |b| {
        b.iter(|| black_box(sample_local_business_json_macro_value()));
    });
    to_value.bench_function("typed_serialize", |b| {
        b.iter(|| black_box(serde_json::to_value(sample_local_business_serde_document()).unwrap()));
    });
    to_value.finish();

    let mut to_string = c.benchmark_group("json/ld_family/local_business/to_string");

    to_string.bench_function("google_typed", |b| {
        b.iter(|| {
            black_box(
                sample_local_business_typed_document()
                    .to_json_string()
                    .unwrap(),
            )
        });
    });
    to_string.bench_function("explicit_builder", |b| {
        b.iter(|| {
            black_box(
                sample_local_business_explicit_document()
                    .to_json_string()
                    .unwrap(),
            )
        });
    });
    to_string.bench_function("json_macro", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&sample_local_business_json_macro_value()).unwrap())
        });
    });
    to_string.bench_function("typed_serialize", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&sample_local_business_serde_document()).unwrap())
        });
    });
    to_string.finish();
}

fn bench_json_ld_media_video(c: &mut Criterion) {
    let mut to_value = c.benchmark_group("json/ld_family/media_video/to_value");

    to_value.bench_function("google_typed", |b| {
        b.iter(|| black_box(sample_video_typed_document().to_value()));
    });
    to_value.bench_function("explicit_builder", |b| {
        b.iter(|| black_box(sample_video_explicit_document().to_value()));
    });
    to_value.bench_function("json_macro", |b| {
        b.iter(|| black_box(sample_video_json_macro_value()));
    });
    to_value.bench_function("typed_serialize", |b| {
        b.iter(|| black_box(serde_json::to_value(sample_video_serde_document()).unwrap()));
    });
    to_value.finish();

    let mut to_string = c.benchmark_group("json/ld_family/media_video/to_string");

    to_string.bench_function("google_typed", |b| {
        b.iter(|| black_box(sample_video_typed_document().to_json_string().unwrap()));
    });
    to_string.bench_function("explicit_builder", |b| {
        b.iter(|| black_box(sample_video_explicit_document().to_json_string().unwrap()));
    });
    to_string.bench_function("json_macro", |b| {
        b.iter(|| black_box(serde_json::to_string(&sample_video_json_macro_value()).unwrap()));
    });
    to_string.bench_function("typed_serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(&sample_video_serde_document()).unwrap()));
    });
    to_string.finish();
}

fn bench_json_ld_object_store_backends(c: &mut Criterion) {
    let mut product = c.benchmark_group("json/object_store/product_to_value");
    product.bench_function("serde_map", |b| {
        b.iter(|| black_box(build_product_with::<SerdeMapBackend>()));
    });
    product.bench_function("indexmap", |b| {
        b.iter(|| black_box(build_product_with::<IndexMapBackend>()));
    });
    product.bench_function("hashbrown", |b| {
        b.iter(|| black_box(build_product_with::<HashBrownBackend>()));
    });
    product.bench_function("hashbrown_ahash", |b| {
        b.iter(|| black_box(build_product_with::<HashBrownAHashBackend>()));
    });
    product.finish();

    let mut local_business = c.benchmark_group("json/object_store/local_business_to_value");
    local_business.bench_function("serde_map", |b| {
        b.iter(|| black_box(build_local_business_with::<SerdeMapBackend>()));
    });
    local_business.bench_function("indexmap", |b| {
        b.iter(|| black_box(build_local_business_with::<IndexMapBackend>()));
    });
    local_business.bench_function("hashbrown", |b| {
        b.iter(|| black_box(build_local_business_with::<HashBrownBackend>()));
    });
    local_business.bench_function("hashbrown_ahash", |b| {
        b.iter(|| black_box(build_local_business_with::<HashBrownAHashBackend>()));
    });
    local_business.finish();

    let mut media_video = c.benchmark_group("json/object_store/media_video_to_value");
    media_video.bench_function("serde_map", |b| {
        b.iter(|| black_box(build_media_video_with::<SerdeMapBackend>()));
    });
    media_video.bench_function("indexmap", |b| {
        b.iter(|| black_box(build_media_video_with::<IndexMapBackend>()));
    });
    media_video.bench_function("hashbrown", |b| {
        b.iter(|| black_box(build_media_video_with::<HashBrownBackend>()));
    });
    media_video.bench_function("hashbrown_ahash", |b| {
        b.iter(|| black_box(build_media_video_with::<HashBrownAHashBackend>()));
    });
    media_video.finish();
}

criterion_group!(
    benches,
    bench_json_ld_to_value,
    bench_json_ld_to_string,
    bench_json_ld_local_business,
    bench_json_ld_media_video,
    bench_json_ld_object_store_backends
);
criterion_main!(benches);
