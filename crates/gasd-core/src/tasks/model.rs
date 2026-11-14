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
    /// Compatibility projection of `status == TaskStatus::Done`.
    pub done: bool,
    pub position: i64,
}

/// Legacy title, section, and completion update. Non-done statuses are preserved
/// unless completion changes. Use `set_task_status` for explicit status changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskUpdate {
    pub title: String,
    pub section_id: i64,
    pub done: bool,
}

/// Active tasks are ordered within their section; completed tasks share one list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TaskList {
    Active { section_id: i64 },
    Done,
}
