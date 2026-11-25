//! JSON-LD google.
#![allow(
    clippy::indexing_slicing,
    reason = "these assertions exercise public JSON indexing on fixed test fixtures"
)]
use rs_json::{
    JsonLdArray, JsonLdDocument, JsonLdObject, JsonValue,
    ld::google::{
        BreadcrumbList, BreadcrumbListItem, MerchantReturnPolicy, Offer, OfferShippingDetails,
        Organization, Product, ProductGroup,
    },
};

#[test]
fn breadcrumb_list_builder_renders_google_shape() {
    let document = BreadcrumbList::new()
        .item(BreadcrumbListItem::new(1, "Books").item("https://example.com/books"))
        .item(BreadcrumbListItem::new(2, "Science Fiction"))
        .into_document();

    let value = document.to_value();

    assert_eq!(
        value["@context"],
        JsonValue::String(String::from("https://schema.org"))
    );
    assert_eq!(
        value["@type"],
        JsonValue::String(String::from("BreadcrumbList"))
    );
    assert_eq!(value["itemListElement"][0]["position"], JsonValue::from(1));
    assert_eq!(
        value["itemListElement"][0]["name"],
        JsonValue::String(String::from("Books"))
    );
    assert_eq!(
        value["itemListElement"][0]["item"],
        JsonValue::String(String::from("https://example.com/books"))
    );
    assert_eq!(
        value["itemListElement"][1]["@type"],
        JsonValue::String(String::from("ListItem"))
    );
}

#[test]
fn organization_builder_supports_same_as_and_return_policy() {
    let policy = MerchantReturnPolicy::new("https://schema.org/MerchantReturnFiniteReturnWindow")
        .merchant_return_days(30)
        .applicable_country("US");
    let document = Organization::new("Example Online Store")
        .schema_type("OnlineStore")
        .url("https://example.com")
        .logo_url("https://example.com/logo.png")
        .same_as("https://www.instagram.com/example")
        .same_as("https://www.youtube.com/@example")
        .merchant_return_policy(policy)
        .into_document();

    let value = document.to_value();

    assert_eq!(
        value["@type"],
        JsonValue::String(String::from("OnlineStore"))
    );
    assert_eq!(
        value["sameAs"][0],
        JsonValue::String(String::from("https://www.instagram.com/example"))
    );
    assert_eq!(
        value["hasMerchantReturnPolicy"]["@type"],
        JsonValue::String(String::from("MerchantReturnPolicy"))
    );
}

#[test]
fn product_group_builder_supports_variants_and_offer_overrides() {
    let shipping = OfferShippingDetails::new(
        "US",
        JsonLdObject::new()
            .ty("MonetaryAmount")
            .field("value", "5.99")
            .field("currency", "USD"),
    );
    let policy = MerchantReturnPolicy::new("https://schema.org/MerchantReturnFiniteReturnWindow")
        .merchant_return_days(30);

    let variant = Product::new("Wool winter coat - green, size small")
        .sku("COAT-GREEN-S")
        .image("https://example.com/coat-green-s.png")
        .offer(
            Offer::new("149.00", "USD")
                .availability("https://schema.org/InStock")
                .url("https://example.com/products/coat-green-s")
                .shipping_details(shipping)
                .merchant_return_policy(policy),
        );

    let document = ProductGroup::new("Wool winter coat", "coat-001")
        .description("Wool winter coat for cold weather climates")
        .brand_name("Example Apparel")
        .varies_by_schema_property("color")
        .varies_by_schema_property("size")
        .variant(variant)
        .into_document();

    let value = document.to_value();

    assert_eq!(
        value["@type"],
        JsonValue::String(String::from("ProductGroup"))
    );
    assert_eq!(
        value["variesBy"][0],
        JsonValue::String(String::from("https://schema.org/color"))
    );
    assert_eq!(
        value["variesBy"][1],
        JsonValue::String(String::from("https://schema.org/size"))
    );
    assert_eq!(
        value["hasVariant"][0]["inProductGroupWithID"],
        JsonValue::String(String::from("coat-001"))
    );
    assert_eq!(
        value["hasVariant"][0]["offers"]["shippingDetails"]["@type"],
        JsonValue::String(String::from("OfferShippingDetails"))
    );
    assert_eq!(
        value["hasVariant"][0]["offers"]["hasMerchantReturnPolicy"]["@type"],
        JsonValue::String(String::from("MerchantReturnPolicy"))
    );
}

#[test]
fn google_builders_can_be_composed_into_top_level_graphs() {
    let organization = Organization::new("Example Corporation").url("https://example.com");
    let breadcrumb = BreadcrumbList::new()
        .item(BreadcrumbListItem::new(1, "Home").item("https://example.com"))
        .item(BreadcrumbListItem::new(2, "Products"));

    let document = JsonLdDocument::new().context("https://schema.org").graph(
        JsonLdArray::new()
            .item(organization)
            .item(breadcrumb.into_object()),
    );

    let value = document.to_value();

    assert_eq!(
        value["@graph"][0]["@type"],
        JsonValue::String(String::from("Organization"))
    );
    assert_eq!(
        value["@graph"][1]["@type"],
        JsonValue::String(String::from("BreadcrumbList"))
    );
}
