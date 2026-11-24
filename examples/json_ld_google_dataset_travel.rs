use rs_json::ld::google::{
    data_travel::{Accommodation, DataCatalog, DataDownload, Dataset, VacationRental},
    jobs_events::PostalAddress,
    profiles::ProfilePerson,
    reviews::AggregateRating,
};

fn main() {
    let dataset = Dataset::new(
        "Regional weather observations",
        "Daily climate observations exported as open data.",
        ProfilePerson::new("Example Data Team"),
    )
    .distribution(
        DataDownload::new("https://data.example.com/weather.csv").encoding_format("text/csv"),
    )
    .included_in_data_catalog(DataCatalog::new("Example Open Data"))
    .keyword("weather")
    .accessible_for_free(true);

    let rental = VacationRental::new(
        "Desert escape villa",
        "villa-001",
        25.2048,
        55.2708,
        Accommodation::new(6)
            .number_of_rooms(3)
            .bed("King bed", 2)
            .amenity("Pool", true),
    )
    .address(
        PostalAddress::new("AE")
            .street_address("Desert Road 10")
            .locality("Dubai"),
    )
    .aggregate_rating(AggregateRating::new("4.7", 48));

    println!(
        "{}",
        dataset.into_document().to_json_string_pretty().unwrap()
    );
    println!(
        "{}",
        rental.into_document().to_json_string_pretty().unwrap()
    );
}
