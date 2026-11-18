use gasd_core::{Error, Service, SqliteRepository, TaskStatus, TaskUpdate};

#[test]
fn subtasks_persist_and_remain_independent_of_parent_edits() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("subtasks.sqlite3");
    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    let section = app.create_section("Work").unwrap();
    let task = app.create_task(section.id, "Review").unwrap();
    assert!(task.subtasks.is_empty());
    let first = app
        .create_subtask(task.id, "  Check tests  ")
        .unwrap()
        .subtasks[0]
        .clone();
    let task = app.create_subtask(task.id, "Approve").unwrap();
    assert_eq!(task.subtasks[0].title, "Check tests");
    assert!(!first.completed);
    let edited = app
        .update_subtask(task.id, first.id, "Run tests", true)
        .unwrap();
    assert_eq!(edited.subtasks[0].title, "Run tests");
    assert!(edited.subtasks[0].completed);
    assert_eq!(edited.status_changed_at, task.status_changed_at);
    assert_eq!(edited.status, TaskStatus::Open);
    let moved = app
        .update_task(
            task.id,
            TaskUpdate {
                title: "Review again".into(),
                description: "Details".into(),
                section_id: section.id,
                status: TaskStatus::Done,
            },
        )
        .unwrap();
    assert_eq!(moved.subtasks, edited.subtasks);
    drop(app);

    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    assert_eq!(app.get_task(task.id).unwrap(), moved);
    assert_eq!(app.get_tasks().unwrap(), [moved]);
    assert!(
        !app.update_subtask(task.id, first.id, "Run tests", false)
            .unwrap()
            .subtasks[0]
            .completed
    );
    let remaining = app.delete_subtask(task.id, first.id).unwrap();
    assert_eq!(remaining.subtasks.len(), 1);
    assert_eq!(remaining.subtasks[0].title, "Approve");
}

#[test]
fn validation_and_parent_scoping_prevent_invalid_changes() {
    let mut app = Service::new(SqliteRepository::in_memory().unwrap());
    let section = app.create_section("Work").unwrap();
    let task = app.create_task(section.id, "Review").unwrap();
    let other = app.create_task(section.id, "Other").unwrap();
    assert!(matches!(
        app.create_subtask(task.id, " \n "),
        Err(Error::Validation { .. })
    ));
    assert!(matches!(
        app.create_subtask(999, "Missing"),
        Err(Error::NotFound { .. })
    ));
    let saved = app.create_subtask(task.id, "Check").unwrap();
    let id = saved.subtasks[0].id;
    assert!(matches!(
        app.update_subtask(task.id, id, "", true),
        Err(Error::Validation { .. })
    ));
    assert!(matches!(
        app.update_subtask(other.id, id, "Wrong", true),
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        app.delete_subtask(other.id, id),
        Err(Error::NotFound { .. })
    ));
    assert_eq!(app.get_task(task.id).unwrap(), saved);
    app.delete_task(task.id).unwrap();
    assert!(matches!(
        app.delete_subtask(task.id, id),
        Err(Error::NotFound { .. })
    ));
}

#[test]
fn deleting_a_section_cascades_to_subtasks() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("subtasks.sqlite3");
    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    let section = app.create_section("Work").unwrap();
    let task = app.create_task(section.id, "Review").unwrap();
    app.create_subtask(task.id, "Check").unwrap();
    app.delete_section(section.id).unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    let count: i64 = connection
        .query_row("SELECT count(*) FROM subtasks", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}
