use rs_json::{JsonLdArray, JsonLdDocument, JsonLdObject, JsonValue};

#[test]
fn json_ld_document_builds_core_fields() {
    let document = JsonLdDocument::new()
        .context("https://schema.org")
        .ty("Product")
        .id("https://example.com/products/widget")
        .field("name", "Widget")
        .field("sku", "WIDGET-1");

    let value = document.to_value();

    assert_eq!(
        value["@context"],
        JsonValue::String(String::from("https://schema.org"))
    );
    assert_eq!(value["@type"], JsonValue::String(String::from("Product")));
    assert_eq!(
        value["@id"],
        JsonValue::String(String::from("https://example.com/products/widget"))
    );
    assert_eq!(value["name"], JsonValue::String(String::from("Widget")));
    assert_eq!(value["sku"], JsonValue::String(String::from("WIDGET-1")));
}

#[test]
fn json_ld_document_supports_nested_objects_and_arrays() {
    let images = JsonLdArray::new()
        .item("https://example.com/widget-1.png")
        .item("https://example.com/widget-2.png");
    let offer = JsonLdObject::new()
        .ty("Offer")
        .field("price", "19.99")
        .field("priceCurrency", "USD");
    let document = JsonLdDocument::new()
        .context("https://schema.org")
        .ty("Product")
        .field("name", "Widget")
        .field("image", images)
        .field("offers", offer);

    let value = document.to_value();

    assert_eq!(
        value["image"][0],
        JsonValue::String(String::from("https://example.com/widget-1.png"))
    );
    assert_eq!(
        value["image"][1],
        JsonValue::String(String::from("https://example.com/widget-2.png"))
    );
    assert_eq!(
        value["offers"]["@type"],
        JsonValue::String(String::from("Offer"))
    );
    assert_eq!(
        value["offers"]["price"],
        JsonValue::String(String::from("19.99"))
    );
}

#[test]
fn json_ld_document_accepts_raw_value_escape_hatches() {
    let aggregate_rating = serde_json::json!({
        "@type": "AggregateRating",
        "ratingValue": "4.8",
        "reviewCount": 27
    });
    let document = JsonLdDocument::new()
        .context("https://schema.org")
        .ty("Product")
        .field("name", "Widget")
        .field("aggregateRating", aggregate_rating);

    let value = document.to_value();

    assert_eq!(
        value["aggregateRating"]["@type"],
        JsonValue::String(String::from("AggregateRating"))
    );
    assert_eq!(value["aggregateRating"]["reviewCount"], JsonValue::from(27));
}

#[test]
fn json_ld_document_serializes_keys_canonically() {
    let document = JsonLdDocument::new()
        .context("https://schema.org")
        .ty("Product")
        .field("name", "Widget")
        .field("description", "Short description")
        .field(
            "offers",
            JsonLdObject::new()
                .field("priceCurrency", "USD")
                .field("price", "19.99")
                .ty("Offer"),
        );

    assert_eq!(
        document.to_json_string().unwrap(),
        r#"{"@context":"https://schema.org","@type":"Product","description":"Short description","name":"Widget","offers":{"@type":"Offer","price":"19.99","priceCurrency":"USD"}}"#
    );
}

#[test]
fn json_ld_document_renders_script_tag() {
    let document = JsonLdDocument::new()
        .context("https://schema.org")
        .ty("BreadcrumbList")
        .field(
            "itemListElement",
            JsonLdArray::new().item(
                JsonLdObject::new()
                    .ty("ListItem")
                    .field("position", 1)
                    .field("name", "Books"),
            ),
        );

    let script = document.to_script_tag().unwrap();

    assert!(script.starts_with("<script type=\"application/ld+json\">"));
    assert!(script.ends_with("</script>"));
    assert!(script.contains("\"@context\":\"https://schema.org\""));
    assert!(script.contains("\"@type\":\"BreadcrumbList\""));
}

#[test]
fn json_ld_document_pretty_output_is_deterministic() {
    let document = JsonLdDocument::new()
        .context("https://schema.org")
        .ty("Organization")
        .field("name", "Acme")
        .field("url", "https://example.com");

    let pretty = document.to_json_string_pretty().unwrap();

    assert!(pretty.contains('\n'));
    assert_eq!(
        pretty,
        "{\n  \"@context\": \"https://schema.org\",\n  \"@type\": \"Organization\",\n  \"name\": \"Acme\",\n  \"url\": \"https://example.com\"\n}"
    );
}
