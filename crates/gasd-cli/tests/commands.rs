use std::{
    path::Path,
    process::{Command, Output},
};

use serde_json::Value;

fn invoke(path: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gasd"))
        .env("GASD_DATABASE", path)
        .args(args)
        .output()
        .unwrap()
}

fn json(path: &Path, args: &[&str]) -> Value {
    let mut args = args.to_vec();
    args.push("--json");
    let output = invoke(path, &args);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn command_workflow_persists_and_preserves_unspecified_fields() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("nested/tasks.sqlite3");
    assert_eq!(json(&db, &["list"]), serde_json::json!([]));
    assert_eq!(json(&db, &["add", "section", "Personal"])[0]["id"], 1);
    json(&db, &["add", "section", "Work"]);
    let task = json(&db, &["add", "task", "Read", "--section", "1"]);
    assert_eq!(task[0]["title"], "Read");
    assert_eq!(task[0]["status"], "open");
    assert!(task[0].get("done").is_none());
    assert!(task[0]["statusChangedAt"].as_i64().unwrap() > 0);
    let task = json(&db, &["update", "task", "1", "--status", "done"]);
    assert_eq!(task[0]["status"], "done");
    assert_eq!(task[0]["title"], "Read");
    assert_eq!(task[0]["sectionId"], 1);
    assert_eq!(
        json(&db, &["list", "tasks", "--status", "open"]),
        serde_json::json!([])
    );
    assert_eq!(
        json(&db, &["list", "tasks", "--status", "done"])
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let task = json(
        &db,
        &[
            "update",
            "task",
            "1",
            "--status",
            "inProgress",
            "--title",
            "Write",
            "--section",
            "2",
        ],
    );
    assert_eq!(task[0]["status"], "inProgress");
    assert_eq!(task[0]["title"], "Write");
    assert_eq!(task[0]["sectionId"], 2);
    let timestamp = task[0]["statusChangedAt"].clone();
    let edited = json(&db, &["update", "task", "1", "--title", "Write more"]);
    assert_eq!(edited[0]["status"], "inProgress");
    assert_eq!(edited[0]["statusChangedAt"], timestamp);
    for status in ["open", "inProgress", "waiting", "done"] {
        let updated = json(&db, &["update", "task", "1", "--status", status]);
        assert_eq!(updated[0]["status"], status);
        let filtered = json(
            &db,
            &["list", "tasks", "--status", status, "--section", "2"],
        );
        assert_eq!(filtered, updated);
        assert_eq!(
            json(
                &db,
                &["list", "tasks", "--status", status, "--section", "1"]
            ),
            serde_json::json!([])
        );
    }
    assert_eq!(
        json(&db, &["list", "tasks", "--section", "1"]),
        serde_json::json!([])
    );
    json(&db, &["update", "section", "2", "--name", "Office"]);
    assert_eq!(json(&db, &["list", "sections"])[1]["name"], "Office");
}

#[test]
fn descriptions_can_be_set_preserved_listed_and_cleared() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("tasks.sqlite3");
    json(&db, &["add", "section", "Work"]);
    let task = json(&db, &["add", "task", "Review", "--section", "1"]);
    assert_eq!(task[0]["description"], "");
    let description = "Check changes\nThen approve\t\u{1b}[31m";
    let edited = json(&db, &["update", "task", "1", "--description", description]);
    assert_eq!(edited[0]["description"], description);
    assert_eq!(edited[0]["statusChangedAt"], task[0]["statusChangedAt"]);
    let renamed = json(&db, &["update", "task", "1", "--title", "Review again"]);
    assert_eq!(renamed[0]["description"], description);
    assert_eq!(json(&db, &["list"]), renamed);
    let output = invoke(&db, &["list"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("Check changes\\nThen approve\\t\\u{1b}[31m"));
    let cleared = json(&db, &["update", "task", "1", "--description", ""]);
    assert_eq!(cleared[0]["description"], "");
    assert_eq!(cleared[0]["title"], "Review again");
}

#[test]
fn status_filters_select_exact_matches_without_changing_order() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("db.sqlite3");
    json(&db, &["add", "section", "Work"]);
    for (index, status) in ["open", "inProgress", "waiting", "done"].iter().enumerate() {
        let id = (index + 1).to_string();
        json(&db, &["add", "task", status, "--section", "1"]);
        json(&db, &["update", "task", &id, "--status", status]);
    }
    assert_eq!(json(&db, &["list"]).as_array().unwrap().len(), 4);
    for (index, status) in ["open", "inProgress", "waiting", "done"].iter().enumerate() {
        let rows = json(&db, &["list", "tasks", "--status", status]);
        assert_eq!(rows.as_array().unwrap().len(), 1);
        assert_eq!(rows[0]["id"], index + 1);
        assert_eq!(rows[0]["status"], *status);
    }
}

