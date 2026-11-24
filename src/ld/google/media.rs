use crate::{JsonLdDocument, JsonLdObject, JsonValue};

use super::{SCHEMA_ORG_CONTEXT, values_to_json};

/// A Google-friendly `Clip` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Clip {
    name: String,
    start_offset: u64,
    end_offset: u64,
    url: String,
    extra: JsonLdObject,
}

/// A Google-friendly `BroadcastEvent` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct BroadcastEvent {
    start_date: String,
    end_date: Option<String>,
    is_live_broadcast: bool,
    extra: JsonLdObject,
}

/// A Google-friendly `VideoObject` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoObject {
    name: String,
    description: String,
    upload_date: String,
    thumbnail_urls: Vec<JsonValue>,
    duration: Option<String>,
    embed_url: Option<String>,
    content_url: Option<String>,
    expires: Option<String>,
    clips: Vec<JsonValue>,
    broadcast_event: Option<JsonValue>,
    regions_allowed: Vec<String>,
    family_friendly: Option<bool>,
    extra: JsonLdObject,
}

impl Clip {
    /// Creates a `Clip` builder with its required fields.
    #[must_use]
    pub fn new<N, U>(name: N, start_offset: u64, end_offset: u64, url: U) -> Self
    where
        N: Into<String>,
        U: Into<String>,
    {
        Self {
            name: name.into(),
            start_offset,
            end_offset,
            url: url.into(),
            extra: JsonLdObject::new(),
        }
    }

