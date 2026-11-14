use gasd_core::{Error, Service, SqliteRepository, Task, TaskList, TaskStatus, TaskUpdate};
use rusqlite::Connection;

fn service() -> Service {
    Service::new(SqliteRepository::in_memory().unwrap())
}

fn update(task: &Task, section_id: i64, done: bool) -> TaskUpdate {
    TaskUpdate {
        title: task.title.clone(),
        section_id,
        status: if done {
            TaskStatus::Done
        } else {
            TaskStatus::Open
        },
    }
}

fn task_ids(service: &Service) -> Vec<i64> {
    service
        .get_tasks()
        .unwrap()
        .iter()
        .map(|task| task.id)
        .collect()
}

#[test]
fn section_and_task_crud() {
    let mut app = service();
    assert!(app.get_sections().unwrap().is_empty());
    assert!(app.get_tasks().unwrap().is_empty());

    let section = app.create_section("  Personal  ").unwrap();
    assert_eq!(section.name, "Personal");
    assert_eq!(app.get_section(section.id).unwrap(), section);
    let renamed = app.rename_section(section.id, "  Home  ").unwrap();
    assert_eq!(renamed.name, "Home");

    let task = app.create_task(section.id, "  Read a book  ").unwrap();
    assert_eq!(task.title, "Read a book");
    assert_eq!(task.status, TaskStatus::Open);
    assert_eq!(app.get_task(task.id).unwrap(), task);
    let mut changes = update(&task, section.id, true);
    changes.title = "  Read two books  ".into();
    let edited = app.update_task(task.id, changes).unwrap();
    assert_eq!(edited.title, "Read two books");
    assert_eq!(edited.status, TaskStatus::Done);

    app.delete_task(task.id).unwrap();
    assert!(app.get_tasks().unwrap().is_empty());
    app.delete_section(section.id).unwrap();
    assert!(app.get_sections().unwrap().is_empty());
}

#[test]
fn blank_input_is_rejected_without_modifying_existing_data() {
    let mut app = service();
    assert!(matches!(
        app.create_section(" \n\t\u{2003}"),
        Err(Error::Validation { field: "name" })
    ));
    let section = app.create_section("Work").unwrap();
    let task = app.create_task(section.id, "Review").unwrap();
    assert!(matches!(
        app.rename_section(section.id, " "),
        Err(Error::Validation { .. })
    ));
    assert!(matches!(
        app.create_task(section.id, "\t"),
        Err(Error::Validation { field: "title" })
    ));
    let mut changes = update(&task, section.id, true);
    changes.title = "\n".into();
    assert!(matches!(
        app.update_task(task.id, changes),
        Err(Error::Validation { .. })
    ));
    assert_eq!(app.get_section(section.id).unwrap(), section);
    assert_eq!(app.get_task(task.id).unwrap(), task);
}

