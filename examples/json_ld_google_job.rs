//! JSON-LD google job.
use rs_json::{
    JsonLdObject,
    ld::google::{
        business::Organization,
        jobs_events::{JobPosting, Place, PostalAddress},
    },
};

fn main() {
    let salary = JsonLdObject::new()
        .ty("MonetaryAmount")
        .field("currency", "USD")
        .field(
            "value",
            JsonLdObject::new()
                .ty("QuantitativeValue")
                .field("value", 85)
                .field("unitText", "HOUR"),
        );
    let location = Place::new(
        "Example HQ",
        PostalAddress::new("US")
            .street_address("555 Clancy St")
            .locality("Detroit")
            .region("MI")
            .postal_code("48201"),
    );
    let organization = Organization::new("Example Corp")
        .url("https://example.com")
        .logo_url("https://example.com/logo.png");

    let document = JobPosting::new(
        "Senior Rust Engineer",
        "<p>Build and ship high-performance Rust systems.</p>",
        "2026-04-08",
        organization,
    )
    .valid_through("2026-05-08T23:59:59Z")
    .job_location(location)
    .remote_country("Canada")
    .employment_type("FULL_TIME")
    .identifier_value("Example Corp", "job-123")
    .base_salary(salary)
    .direct_apply(true)
    .into_document();

    println!("{}", document.to_json_string_pretty().unwrap());
}
