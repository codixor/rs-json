use crate::{JsonLdArray, JsonLdDocument, JsonLdObject, JsonValue};

use super::SCHEMA_ORG_CONTEXT;

/// A Google-friendly `BreadcrumbList` builder.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BreadcrumbList {
    items: Vec<BreadcrumbListItem>,
}

/// A typed `ListItem` entry for a `BreadcrumbList`.
#[derive(Debug, Clone, PartialEq)]
pub struct BreadcrumbListItem {
    position: u64,
    name: String,
    item: Option<JsonValue>,
}

impl BreadcrumbList {
    /// Creates an empty breadcrumb list.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one breadcrumb entry.
    #[must_use]
    pub fn item(mut self, item: BreadcrumbListItem) -> Self {
        self.items.push(item);
        self
    }

    /// Returns the breadcrumb list as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut item_list = JsonLdArray::new();
        for item in self.items {
            item_list = item_list.item(item);
        }

        JsonLdObject::new()
            .ty("BreadcrumbList")
            .field("itemListElement", item_list)
    }

    /// Returns the breadcrumb list as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl BreadcrumbListItem {
    /// Creates a breadcrumb item with its required position and label.
    #[must_use]
    pub fn new<N>(position: u64, name: N) -> Self
    where
        N: Into<String>,
    {
        Self {
            position,
            name: name.into(),
            item: None,
        }
    }

    /// Sets the linked target for the breadcrumb item.
    #[must_use]
    pub fn item<V>(mut self, item: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.item = Some(item.into());
        self
    }

    /// Returns the breadcrumb item as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("ListItem")
            .field("position", self.position)
            .field("name", self.name);

        if let Some(item) = self.item {
            object = object.field("item", item);
        }

        object
    }
}

impl From<BreadcrumbList> for JsonLdObject {
    fn from(value: BreadcrumbList) -> Self {
        value.into_object()
    }
}

impl From<BreadcrumbList> for JsonLdDocument {
    fn from(value: BreadcrumbList) -> Self {
        value.into_document()
    }
}

impl From<BreadcrumbList> for JsonValue {
    fn from(value: BreadcrumbList) -> Self {
        value.into_object().into()
    }
}

impl From<BreadcrumbListItem> for JsonLdObject {
    fn from(value: BreadcrumbListItem) -> Self {
        value.into_object()
    }
}

impl From<BreadcrumbListItem> for JsonValue {
    fn from(value: BreadcrumbListItem) -> Self {
        value.into_object().into()
    }
}