#[test]
fn missing_entities_return_typed_errors() {
    let mut app = service();
    assert!(matches!(
        app.get_section(99),
        Err(Error::NotFound {
            entity: "section",
            id: 99
        })
    ));
    assert!(matches!(
        app.get_task(99),
        Err(Error::NotFound {
            entity: "task",
            id: 99
        })
    ));
    assert!(matches!(
        app.rename_section(99, "Work"),
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        app.delete_section(99),
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(app.delete_task(99), Err(Error::NotFound { .. })));
    assert!(matches!(
        app.create_task(99, "Orphan"),
        Err(Error::NotFound {
            entity: "section",
            ..
        })
    ));
    let changes = TaskUpdate {
        title: "Missing".into(),
        section_id: 99,
        status: TaskStatus::Open,
    };
    assert!(matches!(
        app.update_task(99, changes),
        Err(Error::NotFound { entity: "task", .. })
    ));
    assert!(matches!(
        app.reorder_tasks(TaskList::Active { section_id: 99 }, &[]),
        Err(Error::NotFound { .. })
    ));
}

#[test]
fn invalid_move_rolls_back_all_task_changes() {
    let mut app = service();
    let section = app.create_section("Work").unwrap();
    let task = app.create_task(section.id, "Review").unwrap();
    let changes = TaskUpdate {
        title: "Changed".into(),
        section_id: 99,
        status: TaskStatus::Done,
    };
    assert!(matches!(
        app.update_task(task.id, changes),
        Err(Error::NotFound {
            entity: "section",
            ..
        })
    ));
    assert_eq!(app.get_task(task.id).unwrap(), task);
    // A failed transaction must leave the connection usable.
    app.create_task(section.id, "Next").unwrap();
}

#[test]
fn section_order_is_persisted_and_invalid_orders_are_atomic() {
    let mut app = service();
    let a = app.create_section("A").unwrap();
    let b = app.create_section("B").unwrap();
    app.reorder_sections(&[b.id, a.id]).unwrap();
    let before = app.get_sections().unwrap();
    assert_eq!(
        before.iter().map(|section| section.id).collect::<Vec<_>>(),
        [b.id, a.id]
    );
    for ids in [
        vec![],
        vec![a.id],
        vec![a.id, a.id],
        vec![a.id, 999],
        vec![a.id, b.id, 999],
    ] {
        assert!(matches!(
            app.reorder_sections(&ids),
            Err(Error::InvalidOrder)
        ));
        assert_eq!(app.get_sections().unwrap(), before);
    }
    let c = app.create_section("C").unwrap();
    assert!(c.position > before[1].position);
}

#[test]
fn task_order_is_scoped_and_invalid_orders_are_atomic() {
    let mut app = service();
    let a = app.create_section("A").unwrap();
    let b = app.create_section("B").unwrap();
    let first = app.create_task(a.id, "First").unwrap();
    let second = app.create_task(a.id, "Second").unwrap();
    let other = app.create_task(b.id, "Other").unwrap();
    let list = TaskList::Active { section_id: a.id };
    app.reorder_tasks(list, &[second.id, first.id]).unwrap();
    assert_eq!(task_ids(&app), [second.id, first.id, other.id]);
    let before = app.get_tasks().unwrap();
    for ids in [
        vec![],
        vec![first.id],
        vec![first.id, first.id],
        vec![first.id, other.id],
    ] {
        assert!(matches!(
            app.reorder_tasks(list, &ids),
            Err(Error::InvalidOrder)
        ));
        assert_eq!(app.get_tasks().unwrap(), before);
    }
    app.reorder_sections(&[b.id, a.id]).unwrap();
    assert_eq!(task_ids(&app), [other.id, second.id, first.id]);
}

#[test]
fn moves_completion_and_reopening_append_to_the_correct_list() {
    let mut app = service();
    let a = app.create_section("A").unwrap();
    let b = app.create_section("B").unwrap();
    let first = app.create_task(a.id, "First").unwrap();
    let second = app.create_task(b.id, "Second").unwrap();
    let moved = app
        .update_task(first.id, update(&first, b.id, false))
        .unwrap();
    assert!(moved.position > second.position);
    assert_eq!(task_ids(&app), [second.id, first.id]);

    app.update_task(first.id, update(&first, b.id, true))
        .unwrap();
    app.update_task(second.id, update(&second, b.id, true))
        .unwrap();
    assert_eq!(task_ids(&app), [first.id, second.id]);
    app.reorder_tasks(TaskList::Done, &[second.id, first.id])
        .unwrap();
    assert_eq!(task_ids(&app), [second.id, first.id]);

    // A completed task keeps its done-list position when its section changes.
    let done = app.get_task(first.id).unwrap();
    let changed = app
        .update_task(first.id, update(&first, a.id, true))
        .unwrap();
    assert_eq!(changed.position, done.position);
    let active = app.create_task(a.id, "Active").unwrap();
    app.update_task(first.id, update(&first, a.id, false))
        .unwrap();
    assert_eq!(task_ids(&app), [active.id, first.id, second.id]);
}

#[test]
fn deleting_sections_cascades_to_active_and_completed_tasks() {
    let mut app = service();
    let a = app.create_section("A").unwrap();
    let b = app.create_section("B").unwrap();
    app.create_task(a.id, "Active").unwrap();
    let done = app.create_task(a.id, "Done").unwrap();
    app.update_task(done.id, update(&done, a.id, true)).unwrap();
    let retained = app.create_task(b.id, "Retained").unwrap();
    app.delete_section(a.id).unwrap();
    assert_eq!(app.get_tasks().unwrap(), [retained]);
}

#[test]
fn file_database_survives_reopening_and_connections_share_data() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tasks.sqlite3");
    let mut first = Service::new(SqliteRepository::open(&path).unwrap());
    let section = first.create_section("Work").unwrap();
    let a = first.create_task(section.id, "A").unwrap();
    let b = first.create_task(section.id, "B").unwrap();
    first
        .reorder_tasks(
            TaskList::Active {
                section_id: section.id,
            },
            &[b.id, a.id],
        )
        .unwrap();
    let mut second = Service::new(SqliteRepository::open(&path).unwrap());
    assert_eq!(second.get_tasks().unwrap(), first.get_tasks().unwrap());
    let c = second.create_task(section.id, "C").unwrap();
    // Stale UI orders cannot silently omit tasks added by another client.
    assert!(matches!(
        first.reorder_tasks(
            TaskList::Active {
                section_id: section.id
            },
            &[b.id, a.id]
        ),
        Err(Error::InvalidOrder)
    ));
    let expected = first.get_tasks().unwrap();
    drop(first);
    drop(second);
    let reopened = Service::new(SqliteRepository::open(&path).unwrap());
    assert_eq!(reopened.get_tasks().unwrap(), expected);
    assert_eq!(task_ids(&reopened), [b.id, a.id, c.id]);
}

