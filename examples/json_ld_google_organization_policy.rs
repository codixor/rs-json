use rs_json::ld::google::{business::Organization, commerce::MerchantReturnPolicy};

fn main() {
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

    println!("{}", document.to_json_string_pretty().unwrap());
}
