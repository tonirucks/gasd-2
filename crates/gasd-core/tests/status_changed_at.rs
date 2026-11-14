use std::time::{SystemTime, UNIX_EPOCH};

use gasd_core::{Error, Service, SqliteRepository, TaskList, TaskStatus, TaskUpdate};
use rusqlite::Connection;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

// Use a known earlier timestamp rather than sleeping to test clock updates.
fn reset_timestamp(connection: &Connection, id: i64) {
    connection
        .execute("UPDATE tasks SET status_changed_at = 1 WHERE id = ?1", [id])
        .unwrap();
}

#[test]
fn creation_and_every_status_transition_track_time() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tasks.sqlite3");
    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    let connection = Connection::open(&path).unwrap();
    let section = app.create_section("Work").unwrap();
    let before = now_ms();
    let task = app.create_task(section.id, "Review").unwrap();
    assert!((before..=now_ms()).contains(&task.status_changed_at));
    let json = serde_json::to_value(&task).unwrap();
    assert_eq!(json["statusChangedAt"], task.status_changed_at);

    let statuses = [
        TaskStatus::Open,
        TaskStatus::InProgress,
        TaskStatus::Waiting,
        TaskStatus::Done,
    ];
    for source in statuses {
        for destination in statuses {
            app.set_task_status(task.id, source).unwrap();
            reset_timestamp(&connection, task.id);
            let before = now_ms();
            let updated = app.set_task_status(task.id, destination).unwrap();
            if source == destination {
                assert_eq!(updated.status_changed_at, 1);
            } else {
                assert!((before..=now_ms()).contains(&updated.status_changed_at));
            }
            let reopened = Service::new(SqliteRepository::open(&path).unwrap());
            assert_eq!(reopened.get_task(task.id).unwrap(), updated);
            assert_eq!(app.get_tasks().unwrap(), [updated]);
        }
    }
}

#[test]
fn legacy_updates_placement_and_rollback_handle_timestamps() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tasks.sqlite3");
    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    let connection = Connection::open(&path).unwrap();
    let a = app.create_section("A").unwrap();
    let b = app.create_section("B").unwrap();
    let task = app.create_task(a.id, "Review").unwrap();
    app.set_task_status(task.id, TaskStatus::Waiting).unwrap();
    reset_timestamp(&connection, task.id);

    let edited = app
        .update_task(
            task.id,
            TaskUpdate {
                title: "Edited".into(),
                section_id: b.id,
                done: false,
            },
        )
        .unwrap();
    assert_eq!(edited.status_changed_at, 1);
    let active = TaskList::Active { section_id: a.id };
    app.place_task(task.id, active, &[task.id]).unwrap();
    app.reorder_tasks(active, &[task.id]).unwrap();
    assert_eq!(app.get_task(task.id).unwrap().status_changed_at, 1);
    assert!(matches!(
        app.place_task(task.id, TaskList::Done, &[]),
        Err(Error::InvalidOrder)
    ));
    assert_eq!(app.get_task(task.id).unwrap().status_changed_at, 1);

    for done in [true, false] {
        reset_timestamp(&connection, task.id);
        let changed = app
            .update_task(
                task.id,
                TaskUpdate {
                    title: "Edited".into(),
                    section_id: a.id,
                    done,
                },
            )
            .unwrap();
        assert!(changed.status_changed_at > 1);
    }
    for list in [TaskList::Done, active] {
        reset_timestamp(&connection, task.id);
        app.place_task(task.id, list, &[task.id]).unwrap();
        assert!(app.get_task(task.id).unwrap().status_changed_at > 1);
        reset_timestamp(&connection, task.id);
        app.place_task(task.id, list, &[task.id]).unwrap();
        assert_eq!(app.get_task(task.id).unwrap().status_changed_at, 1);
    }
}

#[test]
fn initial_schema_sets_timestamps_and_reopening_preserves_them() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tasks.sqlite3");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(include_str!("../migrations/001_initial.sql"))
        .unwrap();
    let before = now_ms();
    connection
        .execute_batch(
            "INSERT INTO sections(id, name, position) VALUES (7, 'Work', 3);
         INSERT INTO tasks(id, title, section_id, done, position, status) VALUES
             (10, 'Review', 7, 0, 5, 'waiting'), (11, 'Completed', 7, 1, 2, 'done');
         PRAGMA user_version = 1;",
        )
        .unwrap();
    let app = Service::new(SqliteRepository::open(&path).unwrap());
    let tasks = app.get_tasks().unwrap();
    for task in &tasks {
        assert!((before..=now_ms()).contains(&task.status_changed_at));
        assert_eq!(task.section_id, 7);
    }
    assert_eq!(
        (
            tasks[0].id,
            tasks[0].title.as_str(),
            tasks[0].position,
            tasks[0].status
        ),
        (10, "Review", 5, TaskStatus::Waiting)
    );
    assert_eq!(
        (
            tasks[1].id,
            tasks[1].title.as_str(),
            tasks[1].position,
            tasks[1].status
        ),
        (11, "Completed", 2, TaskStatus::Done)
    );
    let reopened = Service::new(SqliteRepository::open(&path).unwrap());
    assert_eq!(reopened.get_tasks().unwrap(), tasks);
    assert!(
        connection
            .execute("UPDATE tasks SET status_changed_at = -1 WHERE id = 10", [])
            .is_err()
    );
}