#[test]
fn invalid_commands_fail_without_changes() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("db.sqlite3");
    json(&db, &["add", "section", "Personal"]);
    json(&db, &["add", "task", "Read", "--section", "1"]);
    for args in [
        vec!["update", "task", "1"],
        vec!["update", "task", "1", "--done", "--active"],
        vec!["update", "task", "999", "--status", "done"],
        vec!["update", "task", "1", "--status", "invalid"],
        vec!["update", "task", "1", "--status", "in-progress"],
        vec![
            "update",
            "task",
            "1",
            "--status",
            "waiting",
            "--title",
            "Changed",
            "--section",
            "999",
        ],
        vec!["update", "task", "1", "--title", " "],
        vec!["update", "task", "1", "--section", "999"],
        vec!["add", "section", " "],
        vec!["add", "task", "Missing section"],
        vec!["list", "tasks", "--done", "--active"],
        vec!["list", "tasks", "--status", "invalid"],
        vec!["list", "tasks", "--active"],
        vec!["update", "task", "1", "--done"],
        vec!["list", "tasks", "--section", "999"],
    ] {
        let output = invoke(&db, &args);
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    let task = json(&db, &["list"]);
    assert_eq!(task[0]["title"], "Read");
    assert_eq!(task[0]["status"], "open");
    assert_eq!(task[0]["sectionId"], 1);
}

#[test]
fn explicit_path_overrides_shared_environment_and_help_does_not_open_database() {
    let dir = tempfile::tempdir().unwrap();
    let env_db = dir.path().join("unused.sqlite3");
    let explicit = dir.path().join("explicit.sqlite3");
    assert!(invoke(&env_db, &["--help"]).status.success());
    assert!(invoke(&env_db, &["--version"]).status.success());
    assert!(!env_db.exists());
    let output = invoke(
        &env_db,
        &[
            "--database",
            explicit.to_str().unwrap(),
            "add",
            "section",
            "Explicit",
        ],
    );
    assert!(output.status.success());
    assert!(explicit.exists());
    assert!(!env_db.exists());
    assert_eq!(
        json(&explicit, &["list", "sections"])[0]["name"],
        "Explicit"
    );
}

#[test]
fn human_output_contains_ids_and_status() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("db.sqlite3");
    json(&db, &["add", "section", "Personal"]);
    let output = invoke(&db, &["add", "task", "Read\nnow", "--section", "1"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ID\tSECTION\tSTATUS\tTITLE"));
    assert!(stdout.contains("1\t1\topen\tRead\\nnow"));
    for status in ["inProgress", "waiting", "done"] {
        let output = invoke(&db, &["update", "task", "1", "--status", status]);
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains(&format!("1\t1\t{status}\tRead\\nnow"))
        );
    }
}

#[test]
fn clean_requires_confirmation_and_removes_without_opening_the_database() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("old.sqlite3");
    // Intentionally not SQLite: clean must not try to open or migrate it.
    let original = b"outdated or invalid database";
    std::fs::write(&db, original).unwrap();
    let output = invoke(&db, &["clean"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(std::fs::read(&db).unwrap(), original);
    assert!(invoke(&db, &["clean", "--help"]).status.success());
    assert_eq!(std::fs::read(&db).unwrap(), original);

    let result = json(&db, &["clean", "--yes"]);
    assert_eq!(
        result,
        serde_json::json!([{ "database": db, "removed": true }])
    );
    assert!(!db.exists());
    assert_eq!(json(&db, &["list"]), serde_json::json!([]));
    assert_eq!(json(&db, &["list", "sections"]), serde_json::json!([]));
    // Fresh migrations must include the latest task status/timestamp schema.
    json(&db, &["add", "section", "Fresh"]);
    let tasks = json(&db, &["add", "task", "Review", "--section", "1"]);
    assert_eq!(tasks[0]["status"], "open");
    assert!(tasks[0]["statusChangedAt"].as_i64().unwrap() > 0);
}

#[test]
fn clean_honors_explicit_path_and_missing_files_do_not_create_directories() {
    let dir = tempfile::tempdir().unwrap();
    let env_db = dir.path().join("keep.sqlite3");
    let explicit = dir.path().join("remove.sqlite3");
    std::fs::write(&env_db, "keep").unwrap();
    std::fs::write(&explicit, "remove").unwrap();
    let output = invoke(
        &env_db,
        &["--database", explicit.to_str().unwrap(), "clean", "--yes"],
    );
    assert!(output.status.success());
    assert!(!explicit.exists());
    assert_eq!(std::fs::read_to_string(&env_db).unwrap(), "keep");
    let missing = dir.path().join("missing/db.sqlite3");
    assert_eq!(
        json(&missing, &["clean", "--yes"]),
        serde_json::json!([{ "database": missing, "removed": false }])
    );
    assert!(!missing.parent().unwrap().exists());
}

#[test]
fn clean_rejects_empty_paths_and_does_not_remove_directories() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("keep.txt");
    std::fs::write(&marker, "keep").unwrap();
    let output = invoke(dir.path(), &["clean", "--yes"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(std::fs::read_to_string(&marker).unwrap(), "keep");
    let output = invoke(dir.path(), &["--database", "", "clean", "--yes"]);
    assert_eq!(output.status.code(), Some(2));
    let output = invoke(Path::new(""), &["clean", "--yes"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("database path must not be empty"));
}
