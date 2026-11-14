use gasd_core::{Error, Service, SqliteRepository, TaskList, TaskStatus, TaskUpdate};
use rusqlite::Connection;

#[test]
fn statuses_serialize_with_the_public_camel_case_names() {
    for (status, name) in [
        (TaskStatus::Open, "open"),
        (TaskStatus::InProgress, "inProgress"),
        (TaskStatus::Waiting, "waiting"),
        (TaskStatus::Done, "done"),
    ] {
        let json = serde_json::json!(name);
        assert_eq!(serde_json::to_value(status).unwrap(), json);
        assert_eq!(serde_json::from_value::<TaskStatus>(json).unwrap(), status);
    }
    assert!(serde_json::from_value::<TaskStatus>(serde_json::json!("invalid")).is_err());
}

#[test]
fn all_status_transitions_preserve_fields_and_use_the_correct_list() {
    let mut app = Service::new(SqliteRepository::in_memory().unwrap());
    let section = app.create_section("Work").unwrap();
    let task = app.create_task(section.id, "Review").unwrap();
    assert_eq!(task.status, TaskStatus::Open);
    app.create_task(section.id, "Next").unwrap();

    for source in [
        TaskStatus::Open,
        TaskStatus::InProgress,
        TaskStatus::Waiting,
        TaskStatus::Done,
    ] {
        for destination in [
            TaskStatus::Open,
            TaskStatus::InProgress,
            TaskStatus::Waiting,
            TaskStatus::Done,
        ] {
            let before = app.set_task_status(task.id, source).unwrap();
            let after = app.set_task_status(task.id, destination).unwrap();
            assert_eq!(after.status, destination);
            assert_eq!(after.done, destination.is_done());
            assert_eq!(after.title, task.title);
            assert_eq!(after.section_id, task.section_id);
            if source.is_done() == destination.is_done() {
                assert_eq!(after.position, before.position);
            } else if !destination.is_done() {
                assert_eq!(app.get_tasks().unwrap().last().unwrap().id, task.id);
            }
            assert_eq!(app.get_task(task.id).unwrap(), after);
        }
    }
    assert!(matches!(
        app.set_task_status(999, TaskStatus::Waiting),
        Err(Error::NotFound {
            entity: "task",
            id: 999
        })
    ));
}

#[test]
fn legacy_updates_and_placement_preserve_active_statuses() {
    let mut app = Service::new(SqliteRepository::in_memory().unwrap());
    let a = app.create_section("A").unwrap();
    let b = app.create_section("B").unwrap();
    let task = app.create_task(a.id, "Review").unwrap();
    for status in [TaskStatus::InProgress, TaskStatus::Waiting] {
        app.set_task_status(task.id, status).unwrap();
        let edited = app
            .update_task(
                task.id,
                TaskUpdate {
                    title: "Edited".into(),
                    section_id: a.id,
                    done: false,
                },
            )
            .unwrap();
        assert_eq!(edited.status, status);
        app.place_task(task.id, TaskList::Active { section_id: b.id }, &[task.id])
            .unwrap();
        assert_eq!(app.get_task(task.id).unwrap().status, status);
        let before = app.get_task(task.id).unwrap();
        assert!(matches!(
            app.place_task(task.id, TaskList::Done, &[]),
            Err(Error::InvalidOrder)
        ));
        assert_eq!(app.get_task(task.id).unwrap(), before);
        app.place_task(task.id, TaskList::Done, &[task.id]).unwrap();
        assert_eq!(app.get_task(task.id).unwrap().status, TaskStatus::Done);
        let reopened = app
            .update_task(
                task.id,
                TaskUpdate {
                    title: task.title.clone(),
                    section_id: a.id,
                    done: false,
                },
            )
            .unwrap();
        assert_eq!(reopened.status, TaskStatus::Open);
    }
}

#[test]
fn initial_schema_persists_statuses_and_enforces_consistency() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tasks.sqlite3");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(include_str!("../migrations/001_initial.sql"))
        .unwrap();
    connection
        .execute_batch(
            "INSERT INTO sections(id, name, position) VALUES (7, 'Work', 3);
         INSERT INTO tasks(id, title, section_id, done, position, status) VALUES
             (10, 'Active', 7, 0, 5, 'open'), (11, 'Completed', 7, 1, 2, 'done');
         PRAGMA user_version = 1;",
        )
        .unwrap();
    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    let tasks = app.get_tasks().unwrap();
    assert_eq!(
        tasks
            .iter()
            .map(|task| (task.id, task.section_id, task.position, task.status))
            .collect::<Vec<_>>(),
        [(10, 7, 5, TaskStatus::Open), (11, 7, 2, TaskStatus::Done),]
    );
    for status in [TaskStatus::InProgress, TaskStatus::Waiting] {
        let expected = app.set_task_status(10, status).unwrap();
        let reopened = Service::new(SqliteRepository::open(&path).unwrap());
        assert_eq!(reopened.get_task(10).unwrap(), expected);
    }
    assert_eq!(app.create_task(7, "New").unwrap().status, TaskStatus::Open);
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 1);
    for sql in [
        "UPDATE tasks SET status = 'invalid' WHERE id = 10",
        "UPDATE tasks SET status = 'done' WHERE id = 10",
        "UPDATE tasks SET done = 1 WHERE id = 10",
        "INSERT INTO tasks(title, section_id, done, position) VALUES ('Invalid', 7, 1, 0)",
    ] {
        assert!(connection.execute(sql, []).is_err());
    }
}
