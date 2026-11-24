use rs_json::ld::google::{
    business::LocalBusiness, jobs_events::PostalAddress, reviews::AggregateRating,
};

fn main() {
    let document = LocalBusiness::new(
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
    .geo_coordinates(25.2048, 55.2708)
    .opening_hours("https://schema.org/Monday", "08:00", "17:00")
    .aggregate_rating(AggregateRating::new("4.8", 120))
    .serves_cuisine("Coffee")
    .into_document();

    println!("{}", document.to_json_string_pretty().unwrap());
}
