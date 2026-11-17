use gasd_core::{Error, Service, SqliteRepository, TaskStatus, TaskUpdate, jira::JiraLink};

fn link(key: &str) -> JiraLink {
    JiraLink {
        key: key.into(),
        url: format!("https://team.atlassian.net/browse/{key}"),
    }
}

#[test]
fn saved_queries_support_crud_and_validation() {
    let mut app = Service::new(SqliteRepository::in_memory().unwrap());
    assert!(app.get_jira_queries().unwrap().is_empty());
    let first = app
        .save_jira_query(None, " Mine ", " assignee = currentUser() ")
        .unwrap();
    let second = app.save_jira_query(None, "Team", "project = TEAM").unwrap();
    assert_eq!(first.name, "Mine");
    assert_eq!(first.jql, "assignee = currentUser()");
    assert_eq!(
        app.get_jira_queries().unwrap(),
        [first.clone(), second.clone()]
    );
    let edited = app
        .save_jira_query(Some(first.id), "Open", "resolution = Unresolved")
        .unwrap();
    assert_eq!(edited.id, first.id);
    for (name, jql) in [(" ", "project = TEAM"), ("Name", "\n")] {
        assert!(matches!(
            app.save_jira_query(Some(first.id), name, jql),
            Err(Error::Validation { .. })
        ));
    }
    assert_eq!(app.get_jira_queries().unwrap()[0], edited);
    app.delete_jira_query(first.id).unwrap();
    assert_eq!(app.get_jira_queries().unwrap(), [second]);
    assert!(matches!(
        app.delete_jira_query(first.id),
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        app.save_jira_query(Some(first.id), "Missing", "project = TEAM"),
        Err(Error::NotFound { .. })
    ));
}

#[test]
fn one_link_per_task_survives_edits_and_can_be_removed_offline() {
    let mut app = Service::new(SqliteRepository::in_memory().unwrap());
    let section = app.create_section("Work").unwrap();
    let task = app.create_task(section.id, "Review").unwrap();
    assert!(task.jira.is_none());
    let linked = app.link_jira_task(task.id, Some(link("TEAM-1"))).unwrap();
    assert_eq!(linked.status_changed_at, task.status_changed_at);
    assert_eq!(linked.jira, Some(link("TEAM-1")));
    app.link_jira_task(task.id, Some(link("TEAM-2"))).unwrap();
    let edited = app
        .update_task(
            task.id,
            TaskUpdate {
                title: "Edited".into(),
                section_id: section.id,
                status: TaskStatus::Waiting,
            },
        )
        .unwrap();
    assert_eq!(edited.jira, Some(link("TEAM-2")));
    assert_eq!(app.get_tasks().unwrap()[0], edited);
    let query = app.save_jira_query(None, "Team", "project = TEAM").unwrap();
    app.delete_jira_query(query.id).unwrap();
    assert_eq!(app.get_task(task.id).unwrap().jira, Some(link("TEAM-2")));
    assert!(app.link_jira_task(task.id, None).unwrap().jira.is_none());
    assert!(matches!(
        app.link_jira_task(999, Some(link("TEAM-1"))),
        Err(Error::NotFound { .. })
    ));
}

#[test]
fn initial_schema_supports_tasks_and_persists_queries_and_links() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("jira.sqlite3");
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch(include_str!("../migrations/001_initial.sql"))
        .unwrap();
    db.execute_batch(
        "INSERT INTO sections(id, name, position) VALUES (1, 'Work', 0);
        INSERT INTO tasks(id, title, section_id, position) VALUES (1, 'Existing', 1, 0);
        PRAGMA user_version = 1;",
    )
    .unwrap();
    drop(db);
    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    let original = app.get_task(1).unwrap();
    assert_eq!(original.title, "Existing");
    assert!(original.jira.is_none());
    let query = app
        .save_jira_query(None, "Mine", "assignee = currentUser()")
        .unwrap();
    let linked = app.link_jira_task(1, Some(link("TEAM-1"))).unwrap();
    drop(app);
    let app = Service::new(SqliteRepository::open(&path).unwrap());
    assert_eq!(app.get_jira_queries().unwrap(), [query]);
    assert_eq!(app.get_task(1).unwrap(), linked);
    assert_eq!(linked.status_changed_at, original.status_changed_at);
}
