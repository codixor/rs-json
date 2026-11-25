//! JSON-LD google breadcrumb.
use rs_json::ld::google::navigation::{BreadcrumbList, BreadcrumbListItem};

fn main() {
    let document = BreadcrumbList::new()
        .item(BreadcrumbListItem::new(1, "Home").item("https://example.com"))
        .item(BreadcrumbListItem::new(2, "Products").item("https://example.com/products"))
        .item(BreadcrumbListItem::new(3, "Winter coats"))
        .into_document();

    println!("{}", document.to_script_tag().unwrap());
}
