use chrono::{DateTime, Utc};
use serde::Deserialize;

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
