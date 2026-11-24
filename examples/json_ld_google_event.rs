use rs_json::ld::google::{
    business::Organization,
    commerce::Offer,
    jobs_events::{Event, Place, PostalAddress, VirtualLocation},
};

fn main() {
    let venue = Place::new(
        "Warehouse Four",
        PostalAddress::new("AE")
            .street_address("Alserkal Avenue")
            .locality("Dubai"),
    );
    let document = Event::new("Rust Dubai Meetup", "2026-05-01T19:00:00+04:00", venue)
        .end_date("2026-05-01T21:00:00+04:00")
        .description("A local meetup focused on performance-oriented Rust.")
        .attendance_mode("https://schema.org/MixedEventAttendanceMode")
        .organizer(Organization::new("Rust Dubai"))
        .performer_name("Community Speakers")
        .offer(
            Offer::new("0", "AED")
                .availability("https://schema.org/InStock")
                .url("https://example.com/events/rust-dubai-meetup"),
        )
        .into_document();

    let online = Event::new(
        "Rust Remote AMA",
        "2026-05-07T18:00:00Z",
        VirtualLocation::new("https://example.com/live"),
    )
    .attendance_mode("https://schema.org/OnlineEventAttendanceMode")
    .organizer_name("Rust Dubai")
    .into_document();

    println!("{}", document.to_json_string_pretty().unwrap());
    println!("{}", online.to_json_string_pretty().unwrap());
}
