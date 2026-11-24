# rs-json Plan

Goal: keep `rs-json` as the workspace’s main lightweight JSON parser crate, with a small stable API, sound lifetimes, typed errors, crate-root proof tests, and measured performance work before any deeper optimization or `kdl` interop lands.

Extended goal: add an additive `rs_json::ld` JSON-LD builder surface for
hand-built schema documents, with a clean non-derive API, Google-oriented
profile builders, and staged proof/benchmark coverage.

## Direction

- [x] Keep the crate library-first.
- [x] Keep the public API intentionally small:
  - [x] `parse(&str) -> Result<JsonItem<'_>, JsonError>`
  - [x] stable `JsonItem` traversal helpers
- [x] Switch the public error type to `thiserror`.
- [x] Move tests out of `src/lib.rs` and into crate-root `tests/`.
- [x] Add golden tests for correctness and regression safety.
- [x] Add a real benchmark matrix baseline.
- [x] Use performance helper crates only after measurement.
- [x] No helper crate is adopted unless it wins on the kept workload mix.
- [x] Keep JSON-LD additive under `rs_json::ld`, not mixed into the parser-first
      zero-copy API.
- [x] Keep the JSON-LD builder useful without requiring caller-side `Serialize`
      derives.
- [x] Keep the non-serde builder path explicit and non-panicking.
- [x] Keep Google Search Central and schema.org as the source of truth for
      Google-targeted JSON-LD profile builders.
- [x] Keep direct `serde_json::Value` escape hatches for advanced callers.

## Current Ownership Decision

- [x] Treat the parser as first-party workspace code.
- [x] Remove stale `jsonic` naming from the active crate surface.
- [x] Keep legacy `json 0.12.4` only as a historical comparator.
- [x] Revisit replacement only if correctness or measured performance makes ownership unattractive later.
- [x] Current benchmark and API-role evidence does not justify replacement for the zero-copy parser role.

## Current JSON-LD Builder Decision

- [x] Use `serde_json::Map<String, Value>` and `serde_json::Value` as the owned
      base representation for hand-built JSON-LD documents.
- [x] Keep `serde_json::json!` as a comparator and convenience macro, not as
      the primary non-serde builder API.
- [x] Keep typed profile builders layered above `Value` / `Map` instead of
      forcing downstream users into `Serialize`-first models.
- [x] Keep the Google JSON-LD surface under a dedicated `ld` module tree.

Current decision notes:
- official `serde_json` docs present `json!` as a natural way to construct
  `Value`, but it is macro-first and expects interpolated values to implement
  Serde serialization
- official docs also note that `json!` can panic if serialization fails or if
  an interpolated map has non-string keys
- official `serde_json::map` docs define `Map<String, Value>` directly as the
  object representation, which is the better base for an explicit non-serde
  builder surface
- Google Search Central documents the supported structured-data gallery and the
  required/recommended properties for product, breadcrumb, organization,
  recipe, review, merchant return policy, and shipping policy builders we want
  to target

## Stage 1: Workspace And Manifest Cleanup

- [x] Align `Cargo.toml` to workspace metadata:
  - [x] edition
  - [x] rust-version
  - [x] license
  - [x] version
  - [x] repository
  - [x] description
  - [x] keywords/categories
- [x] Add `rs-json` to the root workspace.
- [x] Remove the old local release profile override.
- [x] Replace `divan` with `criterion`.
- [x] Keep `README.md` and `PLAN.md` in the crate.

## Stage 2: Error Model With thiserror

- [x] Replace the handwritten error implementation in [error.rs](./src/error.rs) with `thiserror`.
- [x] Keep error payloads lightweight:
  - [x] error index
  - [x] optional nearby extract
- [x] Add stable error kinds for:
  - [x] unexpected EOF
  - [x] invalid token
  - [x] invalid string
  - [x] invalid number
  - [x] invalid object structure
  - [x] invalid array structure
  - [x] trailing characters

## Stage 3: API And Parser Hardening

- [x] Rewrite the parser around lifetimes so the DOM cannot outlive the source input.
- [x] Remove the old raw-pointer `Slice` model.
- [x] Tighten parser correctness for:
  - [x] whitespace handling
  - [x] empty input
  - [x] top-level scalar values
  - [x] string escapes
  - [x] escaped backslashes and quotes
  - [x] unicode escape validation
  - [x] number grammar
  - [x] exponent grammar
  - [x] nesting correctness
  - [x] trailing character rejection

## Stage 4: Tests To Crate-Root And Golden Matrix

- [x] Move inline tests from [lib.rs](./src/lib.rs) into crate-root `tests/`.
- [x] Split tests into focused files:
  - [x] parse basics
  - [x] string handling
  - [x] number handling
  - [x] traversal
  - [x] error behavior
