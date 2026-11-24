use rs_json::{
    JsonLdObject,
    ld::google::{
        BreadcrumbList, BreadcrumbListItem, MerchantReturnPolicy, Offer, OfferShippingDetails,
        Organization, Product,
    },
};

#[test]
fn compact_product_json_ld_matches_golden() {
    let shipping = OfferShippingDetails::new(
        "US",
        JsonLdObject::new()
            .ty("MonetaryAmount")
            .field("value", "5.99")
            .field("currency", "USD"),
    )
    .shipping_label("Standard shipping");
    let policy = MerchantReturnPolicy::new("https://schema.org/MerchantReturnFiniteReturnWindow")
        .merchant_return_days(30)
        .return_fees("https://schema.org/FreeReturn");
    let document = Product::new("Wool winter coat - green, size small")
        .sku("COAT-GREEN-S")
        .description("Soft wool winter coat in green, size small.")
        .image("https://example.com/images/coat-green-s.png")
        .brand_name("Example Apparel")
        .offer(
            Offer::new("149.00", "USD")
                .availability("https://schema.org/InStock")
                .url("https://example.com/products/coat-green-s")
                .shipping_details(shipping)
                .merchant_return_policy(policy),
        )
        .into_document();

    assert_eq!(
        document.to_json_string().unwrap(),
        include_str!("ld/compact_product.json").trim_end()
    );
}

#[test]
fn pretty_organization_json_ld_matches_golden() {
    let policy = MerchantReturnPolicy::new("https://schema.org/MerchantReturnFiniteReturnWindow")
        .merchant_return_days(30)
        .applicable_country("US")
        .return_fees("https://schema.org/FreeReturn");
    let document = Organization::new("Example Online Store")
        .schema_type("OnlineStore")
        .url("https://example.com")
        .logo_url("https://example.com/assets/logo.png")
        .same_as("https://www.instagram.com/example")
        .same_as("https://www.youtube.com/@example")
        .merchant_return_policy(policy)
        .into_document();

    assert_eq!(
        document.to_json_string_pretty().unwrap(),
        include_str!("ld/pretty_organization_policy.json").trim_end()
    );
}

#[test]
fn breadcrumb_script_tag_matches_golden() {
    let document = BreadcrumbList::new()
        .item(BreadcrumbListItem::new(1, "Home").item("https://example.com"))
        .item(BreadcrumbListItem::new(2, "Products").item("https://example.com/products"))
        .item(BreadcrumbListItem::new(3, "Winter coats"))
        .into_document();

    assert_eq!(
        document.to_script_tag().unwrap(),
        include_str!("ld/breadcrumb_script.html").trim_end()
    );
}
