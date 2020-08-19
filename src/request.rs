use crate::model::attribute::SearchableAttributes;

fn check_hits_per_page(max_hits: &Option<u16>) -> bool {
    max_hits.map_or(true, |hits| hits == 20)
}

fn check_page(page: &Option<u32>) -> bool {
    page.map_or(true, |hits| hits == 0)
}

fn check_query(query: &Option<String>) -> bool {
    query.as_deref().unwrap_or("") == ""
}

#[derive(serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    /// The text to search in the index.
    #[serde(skip_serializing_if = "check_query")]
    pub query: Option<String>,

    /// Specify the page to retrieve.
    #[serde(skip_serializing_if = "check_page")]
    pub page: Option<u32>,

    /// Specify the page to retrieve.
    #[serde(skip_serializing_if = "check_hits_per_page")]
    pub hits_per_page: Option<u16>,

    /// Retrieve detailed ranking information.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub get_ranking_info: bool,
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

#[derive(serde::Serialize, Debug, Default)]
pub struct SetSettings {
    searchable_attributes: Option<SearchableAttributes>,
}