- [x] Add golden fixtures under [`tests/golden/`](./tests/golden/).
- [x] Add golden coverage for:
  - [x] valid nested data
  - [x] escaped strings
  - [x] invalid syntax

## Stage 5: Benchmark Matrix

- [x] Replace the old loose bench setup with one Criterion surface.
- [x] Add benchmark groups for:
  - [x] small valid document parse
  - [x] nested document parse
  - [x] invalid document rejection
- [x] Keep comparison benches for:
  - [x] `serde_json`
  - [x] legacy `json 0.12.4`
- [x] Expand the matrix with:
  - [x] string-heavy document parse
  - [x] number-heavy document parse
  - [x] repeated key traversal
  - [x] `simd-json`
  - [x] `sonic-rs`

## Stage 6: Performance Pass

- [x] Profile before changing parser internals.
- [x] Keep the current safe parser as the baseline.
- [x] Only adopt helper crates if they produce a validated `>=40%` win on a real hot path.
- [x] Candidate measured upgrades:
  - [x] `memchr` for quote scanning was benchmarked and rejected because it regressed the common parse lanes
  - [x] `smallvec` or `arrayvec` for tiny containers was prototyped and rejected because inline recursive storage is a poor fit for the recursive zero-copy DOM without adding boxing or a different tree shape
  - [x] `compact_str` or similar is intentionally not adopted because the current parser keeps string data borrowed from the source instead of owning it on the hot path

## Stage 7: Docs, Examples, And Validation

- [x] Add examples for:
  - [x] parse object
  - [x] traverse nested value
  - [x] inspect parse error
- [x] Add validation and bench scripts.
- [x] Keep README aligned with the landed crate state.
- [x] Run:
  - [x] `cargo test`
  - [x] `cargo test --examples`
  - [x] `cargo clippy --all-targets --all-features -D warnings`
  - [x] `cargo doc --no-deps`
  - [x] `cargo check --benches`

## Stage 8: Replacement Rule

- [x] Default to keeping this parser line.
- [x] Only revisit replacement if:
  - [x] conformance gaps prove expensive to maintain
  - [x] the benchmark matrix shows a maintained alternative is materially better for this crate’s role
- [x] Current status:
  - [x] no conformance gap is forcing a switch
  - [x] the measured alternatives are useful comparators, not a clear drop-in replacement for this crate’s zero-copy role

## Stage 9: KDL Interop Contract

- [x] Add a neutral JSON/KDL interop contract once the JSON core is stable.
- [x] Keep conversion through a shared model, not raw text bridging.
- [x] Benchmark the contract path before adopting a conversion strategy in `kdl`.
- [x] Prefer the path that keeps parse performance isolated from interop costs.
- [x] Measured decision from the `kdl` interop bench surface:
  - [x] keep the direct shared-model bridge as the preferred semantic owner
  - [x] keep the generated JSON-on-KDL contract as an additive adapter only when callers actually need a `kdl::Document`
  - [x] avoid introducing a direct `json -> kdl` normal dependency edge that would create a package-cycle hazard

## Stage 10: JSON-LD Core Module

- [x] Add a new additive module tree rooted at [`src/ld/`](./src/ld/).
- [x] Keep parser ownership and JSON-LD builder ownership separate:
  - [x] parsing stays under the current zero-copy parser surface
  - [x] JSON-LD building stays under `rs_json::ld`
- [x] Add a small owned JSON-LD document surface:
  - [x] `JsonLdDocument`
  - [x] `JsonLdObject`
  - [x] `JsonLdArray`
  - [x] builder helpers for `@context`, `@type`, `@id`, and `@graph`
- [x] Keep the core builder backed by `serde_json::Map<String, Value>` and
      `serde_json::Value`.
- [x] Keep the public builder surface explicit rather than macro-only:
  - [x] object field insertion helpers
  - [x] array append helpers
  - [x] nested object/array composition
  - [x] final render to `serde_json::Value`
  - [x] final render to compact / pretty JSON text
  - [x] final render to `<script type="application/ld+json">`
- [x] Keep deterministic object output for tests and goldens.
- [x] Keep raw `serde_json::Value` escape hatches where typed builders are too
      narrow.

Suggested module shape:
- [x] [`src/ld/mod.rs`](./src/ld/mod.rs)
- [x] [`src/ld/builder.rs`](./src/ld/builder.rs)
- [x] [`src/ld/document.rs`](./src/ld/document.rs)
- [x] [`src/ld/google/mod.rs`](./src/ld/google/mod.rs)

Current Stage 10 status:
- the additive `ld` module now ships core owned builders on top of
  `serde_json::Map<String, Value>` and `serde_json::Value`
