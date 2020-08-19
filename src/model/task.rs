use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub struct TaskId(pub(crate) u64);

#[derive(Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum TaskStatus {
    Published,
    NotPublished,
}

impl TaskStatus {
    pub fn completed(self) -> bool {
        matches!(self, Self::Published)
    }
}
