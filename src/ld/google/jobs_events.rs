use crate::{JsonLdArray, JsonLdDocument, JsonLdObject, JsonValue};

use super::{SCHEMA_ORG_CONTEXT, named_thing};

/// A Google-friendly `PostalAddress` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct PostalAddress {
    address_country: String,
    street_address: Option<String>,
    locality: Option<String>,
    region: Option<String>,
    postal_code: Option<String>,
    name: Option<String>,
    extra: JsonLdObject,
}

/// A Google-friendly `Place` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    name: String,
    address: JsonValue,
    url: Option<String>,
    extra: JsonLdObject,
}

/// A Google-friendly `VirtualLocation` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualLocation {
    url: String,
    extra: JsonLdObject,
}

/// A Google-friendly `JobPosting` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct JobPosting {
    title: String,
    description_html: String,
    date_posted: String,
    valid_through: Option<String>,
    hiring_organization: JsonValue,
    job_locations: Vec<JsonValue>,
    applicant_location_requirements: Vec<JsonValue>,
    job_location_type: Option<String>,
    employment_types: Vec<String>,
    identifier: Option<JsonValue>,
    base_salary: Option<JsonValue>,
    direct_apply: Option<bool>,
    extra: JsonLdObject,
}

/// A Google-friendly `Event` builder.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    name: String,
    start_date: String,
    location: JsonValue,
    end_date: Option<String>,
    description: Option<String>,
    status: Option<String>,
    attendance_mode: Option<String>,
    previous_start_date: Option<String>,
    images: Vec<JsonValue>,
    offers: Vec<JsonValue>,
    organizer: Option<JsonValue>,
    performers: Vec<JsonValue>,
    extra: JsonLdObject,
}

