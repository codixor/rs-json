//! JSON-LD hand built.
use rs_json::{JsonLdArray, JsonLdDocument, JsonLdObject};

fn main() {
    let document = JsonLdDocument::new().context("https://schema.org").graph(
        JsonLdArray::new()
            .item(
                JsonLdObject::new()
                    .ty("WebSite")
                    .field("name", "Example Site")
                    .field("url", "https://example.com"),
            )
            .item(
                JsonLdObject::new()
                    .ty("SearchAction")
                    .field("target", "https://example.com/search?q={query}")
                    .field("query-input", "required name=query"),
            ),
    );

    println!("{}", document.to_json_string_pretty().unwrap());
}
