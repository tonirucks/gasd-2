use gasd_core::{Error, Service, SqliteRepository, TaskList, TaskStatus};

#[test]
fn each_status_and_section_has_independent_order_and_membership() {
    let mut app = Service::new(SqliteRepository::in_memory().unwrap());
    let a = app.create_section("A").unwrap();
    let b = app.create_section("B").unwrap();
    let moving = app.create_task(b.id, "Move me").unwrap();
    for status in [
        TaskStatus::Open,
        TaskStatus::InProgress,
        TaskStatus::Waiting,
        TaskStatus::Done,
    ] {
        let first = app.create_task(a.id, "First").unwrap();
        let second = app.create_task(a.id, "Second").unwrap();
        let foreign = app.create_task(b.id, "Other section").unwrap();
        app.set_task_status(first.id, status).unwrap();
        app.set_task_status(second.id, status).unwrap();
        app.set_task_status(foreign.id, status).unwrap();
        let column = TaskList {
            section_id: a.id,
            status,
        };
        let before = app.get_tasks().unwrap();
        for ids in [
            vec![first.id],
            vec![first.id, foreign.id],
            vec![first.id, first.id],
        ] {
            assert!(matches!(
                app.reorder_tasks(column, &ids),
                Err(Error::InvalidOrder)
            ));
            assert_eq!(app.get_tasks().unwrap(), before);
        }
        app.reorder_tasks(column, &[second.id, first.id]).unwrap();
        assert_eq!(app.get_task(second.id).unwrap().position, 0);
        assert_eq!(app.get_task(first.id).unwrap().position, 1);
        let before = app.get_tasks().unwrap();
        assert!(matches!(
            app.place_task(moving.id, column, &[moving.id, first.id]),
            Err(Error::InvalidOrder)
        ));
        assert_eq!(app.get_tasks().unwrap(), before);
        app.place_task(moving.id, column, &[second.id, moving.id, first.id])
            .unwrap();
        let moved = app.get_task(moving.id).unwrap();
        assert_eq!(
            (moved.section_id, moved.status, moved.position),
            (a.id, status, 1)
        );
        // Clear this column for the next status without affecting other columns.
        app.delete_task(first.id).unwrap();
        app.delete_task(second.id).unwrap();
        app.place_task(
            moving.id,
            TaskList {
                section_id: b.id,
                status,
            },
            &[foreign.id, moving.id],
        )
        .unwrap();
    }
}

#[test]
fn older_done_tasks_remain_in_complete_membership_and_reorders_preserve_timestamps() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tasks.sqlite3");
    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    let section = app.create_section("Work").unwrap();
    let old = app.create_task(section.id, "Old done").unwrap();
    let today = app.create_task(section.id, "Today done").unwrap();
    let moving = app.create_task(section.id, "Moving").unwrap();
    app.set_task_status(old.id, TaskStatus::Done).unwrap();
    app.set_task_status(today.id, TaskStatus::Done).unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute(
            "UPDATE tasks SET status_changed_at = 1 WHERE id = ?1",
            [old.id],
        )
        .unwrap();
    let column = TaskList {
        section_id: section.id,
        status: TaskStatus::Done,
    };
    let before = app.get_tasks().unwrap();
    assert!(matches!(
        app.place_task(moving.id, column, &[today.id, moving.id]),
        Err(Error::InvalidOrder)
    ));
    assert_eq!(app.get_tasks().unwrap(), before);
    app.place_task(moving.id, column, &[old.id, moving.id, today.id])
        .unwrap();
    let moved = app.get_task(moving.id).unwrap();
    assert!(moved.status_changed_at > 1);
    assert_eq!(app.get_task(old.id).unwrap().status_changed_at, 1);
    let timestamp = app.get_task(today.id).unwrap().status_changed_at;
    app.place_task(today.id, column, &[old.id, today.id, moving.id])
        .unwrap();
    assert_eq!(app.get_task(today.id).unwrap().status_changed_at, timestamp);
    let reopened = Service::new(SqliteRepository::open(&path).unwrap());
    assert_eq!(app.get_tasks().unwrap(), reopened.get_tasks().unwrap());
}
