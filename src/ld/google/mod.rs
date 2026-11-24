//! Google-oriented JSON-LD profile builders.
//!
//! Family entrypoints:
//! - [`business`]
//! - [`navigation`]
//! - [`publishing`]
//! - [`reviews`]
//! - [`commerce`]
//! - [`jobs_events`]
//! - [`media`]
//! - [`forum_qa`]
//! - [`education`]
//! - [`profiles`]
//! - [`software`]
//! - [`data_travel`]
//!
//! Coverage note:
//! - this family map now covers the planned Google/Search-Central slices for
//!   this crate
//! - richer typed validation for Google-required property sets can still expand
//!   inside each family as the builders mature

use crate::{JsonLdArray, JsonLdObject, JsonValue};

mod breadcrumb;
pub mod business;
pub mod commerce;
mod commerce_types;
mod content;
pub mod data_travel;
pub mod education;
pub mod forum_qa;
pub mod jobs_events;
mod local_business;
pub mod media;
pub mod navigation;
mod organization;
mod product;
pub mod profiles;
pub mod publishing;
mod review;
pub mod reviews;
pub mod software;

pub(crate) const SCHEMA_ORG_CONTEXT: &str = "https://schema.org";

pub use self::breadcrumb::{BreadcrumbList, BreadcrumbListItem};
pub use self::commerce::{
    AggregateOffer, MerchantReturnPolicy, Offer, OfferShippingDetails, Product, ProductGroup,
};
pub use self::content::{Article, BlogPosting, NewsArticle, Recipe};
pub use self::data_travel::{Accommodation, DataCatalog, DataDownload, Dataset, VacationRental};
pub use self::education::{Course, CourseList, FlashcardQuestion, Quiz};
pub use self::forum_qa::{Answer, DiscussionForumPosting, ForumComment, QAPage, Question};
pub use self::jobs_events::{Event, JobPosting, Place, PostalAddress, VirtualLocation};
pub use self::local_business::LocalBusiness;
pub use self::media::{BroadcastEvent, Clip, VideoObject};
pub use self::organization::Organization;
pub use self::profiles::{ProfilePage, ProfilePerson};
pub use self::review::{AggregateRating, Review};
pub use self::software::SoftwareApplication;

pub(crate) fn schema_property_url(name: &str) -> String {
    format!("{SCHEMA_ORG_CONTEXT}/{name}")
}

pub(crate) fn named_thing(schema_type: &str, name: String) -> JsonLdObject {
    JsonLdObject::new().ty(schema_type).field("name", name)
}

pub(crate) fn values_to_json(values: Vec<JsonValue>) -> JsonValue {
    match values.len() {
        0 => JsonValue::Array(Vec::new()),
        1 => values.into_iter().next().unwrap(),
        _ => {
            let mut array = JsonLdArray::new();
            for value in values {
                array = array.item(value);
            }
            array.into()
        }
    }
}

pub(crate) fn strings_to_json(values: Vec<String>) -> JsonValue {
    values_to_json(values.into_iter().map(JsonValue::from).collect())
}
