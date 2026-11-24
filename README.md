# rs-json

`rs-json` is the workspace’s lightweight zero-copy JSON parser crate.

The crate is intentionally small:
- library-first
- zero-copy DOM that borrows from the input
- typed `thiserror` errors
- crate-root tests, examples, and Criterion benches

## Public API

```rust
use rs_json::{JsonError, JsonItem, JsonType, parse};

fn inspect(source: &str) -> Result<JsonItem<'_>, JsonError> {
    parse(source)
}
```

The core public surface is:
- `parse(&str) -> Result<JsonItem<'_>, JsonError>`
- serde-backed typed codec helpers:
  - `to_vec`
  - `to_string`
  - `to_value`
  - `from_slice`
  - `from_str`
  - `from_value`
- `JsonItem` traversal helpers:
  - `as_str`
  - `as_f64`
  - `as_i128`
  - `as_bool`
  - `exists`
  - `is_null`
  - `kind`
  - `elements`
  - `entries`
- additive owned JSON-LD builders:
  - `JsonLdDocument`
  - `JsonLdObject`
  - `JsonLdArray`

`JsonItem::as_str()` returns the raw slice from the source text. For strings, that means:
- quotes are removed
- escapes are preserved exactly as written
- no unescaping allocation happens in the core parser

For typed JSON serialization and deserialization, the crate now also exposes a
workspace-owned serde facade. That keeps downstream crates on `rs_json::*`
instead of wiring `serde_json` directly.

## JSON-LD

`rs-json` now also includes an additive owned JSON-LD builder surface under
`rs_json::ld`.

```rust
use rs_json::{JsonLdDocument, JsonLdObject};

let doc = JsonLdDocument::new()
    .context("https://schema.org")
    .ty("Product")
    .field("name", "Widget")
    .field(
        "offers",
        JsonLdObject::new()
            .ty("Offer")
            .field("price", "19.99")
            .field("priceCurrency", "USD"),
    );

let json = doc.to_json_string().unwrap();
let script = doc.to_script_tag().unwrap();
```

This JSON-LD surface is intentionally:
- additive to the parser API
- explicit and non-macro-first
- usable without caller-side serde derives
- still able to accept raw `serde_json::Value` escape hatches when needed

The current JSON-LD core also keeps the render path lean:
- document JSON rendering serializes the root map directly
- script-tag rendering avoids an extra formatting round
- owned JSON-LD objects and arrays start with small reserved capacity because
  the common schema shapes are not truly empty containers

The first Google-oriented builders now live under `rs_json::ld::google`, with
family entrypoints such as:
- `rs_json::ld::google::commerce`
- `rs_json::ld::google::publishing`
- `rs_json::ld::google::business`
- `rs_json::ld::google::navigation`
- `rs_json::ld::google::reviews`
- `rs_json::ld::google::jobs_events`
- `rs_json::ld::google::media`
- `rs_json::ld::google::forum_qa`
- `rs_json::ld::google::education`
- `rs_json::ld::google::profiles`
- `rs_json::ld::google::software`
- `rs_json::ld::google::data_travel`

```rust
use rs_json::ld::google::{
    commerce::{Offer, Product, ProductGroup},
    navigation::{BreadcrumbList, BreadcrumbListItem},
};

let coat = Product::new("Wool winter coat - green, size small")
    .sku("COAT-GREEN-S")
    .in_product_group_with_id("coat-001")
    .offer(Offer::new("149.00", "USD"));

let document = ProductGroup::new("Wool winter coat", "coat-001")
    .varies_by_schema_property("color")
    .varies_by_schema_property("size")
    .variant(coat)
    .into_document();

let breadcrumb = BreadcrumbList::new()
    .item(BreadcrumbListItem::new(1, "Home").item("https://example.com"))
    .item(BreadcrumbListItem::new(2, "Coats"));

let _json = document.to_json_string().unwrap();
let _script = breadcrumb.into_document().to_script_tag().unwrap();
```

Supported Google-oriented builders today:
- business/navigation:
  - `Organization`
  - `LocalBusiness`
  - `BreadcrumbList`
