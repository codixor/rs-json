//! JSON-LD.
use criterion::{Criterion, criterion_group, criterion_main};
use serde::Serialize;
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

criterion_group!(
    benches,
    bench_json_ld_to_value,
    bench_json_ld_to_string,
    bench_json_ld_local_business,
    bench_json_ld_media_video,
);
criterion_main!(benches);