#[test]
fn schema_constraints_protect_against_invalid_direct_writes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tasks.sqlite3");
    let mut repository = SqliteRepository::open(&path).unwrap();
    // The repository enables foreign keys on its own connection.
    repository.create_section("Work").unwrap();
    assert!(matches!(
        repository.create_task(999, "Orphan"),
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        repository.create_section(" "),
        Err(Error::Storage(_))
    ));
    let connection = Connection::open(&path).unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    assert!(
        connection
            .execute(
                "INSERT INTO tasks(title, section_id, position) VALUES ('Orphan', 999, 0)",
                []
            )
            .is_err()
    );
    assert!(
        connection
            .execute(
                "INSERT INTO tasks(title, section_id, status, position) VALUES ('Invalid', 1, 'invalid', 0)",
                []
            )
            .is_err()
    );
    assert!(
        connection
            .execute(
                "INSERT INTO sections(name, position) VALUES ('Invalid', -1)",
                []
            )
            .is_err()
    );
}

#[test]
fn storage_errors_preserve_the_database_error_as_their_source() {
    let mut repository = SqliteRepository::in_memory().unwrap();
    let error = repository.create_section(" ").unwrap_err();
    assert!(matches!(&error, Error::Storage(_)));
    let source = std::error::Error::source(&error).unwrap();
    assert!(source.is::<rusqlite::Error>());
    assert!(error.to_string().contains(&source.to_string()));
}