impl PostalAddress {
    /// Creates a `PostalAddress` builder with its required `addressCountry`.
    #[must_use]
    pub fn new<C>(address_country: C) -> Self
    where
        C: Into<String>,
    {
        Self {
            address_country: address_country.into(),
            street_address: None,
            locality: None,
            region: None,
            postal_code: None,
            name: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Sets `streetAddress`.
    #[must_use]
    pub fn street_address<S>(mut self, street_address: S) -> Self
    where
        S: Into<String>,
    {
        self.street_address = Some(street_address.into());
        self
    }

    /// Sets `addressLocality`.
    #[must_use]
    pub fn locality<L>(mut self, locality: L) -> Self
    where
        L: Into<String>,
    {
        self.locality = Some(locality.into());
        self
    }

    /// Sets `addressRegion`.
    #[must_use]
    pub fn region<R>(mut self, region: R) -> Self
    where
        R: Into<String>,
    {
        self.region = Some(region.into());
        self
    }

    /// Sets `postalCode`.
    #[must_use]
    pub fn postal_code<P>(mut self, postal_code: P) -> Self
    where
        P: Into<String>,
    {
        self.postal_code = Some(postal_code.into());
        self
    }

    /// Sets the single-line address `name`.
    #[must_use]
    pub fn name<N>(mut self, name: N) -> Self
    where
        N: Into<String>,
    {
        self.name = Some(name.into());
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

    /// Returns the address as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("PostalAddress")
            .field("addressCountry", self.address_country);

        if let Some(street_address) = self.street_address {
            object = object.field("streetAddress", street_address);
        }

        if let Some(locality) = self.locality {
            object = object.field("addressLocality", locality);
        }

        if let Some(region) = self.region {
            object = object.field("addressRegion", region);
        }

        if let Some(postal_code) = self.postal_code {
            object = object.field("postalCode", postal_code);
        }

        if let Some(name) = self.name {
            object = object.field("name", name);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl Place {
    /// Creates a `Place` builder with the Google-recommended `name` and `address`.
    #[must_use]
    pub fn new<N, A>(name: N, address: A) -> Self
    where
        N: Into<String>,
        A: Into<JsonValue>,
    {
        Self {
            name: name.into(),
            address: address.into(),
            url: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Sets `url`.
    #[must_use]
    pub fn url<U>(mut self, url: U) -> Self
    where
        U: Into<String>,
    {
        self.url = Some(url.into());
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

    /// Returns the place as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("Place")
            .field("name", self.name)
            .field("address", self.address);

        if let Some(url) = self.url {
            object = object.field("url", url);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl VirtualLocation {
    /// Creates a `VirtualLocation` builder with its required `url`.
    #[must_use]
    pub fn new<U>(url: U) -> Self
    where
        U: Into<String>,
    {
        Self {
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

    /// Returns the virtual location as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("VirtualLocation")
            .field("url", self.url);

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }
}

impl JobPosting {
    /// Creates a `JobPosting` builder with the Google-required core fields.
    #[must_use]
    pub fn new<T, D, P, O>(
        title: T,
        description_html: D,
        date_posted: P,
        hiring_organization: O,
    ) -> Self
    where
        T: Into<String>,
        D: Into<String>,
        P: Into<String>,
        O: Into<JsonValue>,
    {
        Self {
            title: title.into(),
            description_html: description_html.into(),
            date_posted: date_posted.into(),
            valid_through: None,
            hiring_organization: hiring_organization.into(),
            job_locations: Vec::new(),
            applicant_location_requirements: Vec::new(),
            job_location_type: None,
            employment_types: Vec::new(),
            identifier: None,
            base_salary: None,
            direct_apply: None,
            extra: JsonLdObject::new(),
        }
    }

    /// Sets `validThrough`.
    #[must_use]
    pub fn valid_through<V>(mut self, valid_through: V) -> Self
    where
        V: Into<String>,
    {
        self.valid_through = Some(valid_through.into());
        self
    }

    /// Adds one `jobLocation`.
    #[must_use]
    pub fn job_location<V>(mut self, job_location: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.job_locations.push(job_location.into());
        self
    }

    /// Marks the posting as 100% remote and adds an allowed applicant country.
    #[must_use]
    pub fn remote_country<C>(mut self, country_name: C) -> Self
    where
        C: Into<String>,
    {
        self.job_location_type = Some(String::from("TELECOMMUTE"));
        self.applicant_location_requirements.push(
            JsonLdObject::new()
                .ty("Country")
                .field("name", country_name.into())
                .into(),
        );
        self
    }

    /// Adds one raw `applicantLocationRequirements` entry.
    #[must_use]
    pub fn applicant_location_requirement<V>(mut self, requirement: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.applicant_location_requirements
            .push(requirement.into());
        self
    }

    /// Sets `jobLocationType`, for example `TELECOMMUTE`.
    #[must_use]
    pub fn job_location_type<T>(mut self, job_location_type: T) -> Self
    where
        T: Into<String>,
    {
        self.job_location_type = Some(job_location_type.into());
        self
    }

    /// Adds one `employmentType`.
    #[must_use]
    pub fn employment_type<T>(mut self, employment_type: T) -> Self
    where
        T: Into<String>,
    {
        self.employment_types.push(employment_type.into());
        self
    }

    /// Sets `identifier` from a raw JSON-LD value.
    #[must_use]
    pub fn identifier<V>(mut self, identifier: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.identifier = Some(identifier.into());
        self
    }

    /// Sets a simple `PropertyValue` identifier pair.
    #[must_use]
    pub fn identifier_value<N, V>(self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<JsonValue>,
    {
        self.identifier(
            JsonLdObject::new()
                .ty("PropertyValue")
                .field("name", name.into())
                .field("value", value),
        )
    }

    /// Sets `baseSalary` from a raw JSON-LD value.
    #[must_use]
    pub fn base_salary<V>(mut self, base_salary: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.base_salary = Some(base_salary.into());
        self
    }

    /// Sets `directApply`.
    #[must_use]
    pub fn direct_apply(mut self, direct_apply: bool) -> Self {
        self.direct_apply = Some(direct_apply);
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

    /// Returns the job posting as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("JobPosting")
            .field("title", self.title)
            .field("description", self.description_html)
            .field("datePosted", self.date_posted)
            .field("hiringOrganization", self.hiring_organization);

        if let Some(valid_through) = self.valid_through {
            object = object.field("validThrough", valid_through);
        }

        if !self.job_locations.is_empty() {
            object = object.field("jobLocation", values_to_json(self.job_locations));
        }

        if !self.applicant_location_requirements.is_empty() {
            object = object.field(
                "applicantLocationRequirements",
                values_to_json(self.applicant_location_requirements),
            );
        }

        if let Some(job_location_type) = self.job_location_type {
            object = object.field("jobLocationType", job_location_type);
        }

        if !self.employment_types.is_empty() {
            object = object.field("employmentType", strings_to_json(self.employment_types));
        }

        if let Some(identifier) = self.identifier {
            object = object.field("identifier", identifier);
        }

        if let Some(base_salary) = self.base_salary {
            object = object.field("baseSalary", base_salary);
        }

        if let Some(direct_apply) = self.direct_apply {
            object = object.field("directApply", direct_apply);
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the job posting as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl Event {
    /// Creates an `Event` builder with the Google-required core fields.
    #[must_use]
    pub fn new<N, S, L>(name: N, start_date: S, location: L) -> Self
    where
        N: Into<String>,
        S: Into<String>,
        L: Into<JsonValue>,
    {
        Self {
            name: name.into(),
            start_date: start_date.into(),
            location: location.into(),
            end_date: None,
            description: None,
            status: None,
            attendance_mode: None,
            previous_start_date: None,
            images: Vec::new(),
            offers: Vec::new(),
            organizer: None,
            performers: Vec::new(),
            extra: JsonLdObject::new(),
        }
    }

    /// Replaces `location`.
    #[must_use]
    pub fn location<V>(mut self, location: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.location = location.into();
        self
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

    /// Sets `description`.
    #[must_use]
    pub fn description<D>(mut self, description: D) -> Self
    where
        D: Into<String>,
    {
        self.description = Some(description.into());
        self
    }

    /// Sets `eventStatus`.
    #[must_use]
    pub fn event_status<S>(mut self, event_status: S) -> Self
    where
        S: Into<String>,
    {
        self.status = Some(event_status.into());
        self
    }

    /// Sets `eventAttendanceMode`.
    #[must_use]
    pub fn attendance_mode<M>(mut self, attendance_mode: M) -> Self
    where
        M: Into<String>,
    {
        self.attendance_mode = Some(attendance_mode.into());
        self
    }

    /// Sets `previousStartDate`.
    #[must_use]
    pub fn previous_start_date<D>(mut self, previous_start_date: D) -> Self
    where
        D: Into<String>,
    {
        self.previous_start_date = Some(previous_start_date.into());
        self
    }

    /// Adds one `image`.
    #[must_use]
    pub fn image<V>(mut self, image: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.images.push(image.into());
        self
    }

    /// Adds one `offers` entry.
    #[must_use]
    pub fn offer<V>(mut self, offer: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.offers.push(offer.into());
        self
    }

    /// Sets `organizer`.
    #[must_use]
    pub fn organizer<V>(mut self, organizer: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.organizer = Some(organizer.into());
        self
    }

    /// Sets a simple organization `organizer`.
    #[must_use]
    pub fn organizer_name<N>(mut self, organizer_name: N) -> Self
    where
        N: Into<String>,
    {
        self.organizer = Some(
            JsonLdObject::new()
                .ty("Organization")
                .field("name", organizer_name.into())
                .into(),
        );
        self
    }

    /// Adds one `performer`.
    #[must_use]
    pub fn performer<V>(mut self, performer: V) -> Self
    where
        V: Into<JsonValue>,
    {
        self.performers.push(performer.into());
        self
    }

    /// Adds a simple performing-group `performer`.
    #[must_use]
    pub fn performer_name<N>(mut self, performer_name: N) -> Self
    where
        N: Into<String>,
    {
        self.performers.push(JsonValue::from(named_thing(
            "PerformingGroup",
            performer_name.into(),
        )));
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

    /// Returns the event as a plain JSON-LD object.
    #[must_use]
    pub fn into_object(self) -> JsonLdObject {
        let mut object = JsonLdObject::new()
            .ty("Event")
            .field("name", self.name)
            .field("startDate", self.start_date)
            .field("location", self.location);

        if let Some(end_date) = self.end_date {
            object = object.field("endDate", end_date);
        }

        if let Some(description) = self.description {
            object = object.field("description", description);
        }

        if let Some(event_status) = self.status {
            object = object.field("eventStatus", event_status);
        }

        if let Some(attendance_mode) = self.attendance_mode {
            object = object.field("eventAttendanceMode", attendance_mode);
        }

        if let Some(previous_start_date) = self.previous_start_date {
            object = object.field("previousStartDate", previous_start_date);
        }

        if !self.images.is_empty() {
            object = object.field("image", values_to_json(self.images));
        }

        if !self.offers.is_empty() {
            object = object.field("offers", values_to_json(self.offers));
        }

        if let Some(organizer) = self.organizer {
            object = object.field("organizer", organizer);
        }

        if !self.performers.is_empty() {
            object = object.field("performer", values_to_json(self.performers));
        }

        for (key, value) in self.extra.into_map() {
            object = object.field(key, value);
        }

        object
    }

    /// Returns the event as a top-level JSON-LD document.
    #[must_use]
    pub fn into_document(self) -> JsonLdDocument {
        JsonLdDocument::from_object(self.into_object()).context(SCHEMA_ORG_CONTEXT)
    }
}

impl From<PostalAddress> for JsonLdObject {
    fn from(value: PostalAddress) -> Self {
        value.into_object()
    }
}

impl From<PostalAddress> for JsonValue {
    fn from(value: PostalAddress) -> Self {
        value.into_object().into()
    }
}

impl From<Place> for JsonLdObject {
    fn from(value: Place) -> Self {
        value.into_object()
    }
}

impl From<Place> for JsonValue {
    fn from(value: Place) -> Self {
        value.into_object().into()
    }
}

impl From<VirtualLocation> for JsonLdObject {
    fn from(value: VirtualLocation) -> Self {
        value.into_object()
    }
}

impl From<VirtualLocation> for JsonValue {
    fn from(value: VirtualLocation) -> Self {
        value.into_object().into()
    }
}

impl From<JobPosting> for JsonLdObject {
    fn from(value: JobPosting) -> Self {
        value.into_object()
    }
}

impl From<JobPosting> for JsonLdDocument {
    fn from(value: JobPosting) -> Self {
        value.into_document()
    }
}

impl From<JobPosting> for JsonValue {
    fn from(value: JobPosting) -> Self {
        value.into_object().into()
    }
}

impl From<Event> for JsonLdObject {
    fn from(value: Event) -> Self {
        value.into_object()
    }
}

impl From<Event> for JsonLdDocument {
    fn from(value: Event) -> Self {
        value.into_document()
    }
}

impl From<Event> for JsonValue {
    fn from(value: Event) -> Self {
        value.into_object().into()
    }
}

fn values_to_json(values: Vec<JsonValue>) -> JsonValue {
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

fn strings_to_json(values: Vec<String>) -> JsonValue {
    values_to_json(values.into_iter().map(JsonValue::from).collect())
}
