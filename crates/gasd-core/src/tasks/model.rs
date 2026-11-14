use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    #[default]
    Open,
    InProgress,
    Waiting,
    Done,
}

impl TaskStatus {
    pub fn is_done(self) -> bool {
        self == Self::Done
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub section_id: i64,
    pub status: TaskStatus,
    /// UTC Unix timestamp in milliseconds, initialized at creation and updated
    /// only when the status changes.
    pub status_changed_at: i64,
    pub position: i64,
}

/// Complete replacement of editable task fields. Position is managed separately.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskUpdate {
    pub title: String,
    pub section_id: i64,
    pub status: TaskStatus,
}

/// Tasks are ordered within each section/status column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskList {
    pub section_id: i64,
    pub status: TaskStatus,
}