#[test]
fn database_failure_mid_reorder_rolls_back_earlier_writes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tasks.sqlite3");
    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    let a = app.create_section("A").unwrap();
    let b = app.create_section("B").unwrap();
    let before = app.get_sections().unwrap();
    let connection = Connection::open(&path).unwrap();
    // B is updated first; fail on A to exercise rollback after a successful write.
    connection
        .execute_batch(&format!(
            "CREATE TRIGGER fail_reorder BEFORE UPDATE OF position ON sections
         WHEN OLD.id = {} BEGIN SELECT RAISE(ABORT, 'injected failure'); END;",
            a.id,
        ))
        .unwrap();
    assert!(matches!(
        app.reorder_sections(&[b.id, a.id]),
        Err(Error::Storage(_))
    ));
    assert_eq!(app.get_sections().unwrap(), before);
    connection
        .execute_batch("DROP TRIGGER fail_reorder")
        .unwrap();
    app.reorder_sections(&[b.id, a.id]).unwrap();
}

#[test]
fn newer_schema_is_rejected_without_modification() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("future.sqlite3");
    let connection = Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 99).unwrap();
    assert!(matches!(
        SqliteRepository::open(&path),
        Err(Error::UnsupportedSchema {
            found: 99,
            supported: 1
        })
    ));
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 99);
}

#[test]
fn placing_tasks_moves_and_orders_the_destination_atomically() {
    let mut app = service();
    let a = app.create_section("A").unwrap();
    let b = app.create_section("B").unwrap();
    let first = app.create_task(a.id, "First").unwrap();
    let second = app.create_task(b.id, "Second").unwrap();
    let destination = TaskList::Active { section_id: b.id };
    let before = app.get_tasks().unwrap();
    for ids in [
        vec![first.id],
        vec![second.id],
        vec![first.id, first.id],
        vec![first.id, 999],
    ] {
        assert!(matches!(
            app.place_task(first.id, destination, &ids),
            Err(Error::InvalidOrder)
        ));
        assert_eq!(app.get_tasks().unwrap(), before);
    }
    app.place_task(first.id, destination, &[first.id, second.id])
        .unwrap();
    assert_eq!(task_ids(&app), [first.id, second.id]);
    assert_eq!(app.get_task(first.id).unwrap().section_id, b.id);
    app.place_task(first.id, TaskList::Done, &[first.id])
        .unwrap();
    assert_eq!(app.get_task(first.id).unwrap().section_id, b.id);
    app.place_task(second.id, TaskList::Done, &[second.id, first.id])
        .unwrap();
    assert_eq!(task_ids(&app), [second.id, first.id]);
    app.place_task(first.id, TaskList::Active { section_id: a.id }, &[first.id])
        .unwrap();
    assert_eq!(app.get_task(first.id).unwrap().status, TaskStatus::Open);
    assert_eq!(app.get_task(first.id).unwrap().section_id, a.id);
    assert!(matches!(
        app.place_task(first.id, TaskList::Active { section_id: 999 }, &[first.id]),
        Err(Error::NotFound { .. })
    ));
}

#[test]
fn task_list_ipc_uses_tagged_camel_case() {
    let active = TaskList::Active { section_id: 42 };
    let json = serde_json::to_value(active).unwrap();
    assert_eq!(
        json,
        serde_json::json!({ "kind": "active", "sectionId": 42 })
    );
    assert_eq!(serde_json::from_value::<TaskList>(json).unwrap(), active);
    assert_eq!(
        serde_json::to_value(TaskList::Done).unwrap(),
        serde_json::json!({ "kind": "done" })
    );
}

#[test]
fn empty_lists_can_be_reordered_and_deleted_ids_are_not_reused() {
    let mut app = service();
    app.reorder_sections(&[]).unwrap();
    app.reorder_tasks(TaskList::Done, &[]).unwrap();
    let section = app.create_section("A").unwrap();
    app.reorder_tasks(
        TaskList::Active {
            section_id: section.id,
        },
        &[],
    )
    .unwrap();
    let task = app.create_task(section.id, "A").unwrap();
    app.delete_task(task.id).unwrap();
    assert!(app.create_task(section.id, "B").unwrap().id > task.id);
    app.delete_section(section.id).unwrap();
    assert!(app.create_section("B").unwrap().id > section.id);
}
