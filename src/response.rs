use crate::model::task::{TaskId, TaskStatus};
use chrono::{DateTime, Utc};
use serde::Deserialize;

/// A unit struct (like `()`), but as a standard struct with no fields,
/// this allows for serde to "flatten" with it (a no-op, given the lack of anything to {de,}serialize)
#[derive(Deserialize, Debug)]
pub struct FlattenEmpty {}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectUpdateResponse {
    pub updated_at: DateTime<Utc>,

    #[serde(rename = "taskID")]
    pub task_id: TaskId,

    #[serde(rename = "objectID")]
    pub object_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectDeleteResponse {
    pub deleted_at: DateTime<Utc>,

    #[serde(rename = "taskID")]
    pub task_id: TaskId,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse<T = FlattenEmpty> {
    pub hits: Vec<Hit<T>>,

    pub page: usize,

    #[serde(rename = "nbHits")]
    pub hit_count: usize,

    #[serde(rename = "nbPages")]
    pub page_count: usize,

    pub hits_per_page: usize,

    #[serde(rename = "processingTimeMS")]
    pub processing_time_ms: usize,

    pub query: String,

    pub parsed_query: Option<String>,

    pub params: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Hit<T> {
    #[serde(rename = "objectID")]
    pub object_id: String,

    // fixme: fix this and reimplement.
    // // todo: this can be single OR Vec, handle both cases
    // #[serde(rename = "_highlightResult")]
    // #[serde(default)]
    // pub highlight_result: HashMap<String, HighlightResult>,

    // #[serde(rename = "_snippetResult")]
    // #[serde(default)]
    // pub snippet_result: HashMap<String, SnippetResult>,
    #[serde(rename = "_rankingInfo")]
    pub ranking_info: Option<RankingInfo>,

    #[serde(rename = "_distinctSeqID")]
    pub distinct_seq_id: Option<usize>,

    #[serde(flatten)]
    pub inner: T,
}

#[derive(Eq, PartialEq, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum MatchLevel {
    None,
    Partial,
    Full,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HighlightResult {
    pub value: String,
    pub match_level: MatchLevel,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SnippetResult {
    pub value: String,
    pub match_level: MatchLevel,
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TaskStatusResponse {
    pub status: TaskStatus,
    pub pending_task: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SettingsUpdateResponse {
    pub updated_at: DateTime<Utc>,

    #[serde(rename = "taskID")]
    pub task_id: TaskId,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BatchWriteResponse {
    #[serde(rename = "taskID")]
    pub task_id: TaskId,

    #[serde(rename = "objectIDs")]
    #[serde(default)]
    pub object_ids: Vec<String>,
}