- the current builder surface already supports:
  - top-level document construction
  - nested object/array composition
  - `@context`, `@type`, `@id`, and `@graph`
  - compact JSON, pretty JSON, and script-tag rendering
  - raw `serde_json::Value` escape hatches for unsupported properties
- Google-specific profile builders remain staged for the next module layer
- the current kept core optimization is:
  - direct JSON string rendering from the root map without cloning through an
    intermediate `Value`
  - small default reserved capacity for owned JSON-LD objects and arrays to
    reduce tiny-container reallocation in common schema shapes

## Stage 11: Google JSON-LD Profile Builders

- [x] Add a Google/Search-Central-oriented profile layer under
      [`src/ld/google/`](./src/ld/google/).
- [x] Group the Google layer into feature families instead of a flat export
      surface.
- [x] Keep Google Search Central plus schema.org as the compatibility baseline.
- [x] Add generic/high-value builders first:
  - [x] `Organization`
  - [x] `BreadcrumbList`
  - [x] `Article`
  - [x] `Recipe`
  - [x] `Review`
  - [x] `AggregateRating`
- [x] Add commerce-oriented builders:
  - [x] `ProductGroup`
  - [x] `Product`
  - [x] `Offer`
  - [x] `AggregateOffer`
  - [x] `MerchantReturnPolicy`
  - [x] `OfferShippingDetails`
- [x] Add jobs/events-oriented builders:
  - [x] `JobPosting`
  - [x] `Event`
  - [x] `Place`
  - [x] `PostalAddress`
  - [x] `VirtualLocation`
- [x] Add local/business-oriented builders:
  - [x] `LocalBusiness`
- [x] Add media/video-oriented builders:
  - [x] `VideoObject`
  - [x] `Clip`
  - [x] `BroadcastEvent`
- [x] Add forum / Q&A-oriented builders:
  - [x] `QAPage`
  - [x] `Question`
  - [x] `Answer`
  - [x] `DiscussionForumPosting`
  - [x] `ForumComment`
- [x] Add education-oriented builders:
  - [x] `Quiz`
  - [x] `FlashcardQuestion`
  - [x] `Course`
  - [x] `CourseList`
- [x] Add profile / software / data / travel-oriented builders:
  - [x] `ProfilePage`
  - [x] `ProfilePerson`
  - [x] `SoftwareApplication`
  - [x] `Dataset`
  - [x] `DataCatalog`
  - [x] `DataDownload`
  - [x] `VacationRental`
  - [x] `Accommodation`
- [x] Keep product-variant grouping support:
  - [x] `ProductGroup.variesBy`
  - [x] `ProductGroup.hasVariant`
  - [x] `ProductGroup.productGroupID`
- [x] Keep organization-level merchant policy support:
  - [x] `Organization.hasMerchantReturnPolicy`
- [x] Keep offer-level override support:
  - [x] `Offer.hasMerchantReturnPolicy`
  - [x] `Offer.shippingDetails`
- [x] Keep the builder able to emit either:
  - [x] a single top-level JSON-LD node
  - [x] a top-level `@graph`
- [x] Keep a coverage matrix for broader Google Search Gallery types so the
      module can expand toward fuller Google-supported coverage without losing
      scope control.

## Stage 12: JSON-LD Validation, Goldens, And Benchmarking

- [x] Add crate-root tests for JSON-LD core building.
- [x] Add golden fixtures under [`tests/ld/`](./tests/ld/) for:
  - [x] compact JSON-LD output
  - [x] pretty JSON-LD output
  - [x] script-tag output
  - [x] Google-oriented product / organization / breadcrumb examples
- [x] Add proof tests for Google-required-property enforcement where the typed
      API can guarantee it.
- [x] Add examples showing:
  - [x] hand-built JSON-LD without serde derives
  - [x] product snippet
  - [x] organization + merchant return policy
  - [x] breadcrumb list
  - [x] job posting
  - [x] event
  - [x] local business
  - [x] video
  - [x] forum / Q&A
  - [x] education
  - [x] profile / software
  - [x] dataset / travel
- [x] Add Criterion coverage for the chosen builder path.
- [x] Add dedicated JSON-LD family benchmark slices for the heavier Google
      builder families.
  - [x] local/business
  - [x] media/video
  Family benchmark note:
  - `json/ld_family/local_business/to_value`
    - `google_typed`: `18.186-21.349 µs`
    - `explicit_builder`: `18.704-20.083 µs`
    - `json_macro`: `9.1738-10.446 µs`
    - `typed_serialize`: `9.2333-9.9941 µs`
  - `json/ld_family/local_business/to_string`
    - `google_typed`: `20.268-23.073 µs`
    - `explicit_builder`: `19.485-21.600 µs`
    - `json_macro`: `10.643-11.962 µs`
    - `typed_serialize`: `1.4979-1.7163 µs`
  - `json/ld_family/media_video/to_value`
    - `google_typed`: `14.148-15.146 µs`
    - `explicit_builder`: `13.523-14.106 µs`
    - `json_macro`: `6.6770-6.9861 µs`
    - `typed_serialize`: `7.0689-7.5433 µs`
  - `json/ld_family/media_video/to_string`
    - `google_typed`: `15.490-17.015 µs`
    - `explicit_builder`: `14.937-15.825 µs`
    - `json_macro`: `8.2229-8.8026 µs`
    - `typed_serialize`: `1.3233-1.3988 µs`
