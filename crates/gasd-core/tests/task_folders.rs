use gasd_core::{Error, Service, SqliteRepository, TaskStatus, TaskUpdate};

#[test]
fn links_persist_and_picker_lists_distinct_folders_from_existing_tasks() {
    let directory = tempfile::tempdir().unwrap();
    let db = directory.path().join("tasks.sqlite3");
    let folder = directory.path().canonicalize().unwrap();
    let folder = folder.to_str().unwrap();
    let mut app = Service::new(SqliteRepository::open(&db).unwrap());
    let section = app.create_section("Work").unwrap();
    let task = app
        .create_task_with_folder(section.id, "Review", folder)
        .unwrap();
    assert_eq!(task.folder.as_deref(), Some(folder));
    let other = app.create_task(section.id, "Other").unwrap();
    assert_eq!(other.folder, None);
    app.link_task_folder(other.id, Some(folder)).unwrap();
    assert_eq!(app.get_task_folders().unwrap(), vec![folder]);
    app.update_task(
        task.id,
        TaskUpdate {
            title: "Updated".into(),
            description: "Details".into(),
            section_id: section.id,
            status: TaskStatus::Waiting,
        },
    )
    .unwrap();
    drop(app);

    let mut app = Service::new(SqliteRepository::open(&db).unwrap());
    assert_eq!(
        app.get_task(task.id).unwrap().folder.as_deref(),
        Some(folder)
    );
    let timestamp = app.get_task(task.id).unwrap().status_changed_at;
    let unlinked = app.link_task_folder(task.id, None).unwrap();
    assert_eq!(unlinked.folder, None);
    assert_eq!(unlinked.status_changed_at, timestamp);
    assert_eq!(app.get_task_folders().unwrap(), vec![folder]);
    app.link_task_folder(other.id, None).unwrap();
    assert!(app.get_task_folders().unwrap().is_empty());
    app.link_task_folder(task.id, Some(folder)).unwrap();
    app.delete_section(section.id).unwrap();
    assert!(app.get_task_folders().unwrap().is_empty());
}

#[test]
fn invalid_folders_and_missing_tasks_do_not_change_history() {
    let directory = tempfile::tempdir().unwrap();
    let mut app = Service::new(SqliteRepository::in_memory().unwrap());
    let section = app.create_section("Work").unwrap();
    let missing = directory.path().join("missing");
    assert!(matches!(
        app.create_task_with_folder(section.id, "Review", missing.to_str().unwrap()),
        Err(Error::InvalidFolder)
    ));
    let file = directory.path().join("file");
    std::fs::write(&file, "").unwrap();
    assert!(matches!(
        app.create_task_with_folder(section.id, "Review", file.to_str().unwrap()),
        Err(Error::InvalidFolder)
    ));
    assert!(matches!(
        app.link_task_folder(999, Some(directory.path().to_str().unwrap())),
        Err(Error::NotFound { .. })
    ));
    assert!(app.get_tasks().unwrap().is_empty());
    assert!(app.get_task_folders().unwrap().is_empty());
}