    /// Adds an extra field for unsupported-but-needed properties.
    #[must_use]
    pub fn field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.extra = self.extra.field(key, value);
        self
    }

    /// Returns the clip as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("Clip")
            .field("name", self.name)
            .field("startOffset", self.start_offset)
            .field("endOffset", self.end_offset)
            .field("url", self.url);

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl BroadcastEvent {
    /// Creates a `BroadcastEvent` builder with its required start date.
    #[must_use]
    pub fn new<S>(start_date: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            start_date: start_date.into(),
            end_date: None,
            is_live_broadcast: true,
            extra: JsonLdObject::new(),
        }
    }

    /// Sets `endDate`.
    #[must_use]
    pub fn end_date<D>(mut self, end_date: D) -> Self
    where
        D: Into<String>,
    {
        self.end_date = Some(end_date.into());
        self
    }

    /// Sets `isLiveBroadcast`.
    #[must_use]
    pub fn is_live_broadcast(mut self, is_live_broadcast: bool) -> Self {
        self.is_live_broadcast = is_live_broadcast;
        self
    }

    /// Adds an extra field for unsupported-but-needed properties.
    #[must_use]
    pub fn field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.extra = self.extra.field(key, value);
        self
    }

    /// Returns the broadcast event as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("BroadcastEvent")
            .field("isLiveBroadcast", self.is_live_broadcast)
            .field("startDate", self.start_date);

        if let Some(end_date) = self.end_date {
            object = object.field("endDate", end_date);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl VideoObject {
    /// Creates a `VideoObject` builder with the Google-required core fields.
    #[must_use]
    pub fn new<N, D, U, T>(name: N, description: D, upload_date: U, thumbnail_url: T) -> Self
    where
        N: Into<String>,
        D: Into<String>,
        U: Into<String>,
        T: Into<JsonValue>,
    {
        Self {
            name: name.into(),
            description: description.into(),
            upload_date: upload_date.into(),
            thumbnail_urls: vec![thumbnail_url.into()],
            duration: None,
            embed_url: None,
            content_url: None,
            expires: None,
            clips: Vec::new(),
            broadcast_event: None,
            regions_allowed: Vec::new(),
            family_friendly: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Adds one `thumbnailUrl`.
    #[must_use]
    pub fn thumbnail<V>(mut self, thumbnail_url: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.thumbnail_urls.push(thumbnail_url.into());
        self
    }

    /// Sets `duration`.
    #[must_use]
    pub fn duration<D>(mut self, duration: D) -> Self
    where
        D: Into<String>,
    {
        self.duration = Some(duration.into());
        self
    }

    /// Sets `embedUrl`.
    #[must_use]
    pub fn embed_url<U>(mut self, embed_url: U) -> Self
    where
        U: Into<String>,
    {
        self.embed_url = Some(embed_url.into());
        self
    }

    /// Sets `contentUrl`.
    #[must_use]
    pub fn content_url<U>(mut self, content_url: U) -> Self
    where
        U: Into<String>,
    {
        self.content_url = Some(content_url.into());
        self
    }

    /// Sets `expires`.
    #[must_use]
    pub fn expires<E>(mut self, expires: E) -> Self
    where
        E: Into<String>,
    {
        self.expires = Some(expires.into());
        self
    }

    /// Adds one `Clip` entry via `hasPart`.
    #[must_use]
    pub fn clip<V>(mut self, clip: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.clips.push(clip.into());
        self
    }

    /// Sets `publication` from a `BroadcastEvent`.
    #[must_use]
    pub fn broadcast_event<V>(mut self, broadcast_event: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.broadcast_event = Some(broadcast_event.into());
        self
    }

    /// Adds one `regionsAllowed` entry.
    #[must_use]
    pub fn region_allowed<R>(mut self, region: R) -> Self
    where
        R: Into<String>,
    {
        self.regions_allowed.push(region.into());
        self
    }

    /// Sets `isFamilyFriendly`.
    #[must_use]
    pub fn family_friendly(mut self, family_friendly: bool) -> Self {
        self.family_friendly = Some(family_friendly);
        self
    }

    /// Adds an extra field for unsupported-but-needed properties.
    #[must_use]
    pub fn field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<JsonValue>,
    {
        self.extra = self.extra.field(key, value);
        self
    }

    /// Returns the video object as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("VideoObject")
            .field("name", self.name)
            .field("description", self.description)
            .field("uploadDate", self.upload_date)
            .field("thumbnailUrl", values_to_json(self.thumbnail_urls));

        if let Some(duration) = self.duration {
            object = object.field("duration", duration);
        }

        if let Some(embed_url) = self.embed_url {
            object = object.field("embedUrl", embed_url);
        }

        if let Some(content_url) = self.content_url {
            object = object.field("contentUrl", content_url);
        }

        if let Some(expires) = self.expires {
            object = object.field("expires", expires);
        }

        if !self.clips.is_empty() {
            object = object.field("hasPart", values_to_json(self.clips));
        }

        if let Some(broadcast_event) = self.broadcast_event {
            object = object.field("publication", broadcast_event);
        }

        if !self.regions_allowed.is_empty() {
            object = object.field(
                "regionsAllowed",
                super::strings_to_json(self.regions_allowed),
            );
        }

        if let Some(family_friendly) = self.family_friendly {
            object = object.field("isFamilyFriendly", family_friendly);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the video object as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl From<Clip> for JsonLdObject {
    fn from(value: Clip) -> Self {
        value.into_object()
    }
}

impl From<Clip> for JsonValue {
    fn from(value: Clip) -> Self {
        value.into_object().into()
    }
}

impl From<BroadcastEvent> for JsonLdObject {
    fn from(value: BroadcastEvent) -> Self {
        value.into_object()
    }
}

impl From<BroadcastEvent> for JsonValue {
    fn from(value: BroadcastEvent) -> Self {
        value.into_object().into()
    }
}

impl From<VideoObject> for JsonLdObject {
    fn from(value: VideoObject) -> Self {
        value.into_object()
    }
}

impl From<VideoObject> for JsonLdDocument {
    fn from(value: VideoObject) -> Self {
        value.into_document()
    }
}

impl From<VideoObject> for JsonValue {
    fn from(value: VideoObject) -> Self {
        value.into_object().into()
    }
}
