use rs_json::ld::google::{business::Organization, publishing::NewsArticle};

fn main() {
    let publisher = Organization::new("Example Newsroom")
        .url("https://example.com")
        .logo_url("https://example.com/logo.png")
        .into_object();
    let document = NewsArticle::new("Workspace launches typed JSON-LD families")
        .image("https://example.com/news/typed-json-ld.png")
        .date_published("2026-04-08T08:00:00Z")
        .date_modified("2026-04-08T09:00:00Z")
        .author_name("Example Reporter")
        .publisher(publisher)
        .main_entity_of_page("https://example.com/news/typed-json-ld")
        .into_document();

    println!("{}", document.to_json_string_pretty().unwrap());
}
