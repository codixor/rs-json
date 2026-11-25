//! JSON-LD google product.
use rs_json::{
    JsonLdObject,
    ld::google::commerce::{MerchantReturnPolicy, Offer, OfferShippingDetails, Product},
};

fn main() {
    let shipping = OfferShippingDetails::new(
        "US",
        JsonLdObject::new()
            .ty("MonetaryAmount")
            .field("value", "5.99")
            .field("currency", "USD"),
    )
    .shipping_label("Standard shipping");
    let policy = MerchantReturnPolicy::new("https://schema.org/MerchantReturnFiniteReturnWindow")
        .merchant_return_days(30)
        .return_fees("https://schema.org/FreeReturn");
    let document = Product::new("Wool winter coat - green, size small")
        .sku("COAT-GREEN-S")
        .description("Soft wool winter coat in green, size small.")
        .image("https://example.com/images/coat-green-s.png")
        .brand_name("Example Apparel")
        .offer(
            Offer::new("149.00", "USD")
                .availability("https://schema.org/InStock")
                .url("https://example.com/products/coat-green-s")
                .shipping_details(shipping)
                .merchant_return_policy(policy),
        )
        .into_document();

    println!("{}", document.to_json_string().unwrap());
}
