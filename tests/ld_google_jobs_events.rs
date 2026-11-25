//! JSON-LD google jobs events.
#![allow(
    clippy::indexing_slicing,
    reason = "these assertions exercise public JSON indexing on fixed test fixtures"
)]
use rs_json::{
    JsonLdObject, JsonValue,
    ld::google::{
        business::Organization,
        commerce::Offer,
        jobs_events::{Event, JobPosting, Place, PostalAddress, VirtualLocation},
    },
};

#[test]
fn job_posting_builder_supports_physical_and_remote_google_shape() {
    let location = Place::new(
        "Example HQ",
        PostalAddress::new("US")
            .street_address("555 Clancy St")
            .locality("Detroit")
            .region("MI")
            .postal_code("48201"),
    );
    let hiring_organization = Organization::new("Example Corp")
        .url("https://example.com")
        .logo_url("https://example.com/logo.png");

    let value = JobPosting::new(
        "Senior Rust Engineer",
        "<p>Build and ship core Rust systems.</p>",
        "2026-04-08",
        hiring_organization,
    )
    .valid_through("2026-05-08T23:59:59Z")
    .job_location(location)
    .remote_country("Canada")
    .employment_type("FULL_TIME")
    .employment_type("CONTRACTOR")
    .identifier_value("Example Corp", "job-123")
    .direct_apply(true)
    .into_document()
    .to_value();

    assert_eq!(
        value["@type"],
        JsonValue::String(String::from("JobPosting"))
    );
    assert_eq!(
        value["title"],
        JsonValue::String(String::from("Senior Rust Engineer"))
    );
    assert_eq!(
        value["jobLocation"]["@type"],
        JsonValue::String(String::from("Place"))
    );
    assert_eq!(
        value["jobLocation"]["address"]["addressCountry"],
        JsonValue::String(String::from("US"))
    );
    assert_eq!(
        value["jobLocationType"],
        JsonValue::String(String::from("TELECOMMUTE"))
    );
    assert_eq!(
        value["applicantLocationRequirements"]["name"],
        JsonValue::String(String::from("Canada"))
    );
    assert_eq!(
        value["employmentType"][0],
        JsonValue::String(String::from("FULL_TIME"))
    );
    assert_eq!(
        value["employmentType"][1],
        JsonValue::String(String::from("CONTRACTOR"))
    );
    assert_eq!(
        value["identifier"]["@type"],
        JsonValue::String(String::from("PropertyValue"))
    );
}

#[test]
fn event_builder_supports_place_and_virtual_shapes() {
    let place = Place::new(
        "Warehouse Four",
        PostalAddress::new("AE")
            .street_address("Alserkal Avenue")
            .locality("Dubai"),
    );
    let value = Event::new("Rust Dubai Meetup", "2026-05-01T19:00:00+04:00", place)
        .end_date("2026-05-01T21:00:00+04:00")
        .description("A local meetup focused on performance-oriented Rust.")
        .attendance_mode("https://schema.org/MixedEventAttendanceMode")
        .organizer_name("Rust Dubai")
        .performer_name("Community Speakers")
        .offer(
            Offer::new("0", "AED")
                .availability("https://schema.org/InStock")
                .url("https://example.com/events/rust-dubai-meetup"),
        )
        .image("https://example.com/events/rust-dubai-meetup.png")
        .into_document()
        .to_value();

    assert_eq!(value["@type"], JsonValue::String(String::from("Event")));
    assert_eq!(
        value["location"]["@type"],
        JsonValue::String(String::from("Place"))
    );
    assert_eq!(
        value["location"]["name"],
        JsonValue::String(String::from("Warehouse Four"))
    );
    assert_eq!(
        value["organizer"]["name"],
        JsonValue::String(String::from("Rust Dubai"))
    );
    assert_eq!(
        value["performer"]["name"],
        JsonValue::String(String::from("Community Speakers"))
    );
    assert_eq!(
        value["offers"]["@type"],
        JsonValue::String(String::from("Offer"))
    );

    let virtual_value = Event::new(
        "Rust Remote AMA",
        "2026-05-07T18:00:00Z",
        VirtualLocation::new("https://example.com/live"),
    )
    .attendance_mode("https://schema.org/OnlineEventAttendanceMode")
    .into_document()
    .to_value();

    assert_eq!(
        virtual_value["location"]["@type"],
        JsonValue::String(String::from("VirtualLocation"))
    );
    assert_eq!(
        virtual_value["location"]["url"],
        JsonValue::String(String::from("https://example.com/live"))
    );
}

#[test]
fn jobs_events_builders_compose_inside_other_json_ld_nodes() {
    let location = Place::new(
        "Example HQ",
        PostalAddress::new("US").locality("Detroit").region("MI"),
    );
    let value = JsonLdObject::new()
        .ty("ItemList")
        .field(
            "itemListElement",
            rs_json::JsonLdArray::new()
                .item(
                    JobPosting::new(
                        "Backend Rust Engineer",
                        "<p>Ship backend services.</p>",
                        "2026-04-08",
                        Organization::new("Example Corp"),
                    )
                    .job_location(location.clone()),
                )
                .item(
                    Event::new("Hiring Event", "2026-05-09T18:00:00-04:00", location)
                        .organizer(Organization::new("Example Corp")),
                ),
        )
        .to_value();

    assert_eq!(
        value["itemListElement"][0]["@type"],
        JsonValue::String(String::from("JobPosting"))
    );
    assert_eq!(
        value["itemListElement"][1]["@type"],
        JsonValue::String(String::from("Event"))
    );
}
