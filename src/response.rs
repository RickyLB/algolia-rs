use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectUpdateResponse {
    pub updated_at: DateTime<Utc>,

    #[serde(rename = "taskID")]
    pub task_id: usize,

    #[serde(rename = "objectID")]
    pub object_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectDeleteResponse {
    pub deleted_at: DateTime<Utc>,

    #[serde(rename = "taskID")]
    pub task_id: usize,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse<T> {
    hits: Vec<Hit<T>>,

    page: usize,

    #[serde(rename = "nbHits")]
    hit_count: usize,

    #[serde(rename = "nbPages")]
    page_count: usize,

    hits_per_page: usize,

    #[serde(rename = "processingTimeMS")]
    processing_time_ms: usize,

    query: String,

    parsed_query: String,

    params: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Hit<T> {
    #[serde(rename = "objectID")]
    pub object_id: String,

    #[serde(rename = "_highlightResult")]
    #[serde(default)]
    pub highlight_result: HashMap<String, HighlightResult>,

    #[serde(rename = "_snippetResult")]
    #[serde(default)]
    pub snippet_result: HashMap<String, SnippetResult>,

    #[serde(rename = "_rankingInfo")]
    pub ranking_info: Option<RankingInfo>,

    #[serde(flatten)]
    pub inner: T,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HighlightResult {
    pub value: String,
    pub match_level: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SnippetResult {
    pub value: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RankingInfo {
    #[serde(rename = "nbTypos")]
    pub typo_count: usize,

    pub first_matched_word: usize,

    pub proximity_distance: usize,

    pub user_score: usize,

    pub geo_distance: usize,

    pub geo_precision: usize,

    #[serde(rename = "nbExactWords")]
    pub exact_word_count: usize,
}
