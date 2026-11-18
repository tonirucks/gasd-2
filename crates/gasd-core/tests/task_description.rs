use gasd_core::{Service, SqliteRepository, TaskList, TaskStatus, TaskUpdate};

#[test]
fn version_one_upgrades_without_losing_existing_data() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tasks.sqlite3");
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch(include_str!("../migrations/001_initial.sql"))
        .unwrap();
    connection
        .execute_batch(
            "INSERT INTO sections(id, name, position) VALUES (7, 'Work', 3);
             INSERT INTO tasks(id, title, section_id, position, status, status_changed_at, jira_key, jira_url)
                 VALUES (10, 'Review', 7, 5, 'waiting', 123, 'TEAM-1', 'https://team.atlassian.net/browse/TEAM-1');
             INSERT INTO jira_queries(id, name, jql) VALUES (2, 'Team', 'project = TEAM');
             INSERT INTO app_settings(key, value) VALUES ('example', 'preserved');
             PRAGMA user_version = 1;",
        )
        .unwrap();

    let app = Service::new(SqliteRepository::open(&path).unwrap());
    let task = app.get_task(10).unwrap();
    assert_eq!(task.description, "");
    assert_eq!(task.title, "Review");
    assert_eq!(task.section_id, 7);
    assert_eq!(task.position, 5);
    assert_eq!(task.status, TaskStatus::Waiting);
    assert_eq!(task.status_changed_at, 123);
    assert_eq!(task.jira.as_ref().unwrap().key, "TEAM-1");
    assert_eq!(
        task.jira.as_ref().unwrap().url,
        "https://team.atlassian.net/browse/TEAM-1"
    );
    assert_eq!(app.get_section(7).unwrap().position, 3);
    assert_eq!(app.get_section(7).unwrap().name, "Work");
    assert_eq!(app.get_jira_queries().unwrap()[0].jql, "project = TEAM");
    let setting: String = connection
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'example'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(setting, "preserved");
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 2);
    drop(app);

    // Reopening must not attempt to add the column again.
    let app = Service::new(SqliteRepository::open(&path).unwrap());
    assert_eq!(app.get_task(10).unwrap(), task);
}

#[test]
fn descriptions_survive_reopening_moving_and_status_changes_and_can_be_cleared() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tasks.sqlite3");
    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    let section = app.create_section("Work").unwrap();
    let task = app.create_task(section.id, "Review").unwrap();
    assert_eq!(task.description, "");
    let description = "  Context: café ☕\n\n- Check the changes\n- Approve <when ready>  ";
    let edited = app
        .update_task(
            task.id,
            TaskUpdate {
                title: task.title.clone(),
                description: description.into(),
                section_id: task.section_id,
                status: task.status,
            },
        )
        .unwrap();
    assert_eq!(edited.description, description);
    assert_eq!(edited.position, task.position);
    assert_eq!(edited.status_changed_at, task.status_changed_at);
    assert_eq!(
        serde_json::to_value(&edited).unwrap()["description"],
        description
    );
    drop(app);

    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    assert_eq!(app.get_task(task.id).unwrap(), edited);
    assert_eq!(app.get_tasks().unwrap(), [edited]);
    let other = app.create_section("Personal").unwrap();
    let destination = TaskList {
        section_id: other.id,
        status: TaskStatus::Waiting,
    };
    app.place_task(task.id, destination, &[task.id]).unwrap();
    app.reorder_tasks(destination, &[task.id]).unwrap();
    let moved = app.set_task_status(task.id, TaskStatus::Done).unwrap();
    assert_eq!(moved.description, description);
    let cleared = app
        .update_task(
            task.id,
            TaskUpdate {
                title: moved.title,
                description: String::new(),
                section_id: moved.section_id,
                status: moved.status,
            },
        )
        .unwrap();
    assert_eq!(cleared.description, "");
    assert_eq!(app.get_task(task.id).unwrap(), cleared);
}
