use gasd_core::{Service, SqliteRepository, settings::JiraSettings};

#[test]
fn settings_persist_without_changing_tasks_or_queries() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("settings.sqlite3");
    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    assert!(app.get_jira_settings().unwrap().is_none());
    let section = app.create_section("Work").unwrap();
    let task = app.create_task(section.id, "Review").unwrap();
    let query = app
        .save_jira_query(None, "Mine", "assignee = currentUser()")
        .unwrap();
    let settings = JiraSettings {
        site: "https://team.atlassian.net".into(),
        email: "me@example.com".into(),
    };
    app.save_jira_settings(&settings).unwrap();
    drop(app);
    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    assert_eq!(app.get_jira_settings().unwrap(), Some(settings));
    app.clear_jira_settings().unwrap();
    app.clear_jira_settings().unwrap();
    assert!(app.get_jira_settings().unwrap().is_none());
    assert_eq!(app.get_task(task.id).unwrap(), task);
    assert_eq!(app.get_jira_queries().unwrap(), [query]);
}

#[test]
fn initial_schema_supports_jira_queries_and_settings() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("initial.sqlite3");
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch(include_str!("../migrations/001_initial.sql"))
        .unwrap();
    db.execute_batch("INSERT INTO jira_queries(id, name, jql) VALUES (1, 'Mine', 'assignee = currentUser()'); PRAGMA user_version = 1;").unwrap();
    drop(db);
    let mut app = Service::new(SqliteRepository::open(&path).unwrap());
    assert_eq!(app.get_jira_queries().unwrap()[0].name, "Mine");
    app.save_jira_settings(&JiraSettings {
        site: "https://team.atlassian.net".into(),
        email: "me@example.com".into(),
    })
    .unwrap();
    assert!(app.get_jira_settings().unwrap().is_some());
}

#[test]
fn invalid_settings_do_not_overwrite_saved_values() {
    let mut app = Service::new(SqliteRepository::in_memory().unwrap());
    let original = JiraSettings {
        site: "https://team.atlassian.net".into(),
        email: "me@example.com".into(),
    };
    app.save_jira_settings(&original).unwrap();
    for settings in [
        JiraSettings {
            site: " ".into(),
            email: original.email.clone(),
        },
        JiraSettings {
            site: original.site.clone(),
            email: " ".into(),
        },
    ] {
        assert!(app.save_jira_settings(&settings).is_err());
        assert_eq!(app.get_jira_settings().unwrap(), Some(original.clone()));
    }
}