- [x] Compare alternate object-store backends against the current
      `serde_json::Map` construction path before changing JSON-LD internals.
  Object-store comparison note:
  - `json/object_store/product_to_value`
    - `serde_map`: `5.3613-5.8124 µs`
    - `indexmap`: `7.5984-9.1035 µs`
    - `hashbrown`: `8.7295-13.735 µs`
    - `hashbrown_ahash`: `6.7480-8.5040 µs`
  - `json/object_store/local_business_to_value`
    - `serde_map`: `8.4175-9.0234 µs`
    - `indexmap`: `11.266-11.919 µs`
    - `hashbrown`: `9.8553-10.987 µs`
    - `hashbrown_ahash`: `11.539-14.601 µs`
  - `json/object_store/media_video_to_value`
    - `serde_map`: `6.9931-8.0526 µs`
    - `indexmap`: `8.7192-9.4702 µs`
    - `hashbrown`: `7.7184-8.1750 µs`
    - `hashbrown_ahash`: `7.8000-8.4609 µs`
  Decision:
  - keep `serde_json::Map` as the JSON-LD object store for now
  - `indexmap` is consistently slower on these materialized document shapes
  - `hashbrown` variants are not strong enough or stable enough to justify an
    internal switch
  - no object-store backend change is currently justified
- [x] Benchmark the JSON-LD builder baseline before trying helper crates.
- [x] Benchmark builder variants before locking the surface:
  - [x] `serde_json::Map` / `Value` explicit builder
  - [x] `serde_json::json!` macro baseline
  - [x] typed `Serialize` model baseline where useful
- [x] Defer helper-crate experiments until a proven JSON-LD builder hotspot
      appears.
  Candidates kept ready for measured follow-up only:
  - `smallvec`
  - `arrayvec`
  - `indexmap`
  - `hashbrown`
  - `compact_str`
  - `smol_str`
  - `tinyvec`
  - `itoa`
  - `ryu`
  - `bumpalo`
  - `phf`
  - `ahash`
- [x] Keep helper crates out of the implementation until a hotspot proves they
      are worth adding.
- [x] Keep the explicit non-serde builder as the chosen public path.
  Benchmark note:
  - `json/ld_to_value`
    - `google_typed`: `11.294-12.025 µs`
    - `explicit_builder`: `10.882-11.805 µs`
    - `json_macro`: `5.2365-5.7830 µs`
    - `typed_serialize`: `5.1135-5.7106 µs`
  - `json/ld_to_string`
    - `google_typed`: `7.1316-7.5764 µs`
    - `explicit_builder`: `6.5074-7.3861 µs`
    - `json_macro`: `6.5142-7.0038 µs`
    - `typed_serialize`: `964.09 ns-1.0471 µs`
  Decision:
  - macro and typed-serialize baselines are faster
  - the explicit/google builder path remains the right public surface because it
    stays derive-free, explicit, and reusable for the intended non-serde case
  - the kept JSON-LD core optimization materially improved the shared
    `to_string` path for both the generic product lanes and the heavier
    local/business and media/video family slices
- [x] Run:
  - [x] `cargo test`
  - [x] `cargo test --examples`
  - [x] `cargo clippy --all-targets --all-features -D warnings`
  - [x] `cargo doc --no-deps`
  - [x] `cargo check --benches`

## Success Criteria

- [x] `rs-json` is the clear main lightweight JSON parser crate in the workspace.
- [x] Public errors use `thiserror`.
- [x] Tests live under crate-root `tests/`.
- [x] Golden fixtures protect parser behavior.
- [x] Benchmark matrix exists and is reproducible.
- [x] Performance helpers are adopted only where measurement justifies them.
- [x] `kdl` interop is added through a measured contract layer rather than ad-hoc text conversion.
- [x] `rs_json::ld` provides a clean non-serde JSON-LD builder surface.
- [x] Google-oriented JSON-LD builders live under `rs_json::ld::google`.
- [x] Product, organization, breadcrumb, and merchant policy builders are
      covered by tests and examples.
- [x] Jobs/events builders are covered by tests and examples.
- [x] The chosen JSON-LD builder path is justified against `serde_json::json!`
      and typed `Serialize` baselines.
