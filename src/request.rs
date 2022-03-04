use crate::{
    filter::{EmptyFilter, Filterable},
    model::attribute::{FacetAttribute, SearchableAttributes},
};

use crate::filter::{CommonFilter, CommonFilterKind};
use chrono::{DateTime, Utc};
use serde::{ser::SerializeMap, Serialize};

/// Perform multiple write operations in a single API call.
/// In order to reduce the amount of time spent on network round trips, you can perform multiple write operations at once.
/// All operations will be applied in the order they are specified.
#[derive(Serialize)]
pub struct BatchWriteRequests {
    /// List of operations to batch.
    pub requests: Vec<BatchWriteRequest>,
}

#[derive(Serialize)]
pub enum UnimplementedOperation {}

// todo: links
/// A singular request as part of a batch.
#[derive(Serialize)]
#[serde(tag = "action", content = "body")]
#[serde(rename_all = "camelCase")]
pub enum BatchWriteRequest {
    /// Unimplemented.
    AddObject(UnimplementedOperation),
    /// Add or replace an existing object.
    /// You must set the `object_id` attribute to indicate the object to update.
    /// Equivalent to Add/update an object by ID.
    UpdateObject {
        #[serde(flatten)]
        body: serde_json::Map<String, serde_json::Value>,
        #[serde(rename = "objectID")]
        object_id: String,
    },
    /// Partially update an object.
    /// You must set the `object_id` attribute to indicate the object to update.
    /// Equivalent to Partially update an object.
    PartialUpdateObject {
        #[serde(flatten)]
        body: serde_json::Map<String, serde_json::Value>,
        #[serde(rename = "objectID")]
        object_id: String,
    },
    /// Same as `Self::PartialUpdateObject`, except that the object is not created if
    /// the object designated by `object_id` does not exist.
    PartialUpdateObjectNoCreate {
        #[serde(flatten)]
        body: serde_json::Map<String, serde_json::Value>,
        #[serde(rename = "objectID")]
        object_id: String,
    },
    /// Unimplemented.
    DeleteObject(UnimplementedOperation),

    /// Unimplemented.
    Delete(UnimplementedOperation),

    /// Unimplemented.
    Clear(UnimplementedOperation),
}

#[test]
fn test() {
    dbg!(serde_json::to_string_pretty(&BatchWriteRequests {
        requests: vec![BatchWriteRequest::UpdateObject {
            object_id: "hiii".to_owned(),
            body: serde_json::Map::new()
        }]
    })
    .unwrap());
}

#[derive(Default)]
pub struct SearchQuery<'a, T: CommonFilterKind, U: Filterable = EmptyFilter> {
    /// The text to search in the index.
    pub query: Option<&'a str>,

    /// Specify the page to retrieve.
    pub page: Option<u32>,

    /// Specify the number of hits to retrieve per page.
    pub hits_per_page: Option<u16>,

    /// Search filters.
    pub filters: Option<U>,

    /// Optional filters, passed as a part of the request body.
    pub optional_filters: Option<Vec<CommonFilter<T>>>,

    /// Retrieve detailed ranking information.
    pub get_ranking_info: bool,

    /// Whether to sum the scores of scored Or filters
    pub sum_or_filters_scores: bool,

    /// Maximum number of hits accessible via pagination
    pub pagination_limited_to: Option<u32>,
}

// can't use the derive macro due to a lack of T: Serialize bound
impl<T: CommonFilterKind, U: Filterable> serde::Serialize for SearchQuery<'_, T, U> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        if let Some(query) = self.query.filter(|it| !it.is_empty()) {
            map.serialize_entry("query", query)?;
        }

        if let Some(page) = self.page.filter(|&it| it != 0) {
            map.serialize_entry("page", &page)?;
        }

        if let Some(hits_per_page) = self.hits_per_page.filter(|&it| it != 20) {
            map.serialize_entry("hitsPerPage", &hits_per_page)?;
        }

        if let Some(filters) = &self.filters {
            map.serialize_entry("filters", &format_args!("{}", filters))?;
        }

        // algolia will guess this to be true by default.
        if !self.get_ranking_info {
            map.serialize_entry("getRankingInfo", &false)?;
        }

        // algolia will guess this to the false by default.
        if self.sum_or_filters_scores {
            map.serialize_entry("sumOrFiltersScores", &true)?;
        }

        // algolia will guess this to the false by default.
        if let Some(pagination_limited_to) = self.pagination_limited_to.filter(|&it| it != 0) {
            map.serialize_entry("paginationLimitedTo", &pagination_limited_to)?;
        }

        map.end()
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialUpdateQuery {
    /// When true, a partial update on a nonexistent object will create the object, assuming an empty object as the basis.
    /// When false, a partial update on a nonexistent object will be ignored.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub create_if_not_exists: bool,
}

impl Default for PartialUpdateQuery {
    fn default() -> Self {
        Self {
            create_if_not_exists: true,
        }
    }
}

#[derive(serde::Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetSettings {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub searchable_attributes: Option<SearchableAttributes>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub attributes_for_faceting: Option<Vec<FacetAttribute>>,
}

#[derive(serde::Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct VirtualKeyRestrictions {
    /// An identifier used by the rate-limit system to differentiate users using the same IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_token: Option<String>,

    /// Expiration date of the API key.
    #[serde(serialize_with = "datetime_timestamp::serialize_optional")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
    // todo:
    // restrictIndices
    // referers
    // restrictSources
    // searchOptions (flattened)
}

mod datetime_timestamp {
    use chrono::{DateTime, Utc};
    use serde::Serializer;

    // this will _probably_ be useful later?
    #[allow(dead_code)]
    pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(dt.timestamp())
    }

    pub fn serialize_optional<S>(
        dt: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dt.as_ref() {
            Some(dt) => serializer.serialize_some(&dt.timestamp()),
            None => serializer.serialize_none(),
        }
    }
}