- publishing:
  - `Article`
  - `NewsArticle`
  - `BlogPosting`
  - `Recipe`
- reviews:
  - `Review`
  - `AggregateRating`
- commerce:
  - `ProductGroup`
  - `Product`
  - `Offer`
  - `AggregateOffer`
  - `MerchantReturnPolicy`
  - `OfferShippingDetails`
- jobs/events:
  - `JobPosting`
  - `Event`
  - `Place`
  - `PostalAddress`
  - `VirtualLocation`
- media:
  - `VideoObject`
  - `Clip`
  - `BroadcastEvent`
- forum / Q&A:
  - `QAPage`
  - `Question`
  - `Answer`
  - `DiscussionForumPosting`
  - `ForumComment`
- education:
  - `Quiz`
  - `FlashcardQuestion`
  - `Course`
  - `CourseList`
- profiles / software:
  - `ProfilePage`
  - `ProfilePerson`
  - `SoftwareApplication`
- data / travel:
  - `Dataset`
  - `DataCatalog`
  - `DataDownload`
  - `VacationRental`
  - `Accommodation`

## Examples

Run the examples with:

```sh
cargo run -p rs-json@0.1.0 --example parse_object
cargo run -p rs-json@0.1.0 --example traverse_nested
cargo run -p rs-json@0.1.0 --example inspect_error
cargo run -p rs-json@0.1.0 --example json_ld_hand_built
cargo run -p rs-json@0.1.0 --example json_ld_google_product
cargo run -p rs-json@0.1.0 --example json_ld_google_organization_policy
cargo run -p rs-json@0.1.0 --example json_ld_google_breadcrumb
cargo run -p rs-json@0.1.0 --example json_ld_google_news
cargo run -p rs-json@0.1.0 --example json_ld_google_job
cargo run -p rs-json@0.1.0 --example json_ld_google_event
cargo run -p rs-json@0.1.0 --example json_ld_google_local_business
cargo run -p rs-json@0.1.0 --example json_ld_google_video
cargo run -p rs-json@0.1.0 --example json_ld_google_forum_qa
cargo run -p rs-json@0.1.0 --example json_ld_google_education
cargo run -p rs-json@0.1.0 --example json_ld_google_profile_software
cargo run -p rs-json@0.1.0 --example json_ld_google_dataset_travel
```

## Validation

```sh
bash rs-json/scripts/validate.sh
```

That covers:
- tests
- examples
- strict clippy
- docs
- bench compilation

## Benchmarks

```sh
bash rs-json/scripts/bench.sh
bash rs-json/scripts/bench-json-ld.sh
```

The current Criterion surface covers:
- small document parse
- nested document parse
- string-heavy document parse
- number-heavy document parse
- repeated key traversal
- invalid document rejection
- JSON-LD builder and render baselines
- dedicated JSON-LD family slices for:
  - local/business
  - media/video
- bench-only JSON-LD object-store comparison slices for:
  - current `serde_json::Map`
  - `indexmap`
  - `hashbrown`
  - `hashbrown` + `ahash`

Current comparison baselines:
- `serde_json`
- legacy `json 0.12.4`
- `simd-json` on `x86_64` / `aarch64`
- `sonic-rs` on `x86_64` / `aarch64`

The latest measured decision was conservative on purpose: a `memchr`-based
string-scan experiment did not clear the adoption bar because it regressed the
common small and nested parse lanes, so the parser keeps the simpler baseline
for now.

The same rule closed the remaining helper-crate ideas too:
- `smallvec` / `arrayvec` are a poor fit for the recursive zero-copy DOM unless
  we change the tree shape with extra indirection
- `compact_str` is not justified because the hot path intentionally borrows
  strings from the source instead of owning them

## Direction

This crate is now treated as first-party workspace code, not an untouched donor snapshot. The current parser line is owned here and can evolve behind the small public API.

Future interop work with `kdl` now follows a measured rule:

- use the shared neutral interop model as the preferred bridge
- use the generated JSON-on-KDL contract only when the caller actually needs a
  KDL `Document`

That keeps JSON parsing performance work independent from format-conversion policy and avoids forcing JSON through KDL-shaped structure when the shared model is enough.
