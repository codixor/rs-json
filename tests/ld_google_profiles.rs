use rs_json::{
    JsonLdObject, JsonValue,
    ld::google::{
        commerce::{AggregateOffer, MerchantReturnPolicy, Offer, OfferShippingDetails},
        publishing::{Article, BlogPosting, NewsArticle, Recipe},
        reviews::{AggregateRating, Review},
    },
};

#[test]
fn article_builder_renders_expected_google_shape() {
    let value = Article::new("How to build clean JSON-LD")
        .schema_type("TechArticle")
        .image("https://example.com/article-cover.png")
        .date_published("2026-04-08")
        .date_modified("2026-04-09")
        .author_name("Example Author")
        .publisher(
            JsonLdObject::new()
                .ty("Organization")
                .field("name", "Example Corp"),
        )
        .main_entity_of_page("https://example.com/articles/json-ld")
        .into_document()
        .to_value();

    assert_eq!(
        value["@type"],
        JsonValue::String(String::from("TechArticle"))
    );
    assert_eq!(
        value["headline"],
        JsonValue::String(String::from("How to build clean JSON-LD"))
    );
    assert_eq!(
        value["author"]["name"],
        JsonValue::String(String::from("Example Author"))
    );
}

#[test]
fn publishing_family_supports_news_and_blog_article_types() {
    let news = NewsArticle::new("Workspace launch update")
        .date_published("2026-04-08")
        .into_document()
        .to_value();
    let blog = BlogPosting::new("How we structure JSON-LD builders")
        .author_name("Example Author")
        .into_document()
        .to_value();

    assert_eq!(
        news["@type"],
        JsonValue::String(String::from("NewsArticle"))
    );
    assert_eq!(
        blog["@type"],
        JsonValue::String(String::from("BlogPosting"))
    );
}

#[test]
fn recipe_builder_supports_rating_and_review() {
    let rating = AggregateRating::new("4.9", 24).review_count(18);
    let review = Review::new("Example Baker")
        .review_body("Worked perfectly for our weekend brunch.")
        .review_rating(AggregateRating::new("5", 1));

    let value = Recipe::new("Weekend pancakes")
        .image("https://example.com/pancakes.png")
        .author_name("Example Kitchen")
        .prep_time("PT10M")
        .cook_time("PT15M")
        .total_time("PT25M")
        .recipe_yield("4 servings")
        .ingredient("2 cups flour")
        .ingredient("2 eggs")
        .instruction_text("Whisk ingredients.")
        .instruction_text("Cook on a hot pan.")
        .aggregate_rating(rating)
        .review(review)
        .into_document()
        .to_value();

    assert_eq!(value["@type"], JsonValue::String(String::from("Recipe")));
    assert_eq!(
        value["aggregateRating"]["@type"],
        JsonValue::String(String::from("AggregateRating"))
    );
    assert_eq!(
        value["review"]["@type"],
        JsonValue::String(String::from("Review"))
    );
}

#[test]
fn aggregate_offer_builder_renders_expected_shape() {
    let value = AggregateOffer::new("129.00", "159.00", "USD")
        .offer_count(3)
        .availability("https://schema.org/InStock")
        .offer(Offer::new("129.00", "USD"))
        .into_object()
        .to_value();

    assert_eq!(
        value["@type"],
        JsonValue::String(String::from("AggregateOffer"))
    );
    assert_eq!(value["offerCount"], JsonValue::from(3));
    assert_eq!(
        value["offers"]["@type"],
        JsonValue::String(String::from("Offer"))
    );
}

#[test]
fn merchant_policy_and_shipping_builders_keep_required_core_fields() {
    let policy = MerchantReturnPolicy::new("https://schema.org/MerchantReturnFiniteReturnWindow")
        .merchant_return_days(30)
        .into_object()
        .to_value();
    let shipping = OfferShippingDetails::new(
        "AE",
        JsonLdObject::new()
            .ty("MonetaryAmount")
            .field("value", "20.00")
            .field("currency", "AED"),
    )
    .into_object()
    .to_value();

    assert_eq!(
        policy["returnPolicyCategory"],
        JsonValue::String(String::from(
            "https://schema.org/MerchantReturnFiniteReturnWindow"
        ))
    );
    assert_eq!(policy["merchantReturnDays"], JsonValue::from(30));
    assert_eq!(
        shipping["shippingDestination"]["addressCountry"],
        JsonValue::String(String::from("AE"))
    );
    assert_eq!(
        shipping["shippingRate"]["@type"],
        JsonValue::String(String::from("MonetaryAmount"))
    );
}
