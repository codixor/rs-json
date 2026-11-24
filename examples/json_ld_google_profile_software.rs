use rs_json::ld::google::{
    profiles::{ProfilePage, ProfilePerson},
    reviews::AggregateRating,
    software::SoftwareApplication,
};

fn main() {
    let profile = ProfilePage::new(
        ProfilePerson::new("Example Creator")
            .alternate_name("@example")
            .description("Builds Rust-first structured data systems.")
            .same_as("https://example.com"),
    )
    .date_created("2026-04-08")
    .date_modified("2026-04-09");

    let software = SoftwareApplication::free("rs-json Schema Builder")
        .price_currency("USD")
        .operating_system("Linux, macOS, Windows")
        .application_category("DeveloperApplication")
        .aggregate_rating(AggregateRating::new("4.9", 240))
        .offer_field("availability", "https://schema.org/InStock");

    println!(
        "{}",
        profile.into_document().to_json_string_pretty().unwrap()
    );
    println!(
        "{}",
        software.into_document().to_json_string_pretty().unwrap()
    );
}
