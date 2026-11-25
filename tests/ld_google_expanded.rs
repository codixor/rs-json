//! JSON-LD google expanded.
#![allow(
    clippy::indexing_slicing,
    reason = "these assertions exercise public JSON indexing on fixed test fixtures"
)]
use rs_json::{
    JsonValue,
    ld::google::{
        business::LocalBusiness,
        data_travel::{Accommodation, DataCatalog, DataDownload, Dataset, VacationRental},
        education::{Course, CourseList, Quiz},
        forum_qa::{Answer, DiscussionForumPosting, ForumComment, QAPage, Question},
        jobs_events::PostalAddress,
        media::{BroadcastEvent, Clip, VideoObject},
        profiles::{ProfilePage, ProfilePerson},
        reviews::AggregateRating,
        software::SoftwareApplication,
    },
};

#[test]
fn local_business_builder_supports_google_local_fields() {
    let value = LocalBusiness::new(
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
    .to_value();

    assert_eq!(
        value["@type"],
        JsonValue::String(String::from("CafeOrCoffeeShop"))
    );
    assert_eq!(
        value["address"]["addressLocality"],
        JsonValue::String(String::from("Dubai"))
    );
    assert_eq!(
        value["geo"]["@type"],
        JsonValue::String(String::from("GeoCoordinates"))
    );
    assert_eq!(
        value["openingHoursSpecification"]["dayOfWeek"],
        JsonValue::String(String::from("https://schema.org/Monday"))
    );
    assert_eq!(
        value["aggregateRating"]["@type"],
        JsonValue::String(String::from("AggregateRating"))
    );
    assert_eq!(
        value["servesCuisine"][0],
        JsonValue::String(String::from("Coffee"))
    );
}

#[test]
fn video_builder_supports_clips_and_live_broadcast_shape() {
    let value = VideoObject::new(
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
    .to_value();

    assert_eq!(
        value["@type"],
        JsonValue::String(String::from("VideoObject"))
    );
    assert_eq!(
        value["hasPart"]["@type"],
        JsonValue::String(String::from("Clip"))
    );
    assert_eq!(
        value["publication"]["@type"],
        JsonValue::String(String::from("BroadcastEvent"))
    );
    assert_eq!(
        value["regionsAllowed"][1],
        JsonValue::String(String::from("US"))
    );
    assert_eq!(value["isFamilyFriendly"], JsonValue::Bool(true));
}

#[test]
fn qa_and_forum_builders_render_expected_google_shapes() {
    let qa_value = QAPage::new(
        Question::new(
            "How do I expose JSON-LD in rs-json?",
            "What is the cleanest way to emit Google JSON-LD from rs-json builders?",
        )
        .author_name("Example User")
        .date_created("2026-04-08T09:00:00Z")
        .accepted_answer(
            Answer::new("Use the typed builders under rs_json::ld::google.")
                .author_name("Maintainer")
                .date_published("2026-04-08T10:00:00Z"),
        )
        .suggested_answer(
            Answer::new("You can also compose raw JsonLdObject values.").author_name("Contributor"),
        ),
    )
    .into_document()
    .to_value();

    assert_eq!(qa_value["@type"], JsonValue::String(String::from("QAPage")));
    assert_eq!(
        qa_value["mainEntity"]["acceptedAnswer"]["@type"],
        JsonValue::String(String::from("Answer"))
    );
    assert_eq!(
        qa_value["mainEntity"]["suggestedAnswer"]["@type"],
        JsonValue::String(String::from("Answer"))
    );

    let forum_value = DiscussionForumPosting::new(
        "How are you structuring JSON-LD families?",
        "We split them into commerce, publishing, jobs/events, media, and more.",
        "Example Maintainer",
        "2026-04-08T11:00:00Z",
    )
    .comment(
        ForumComment::new(
            "This makes the API much easier to discover.",
            "Example Reader",
            "2026-04-08T11:30:00Z",
        )
        .upvote_count(4),
    )
    .shared_content(
        VideoObject::new(
            "Family overview",
            "Overview clip",
            "2026-04-08T10:00:00Z",
            "https://example.com/families.png",
        )
        .duration("PT1M"),
    )
    .interaction_statistic("https://schema.org/LikeAction", 12)
    .into_document()
    .to_value();

    assert_eq!(
        forum_value["@type"],
        JsonValue::String(String::from("DiscussionForumPosting"))
    );
    assert_eq!(
        forum_value["comment"]["@type"],
        JsonValue::String(String::from("Comment"))
    );
    assert_eq!(
        forum_value["sharedContent"]["@type"],
        JsonValue::String(String::from("VideoObject"))
    );
    assert_eq!(
        forum_value["interactionStatistic"]["@type"],
        JsonValue::String(String::from("InteractionCounter"))
    );
}

#[test]
fn education_builders_support_quiz_and_course_list_shapes() {
    let quiz_value = Quiz::new()
        .name("Rust JSON-LD Fundamentals")
        .about("A short flashcard quiz about structured data.")
        .educational_level("Beginner")
        .flashcard("What does @context usually point to?", "https://schema.org")
        .into_document()
        .to_value();

    assert_eq!(quiz_value["@type"], JsonValue::String(String::from("Quiz")));
    assert_eq!(
        quiz_value["hasPart"]["eduQuestionType"],
        JsonValue::String(String::from("Flashcard"))
    );
    assert_eq!(
        quiz_value["hasPart"]["acceptedAnswer"]["text"],
        JsonValue::String(String::from("https://schema.org"))
    );

    let course_list_value = CourseList::new()
        .course(
            "https://example.com/courses/rs-json",
            Course::new(
                "Structured data with rs-json",
                "Learn how to emit Google JSON-LD with typed builders.",
            )
            .provider_name("Example Academy")
            .image("https://example.com/course.png"),
        )
        .into_document()
        .to_value();

    assert_eq!(
        course_list_value["@type"],
        JsonValue::String(String::from("ItemList"))
    );
    assert_eq!(
        course_list_value["itemListElement"]["item"]["@type"],
        JsonValue::String(String::from("Course"))
    );
    assert_eq!(
        course_list_value["itemListElement"]["item"]["provider"]["name"],
        JsonValue::String(String::from("Example Academy"))
    );
}

#[test]
fn profile_and_software_builders_cover_profile_page_and_offer_shape() {
    let profile = ProfilePerson::new("Example Creator")
        .alternate_name("@example")
        .description("Builds Rust-first structured data systems.")
        .identifier("creator-001")
        .image("https://example.com/avatar.png")
        .same_as("https://example.com")
        .interaction_statistic("https://schema.org/FollowAction", 420);

    let page_value = ProfilePage::new(profile)
        .date_created("2026-04-08")
        .date_modified("2026-04-09")
        .into_document()
        .to_value();

    assert_eq!(
        page_value["@type"],
        JsonValue::String(String::from("ProfilePage"))
    );
    assert_eq!(
        page_value["mainEntity"]["@type"],
        JsonValue::String(String::from("Person"))
    );
    assert_eq!(
        page_value["mainEntity"]["agentInteractionStatistic"]["userInteractionCount"],
        JsonValue::from(420)
    );

    let software_value = SoftwareApplication::free("rs-json Schema Builder")
        .price_currency("USD")
        .operating_system("Linux, macOS, Windows")
        .application_category("DeveloperApplication")
        .aggregate_rating(AggregateRating::new("4.9", 240))
        .offer_field("availability", "https://schema.org/InStock")
        .url("https://example.com/rs-json")
        .into_document()
        .to_value();

    assert_eq!(
        software_value["@type"],
        JsonValue::String(String::from("SoftwareApplication"))
    );
    assert_eq!(software_value["offers"]["price"], JsonValue::from(0));
    assert_eq!(
        software_value["offers"]["availability"],
        JsonValue::String(String::from("https://schema.org/InStock"))
    );
}

#[test]
fn dataset_and_vacation_rental_builders_render_expected_shapes() {
    let dataset_value = Dataset::new(
        "Regional weather observations",
        "Daily climate observations exported as open data.",
        ProfilePerson::new("Example Data Team"),
    )
    .url("https://example.com/datasets/weather")
    .same_as("https://data.example.com/weather")
    .distribution(
        DataDownload::new("https://data.example.com/weather.csv").encoding_format("text/csv"),
    )
    .included_in_data_catalog(DataCatalog::new("Example Open Data"))
    .keyword("weather")
    .keyword("climate")
    .accessible_for_free(true)
    .into_document()
    .to_value();

    assert_eq!(
        dataset_value["@type"],
        JsonValue::String(String::from("Dataset"))
    );
    assert_eq!(
        dataset_value["distribution"]["@type"],
        JsonValue::String(String::from("DataDownload"))
    );
    assert_eq!(
        dataset_value["includedInDataCatalog"]["@type"],
        JsonValue::String(String::from("DataCatalog"))
    );
    assert_eq!(
        dataset_value["keywords"][1],
        JsonValue::String(String::from("climate"))
    );

    let vacation_value = VacationRental::new(
        "Desert escape villa",
        "villa-001",
        25.2048,
        55.2708,
        Accommodation::new(6)
            .number_of_rooms(3)
            .number_of_bathrooms_total(2)
            .bed("King bed", 2)
            .amenity("Pool", true)
            .floor_size(220, "MTK"),
    )
    .image("https://example.com/villa.png")
    .description("A private villa for desert weekend stays.")
    .address(
        PostalAddress::new("AE")
            .street_address("Desert Road 10")
            .locality("Dubai"),
    )
    .aggregate_rating(AggregateRating::new("4.7", 48))
    .into_document()
    .to_value();

    assert_eq!(
        vacation_value["@type"],
        JsonValue::String(String::from("VacationRental"))
    );
    assert_eq!(
        vacation_value["containsPlace"]["@type"],
        JsonValue::String(String::from("Accommodation"))
    );
    assert_eq!(
        vacation_value["containsPlace"]["amenityFeature"]["@type"],
        JsonValue::String(String::from("LocationFeatureSpecification"))
    );
    assert_eq!(
        vacation_value["aggregateRating"]["@type"],
        JsonValue::String(String::from("AggregateRating"))
    );
}
